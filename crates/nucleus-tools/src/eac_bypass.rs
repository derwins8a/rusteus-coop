//! Easy Anti-Cheat bypass preparation and launch configuration.

use nucleus_common::error::NucleusResult;
use nucleus_common::file_util;
use std::fs;
use std::path::{Path, PathBuf};

/// Tool for configuring game startup flags, deploying offline bypass DLLs,
/// and disabling Easy Anti-Cheat (EAC) when launching local multi-instance sessions.
#[derive(Debug, Default, Clone)]
pub struct EacBypassTool;

impl EacBypassTool {
    /// Standard command line flags commonly used to disable or bypass EAC offline.
    pub fn get_standard_offline_args() -> Vec<String> {
        vec![
            "-noeac".to_string(),
            "-eac-nopass".to_string(),
            "-disable-eac".to_string(),
        ]
    }

    /// Recursively searches for `EasyAntiCheat_x64.dll` and `EasyAntiCheat_x86.dll` in `link_folder`
    /// and replaces them with offline bypass replacement DLLs from `utils/EAC Bypass/` (or `utils_dir`).
    /// Creates a `.nucleus_bak` copy of the original DLL before replacing it.
    pub fn deploy_eac_bypass_dlls(
        link_folder: &Path,
        utils_dir: &Path,
    ) -> NucleusResult<Vec<PathBuf>> {
        let mut replaced = Vec::new();
        let eac_util_dir = utils_dir.join("EAC Bypass");

        let x64_bypass = if eac_util_dir.join("EasyAntiCheat_x64.dll").exists() {
            Some(eac_util_dir.join("EasyAntiCheat_x64.dll"))
        } else if utils_dir.join("EasyAntiCheat_x64.dll").exists() {
            Some(utils_dir.join("EasyAntiCheat_x64.dll"))
        } else {
            None
        };

        let x86_bypass = if eac_util_dir.join("EasyAntiCheat_x86.dll").exists() {
            Some(eac_util_dir.join("EasyAntiCheat_x86.dll"))
        } else if utils_dir.join("EasyAntiCheat_x86.dll").exists() {
            Some(utils_dir.join("EasyAntiCheat_x86.dll"))
        } else {
            None
        };

        Self::scan_and_replace_dlls_recursive(
            link_folder,
            &x64_bypass,
            &x86_bypass,
            &mut replaced,
        )?;

        Ok(replaced)
    }

    fn scan_and_replace_dlls_recursive(
        dir: &Path,
        x64_bypass: &Option<PathBuf>,
        x86_bypass: &Option<PathBuf>,
        replaced: &mut Vec<PathBuf>,
    ) -> NucleusResult<()> {
        let entries = match fs::read_dir(dir) {
            Ok(e) => e,
            Err(_) => return Ok(()),
        };

        for entry in entries.flatten() {
            let path = entry.path();
            if path.is_dir() {
                Self::scan_and_replace_dlls_recursive(&path, x64_bypass, x86_bypass, replaced)?;
            } else if let Some(file_name) = path.file_name().and_then(|n| n.to_str()) {
                let lower = file_name.to_ascii_lowercase();
                if lower == "easyanticheat_x64.dll" {
                    if let Some(src) = x64_bypass {
                        let backup = path.with_extension("dll.nucleus_bak");
                        if !backup.exists() {
                            let _ = file_util::copy_file(&path, &backup, false);
                        }
                        file_util::copy_file(src, &path, true)?;
                        replaced.push(path.clone());
                    }
                } else if lower == "easyanticheat_x86.dll" {
                    if let Some(src) = x86_bypass {
                        let backup = path.with_extension("dll.nucleus_bak");
                        if !backup.exists() {
                            let _ = file_util::copy_file(&path, &backup, false);
                        }
                        file_util::copy_file(src, &path, true)?;
                        replaced.push(path.clone());
                    }
                }
            }
        }

        Ok(())
    }

    /// Prepares an EAC bypass by backing up the original protective launcher (e.g., `start_protected_game.exe`)
    /// and replacing it with the target game executable or launcher stub.
    pub fn setup_launcher_bypass(
        game_dir: &Path,
        launcher_name: &str,
        target_game_exe: &str,
    ) -> NucleusResult<PathBuf> {
        file_util::ensure_dir(game_dir)?;

        let original_launcher = game_dir.join(launcher_name);
        let backup_launcher = game_dir.join(format!("{}.nucleus_bak", launcher_name));
        let target_binary = game_dir.join(target_game_exe);

        // Backup original launcher if it exists and hasn't been backed up yet
        if original_launcher.exists() && !backup_launcher.exists() {
            file_util::copy_file(&original_launcher, &backup_launcher, false)?;
        }

        // If target game binary exists, copy it in place of the launcher
        if target_binary.exists() {
            file_util::copy_file(&target_binary, &original_launcher, true)?;
        }

        Ok(original_launcher)
    }

    /// Restores the original protective launcher executable from its backup if present.
    pub fn restore_launcher(game_dir: &Path, launcher_name: &str) -> NucleusResult<()> {
        let original_launcher = game_dir.join(launcher_name);
        let backup_launcher = game_dir.join(format!("{}.nucleus_bak", launcher_name));

        if backup_launcher.exists() {
            file_util::copy_file(&backup_launcher, &original_launcher, true)?;
            let _ = std::fs::remove_file(&backup_launcher);
        }

        Ok(())
    }

    /// Writes an offline configuration file to disable EAC telemetry and enforce offline mode.
    pub fn setup_offline_config(
        target_dir: &Path,
        product_id: Option<&str>,
    ) -> NucleusResult<PathBuf> {
        let eac_dir = target_dir.join("EasyAntiCheat");
        file_util::ensure_dir(&eac_dir)?;

        let mut ini = String::new();
        ini.push_str("[Settings]\r\n");
        if let Some(pid) = product_id {
            ini.push_str(&format!("productid={}\r\n", pid));
        }
        ini.push_str("use_eac=0\r\n");
        ini.push_str("sandboxid=offline\r\n");

        let settings_path = eac_dir.join("anticheat.ini");
        file_util::write_text_file(&settings_path, &ini)?;
        Ok(settings_path)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_eac_standard_args() {
        let args = EacBypassTool::get_standard_offline_args();
        assert!(args.contains(&"-noeac".to_string()));
        assert!(args.contains(&"-eac-nopass".to_string()));
    }

    #[test]
    fn test_eac_bypass_and_restore() {
        let temp_dir =
            std::env::temp_dir().join(format!("nucleus_eac_test_{}", std::process::id()));
        file_util::ensure_dir(&temp_dir).unwrap();

        let launcher = temp_dir.join("start_protected_game.exe");
        let game = temp_dir.join("game.exe");

        file_util::write_text_file(&launcher, "ORIGINAL_LAUNCHER").unwrap();
        file_util::write_text_file(&game, "REAL_GAME_BIN").unwrap();

        EacBypassTool::setup_launcher_bypass(&temp_dir, "start_protected_game.exe", "game.exe")
            .unwrap();

        let content = file_util::read_text_file(&launcher).unwrap();
        assert_eq!(content, "REAL_GAME_BIN");

        EacBypassTool::restore_launcher(&temp_dir, "start_protected_game.exe").unwrap();
        let restored = file_util::read_text_file(&launcher).unwrap();
        assert_eq!(restored, "ORIGINAL_LAUNCHER");

        let _ = std::fs::remove_dir_all(&temp_dir);
    }

    #[test]
    fn test_eac_deploy_bypass_dlls() {
        let temp_dir =
            std::env::temp_dir().join(format!("nucleus_eac_dll_test_{}", std::process::id()));
        let utils_dir = temp_dir.join("utils");
        let eac_utils = utils_dir.join("EAC Bypass");
        file_util::ensure_dir(&eac_utils).unwrap();

        // Create bypass replacement DLLs in utils
        file_util::write_text_file(&eac_utils.join("EasyAntiCheat_x64.dll"), "BYPASS_64").unwrap();
        file_util::write_text_file(&eac_utils.join("EasyAntiCheat_x86.dll"), "BYPASS_86").unwrap();

        // Create game folder with nested original EAC DLLs
        let game_dir = temp_dir.join("GameInstance");
        let sub_bin64 = game_dir.join("EasyAntiCheat");
        file_util::ensure_dir(&sub_bin64).unwrap();
        let orig_dll64 = sub_bin64.join("EasyAntiCheat_x64.dll");
        file_util::write_text_file(&orig_dll64, "ORIG_64").unwrap();

        let sub_bin86 = game_dir.join("Binaries").join("Win32");
        file_util::ensure_dir(&sub_bin86).unwrap();
        let orig_dll86 = sub_bin86.join("EasyAntiCheat_x86.dll");
        file_util::write_text_file(&orig_dll86, "ORIG_86").unwrap();

        let replaced = EacBypassTool::deploy_eac_bypass_dlls(&game_dir, &utils_dir).unwrap();
        assert_eq!(replaced.len(), 2);

        // Check backups were created
        assert!(sub_bin64.join("EasyAntiCheat_x64.dll.nucleus_bak").exists());
        assert!(sub_bin86.join("EasyAntiCheat_x86.dll.nucleus_bak").exists());

        // Check replaced content
        let c64 = file_util::read_text_file(&orig_dll64).unwrap();
        let c86 = file_util::read_text_file(&orig_dll86).unwrap();
        assert_eq!(c64, "BYPASS_64");
        assert_eq!(c86, "BYPASS_86");

        let _ = std::fs::remove_dir_all(&temp_dir);
    }

    #[test]
    fn test_eac_offline_config() {
        let temp_dir =
            std::env::temp_dir().join(format!("nucleus_eac_cfg_test_{}", std::process::id()));
        let cfg = EacBypassTool::setup_offline_config(&temp_dir, Some("halo_mcc")).unwrap();

        assert!(cfg.exists());
        let content = file_util::read_text_file(&cfg).unwrap();
        assert!(content.contains("productid=halo_mcc"));
        assert!(content.contains("use_eac=0"));

        let _ = std::fs::remove_dir_all(&temp_dir);
    }
}
