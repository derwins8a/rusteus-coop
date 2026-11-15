//! Save game and configuration backup and restoration tooling.

use std::path::{Path, PathBuf};
use nucleus_common::error::NucleusResult;
use nucleus_common::file_util;

/// Tool for backing up original user saves and configuration files before session launch,
/// and restoring them when instances exit.
#[derive(Debug, Default, Clone)]
pub struct BackupTool;

impl BackupTool {
    /// Backs up a list of files to the specified backup destination directory.
    pub fn backup_files(files: &[PathBuf], backup_root: &Path) -> NucleusResult<Vec<PathBuf>> {
        file_util::ensure_dir(backup_root)?;
        let mut backed_up = Vec::new();

        for file in files {
            if file.exists() && file.is_file() {
                let dest = file_util::backup_file(file, backup_root)?;
                backed_up.push(dest);
            }
        }

        Ok(backed_up)
    }

    /// Backs up an entire directory recursively to the target backup folder.
    pub fn backup_folder(folder: &Path, backup_target: &Path) -> NucleusResult<()> {
        if folder.exists() && folder.is_dir() {
            file_util::copy_directory(folder, backup_target, true)?;
        }
        Ok(())
    }

    /// Restores a backed up file to its original destination.
    pub fn restore_file(backup_path: &Path, original_path: &Path) -> NucleusResult<()> {
        file_util::restore_file(backup_path, original_path)
    }

    /// Restores an entire backed up folder back to its original location.
    pub fn restore_folder(backup_folder: &Path, original_folder: &Path) -> NucleusResult<()> {
        if backup_folder.exists() && backup_folder.is_dir() {
            file_util::copy_directory(backup_folder, original_folder, true)?;
        }
        Ok(())
    }
}
