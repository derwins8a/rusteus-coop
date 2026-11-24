//! Linux stub implementation of `FilesystemManager`.

#![cfg(target_os = "linux")]

use std::path::{Path, PathBuf};

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
        Err(NucleusError::Unsupported(
            "Filesystem symlink creation not yet implemented on Linux".into(),
        ))
    }

    fn create_directory_symlink(&self, _src: &Path, _dst: &Path) -> NucleusResult<()> {
        Err(NucleusError::Unsupported(
            "Directory symlink creation not yet implemented on Linux".into(),
        ))
    }

    fn create_hardlink(&self, _src: &Path, _dst: &Path) -> NucleusResult<()> {
        Err(NucleusError::Unsupported(
            "Hardlink creation not yet implemented on Linux".into(),
        ))
    }

    fn create_junction(&self, _src: &Path, _dst: &Path) -> NucleusResult<()> {
        Err(NucleusError::Unsupported(
            "NTFS junctions not supported on Linux".into(),
        ))
    }

    fn is_symlink(&self, _path: &Path) -> bool {
        false
    }

    fn create_desktop_shortcut(
        &self,
        name: &str,
        target_exe: &Path,
        arguments: &str,
        working_dir: &Path,
        icon_path: Option<&Path>,
        description: &str,
    ) -> NucleusResult<PathBuf> {
        let sanitized_name: String = name
            .chars()
            .map(|c| if c == '/' || c == '\0' || c == ':' || c == '\\' { '_' } else { c })
            .collect();
        let filename = format!("{}.desktop", sanitized_name);

        let target_dir = dirs::desktop_dir()
            .or_else(|| dirs::data_local_dir().map(|p| p.join("applications")))
            .or_else(|| dirs::home_dir().map(|p| p.join("Desktop")))
            .unwrap_or_else(|| PathBuf::from("."));

        if !target_dir.exists() {
            let _ = std::fs::create_dir_all(&target_dir);
        }

        let shortcut_path = target_dir.join(filename);

        let exec_cmd = if arguments.trim().is_empty() {
            format!("\"{}\"", target_exe.display())
        } else {
            format!("\"{}\" {}", target_exe.display(), arguments.trim())
        };

        let icon_line = if let Some(icon) = icon_path {
            format!("Icon={}\n", icon.display())
        } else {
            String::new()
        };

        let content = format!(
            "[Desktop Entry]\n\
             Type=Application\n\
             Name={name}\n\
             Comment={description}\n\
             Exec={exec_cmd}\n\
             Path={working_dir}\n\
             {icon_line}\
             Terminal=false\n\
             Categories=Game;\n",
            name = name,
            description = description,
            exec_cmd = exec_cmd,
            working_dir = working_dir.display(),
            icon_line = icon_line,
        );

        std::fs::write(&shortcut_path, content)?;

        #[cfg(unix)]
        {
            use std::os::unix::fs::PermissionsExt;
            if let Ok(metadata) = std::fs::metadata(&shortcut_path) {
                let mut perms = metadata.permissions();
                perms.set_mode(0o755);
                let _ = std::fs::set_permissions(&shortcut_path, perms);
            }
        }

        Ok(shortcut_path)
    }
}
