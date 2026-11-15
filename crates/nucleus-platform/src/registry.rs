//! Platform-agnostic registry operations.

use nucleus_common::error::{NucleusError, NucleusResult};
use nucleus_common::types::RegistryBackup;
use std::path::Path;

/// Platform-agnostic registry operations (Windows registry, Wine registry, or virtual registry).
pub trait RegistryManager: Send + Sync {
    /// Read a string value from a registry key.
    fn read_string(&self, key: &str, value_name: &str) -> NucleusResult<String>;

    /// Write a string value to a registry key.
    fn write_string(&self, key: &str, value_name: &str, data: &str) -> NucleusResult<()>;

    /// Write an expandable string value (REG_EXPAND_SZ) to a registry key.
    fn write_expand_string(&self, key: &str, value_name: &str, data: &str) -> NucleusResult<()> {
        let _ = (key, value_name, data);
        Err(NucleusError::Platform(
            "REG_EXPAND_SZ not supported on this platform".to_string(),
        ))
    }

    /// Read a 32-bit DWORD value from a registry key.
    fn read_dword(&self, key: &str, value_name: &str) -> NucleusResult<u32>;

    /// Write a 32-bit DWORD value to a registry key.
    fn write_dword(&self, key: &str, value_name: &str, data: u32) -> NucleusResult<()>;

    /// Delete a specific value within a registry key.
    fn delete_value(&self, key: &str, value_name: &str) -> NucleusResult<()>;

    /// Delete an entire registry key and its subkeys.
    fn delete_key(&self, key: &str) -> NucleusResult<()>;

    /// Create a backup of a registry key for later restoration.
    fn backup_key(&self, key: &str) -> NucleusResult<RegistryBackup>;

    /// Restore a registry key from a previously created backup.
    fn restore_key(&self, backup: &RegistryBackup) -> NucleusResult<()>;

    /// Export a key to a .reg file on disk (reg.exe export).
    fn export_to_file(&self, key: &str, file_path: &Path) -> NucleusResult<()> {
        let _ = (key, file_path);
        Err(NucleusError::Platform(
            "Registry export to file not supported on this platform".to_string(),
        ))
    }

    /// Import a key from a .reg file on disk (reg.exe import).
    fn import_from_file(&self, file_path: &Path) -> NucleusResult<()> {
        let _ = file_path;
        Err(NucleusError::Platform(
            "Registry import from file not supported on this platform".to_string(),
        ))
    }

    /// Check if a registry key exists.
    fn key_exists(&self, key: &str) -> bool;
}
