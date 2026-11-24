//! Platform-agnostic filesystem link operations.

use std::path::{Path, PathBuf};

use nucleus_common::error::NucleusResult;

/// Platform-agnostic filesystem link operations (symlinks, directory symlinks, hardlinks, junctions).
pub trait FilesystemManager: Send + Sync {
    /// Create a symbolic link pointing to `src` at path `dst`.
    fn create_symlink(&self, src: &Path, dst: &Path) -> NucleusResult<()>;

    /// Create a directory symbolic link pointing to `src` at path `dst`.
    fn create_directory_symlink(&self, src: &Path, dst: &Path) -> NucleusResult<()>;

    /// Create a hard link pointing to `src` at path `dst`.
    fn create_hardlink(&self, src: &Path, dst: &Path) -> NucleusResult<()>;

    /// Create an NTFS junction point (Windows) or directory symlink equivalent (Linux).
    fn create_junction(&self, src: &Path, dst: &Path) -> NucleusResult<()>;

    /// Check if a path is a symbolic link or junction.
    fn is_symlink(&self, path: &Path) -> bool;

    /// Returns the filesystem type/format name of the drive/volume containing `path` (e.g. "NTFS", "exFAT", "FAT32").
    fn get_volume_filesystem(&self, path: &Path) -> NucleusResult<String> {
        let _ = path;
        Ok("NTFS".to_string())
    }

    /// Creates a desktop shortcut pointing to `target_exe` with `arguments`.
    /// Returns the path to the created shortcut file (e.g. `<Desktop>/<name>.lnk` on Windows or `<Desktop>/<name>.desktop` on Linux).
    fn create_desktop_shortcut(
        &self,
        name: &str,
        target_exe: &Path,
        arguments: &str,
        working_dir: &Path,
        icon_path: Option<&Path>,
        description: &str,
    ) -> NucleusResult<PathBuf> {
        let _ = (name, target_exe, arguments, working_dir, icon_path, description);
        Err(nucleus_common::error::NucleusError::Unsupported(
            "Desktop shortcut creation not supported on this platform".into(),
        ))
    }
}
