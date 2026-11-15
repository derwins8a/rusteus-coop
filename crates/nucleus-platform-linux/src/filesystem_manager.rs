//! Linux stub implementation of `FilesystemManager`.

#![cfg(target_os = "linux")]

use std::path::Path;

use nucleus_common::error::{NucleusError, NucleusResult};
use nucleus_platform::FilesystemManager;

/// Linux filesystem linking manager.
#[derive(Default)]
pub struct LinuxFilesystemManager;

impl LinuxFilesystemManager {
    /// Creates a new `LinuxFilesystemManager`.
    pub fn new() -> Self {
        Self
    }
}

impl FilesystemManager for LinuxFilesystemManager {
    fn create_symlink(&self, _src: &Path, _dst: &Path) -> NucleusResult<()> {
        Err(NucleusError::Unsupported("Filesystem symlink creation not yet implemented on Linux".into()))
    }

    fn create_directory_symlink(&self, _src: &Path, _dst: &Path) -> NucleusResult<()> {
        Err(NucleusError::Unsupported("Directory symlink creation not yet implemented on Linux".into()))
    }

    fn create_hardlink(&self, _src: &Path, _dst: &Path) -> NucleusResult<()> {
        Err(NucleusError::Unsupported("Hardlink creation not yet implemented on Linux".into()))
    }

    fn create_junction(&self, _src: &Path, _dst: &Path) -> NucleusResult<()> {
        Err(NucleusError::Unsupported("NTFS junctions not supported on Linux".into()))
    }

    fn is_symlink(&self, _path: &Path) -> bool {
        false
    }
}
