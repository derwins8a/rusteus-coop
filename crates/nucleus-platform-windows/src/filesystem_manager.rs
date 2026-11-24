use std::os::windows::ffi::OsStrExt;
use std::os::windows::process::CommandExt;
use std::path::{Path, PathBuf};

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
            .map_err(|e| {
                nucleus_common::error::NucleusError::Platform(format!(
                    "GetVolumeInformationW failed for {}: {}",
                    root, e
                ))
            })?;
        }
        let len = fs_name_buf
            .iter()
            .position(|&c| c == 0)
            .unwrap_or(fs_name_buf.len());
        let fs_name = String::from_utf16_lossy(&fs_name_buf[..len]);
        Ok(fs_name)
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
        let desktop_dir = dirs::desktop_dir()
            .or_else(|| {
                std::env::var_os("USERPROFILE").map(|u| PathBuf::from(u).join("Desktop"))
            })
            .unwrap_or_else(|| PathBuf::from("."));

        if !desktop_dir.exists() {
            let _ = std::fs::create_dir_all(&desktop_dir);
        }

        let clean_name = sanitize_shortcut_name(name);
        let file_name = if clean_name.to_lowercase().ends_with(".lnk") {
            clean_name
        } else {
            format!("{}.lnk", clean_name)
        };
        let shortcut_path = desktop_dir.join(file_name);

        // Try native Windows COM first
        let com_res = unsafe {
            create_shortcut_com(
                &shortcut_path,
                target_exe,
                arguments,
                working_dir,
                icon_path,
                description,
            )
        };

        if let Err(com_err) = com_res {
            tracing::warn!(
                "Windows COM shortcut creation failed ({:?}); falling back to PowerShell WScript.Shell",
                com_err
            );
            create_shortcut_powershell(
                &shortcut_path,
                target_exe,
                arguments,
                working_dir,
                icon_path,
                description,
            )?;
        }

        Ok(shortcut_path)
    }
}

/// Creates a `.lnk` shortcut file using Windows COM (IShellLinkW + IPersistFile).
unsafe fn create_shortcut_com(
    shortcut_path: &Path,
    target_exe: &Path,
    arguments: &str,
    working_dir: &Path,
    icon_path: Option<&Path>,
    description: &str,
) -> windows::core::Result<()> {
    use windows::Win32::Foundation::BOOL;
    use windows::Win32::System::Com::IPersistFile;
    use windows::Win32::System::Com::{
        CoCreateInstance, CoInitializeEx, CLSCTX_INPROC_SERVER, COINIT_APARTMENTTHREADED,
    };
    use windows::Win32::UI::Shell::{IShellLinkW, ShellLink};
    use windows::core::{Interface, PCWSTR};

    unsafe {
        // CoInitializeEx may return S_OK, S_FALSE, or RPC_E_CHANGED_MODE. All are acceptable.
        let _ = CoInitializeEx(None, COINIT_APARTMENTTHREADED);

        let shell_link: IShellLinkW = CoCreateInstance(&ShellLink, None, CLSCTX_INPROC_SERVER)?;

        let target_wide: Vec<u16> = target_exe
            .as_os_str()
            .encode_wide()
            .chain(std::iter::once(0))
            .collect();
        shell_link.SetPath(PCWSTR(target_wide.as_ptr()))?;

        if !arguments.is_empty() {
            let args_wide: Vec<u16> = arguments.encode_utf16().chain(std::iter::once(0)).collect();
            shell_link.SetArguments(PCWSTR(args_wide.as_ptr()))?;
        }

        let work_dir_wide: Vec<u16> = working_dir
            .as_os_str()
            .encode_wide()
            .chain(std::iter::once(0))
            .collect();
        shell_link.SetWorkingDirectory(PCWSTR(work_dir_wide.as_ptr()))?;

        if !description.is_empty() {
            let desc_wide: Vec<u16> = description.encode_utf16().chain(std::iter::once(0)).collect();
            shell_link.SetDescription(PCWSTR(desc_wide.as_ptr()))?;
        }

        if let Some(icon) = icon_path {
            let icon_wide: Vec<u16> = icon
                .as_os_str()
                .encode_wide()
                .chain(std::iter::once(0))
                .collect();
            shell_link.SetIconLocation(PCWSTR(icon_wide.as_ptr()), 0)?;
        }

        let persist_file: IPersistFile = shell_link.cast()?;
        let path_wide: Vec<u16> = shortcut_path
            .as_os_str()
            .encode_wide()
            .chain(std::iter::once(0))
            .collect();
        persist_file.Save(PCWSTR(path_wide.as_ptr()), BOOL(1))?;

        Ok(())
    }
}

/// Fallback shortcut creation using PowerShell with WScript.Shell.
fn create_shortcut_powershell(
    shortcut_path: &Path,
    target_exe: &Path,
    arguments: &str,
    working_dir: &Path,
    icon_path: Option<&Path>,
    description: &str,
) -> NucleusResult<()> {
    let mut ps_script = format!(
        "$ws = New-Object -ComObject WScript.Shell; \
         $s = $ws.CreateShortcut('{}'); \
         $s.TargetPath = '{}'; \
         $s.Arguments = '{}'; \
         $s.WorkingDirectory = '{}'; \
         $s.Description = '{}'; ",
        escape_ps_single_quote(&shortcut_path.to_string_lossy()),
        escape_ps_single_quote(&target_exe.to_string_lossy()),
        escape_ps_single_quote(arguments),
        escape_ps_single_quote(&working_dir.to_string_lossy()),
        escape_ps_single_quote(description),
    );

    if let Some(icon) = icon_path {
        ps_script.push_str(&format!(
            "$s.IconLocation = '{},0'; ",
            escape_ps_single_quote(&icon.to_string_lossy())
        ));
    }
    ps_script.push_str("$s.Save();");

    let status = std::process::Command::new("powershell")
        .args([
            "-NoProfile",
            "-NonInteractive",
            "-ExecutionPolicy",
            "Bypass",
            "-Command",
            &ps_script,
        ])
        .creation_flags(0x08000000) // CREATE_NO_WINDOW
        .status()?;

    if status.success() && shortcut_path.exists() {
        Ok(())
    } else {
        Err(nucleus_common::error::NucleusError::Platform(format!(
            "PowerShell failed to create desktop shortcut at {}",
            shortcut_path.display()
        )))
    }
}

/// Escape single quotes in strings for PowerShell single-quoted literals (`'foo''bar'`).
fn escape_ps_single_quote(s: &str) -> String {
    s.replace('\'', "''")
}

/// Sanitize filename for desktop shortcuts, stripping illegal Windows filesystem characters.
fn sanitize_shortcut_name(name: &str) -> String {
    let clean: String = name
        .chars()
        .map(|c| match c {
            '<' | '>' | ':' | '"' | '/' | '\\' | '|' | '?' | '*' => '_',
            c if (c as u32) < 32 => '_',
            other => other,
        })
        .collect();
    let trimmed = clean.trim().trim_matches('.');
    if trimmed.is_empty() {
        "Game".to_string()
    } else {
        trimmed.to_string()
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
        if canon_str.starts_with(r"\\?\") && canon_str.len() >= 6 && canon_str.as_bytes()[5] == b':'
        {
            let drive = (canon_str.as_bytes()[4] as char).to_ascii_uppercase();
            return format!("{}:\\", drive);
        }
    }
    "C:\\".to_string()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_sanitize_shortcut_name() {
        assert_eq!(
            sanitize_shortcut_name("Game: Subtitle? *1*"),
            "Game_ Subtitle_ _1_"
        );
        assert_eq!(sanitize_shortcut_name("  My Game.lnk  "), "My Game.lnk");
        assert_eq!(sanitize_shortcut_name("..."), "Game");
        assert_eq!(
            sanitize_shortcut_name("Game/With\\Slashes"),
            "Game_With_Slashes"
        );
    }

    #[test]
    fn test_create_desktop_shortcut() {
        let mgr = WindowsFilesystemManager::new();
        let temp_dir = std::env::temp_dir();
        let target_exe = temp_dir.join("nucleus_test_dummy_game.exe");
        let _ = std::fs::write(&target_exe, b"dummy");

        let shortcut_path = mgr
            .create_desktop_shortcut(
                "NucleusCoop_UnitTest_Shortcut",
                &target_exe,
                "--player 1",
                &temp_dir,
                None,
                "Unit Test Shortcut Description",
            )
            .expect("desktop shortcut creation should succeed");

        assert!(shortcut_path.to_string_lossy().ends_with(".lnk"));
        assert!(shortcut_path.exists(), "shortcut file should exist on disk");

        // Clean up test shortcut and dummy exe
        let _ = std::fs::remove_file(&shortcut_path);
        let _ = std::fs::remove_file(&target_exe);
    }
}
