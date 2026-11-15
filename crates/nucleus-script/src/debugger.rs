//! Boa JavaScript runtime debugger bridge and cycle-safe object inspector.

use std::collections::HashSet;
use boa_engine::object::builtins::JsArray;
use boa_engine::{Context, JsArgs, JsNativeError, JsResult, JsString, JsValue, NativeFunction};
use nucleus_common::error::{NucleusError, NucleusResult};
use nucleus_debug::controller::DebugController;
use nucleus_debug::protocol::InspectValue;

/// Recursively inspects a Boa JsValue, preventing infinite recursion on circular references.
pub fn js_value_to_inspect(
    name: &str,
    val: &JsValue,
    ctx: &mut Context,
    max_depth: usize,
    visited: &mut HashSet<usize>,
) -> InspectValue {
    if max_depth == 0 {
        return InspectValue {
            name: name.to_string(),
            type_name: "object".to_string(),
            summary: "{...}".to_string(),
            children: Vec::new(),
        };
    }

    if val.is_null() {
        InspectValue {
            name: name.to_string(),
            type_name: "null".to_string(),
            summary: "null".to_string(),
            children: Vec::new(),
        }
    } else if val.is_undefined() {
        InspectValue {
            name: name.to_string(),
            type_name: "undefined".to_string(),
            summary: "undefined".to_string(),
            children: Vec::new(),
        }
    } else if let Some(b) = val.as_boolean() {
        InspectValue {
            name: name.to_string(),
            type_name: "boolean".to_string(),
            summary: b.to_string(),
            children: Vec::new(),
        }
    } else if let Some(n) = val.as_number() {
        InspectValue {
            name: name.to_string(),
            type_name: "number".to_string(),
            summary: n.to_string(),
            children: Vec::new(),
        }
    } else if let Some(s) = val.as_string() {
        InspectValue {
            name: name.to_string(),
            type_name: "string".to_string(),
            summary: format!("\"{}\"", s.to_std_string_escaped()),
            children: Vec::new(),
        }
    } else if let Some(b) = val.as_bigint() {
        InspectValue {
            name: name.to_string(),
            type_name: "bigint".to_string(),
            summary: format!("{b}n"),
            children: Vec::new(),
        }
    } else if val.is_symbol() {
        InspectValue {
            name: name.to_string(),
            type_name: "symbol".to_string(),
            summary: "Symbol()".to_string(),
            children: Vec::new(),
        }
    } else if let Some(obj) = val.as_object() {
        // Memory identity address used for circular reference detection
        let addr = obj.as_ref() as *const _ as *const () as usize;
        if visited.contains(&addr) {
            return InspectValue {
                name: name.to_string(),
                type_name: "CircularRef".to_string(),
                summary: "[Circular]".to_string(),
                children: Vec::new(),
            };
        }
        visited.insert(addr);

        // Handle Arrays
        if obj.is_array() {
            let js_arr = JsArray::from_object(obj.clone()).unwrap();
            let len = js_arr.length(ctx).unwrap_or(0);
            let summary = format!("Array({len})");

            let mut children = Vec::new();
            let limit = len.min(50);
            for i in 0..limit {
                if let Ok(elem) = js_arr.get(i, ctx) {
                    children.push(js_value_to_inspect(
                        &format!("[{i}]"),
                        &elem,
                        ctx,
                        max_depth - 1,
                        visited,
                    ));
                }
            }

            return InspectValue {
                name: name.to_string(),
                type_name: "array".to_string(),
                summary,
                children,
            };
        }

        // Handle standard Objects
        let keys = obj.own_property_keys(ctx).unwrap_or_default();
        let summary = format!("Object {{ {} keys }}", keys.len());
        let mut children = Vec::new();

        for key in keys.into_iter().take(50) {
            let key_str = match &key {
                boa_engine::property::PropertyKey::String(s) => s.to_std_string_escaped(),
                boa_engine::property::PropertyKey::Index(idx) => idx.get().to_string(),
                boa_engine::property::PropertyKey::Symbol(_) => "[Symbol]".to_string(),
            };

            if let Ok(prop_val) = obj.get(key, ctx) {
                children.push(js_value_to_inspect(
                    &key_str,
                    &prop_val,
                    ctx,
                    max_depth - 1,
                    visited,
                ));
            }
        }

        InspectValue {
            name: name.to_string(),
            type_name: "object".to_string(),
            summary,
            children,
        }
    } else {
        InspectValue {
            name: name.to_string(),
            type_name: "unknown".to_string(),
            summary: "unknown".to_string(),
            children: Vec::new(),
        }
    }
}

/// Registers the native `__nucleus_dbg_stmt`, `__nucleus_dbg_enter`, and `__nucleus_dbg_exit` hooks in the Boa Context.
pub fn register_debug_probes(
    ctx: &mut Context,
    controller: DebugController,
) -> NucleusResult<()> {
    let ctrl_stmt = controller.clone();
    let stmt_func = unsafe {
        NativeFunction::from_closure(move |_this: &JsValue, args: &[JsValue], ctx: &mut Context| -> JsResult<JsValue> {
            let file_id = args.get_or_undefined(0).to_string(ctx)?.to_std_string_escaped();
            let line = args.get_or_undefined(1).to_u32(ctx)?;
            let col = args.get_or_undefined(2).to_u32(ctx)?;
            let eval_closure = args.get(3).cloned();

            let mut eval_fn = |expr: &str| -> Result<String, String> {
                if let Some(ref closure) = eval_closure {
                    if let Some(func_obj) = closure.as_object() {
                        let arg = JsString::from(expr).into();
                        match func_obj.call(&JsValue::undefined(), &[arg], ctx) {
                            Ok(v) => Ok(v.to_string(ctx).map(|s| s.to_std_string_escaped()).unwrap_or_default()),
                            Err(e) => Err(e.to_string()),
                        }
                    } else {
                        Err("Eval closure is not callable".to_string())
                    }
                } else {
                    Err("No eval closure provided".to_string())
                }
            };

            if let Err(e) = ctrl_stmt.on_statement_probe(&file_id, line, col, Some(&mut eval_fn)) {
                return Err(JsNativeError::error().with_message(e.to_string()).into());
            }

            Ok(JsValue::undefined())
        })
    };

    ctx.register_global_callable(
        boa_engine::js_string!("__nucleus_dbg_stmt"),
        4,
        stmt_func,
    ).map_err(|e| NucleusError::Script(format!("Failed registering __nucleus_dbg_stmt: {e}")))?;

    let ctrl_enter = controller.clone();
    let enter_func = unsafe {
        NativeFunction::from_closure(move |_this: &JsValue, _args: &[JsValue], _ctx: &mut Context| -> JsResult<JsValue> {
            ctrl_enter.enter_function();
            Ok(JsValue::undefined())
        })
    };
    ctx.register_global_callable(
        boa_engine::js_string!("__nucleus_dbg_enter"),
        0,
        enter_func,
    ).map_err(|e| NucleusError::Script(format!("Failed registering __nucleus_dbg_enter: {e}")))?;

    let ctrl_exit = controller;
    let exit_func = unsafe {
        NativeFunction::from_closure(move |_this: &JsValue, _args: &[JsValue], _ctx: &mut Context| -> JsResult<JsValue> {
            ctrl_exit.exit_function();
            Ok(JsValue::undefined())
        })
    };
    ctx.register_global_callable(
        boa_engine::js_string!("__nucleus_dbg_exit"),
        0,
        exit_func,
    ).map_err(|e| NucleusError::Script(format!("Failed registering __nucleus_dbg_exit: {e}")))?;

    Ok(())
}
