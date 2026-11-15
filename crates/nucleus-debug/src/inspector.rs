//! Variable diffing, cycle-safe object formatting, and runtime state inspection.

use std::collections::HashMap;
use serde::{Deserialize, Serialize};
use crate::protocol::InspectValue;

/// Classification of changes between two debugger snapshots.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum DiffKind {
    Added,
    Modified { old_value: String },
    Removed,
    Unchanged,
}

/// A detected difference in an inspected variable.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct VariableDiff {
    pub name: String,
    pub diff_kind: DiffKind,
    pub current_value: Option<InspectValue>,
}

/// Helper utility for inspecting variables and computing state diffs.
pub struct StateInspector;

impl StateInspector {
    /// Recursively converts a serde_json::Value into an InspectValue with bounded depth.
    pub fn format_json_value(name: &str, val: &serde_json::Value, max_depth: usize) -> InspectValue {
        if max_depth == 0 {
            return InspectValue {
                name: name.to_string(),
                type_name: "object".to_string(),
                summary: "{...}".to_string(),
                children: Vec::new(),
            };
        }

        match val {
            serde_json::Value::Null => InspectValue {
                name: name.to_string(),
                type_name: "null".to_string(),
                summary: "null".to_string(),
                children: Vec::new(),
            },
            serde_json::Value::Bool(b) => InspectValue {
                name: name.to_string(),
                type_name: "boolean".to_string(),
                summary: b.to_string(),
                children: Vec::new(),
            },
            serde_json::Value::Number(n) => InspectValue {
                name: name.to_string(),
                type_name: "number".to_string(),
                summary: n.to_string(),
                children: Vec::new(),
            },
            serde_json::Value::String(s) => InspectValue {
                name: name.to_string(),
                type_name: "string".to_string(),
                summary: format!("\"{s}\""),
                children: Vec::new(),
            },
            serde_json::Value::Array(arr) => {
                let summary = format!("Array({})", arr.len());
                let children = arr
                    .iter()
                    .enumerate()
                    .take(50) // limit array expansion to first 50 items
                    .map(|(idx, item)| Self::format_json_value(&format!("[{idx}]"), item, max_depth - 1))
                    .collect();

                InspectValue {
                    name: name.to_string(),
                    type_name: "array".to_string(),
                    summary,
                    children,
                }
            }
            serde_json::Value::Object(map) => {
                let summary = format!("Object {{ {} keys }}", map.len());
                let children = map
                    .iter()
                    .take(50) // limit object property expansion to first 50 keys
                    .map(|(k, v)| Self::format_json_value(k, v, max_depth - 1))
                    .collect();

                InspectValue {
                    name: name.to_string(),
                    type_name: "object".to_string(),
                    summary,
                    children,
                }
            }
        }
    }

    /// Compares two sets of variables, identifying added, modified, and removed entries.
    pub fn compute_variable_diff(
        before: &[InspectValue],
        after: &[InspectValue],
    ) -> Vec<VariableDiff> {
        let before_map: HashMap<&str, &InspectValue> =
            before.iter().map(|v| (v.name.as_str(), v)).collect();
        let after_map: HashMap<&str, &InspectValue> =
            after.iter().map(|v| (v.name.as_str(), v)).collect();

        let mut diffs = Vec::new();

        // Check for added or modified variables
        for after_val in after {
            let name = &after_val.name;
            if let Some(before_val) = before_map.get(name.as_str()) {
                if before_val.summary != after_val.summary || before_val.type_name != after_val.type_name {
                    diffs.push(VariableDiff {
                        name: name.clone(),
                        diff_kind: DiffKind::Modified {
                            old_value: before_val.summary.clone(),
                        },
                        current_value: Some(after_val.clone()),
                    });
                } else {
                    diffs.push(VariableDiff {
                        name: name.clone(),
                        diff_kind: DiffKind::Unchanged,
                        current_value: Some(after_val.clone()),
                    });
                }
            } else {
                diffs.push(VariableDiff {
                    name: name.clone(),
                    diff_kind: DiffKind::Added,
                    current_value: Some(after_val.clone()),
                });
            }
        }

        // Check for removed variables
        for before_val in before {
            if !after_map.contains_key(before_val.name.as_str()) {
                diffs.push(VariableDiff {
                    name: before_val.name.clone(),
                    diff_kind: DiffKind::Removed,
                    current_value: None,
                });
            }
        }

        diffs
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_format_json_value_and_cycle_bounds() {
        let json_data = serde_json::json!({
            "ExecutableName": "l4d2.exe",
            "Players": [1, 2, 3],
            "Nested": {
                "Deep": "Value"
            }
        });

        let val = StateInspector::format_json_value("Context", &json_data, 3);
        assert_eq!(val.name, "Context");
        assert_eq!(val.type_name, "object");
        assert_eq!(val.children.len(), 3);
    }

    #[test]
    fn test_compute_variable_diff() {
        let v1 = vec![
            InspectValue {
                name: "counter".to_string(),
                type_name: "number".to_string(),
                summary: "1".to_string(),
                children: Vec::new(),
            },
            InspectValue {
                name: "to_delete".to_string(),
                type_name: "string".to_string(),
                summary: "\"old\"".to_string(),
                children: Vec::new(),
            },
        ];

        let v2 = vec![
            InspectValue {
                name: "counter".to_string(),
                type_name: "number".to_string(),
                summary: "2".to_string(),
                children: Vec::new(),
            },
            InspectValue {
                name: "new_var".to_string(),
                type_name: "boolean".to_string(),
                summary: "true".to_string(),
                children: Vec::new(),
            },
        ];

        let diff = StateInspector::compute_variable_diff(&v1, &v2);

        let counter_diff = diff.iter().find(|d| d.name == "counter").unwrap();
        assert_eq!(
            counter_diff.diff_kind,
            DiffKind::Modified {
                old_value: "1".to_string()
            }
        );

        let new_diff = diff.iter().find(|d| d.name == "new_var").unwrap();
        assert_eq!(new_diff.diff_kind, DiffKind::Added);

        let del_diff = diff.iter().find(|d| d.name == "to_delete").unwrap();
        assert_eq!(del_diff.diff_kind, DiffKind::Removed);
    }
}
