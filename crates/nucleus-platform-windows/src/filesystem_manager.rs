use std::os::windows::process::CommandExt;
use std::path::Path;

use nucleus_common::error::NucleusResult;

use nucleus_platform::FilesystemManager;

/// Windows filesystem linking and junction manager.
#[derive(Default)]
pub struct WindowsFilesystemManager;

impl WindowsFilesystemManager {
    /// Creates a new `WindowsFilesystemManager`.
    pub fn new() -> Self {
        Self
    }
}

impl FilesystemManager for WindowsFilesystemManager {
    fn create_symlink(&self, src: &Path, dst: &Path) -> NucleusResult<()> {
        if let Some(parent) = dst.parent() {
            if !parent.exists() {
                std::fs::create_dir_all(parent)?;
            }
        }
        std::os::windows::fs::symlink_file(src, dst)?;
        Ok(())
    }

    fn create_directory_symlink(&self, src: &Path, dst: &Path) -> NucleusResult<()> {
        if let Some(parent) = dst.parent() {
            if !parent.exists() {
                std::fs::create_dir_all(parent)?;
            }
        }
        std::os::windows::fs::symlink_dir(src, dst)?;
        Ok(())
    }

    fn create_hardlink(&self, src: &Path, dst: &Path) -> NucleusResult<()> {
        if let Some(parent) = dst.parent() {
            if !parent.exists() {
                std::fs::create_dir_all(parent)?;
            }
        }
        std::fs::hard_link(src, dst)?;
        Ok(())
    }

    fn create_junction(&self, src: &Path, dst: &Path) -> NucleusResult<()> {
        if let Some(parent) = dst.parent() {
            if !parent.exists() {
                std::fs::create_dir_all(parent)?;
            }
        }
        // Try directory symlink first
        if std::os::windows::fs::symlink_dir(src, dst).is_ok() {
            return Ok(());
        }
        // Fallback to mklink /J
        let src_str = src.to_string_lossy();
        let dst_str = dst.to_string_lossy();
        let status = std::process::Command::new("cmd")
            .args(["/C", "mklink", "/J", &dst_str, &src_str])
            .creation_flags(0x08000000) // CREATE_NO_WINDOW
            .status()?;
        if status.success() {
            Ok(())
        } else {
            Err(nucleus_common::error::NucleusError::Platform(
                "Failed to create junction point".to_string(),
            ))
        }
    }

    fn is_symlink(&self, path: &Path) -> bool {
        path.symlink_metadata()
            .map(|m| m.file_type().is_symlink())
            .unwrap_or(false)
    }

    fn get_volume_filesystem(&self, path: &Path) -> NucleusResult<String> {
        let root = get_volume_root(path);
        let root_wide: Vec<u16> = root.encode_utf16().chain(std::iter::once(0)).collect();
        let mut fs_name_buf = [0u16; 260];
        unsafe {
            windows::Win32::Storage::FileSystem::GetVolumeInformationW(
                windows::core::PCWSTR(root_wide.as_ptr()),
                None,
                None,
                None,
                None,
                Some(&mut fs_name_buf),
            )
            .map_err(|e| nucleus_common::error::NucleusError::Platform(format!(
                "GetVolumeInformationW failed for {}: {}",
                root, e
            )))?;
        }
        let len = fs_name_buf.iter().position(|&c| c == 0).unwrap_or(fs_name_buf.len());
        let fs_name = String::from_utf16_lossy(&fs_name_buf[..len]);
        Ok(fs_name)
    }
}

/// Helper function to resolve volume root (e.g. "C:\\") from any path.
pub fn get_volume_root(path: &Path) -> String {
    let path_str = path.to_string_lossy();
    let bytes = path_str.as_bytes();
    if bytes.len() >= 2 && bytes[1] == b':' && bytes[0].is_ascii_alphabetic() {
        return format!("{}:\\", (bytes[0] as char).to_ascii_uppercase());
    }
    if path_str.starts_with(r"\\?\") && path_str.len() >= 6 && path_str.as_bytes()[5] == b':' {
        let drive = (path_str.as_bytes()[4] as char).to_ascii_uppercase();
        return format!("{}:\\", drive);
    }
    if let Ok(canonical) = path.canonicalize() {
        let canon_str = canonical.to_string_lossy();
        if canon_str.starts_with(r"\\?\") && canon_str.len() >= 6 && canon_str.as_bytes()[5] == b':' {
            let drive = (canon_str.as_bytes()[4] as char).to_ascii_uppercase();
            return format!("{}:\\", drive);
        }
    }
    "C:\\".to_string()
}

