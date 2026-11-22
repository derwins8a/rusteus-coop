//! Central Extension and Plugin Manager orchestrating discovery, capability verification, and lifecycle hooks.

use semver::Version;
use std::collections::HashMap;
use std::fs;
use std::path::{Path, PathBuf};
use tracing::{error, info, warn};

use crate::js_addon::{JsAddon, sort_addons_by_dependencies};
use crate::legacy_bridge::find_and_parse_legacy_plugins;
use crate::manifest::{LifecycleHookKind, PluginKind, PluginManifest, PluginPermission};
use crate::package::PluginPackager;
use crate::permissions::PermissionStore;
use crate::wasm_host::{WasmHostContext, WasmPluginHost};
use nucleus_common::error::NucleusResult;

/// Current runtime health and activation state of an extension.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum PluginStatus {
    /// Fully verified, capabilities granted, and active.
    Active,
    /// Disabled explicitly by user configuration.
    Disabled,
    /// Requires additional capability grants before execution.
    PendingPermissions,
    /// Incompatible with current host application version.
    IncompatibleVersion(String),
    /// Encountered runtime or initialization fault.
    Failed(String),
}

/// A loaded extension representation.
pub struct LoadedPlugin {
    pub manifest: PluginManifest,
    pub status: PluginStatus,
    pub root_dir: PathBuf,
    pub wasm_host: Option<WasmPluginHost>,
    pub js_addon: Option<JsAddon>,
}

/// Central manager orchestrating plugins, addons, and sandboxed runtimes.
pub struct ExtensionManager {
    plugins_dir: PathBuf,
    addons_dir: PathBuf,
    permissions_path: PathBuf,
    permission_store: PermissionStore,
    loaded_plugins: HashMap<String, LoadedPlugin>,
    app_version: Version,
}

impl ExtensionManager {
    /// Initializes an ExtensionManager with designated directory roots.
    pub fn new(
        plugins_dir: PathBuf,
        addons_dir: PathBuf,
        permissions_path: PathBuf,
        app_version: Version,
    ) -> Self {
        let permission_store =
            PermissionStore::load_from_file(&permissions_path).unwrap_or_default();

        Self {
            plugins_dir,
            addons_dir,
            permissions_path,
            permission_store,
            loaded_plugins: HashMap::new(),
            app_version,
        }
    }

    /// Returns a reference to all loaded plugins.
    pub fn loaded_plugins(&self) -> &HashMap<String, LoadedPlugin> {
        &self.loaded_plugins
    }

    /// Returns a mutable reference to the permission store.
    pub fn permission_store_mut(&mut self) -> &mut PermissionStore {
        &mut self.permission_store
    }

    /// Saves any changes made to the permission store.
    pub fn save_permissions(&self) -> NucleusResult<()> {
        self.permission_store.save_to_file(&self.permissions_path)
    }

    /// Enables or disables an installed extension.
    pub fn set_extension_enabled(&mut self, plugin_id: &str, enabled: bool) {
        if let Some(plugin) = self.loaded_plugins.get_mut(plugin_id) {
            plugin.status = if enabled {
                PluginStatus::Active
            } else {
                PluginStatus::Disabled
            };
        }
    }

    /// Grants a permission to an installed extension and persists the permission store.
    pub fn grant_permission(
        &mut self,
        plugin_id: &str,
        perm: PluginPermission,
    ) -> NucleusResult<()> {
        let binary_sha = if let Some(plugin) = self.loaded_plugins.get(plugin_id) {
            let wasm_file = plugin.root_dir.join(&plugin.manifest.entrypoint);
            if wasm_file.exists() {
                PermissionStore::compute_sha256(&wasm_file).unwrap_or_default()
            } else {
                String::new()
            }
        } else {
            String::new()
        };
        self.permission_store
            .grant_permissions(plugin_id, &binary_sha, &[perm]);
        self.save_permissions()
    }

    /// Installs a `.nucplugin` or `.nucaddon` archive file into the appropriate directory.
    pub fn install_archive(&mut self, archive_path: &Path) -> NucleusResult<String> {
        let inspection = PluginPackager::inspect_package(archive_path)?;
        let plugin_id = inspection.manifest.id.clone();

        let target_dir = match inspection.manifest.kind {
            PluginKind::Wasm | PluginKind::DotnetLegacy => self.plugins_dir.join(&plugin_id),
            PluginKind::JsAddon | PluginKind::ThemePack => self.addons_dir.join(&plugin_id),
        };

        if target_dir.exists() {
            let _ = fs::remove_dir_all(&target_dir);
        }

        PluginPackager::extract_package(archive_path, &target_dir)?;
        info!(
            "Successfully installed extension '{}' to {}",
            plugin_id,
            target_dir.display()
        );

        self.scan_and_load_dir(&target_dir)?;
        Ok(plugin_id)
    }

    /// Scans both plugins and addons directories, discovering and activating extensions.
    pub fn scan_all(&mut self) -> NucleusResult<()> {
        let mut dirs_to_scan = vec![self.plugins_dir.clone(), self.addons_dir.clone()];

        // Also check local workspace/portable fallback roots if they exist
        if let Some(globals) = nucleus_common::Globals::try_get() {
            let inst_addons = globals.install_root.join("addons");
            if inst_addons.exists() && !dirs_to_scan.contains(&inst_addons) {
                dirs_to_scan.push(inst_addons);
            }
            let inst_plugins = globals.install_root.join("plugins");
            if inst_plugins.exists() && !dirs_to_scan.contains(&inst_plugins) {
                dirs_to_scan.push(inst_plugins);
            }
        }
        for candidate in &[
            PathBuf::from("addons"),
            PathBuf::from("plugins"),
            PathBuf::from("nucleus-coop").join("addons"),
            PathBuf::from("nucleus-coop").join("plugins"),
            PathBuf::from("content").join("addons"),
            PathBuf::from("content").join("plugins"),
        ] {
            if candidate.exists() && !dirs_to_scan.contains(candidate) {
                dirs_to_scan.push(candidate.clone());
            }
        }

        for scan_root in dirs_to_scan {
            if !scan_root.exists() {
                let _ = fs::create_dir_all(&scan_root);
                continue;
            }

            let entries = match fs::read_dir(&scan_root) {
                Ok(e) => e,
                Err(err) => {
                    warn!("Failed reading directory {}: {}", scan_root.display(), err);
                    continue;
                }
            };

            for entry in entries.flatten() {
                let path = entry.path();
                if path.is_dir() {
                    let _ = self.scan_and_load_dir(&path);
                }
            }
        }

        Ok(())
    }

    /// Scans an individual plugin/addon folder and registers it.
    pub fn scan_and_load_dir(&mut self, dir: &Path) -> NucleusResult<()> {
        let manifest_path = if dir.join("plugin.toml").exists() {
            dir.join("plugin.toml")
        } else if dir.join("addon.toml").exists() {
            dir.join("addon.toml")
        } else {
            // Check for legacy pluginsInfo.txt in root
            let legacy_entries = find_and_parse_legacy_plugins(dir)?;
            if !legacy_entries.is_empty() {
                info!(
                    "Discovered {} legacy plugin entries in {}",
                    legacy_entries.len(),
                    dir.display()
                );
            }
            return Ok(());
        };

        let manifest_content = fs::read_to_string(&manifest_path)?;
        let manifest = PluginManifest::parse_toml(&manifest_content)?;

        // Verify API compatibility
        if !manifest.is_compatible(&self.app_version) {
            let req_str = manifest
                .min_nucleus_version
                .as_ref()
                .map(|v| v.to_string())
                .unwrap_or_default();
            warn!(
                "Plugin '{}' requires API version {} which does not match host {}",
                manifest.id, req_str, self.app_version
            );
            self.loaded_plugins.insert(
                manifest.id.clone(),
                LoadedPlugin {
                    manifest: manifest.clone(),
                    status: PluginStatus::IncompatibleVersion(req_str),
                    root_dir: dir.to_path_buf(),
                    wasm_host: None,
                    js_addon: None,
                },
            );
            return Ok(());
        }

        match manifest.kind {
            PluginKind::Wasm => {
                let wasm_file = dir.join(&manifest.entrypoint);
                if !wasm_file.exists() {
                    let err = format!("Wasm binary '{}' not found", manifest.entrypoint);
                    self.loaded_plugins.insert(
                        manifest.id.clone(),
                        LoadedPlugin {
                            manifest: manifest.clone(),
                            status: PluginStatus::Failed(err),
                            root_dir: dir.to_path_buf(),
                            wasm_host: None,
                            js_addon: None,
                        },
                    );
                    return Ok(());
                }

                let wasm_bytes = fs::read(&wasm_file)?;
                let file_hash = PermissionStore::compute_sha256(&wasm_file)?;

                // Check permissions
                let perms_ok = self.permission_store.check_all_granted(
                    &manifest.id,
                    &manifest.permissions,
                    &file_hash,
                );

                let status = if perms_ok {
                    PluginStatus::Active
                } else {
                    PluginStatus::PendingPermissions
                };

                let wasm_host = match WasmPluginHost::new(&wasm_bytes, 50_000_000) {
                    Ok(host) => Some(host),
                    Err(e) => {
                        error!("Failed initializing wasm host for '{}': {e}", manifest.id);
                        None
                    }
                };

                self.loaded_plugins.insert(
                    manifest.id.clone(),
                    LoadedPlugin {
                        manifest,
                        status,
                        root_dir: dir.to_path_buf(),
                        wasm_host,
                        js_addon: None,
                    },
                );
            }
            PluginKind::JsAddon => {
                let js_addon = match JsAddon::load_from_dir(dir) {
                    Ok(addon) => Some(addon),
                    Err(e) => {
                        warn!("Failed loading JS addon from {}: {e}", dir.display());
                        None
                    }
                };

                let status = if js_addon.is_some() {
                    PluginStatus::Active
                } else {
                    PluginStatus::Failed("JS addon parsing or entrypoint failure".to_string())
                };

                self.loaded_plugins.insert(
                    manifest.id.clone(),
                    LoadedPlugin {
                        manifest,
                        status,
                        root_dir: dir.to_path_buf(),
                        wasm_host: None,
                        js_addon,
                    },
                );
            }
            PluginKind::ThemePack | PluginKind::DotnetLegacy => {
                self.loaded_plugins.insert(
                    manifest.id.clone(),
                    LoadedPlugin {
                        manifest,
                        status: PluginStatus::Active,
                        root_dir: dir.to_path_buf(),
                        wasm_host: None,
                        js_addon: None,
                    },
                );
            }
        }

        Ok(())
    }

    /// Returns bundled and ordered JavaScript source code for all active JS addons.
    pub fn get_combined_js_addons_code(&self) -> String {
        let mut active_addons = Vec::new();
        for plugin in self.loaded_plugins.values() {
            if plugin.status == PluginStatus::Active {
                if let Some(ref addon) = plugin.js_addon {
                    active_addons.push(addon.clone());
                }
            }
        }

        let sorted = sort_addons_by_dependencies(&active_addons).unwrap_or(active_addons);
        let mut combined = String::new();
        for addon in sorted {
            combined.push_str(&addon.build_injectable_script());
            combined.push('\n');
        }

        combined
    }

    /// Dispatches a multi-instance lifecycle hook across all active Wasm plugins.
    pub fn dispatch_lifecycle_hook(
        &self,
        hook_kind: LifecycleHookKind,
        instance_dir: Option<&Path>,
        arg: i32,
    ) -> Vec<(String, NucleusResult<i32>)> {
        let mut results = Vec::new();
        let hook_str = hook_kind.as_str();

        for (plugin_id, plugin) in &self.loaded_plugins {
            if plugin.status != PluginStatus::Active {
                continue;
            }

            if !plugin.manifest.has_lifecycle_hook(hook_kind) {
                continue;
            }

            if let Some(ref wasm_host) = plugin.wasm_host {
                let context = WasmHostContext::new(
                    plugin_id.clone(),
                    plugin.manifest.permissions.clone(),
                    instance_dir.map(|p| p.to_path_buf()),
                );

                let res = wasm_host
                    .invoke_hook(hook_str, arg, context)
                    .map(|(r, _)| r);
                results.push((plugin_id.clone(), res));
            }
        }

        results
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_extension_manager_creation() {
        let temp_dir = std::env::temp_dir().join(format!("nucleus_ext_mgr_{}", std::process::id()));
        let plugins_dir = temp_dir.join("plugins");
        let addons_dir = temp_dir.join("addons");
        let perms_file = temp_dir.join("permissions.json");

        let manager =
            ExtensionManager::new(plugins_dir, addons_dir, perms_file, Version::new(0, 1, 0));

        assert_eq!(manager.loaded_plugins().len(), 0);
        let _ = fs::remove_dir_all(temp_dir);
    }
}
