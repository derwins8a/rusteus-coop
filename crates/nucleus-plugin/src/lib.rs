//! Nucleus Co-op Extensions, Add-ons & Plugins Ecosystem.
//!
//! Provides a sandboxed WebAssembly execution environment (wasmi), pure JavaScript add-on modules,
//! capability-based security permission vaults, `.nucplugin` package verification, and legacy
//! C# `pluginsInfo.txt` migration bridges.

pub mod js_addon;
pub mod legacy_bridge;
pub mod manager;
pub mod manifest;
pub mod package;
pub mod permissions;
pub mod wasm_host;

pub use js_addon::{JsAddon, sort_addons_by_dependencies};
pub use legacy_bridge::{
    BuiltinToolEquivalent, LegacyPluginInfo, find_and_parse_legacy_plugins,
    parse_legacy_plugins_txt,
};
pub use manager::{ExtensionManager, LoadedPlugin, PluginStatus};
pub use manifest::{
    LifecycleHookKind, PluginKind, PluginManifest, PluginPermission, PluginRequirement,
};
pub use package::{PackageInspection, PluginPackager};
pub use permissions::{PermissionStore, PluginGrant};
pub use wasm_host::{WasmHostContext, WasmPluginHost};
