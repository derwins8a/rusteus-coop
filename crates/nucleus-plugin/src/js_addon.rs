//! Pure JavaScript add-on modules, syntax validation, and dependency ordering.

use std::collections::{HashMap, HashSet};
use std::fs;
use std::path::{Path, PathBuf};
use boa_parser::{Parser, Source};
use nucleus_common::error::{NucleusError, NucleusResult};
use crate::manifest::PluginManifest;

/// An installed pure JavaScript add-on module.
#[derive(Debug, Clone)]
pub struct JsAddon {
    pub manifest: PluginManifest,
    pub root_dir: PathBuf,
    pub entrypoint_path: PathBuf,
    pub source_code: String,
}

impl JsAddon {
    /// Loads and validates a JS add-on module from its installation directory.
    pub fn load_from_dir(dir: &Path) -> NucleusResult<Self> {
        let manifest_path = dir.join("plugin.toml");
        let alt_manifest_path = dir.join("addon.toml");
        let final_manifest_path = if manifest_path.exists() {
            manifest_path
        } else if alt_manifest_path.exists() {
            alt_manifest_path
        } else {
            return Err(NucleusError::Plugin(format!(
                "No manifest found in {}",
                dir.display()
            )));
        };

        let manifest_content = fs::read_to_string(&final_manifest_path)?;
        let manifest = PluginManifest::parse_toml(&manifest_content)?;

        let entry_name = &manifest.entrypoint;
        let entrypoint_path = dir.join(entry_name);
        if !entrypoint_path.exists() {
            return Err(NucleusError::Plugin(format!(
                "JS addon entrypoint '{}' does not exist in {}",
                entry_name,
                dir.display()
            )));
        }

        let source_code = fs::read_to_string(&entrypoint_path)?;

        // Validate JavaScript syntax using boa_parser
        Self::validate_syntax(&source_code, entry_name)?;

        Ok(Self {
            manifest,
            root_dir: dir.to_path_buf(),
            entrypoint_path,
            source_code,
        })
    }

    /// Validates ECMAScript syntax of source code via boa_parser.
    pub fn validate_syntax(source_code: &str, file_name: &str) -> NucleusResult<()> {
        let mut interner = boa_interner::Interner::default();
        let source = Source::from_bytes(source_code.as_bytes());
        let mut parser = Parser::new(source);
        let scope = boa_ast::scope::Scope::new_global();

        parser
            .parse_script(&scope, &mut interner)
            .map_err(|e| NucleusError::Plugin(format!("Syntax error in JS addon '{file_name}': {e}")))?;

        Ok(())
    }

    /// Wraps the JS source code in a self-executing scoped closure exposing the addon's namespace.
    pub fn build_injectable_script(&self) -> String {
        let ns = &self.manifest.js_namespace;
        format!(
            "/* Nucleus Addon: {} v{} */\n(function() {{\n  var __addon_ns = '{}';\n  if (!globalThis.Plugins) globalThis.Plugins = {{}};\n  var exports = {{}};\n  globalThis.Plugins[__addon_ns] = exports;\n  (function(exports, Plugins) {{\n{}\n  }})(exports, globalThis.Plugins);\n}})();\n",
            self.manifest.name,
            self.manifest.version,
            ns,
            self.source_code
        )
    }
}

/// Resolves dependency ordering (topological sort) for a collection of JS addons.
pub fn sort_addons_by_dependencies(addons: &[JsAddon]) -> NucleusResult<Vec<JsAddon>> {
    let mut by_id: HashMap<String, &JsAddon> = HashMap::new();
    for addon in addons {
        by_id.insert(addon.manifest.id.clone(), addon);
    }

    let mut visited: HashSet<String> = HashSet::new();
    let mut visiting: HashSet<String> = HashSet::new();
    let mut sorted: Vec<JsAddon> = Vec::new();

    fn visit<'a>(
        addon_id: &str,
        by_id: &HashMap<String, &'a JsAddon>,
        visited: &mut HashSet<String>,
        visiting: &mut HashSet<String>,
        sorted: &mut Vec<JsAddon>,
    ) -> NucleusResult<()> {
        if visiting.contains(addon_id) {
            return Err(NucleusError::Plugin(format!(
                "Cyclic dependency detected involving addon '{addon_id}'"
            )));
        }
        if !visited.contains(addon_id) {
            visiting.insert(addon_id.to_string());
            if let Some(addon) = by_id.get(addon_id) {
                for (dep_id, _) in &addon.manifest.dependencies {
                    if by_id.contains_key(dep_id) {
                        visit(dep_id, by_id, visited, visiting, sorted)?;
                    }
                }
                visited.insert(addon_id.to_string());
                visiting.remove(addon_id);
                sorted.push((*addon).clone());
            }
        }
        Ok(())
    }

    for addon in addons {
        if !visited.contains(&addon.manifest.id) {
            visit(&addon.manifest.id, &by_id, &mut visited, &mut visiting, &mut sorted)?;
        }
    }

    Ok(sorted)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_js_addon_syntax_validation() {
        let valid_js = "function add(a, b) { return a + b; } exports.add = add;";
        assert!(JsAddon::validate_syntax(valid_js, "test.js").is_ok());

        let invalid_js = "function broken( { return ;";
        assert!(JsAddon::validate_syntax(invalid_js, "bad.js").is_err());
    }

    #[test]
    fn test_topological_sort_addons() {
        let make_addon = |id: &str, deps: &[(&str, &str)]| -> JsAddon {
            let dep_lines = deps
                .iter()
                .map(|(k, v)| format!("\"{}\" = \"{}\"", k, v))
                .collect::<Vec<_>>()
                .join("\n");
            let toml = format!(
                r#"
id = "{id}"
name = "{id}"
js_namespace = "{id}"
version = "1.0.0"
kind = "js-addon"
entrypoint = "main.js"

[dependencies]
{dep_lines}
"#
            );
            let manifest = PluginManifest::parse_toml(&toml).unwrap();
            JsAddon {
                manifest,
                root_dir: PathBuf::new(),
                entrypoint_path: PathBuf::new(),
                source_code: "exports.val = 1;".to_string(),
            }
        };

        let addon_c = make_addon("addon_c", &[("addon_b", "^1.0.0")]);
        let addon_b = make_addon("addon_b", &[("addon_a", "^1.0.0")]);
        let addon_a = make_addon("addon_a", &[]);

        let raw = vec![addon_c, addon_a, addon_b];
        let sorted = sort_addons_by_dependencies(&raw).unwrap();

        let order = sorted.iter().map(|a| a.manifest.id.as_str()).collect::<Vec<_>>();
        assert_eq!(order, vec!["addon_a", "addon_b", "addon_c"]);
    }
}
