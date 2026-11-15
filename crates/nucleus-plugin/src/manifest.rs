//! Plugin manifest definition, permissions, and SemVer constraint matching.

use nucleus_common::error::{NucleusError, NucleusResult};
use semver::{Version, VersionReq};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Type or runtime classification of an extension/plugin.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum PluginKind {
    /// Sandboxed WebAssembly binary module executed via wasmi.
    Wasm,
    /// Pure JavaScript add-on module providing helper functions or hooks.
    JsAddon,
    /// UI visual theme or asset pack.
    ThemePack,
    /// Legacy C# .NET assembly declared via pluginsInfo.txt.
    DotnetLegacy,
}

/// Explicit security capabilities that plugins must declare and users must approve.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum PluginPermission {
    /// Read access to files inside the assigned instance folder.
    InstanceFsRead,
    /// Write access to files inside the assigned instance folder.
    InstanceFsWrite,
    /// Read and write access inside the player's sandboxed environment (%USERPROFILE%\NucleusCoop\<Player>).
    EnvironmentFs,
    /// Read and write operations to Windows registry keys.
    RegistryOps,
    /// Spawning whitelisted child processes or tool utilities.
    ProcessSpawnScoped,
    /// Manipulating game window bounds, styles, or focus.
    WindowControl,
    /// Exposing functions into the handler JavaScript runtime under `Plugins.<namespace>`.
    ScriptApiExport,
    /// Intercepting multi-instance lifecycle events (staging, pre-launch, post-window, teardown).
    LifecycleHooks,
    /// Making outbound network HTTP requests.
    NetworkOutbound,
    /// Unrestricted native access (only for trusted legacy C# binaries).
    UnrestrictedNative,
}

impl PluginPermission {
    /// Human-readable title for the permission prompt.
    pub fn display_title(self) -> &'static str {
        match self {
            Self::InstanceFsRead => "Read Instance Files",
            Self::InstanceFsWrite => "Modify Instance Files",
            Self::EnvironmentFs => "Access Player User Environment",
            Self::RegistryOps => "Modify Windows Registry",
            Self::ProcessSpawnScoped => "Spawn Process Tools",
            Self::WindowControl => "Control Game Windows",
            Self::ScriptApiExport => "Export Script APIs",
            Self::LifecycleHooks => "Hook Multi-Instance Lifecycle",
            Self::NetworkOutbound => "Outbound Network Requests",
            Self::UnrestrictedNative => "Unrestricted Native Execution (DANGEROUS)",
        }
    }
}

/// Multi-instance lifecycle stages where plugins can execute hooks.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum LifecycleHookKind {
    PrePrepare,
    PostStage,
    PreLaunch,
    PostWindow,
    OnTick,
    OnStop,
}

impl LifecycleHookKind {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::PrePrepare => "pre_prepare",
            Self::PostStage => "post_stage",
            Self::PreLaunch => "pre_launch",
            Self::PostWindow => "post_window",
            Self::OnTick => "on_tick",
            Self::OnStop => "on_stop",
        }
    }
}

/// Declared dependency requirement on another plugin.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct PluginRequirement {
    pub id: String,
    pub version_req: VersionReq,
}

/// Standard plugin manifest parsed from `plugin.toml`.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PluginManifest {
    /// Unique reverse-DNS identifier (e.g. "com.community.unity-save-helper").
    pub id: String,
    /// Safe identifier exposed in JavaScript under `Plugins.<js_namespace>`.
    pub js_namespace: String,
    /// Human-readable display name.
    pub name: String,
    /// Semantic version of this plugin release.
    pub version: Version,
    /// Minimum required Nucleus Co-op engine version.
    #[serde(default)]
    pub min_nucleus_version: Option<Version>,
    /// Extension category / runtime kind.
    pub kind: PluginKind,
    /// Relative path to primary entrypoint file (e.g. "plugin.wasm" or "index.js").
    pub entrypoint: String,
    /// Required capability permissions.
    #[serde(default)]
    pub permissions: Vec<PluginPermission>,
    /// Lifecycle hooks subscribed to by this plugin.
    #[serde(default)]
    pub hooks: Vec<LifecycleHookKind>,
    /// Dependencies on other plugins.
    #[serde(default)]
    pub dependencies: HashMap<String, VersionReq>,
}

impl PluginManifest {
    /// Parses and validates a `plugin.toml` string.
    pub fn parse_toml(toml_str: &str) -> NucleusResult<Self> {
        let manifest: Self = toml::from_str(toml_str)
            .map_err(|e| NucleusError::Config(format!("Failed to parse plugin.toml: {}", e)))?;

        // Validate js_namespace is a safe ECMAScript identifier
        if manifest.js_namespace.is_empty()
            || !manifest
                .js_namespace
                .chars()
                .next()
                .unwrap_or('_')
                .is_ascii_alphabetic()
                && manifest.js_namespace.chars().next().unwrap_or('_') != '_'
            || !manifest
                .js_namespace
                .chars()
                .all(|c| c.is_ascii_alphanumeric() || c == '_')
        {
            return Err(NucleusError::Config(format!(
                "Invalid JavaScript namespace '{}' in plugin manifest: must be a valid identifier",
                manifest.js_namespace
            )));
        }

        // Validate entrypoint does not contain directory traversal
        for comp in std::path::Path::new(&manifest.entrypoint).components() {
            if let std::path::Component::ParentDir = comp {
                return Err(NucleusError::Config(format!(
                    "Plugin entrypoint '{}' contains illegal parent directory traversal",
                    manifest.entrypoint
                )));
            }
        }

        Ok(manifest)
    }

    /// Checks if this plugin has registered for the specified lifecycle hook.
    pub fn has_lifecycle_hook(&self, hook: LifecycleHookKind) -> bool {
        self.hooks.contains(&hook)
    }

    /// Checks if this plugin is compatible with the given application version.
    pub fn is_compatible(&self, app_version: &Version) -> bool {
        if let Some(ref min_ver) = self.min_nucleus_version {
            app_version >= min_ver
        } else {
            true
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_valid_plugin_manifest_parsing() {
        let toml_data = r#"
            id = "com.example.test-plugin"
            js_namespace = "testHelper"
            name = "Test Helper Plugin"
            version = "1.2.3"
            kind = "wasm"
            entrypoint = "bin/plugin.wasm"
            permissions = ["instance_fs_read", "instance_fs_write"]
            hooks = ["post_stage", "on_stop"]

            [dependencies]
            "com.example.core-lib" = "^1.0.0"
        "#;

        let manifest = PluginManifest::parse_toml(toml_data).unwrap();
        assert_eq!(manifest.id, "com.example.test-plugin");
        assert_eq!(manifest.js_namespace, "testHelper");
        assert_eq!(manifest.version, Version::parse("1.2.3").unwrap());
        assert_eq!(manifest.kind, PluginKind::Wasm);
        assert_eq!(manifest.entrypoint, "bin/plugin.wasm");
        assert_eq!(manifest.permissions.len(), 2);
        assert_eq!(manifest.hooks.len(), 2);
        assert!(manifest.dependencies.contains_key("com.example.core-lib"));
    }

    #[test]
    fn test_invalid_namespace_rejected() {
        let toml_data = r#"
            id = "com.example.bad"
            js_namespace = "bad-namespace!"
            name = "Bad"
            version = "1.0.0"
            kind = "wasm"
            entrypoint = "plugin.wasm"
        "#;
        assert!(PluginManifest::parse_toml(toml_data).is_err());
    }
}
