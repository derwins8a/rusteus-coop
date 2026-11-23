//! Security permissions vault, SHA-256 tamper validation, and filesystem confinement.

use crate::manifest::PluginPermission;
use nucleus_common::error::NucleusResult;
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use std::collections::{HashMap, HashSet};
use std::fs;
use std::path::Path;

/// Persisted user capability grants for a specific plugin binary version.
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct PluginGrant {
    pub plugin_id: String,
    pub binary_sha256: String,
    pub granted_permissions: HashSet<PluginPermission>,
    pub granted_at_unix: u64,
}

/// Persistent store managing plugin trust and capability grants.
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct PermissionStore {
    #[serde(default)]
    grants: HashMap<String, PluginGrant>,
}

impl PermissionStore {
    /// Loads the permission store from `<app_data>/plugins/permissions.json` or content directory.
    pub fn load_from_file(path: &Path) -> NucleusResult<Self> {
        if !path.exists() {
            return Ok(Self::default());
        }
        let content = fs::read_to_string(path)?;
        let store = serde_json::from_str(&content)?;
        Ok(store)
    }

    /// Saves the permission store to disk.
    pub fn save_to_file(&self, path: &Path) -> NucleusResult<()> {
        if let Some(parent) = path.parent() {
            let _ = fs::create_dir_all(parent);
        }
        let content = serde_json::to_string_pretty(self)?;
        fs::write(path, content)?;
        Ok(())
    }

    /// Computes the SHA-256 hex digest of a binary file.
    pub fn compute_sha256(path: &Path) -> NucleusResult<String> {
        let bytes = fs::read(path)?;
        let mut hasher = Sha256::new();
        hasher.update(&bytes);
        Ok(hex::encode(hasher.finalize()))
    }

    /// Grants a set of permissions to a plugin pinned to its current binary SHA-256 hash.
    pub fn grant_permissions(
        &mut self,
        plugin_id: &str,
        binary_sha256: &str,
        permissions: &[PluginPermission],
    ) {
        let grant = PluginGrant {
            plugin_id: plugin_id.to_string(),
            binary_sha256: binary_sha256.to_string(),
            granted_permissions: permissions.iter().copied().collect(),
            granted_at_unix: std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap_or_default()
                .as_secs(),
        };
        self.grants.insert(plugin_id.to_string(), grant);
    }

    /// Revokes all granted permissions for a plugin.
    pub fn revoke_plugin(&mut self, plugin_id: &str) {
        self.grants.remove(plugin_id);
    }

    /// Checks if a plugin is authorized for a specific permission, verifying its binary hash has NOT changed.
    pub fn is_authorized(
        &self,
        plugin_id: &str,
        current_binary_sha256: &str,
        required_permission: PluginPermission,
    ) -> bool {
        if let Some(grant) = self.grants.get(plugin_id) {
            // Anti-tamper check: if on-disk hash does not match granted hash, authorization fails immediately
            if grant.binary_sha256 == current_binary_sha256 {
                return grant.granted_permissions.contains(&required_permission);
            }
        }
        false
    }

    /// Returns ungranted permissions for a plugin given its required capabilities.
    pub fn missing_permissions(
        &self,
        plugin_id: &str,
        current_binary_sha256: &str,
        required_permissions: &[PluginPermission],
    ) -> Vec<PluginPermission> {
        required_permissions
            .iter()
            .copied()
            .filter(|&p| !self.is_authorized(plugin_id, current_binary_sha256, p))
            .collect()
    }

    /// Checks if all required capabilities have been granted for a plugin binary.
    pub fn check_all_granted(
        &self,
        plugin_id: &str,
        required_permissions: &[PluginPermission],
        current_binary_sha256: &str,
    ) -> bool {
        self.missing_permissions(plugin_id, current_binary_sha256, required_permissions)
            .is_empty()
    }

    /// Verifies that a target path is strictly confined within the allowed root directory,
    /// preventing directory traversal escapes (`..` or junction redirects).
    pub fn is_path_confined(root: &Path, target: &Path) -> bool {
        let clean_root = root.canonicalize().unwrap_or_else(|_| root.to_path_buf());
        let clean_target = if target.exists() {
            target
                .canonicalize()
                .unwrap_or_else(|_| target.to_path_buf())
        } else {
            // For not-yet-created target files, check parent canonicalization
            if let Some(parent) = target.parent() {
                if let Ok(clean_parent) = parent.canonicalize() {
                    clean_parent.join(target.file_name().unwrap_or_default())
                } else {
                    target.to_path_buf()
                }
            } else {
                target.to_path_buf()
            }
        };

        clean_target.starts_with(&clean_root)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_permission_tamper_detection() {
        let mut store = PermissionStore::default();
        let plugin = "com.example.mod";
        let hash_v1 = "abcdef1234567890";
        let hash_tampered = "1111111111111111";

        store.grant_permissions(plugin, hash_v1, &[PluginPermission::InstanceFsWrite]);

        assert!(store.is_authorized(plugin, hash_v1, PluginPermission::InstanceFsWrite));
        // Tampered binary fails check
        assert!(!store.is_authorized(plugin, hash_tampered, PluginPermission::InstanceFsWrite));
    }

    #[test]
    fn test_path_confinement() {
        let root = std::env::temp_dir().join(format!("nucleus_conf_test_{}", std::process::id()));
        let _ = fs::create_dir_all(&root);

        let inside = root.join("save.dat");
        let outside = std::env::temp_dir().join("outside.dat");

        assert!(PermissionStore::is_path_confined(&root, &inside));
        assert!(!PermissionStore::is_path_confined(&root, &outside));

        let _ = fs::remove_dir_all(&root);
    }
}
