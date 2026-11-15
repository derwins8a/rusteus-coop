//! WebAssembly (WASM) plugin sandboxing using wasmi with fuel-metering and scoped host imports.

use std::collections::HashSet;
use std::fs;
use std::path::PathBuf;
use tracing::{debug, error, info, warn};
use wasmi::{Caller, Config, Engine, Error as WasmiError, Linker, Module, Store, TypedFunc};

use crate::manifest::PluginPermission;
use crate::permissions::PermissionStore;
use nucleus_common::error::{NucleusError, NucleusResult};

/// Host execution context and permission bounds provided to a sandboxed Wasm instance.
#[derive(Debug)]
pub struct WasmHostContext {
    pub plugin_id: String,
    pub permissions: HashSet<PluginPermission>,
    pub instance_dir: Option<PathBuf>,
    pub logs: Vec<String>,
}

impl WasmHostContext {
    pub fn new<I>(plugin_id: String, permissions: I, instance_dir: Option<PathBuf>) -> Self
    where
        I: IntoIterator<Item = PluginPermission>,
    {
        Self {
            plugin_id,
            permissions: permissions.into_iter().collect(),
            instance_dir,
            logs: Vec::new(),
        }
    }
}

/// Sandboxed WebAssembly instance runtime powered by pure-Rust wasmi.
pub struct WasmPluginHost {
    engine: Engine,
    module: Module,
    default_fuel: u64,
}

impl WasmPluginHost {
    /// Compiles a Wasm binary module and configures the fuel-metered execution engine.
    pub fn new(wasm_bytes: &[u8], default_fuel: u64) -> NucleusResult<Self> {
        let mut config = Config::default();
        config.consume_fuel(true);

        let engine = Engine::new(&config);
        let module = Module::new(&engine, wasm_bytes)
            .map_err(|e| NucleusError::Plugin(format!("Failed to compile Wasm module: {e}")))?;

        Ok(Self {
            engine,
            module,
            default_fuel,
        })
    }

    /// Creates and configures the Linker with host import APIs.
    fn create_linker(&self) -> NucleusResult<Linker<WasmHostContext>> {
        let mut linker = Linker::new(&self.engine);

        // host_log(level: i32, ptr: i32, len: i32)
        linker
            .func_wrap(
                "env",
                "host_log",
                |mut caller: Caller<WasmHostContext>,
                 level: i32,
                 ptr: i32,
                 len: i32|
                 -> Result<(), WasmiError> {
                    let memory = match caller.get_export("memory").and_then(|e| e.into_memory()) {
                        Some(m) => m,
                        None => return Err(WasmiError::new("Memory not exported")),
                    };

                    let p = ptr as usize;
                    let l = len as usize;
                    let data = memory.data(&caller);
                    if p + l > data.len() {
                        return Err(WasmiError::new("Buffer overflow reading log string"));
                    }

                    let msg = String::from_utf8_lossy(&data[p..p + l]).to_string();
                    let plugin_id = caller.data().plugin_id.clone();
                    let log_entry = format!("[wasm:{plugin_id}] {msg}");

                    match level {
                        0 => debug!("{}", log_entry),
                        1 => info!("{}", log_entry),
                        2 => warn!("{}", log_entry),
                        _ => error!("{}", log_entry),
                    }

                    caller.data_mut().logs.push(log_entry);
                    Ok(())
                },
            )
            .map_err(|e| NucleusError::Plugin(format!("Failed registering host_log: {e}")))?;

        // host_read_file(path_ptr: i32, path_len: i32, out_ptr: i32, out_cap: i32) -> i32 (bytes written, or -1 on error)
        linker
            .func_wrap(
                "env",
                "host_read_file",
                |mut caller: Caller<WasmHostContext>,
                 path_ptr: i32,
                 path_len: i32,
                 out_ptr: i32,
                 out_cap: i32|
                 -> Result<i32, WasmiError> {
                    let has_perm = caller
                        .data()
                        .permissions
                        .contains(&PluginPermission::InstanceFsRead);
                    if !has_perm {
                        warn!("Wasm plugin tried reading file without InstanceFsRead permission");
                        return Ok(-1);
                    }

                    let memory = match caller.get_export("memory").and_then(|e| e.into_memory()) {
                        Some(m) => m,
                        None => return Err(WasmiError::new("Memory not exported")),
                    };

                    let pp = path_ptr as usize;
                    let pl = path_len as usize;
                    let mem_data = memory.data(&caller);
                    if pp + pl > mem_data.len() {
                        return Err(WasmiError::new("Buffer overflow reading path"));
                    }

                    let path_str = match std::str::from_utf8(&mem_data[pp..pp + pl]) {
                        Ok(s) => s,
                        Err(_) => return Ok(-1),
                    };

                    let base_dir = match &caller.data().instance_dir {
                        Some(d) => d.clone(),
                        None => return Ok(-1),
                    };

                    let target_path = base_dir.join(path_str);
                    if !PermissionStore::is_path_confined(&base_dir, &target_path) {
                        warn!("Wasm plugin path traversal blocked: {path_str}");
                        return Ok(-1);
                    }

                    let content = match fs::read(&target_path) {
                        Ok(c) => c,
                        Err(_) => return Ok(-1),
                    };

                    let op = out_ptr as usize;
                    let oc = out_cap as usize;
                    if content.len() > oc {
                        return Ok(-1);
                    }

                    let out_slice = memory.data_mut(&mut caller);
                    if op + content.len() > out_slice.len() {
                        return Err(WasmiError::new("Buffer overflow writing output file"));
                    }
                    out_slice[op..op + content.len()].copy_from_slice(&content);

                    Ok(content.len() as i32)
                },
            )
            .map_err(|e| NucleusError::Plugin(format!("Failed registering host_read_file: {e}")))?;

        // host_write_file(path_ptr: i32, path_len: i32, data_ptr: i32, data_len: i32) -> i32 (0 on success, -1 on error)
        linker
            .func_wrap(
                "env",
                "host_write_file",
                |caller: Caller<WasmHostContext>,
                 path_ptr: i32,
                 path_len: i32,
                 data_ptr: i32,
                 data_len: i32|
                 -> Result<i32, WasmiError> {
                    let has_perm = caller
                        .data()
                        .permissions
                        .contains(&PluginPermission::InstanceFsWrite);
                    if !has_perm {
                        warn!("Wasm plugin tried writing file without InstanceFsWrite permission");
                        return Ok(-1);
                    }

                    let memory = match caller.get_export("memory").and_then(|e| e.into_memory()) {
                        Some(m) => m,
                        None => return Err(WasmiError::new("Memory not exported")),
                    };

                    let pp = path_ptr as usize;
                    let pl = path_len as usize;
                    let dp = data_ptr as usize;
                    let dl = data_len as usize;
                    let mem_data = memory.data(&caller);
                    if pp + pl > mem_data.len() || dp + dl > mem_data.len() {
                        return Err(WasmiError::new("Buffer overflow reading write arguments"));
                    }

                    let path_str = match std::str::from_utf8(&mem_data[pp..pp + pl]) {
                        Ok(s) => s,
                        Err(_) => return Ok(-1),
                    };

                    let base_dir = match &caller.data().instance_dir {
                        Some(d) => d.clone(),
                        None => return Ok(-1),
                    };

                    let target_path = base_dir.join(path_str);
                    if !PermissionStore::is_path_confined(&base_dir, &target_path) {
                        warn!("Wasm plugin path traversal blocked on write: {path_str}");
                        return Ok(-1);
                    }

                    let payload = mem_data[dp..dp + dl].to_vec();
                    if let Some(parent) = target_path.parent() {
                        let _ = fs::create_dir_all(parent);
                    }
                    if fs::write(&target_path, payload).is_err() {
                        return Ok(-1);
                    }

                    Ok(0)
                },
            )
            .map_err(|e| {
                NucleusError::Plugin(format!("Failed registering host_write_file: {e}"))
            })?;

        Ok(linker)
    }

    /// Executes an exported lifecycle hook function with a fresh Store and assigned fuel.
    pub fn invoke_hook(
        &self,
        func_name: &str,
        arg: i32,
        context: WasmHostContext,
    ) -> NucleusResult<(i32, WasmHostContext)> {
        let linker = self.create_linker()?;
        let mut store = Store::new(&self.engine, context);
        store
            .set_fuel(self.default_fuel)
            .map_err(|e| NucleusError::Plugin(format!("Failed setting fuel: {e}")))?;

        let instance = linker
            .instantiate(&mut store, &self.module)
            .map_err(|e| NucleusError::Plugin(format!("Instantiation error: {e}")))?
            .start(&mut store)
            .map_err(|e| NucleusError::Plugin(format!("Start error: {e}")))?;

        let func: TypedFunc<i32, i32> = match instance.get_typed_func(&store, func_name) {
            Ok(f) => f,
            Err(_) => {
                // Return 0 if the hook is not exported by the plugin
                return Ok((0, store.into_data()));
            }
        };

        let result = func
            .call(&mut store, arg)
            .map_err(|e| NucleusError::Plugin(format!("Execution trap in '{func_name}': {e}")))?;

        Ok((result, store.into_data()))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    // Minimal valid WebAssembly module exporting test_hook(arg: i32) -> i32 returning arg + 42:
    const SAMPLE_WASM: &[u8] = &[
        0x00, 0x61, 0x73, 0x6d, 0x01, 0x00, 0x00, 0x00, // \0asm, version 1
        0x01, 0x06, 0x01, 0x60, 0x01, 0x7f, 0x01, 0x7f, // Type: (i32) -> i32
        0x03, 0x02, 0x01, 0x00, // Function 0 has type 0
        0x07, 0x0d, 0x01, 0x09, 0x74, 0x65, 0x73, 0x74, 0x5f, 0x68, 0x6f, 0x6f, 0x6b, 0x00,
        0x00, // Export "test_hook" -> func 0
        0x0a, 0x09, 0x01, 0x07, 0x00, 0x20, 0x00, 0x41, 0x2a, 0x6a,
        0x0b, // Code: local.get 0, i32.const 42, i32.add, end
    ];

    #[test]
    fn test_wasm_host_execution_and_fuel() {
        let host = WasmPluginHost::new(SAMPLE_WASM, 100_000).unwrap();
        let ctx = WasmHostContext::new("unit-test-wasm".to_string(), HashSet::new(), None);

        let (ret, _final_ctx) = host.invoke_hook("test_hook", 8, ctx).unwrap();
        assert_eq!(ret, 50); // 8 + 42 = 50
    }
}
