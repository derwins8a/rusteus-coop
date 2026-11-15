//! Nucleus Co-op Extensions, Add-ons & Plugins Ecosystem.
//!
//! Provides a sandboxed WebAssembly execution environment (wasmi), pure JavaScript add-on modules,
//! capability-based security permission vaults, `.nucplugin` package verification, and legacy
//! C# `pluginsInfo.txt` migration bridges.

pub mod manifest;
pub mod permissions;
pub mod package;
pub mod wasm_host;
pub mod js_addon;
pub mod legacy_bridge;
pub mod manager;

pub use manifest::{
    LifecycleHookKind, PluginKind, PluginManifest, PluginPermission, PluginRequirement,
};
pub use permissions::{PermissionStore, PluginGrant};
pub use package::{PackageInspection, PluginPackager};
pub use wasm_host::{WasmHostContext, WasmPluginHost};
pub use js_addon::{sort_addons_by_dependencies, JsAddon};
pub use legacy_bridge::{find_and_parse_legacy_plugins, parse_legacy_plugins_txt, BuiltinToolEquivalent, LegacyPluginInfo};
pub use manager::{ExtensionManager, LoadedPlugin, PluginStatus};
