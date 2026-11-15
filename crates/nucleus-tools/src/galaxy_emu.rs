//! Nemirtingas GOG Galaxy offline emulator configuration and DLL deployment.

use nucleus_common::error::NucleusResult;
use nucleus_common::file_util;
use serde::{Deserialize, Serialize};
use std::fs;
use std::path::{Path, PathBuf};

/// JSON configuration format for `NemirtingasGalaxyEmu.json`.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct GalaxyEmuConfig {
    pub api_version: String,
    pub disable_online_networking: bool,
    pub enable_lan: bool,
    pub enable_overlay: bool,
    pub galaxyid: u64,
    pub language: String,
    pub productid: u64,
    pub savepath: String,
    pub unlock_dlcs: bool,
    pub username: String,
}

impl Default for GalaxyEmuConfig {
    fn default() -> Self {
        Self {
            api_version: "1.100.2.0".to_string(),
            disable_online_networking: false,
            enable_lan: true,
            enable_overlay: false,
            galaxyid: 14601386556348241,
            language: "english".to_string(),
            productid: 2104387650,
            savepath: "appdata".to_string(),
            unlock_dlcs: true,
            username: "Player1".to_string(),
        }
    }
}

/// Tool for configuring and deploying Nemirtingas GOG Galaxy emulator configuration
/// and Galaxy DLLs to enable multi-instance LAN play for GOG Galaxy releases.
#[derive(Debug, Default, Clone)]
pub struct GalaxyEmuTool;

impl GalaxyEmuTool {
    /// Generates the raw JSON string for `NemirtingasGalaxyEmu.json`.
    pub fn generate_json(config: &GalaxyEmuConfig) -> String {
        serde_json::to_string_pretty(config).unwrap_or_else(|_| "{}".to_string())
    }

    /// Recursively deploys Nemirtingas Galaxy Emulator DLLs (`Galaxy.dll` / `Galaxy64.dll`)
    /// into the instance directory matching locations found in `game_root_dir`.
    pub fn deploy_dlls(
        game_root_dir: &Path,
        instance_dir: &Path,
        utils_dir: &Path,
        is_64bit: bool,
    ) -> NucleusResult<Vec<PathBuf>> {
        file_util::ensure_dir(instance_dir)?;
        let mut deployed = Vec::new();
        let galaxy_utils_dir = utils_dir.join("NemirtingasGalaxyEmu");

        let x86_src = galaxy_utils_dir.join("x86").join("Galaxy.dll");
        let x86_fallback = galaxy_utils_dir.join("Galaxy.dll");
        let x64_src = galaxy_utils_dir.join("x64").join("Galaxy64.dll");
        let x64_fallback = galaxy_utils_dir.join("Galaxy64.dll");

        let src_86 = if x86_src.exists() {
            Some(x86_src)
        } else if x86_fallback.exists() {
            Some(x86_fallback)
        } else {
            None
        };
        let src_64 = if x64_src.exists() {
            Some(x64_src)
        } else if x64_fallback.exists() {
            Some(x64_fallback)
        } else {
            None
        };

        let mut found_any = false;
        if game_root_dir.exists() {
            Self::scan_and_deploy_recursive(
                game_root_dir,
                game_root_dir,
                instance_dir,
                &src_86,
                &src_64,
                &mut deployed,
                &mut found_any,
            )?;
        }

        // If no existing Galaxy DLL was found in game tree, deploy to instance root
        if !found_any {
            if is_64bit {
                if let Some(src) = &src_64 {
                    let dest = instance_dir.join("Galaxy64.dll");
                    let _ = file_util::copy_file(src, &dest, true);
                    deployed.push(dest);
                }
            } else if let Some(src) = &src_86 {
                let dest = instance_dir.join("Galaxy.dll");
                let _ = file_util::copy_file(src, &dest, true);
                deployed.push(dest);
            }
        }

        Ok(deployed)
    }

    fn scan_and_deploy_recursive(
        current_dir: &Path,
        root_dir: &Path,
        instance_dir: &Path,
        src_86: &Option<PathBuf>,
        src_64: &Option<PathBuf>,
        deployed: &mut Vec<PathBuf>,
        found_any: &mut bool,
    ) -> NucleusResult<()> {
        let entries = match fs::read_dir(current_dir) {
            Ok(e) => e,
            Err(_) => return Ok(()),
        };

        for entry in entries.flatten() {
            let path = entry.path();
            if path.is_dir() {
                Self::scan_and_deploy_recursive(
                    &path,
                    root_dir,
                    instance_dir,
                    src_86,
                    src_64,
                    deployed,
                    found_any,
                )?;
            } else if let Some(file_name) = path.file_name().and_then(|n| n.to_str()) {
                let lower = file_name.to_ascii_lowercase();
                let is_x64 = lower == "galaxy64.dll";
                let is_x86 = lower == "galaxy.dll";

                if is_x64 || is_x86 {
                    *found_any = true;
                    let rel_folder = path
                        .parent()
                        .and_then(|p| p.strip_prefix(root_dir).ok())
                        .unwrap_or(Path::new(""));
                    let dest_folder = instance_dir.join(rel_folder);
                    let _ = file_util::ensure_dir(&dest_folder);
                    let dest_file = dest_folder.join(file_name);

                    if is_x64 {
                        if let Some(src) = src_64 {
                            let _ = file_util::copy_file(src, &dest_file, true);
                            deployed.push(dest_file);
                        }
                    } else if let Some(src) = src_86 {
                        let _ = file_util::copy_file(src, &dest_file, true);
                        deployed.push(dest_file);
                    }
                }
            }
        }
        Ok(())
    }

    /// Sets up Nemirtingas GOG Galaxy emulator settings (`NemirtingasGalaxyEmu.json`) in the game instance directory,
    /// writing to both `ngalaxye_settings/NemirtingasGalaxyEmu.json` and instance root.
    pub fn setup_galaxy_emu(
        game_instance_dir: &Path,
        instance_index: usize,
        username: &str,
        language: Option<&str>,
        product_id: Option<u64>,
    ) -> NucleusResult<PathBuf> {
        let config = GalaxyEmuConfig {
            api_version: "1.100.2.0".to_string(),
            disable_online_networking: false,
            enable_lan: true,
            enable_overlay: false,
            galaxyid: 14601386556348240 + (instance_index as u64 + 1),
            language: language.unwrap_or("english").to_ascii_lowercase(),
            productid: product_id.unwrap_or(2104387650),
            savepath: "appdata".to_string(),
            unlock_dlcs: true,
            username: username.to_string(),
        };

        let json_content = Self::generate_json(&config);

        // ngalaxye_settings folder
        let settings_dir = game_instance_dir.join("ngalaxye_settings");
        file_util::ensure_dir(&settings_dir)?;
        let settings_path = settings_dir.join("NemirtingasGalaxyEmu.json");
        file_util::write_text_file(&settings_path, &json_content)?;

        // Root of instance folder (legacy support)
        let root_path = game_instance_dir.join("NemirtingasGalaxyEmu.json");
        file_util::write_text_file(&root_path, &json_content)?;

        Ok(settings_path)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_galaxy_emu_generate_json() {
        let config = GalaxyEmuConfig {
            api_version: "1.100.2.0".to_string(),
            disable_online_networking: false,
            enable_lan: true,
            enable_overlay: false,
            galaxyid: 14601386556348242,
            language: "german".to_string(),
            productid: 2104387650,
            savepath: "appdata".to_string(),
            unlock_dlcs: true,
            username: "Player2".to_string(),
        };
        let json = GalaxyEmuTool::generate_json(&config);
        assert!(json.contains("\"galaxyid\": 14601386556348242"));
        assert!(json.contains("\"username\": \"Player2\""));
        assert!(json.contains("\"language\": \"german\""));
        assert!(json.contains("\"productid\": 2104387650"));
    }

    #[test]
    fn test_galaxy_emu_setup_and_deploy() {
        let temp_dir =
            std::env::temp_dir().join(format!("nucleus_galaxy_test_{}", std::process::id()));
        let _ = std::fs::remove_dir_all(&temp_dir);

        let utils_dir = temp_dir.join("utils");
        let galaxy_utils_x64 = utils_dir.join("NemirtingasGalaxyEmu").join("x64");
        file_util::ensure_dir(&galaxy_utils_x64).unwrap();
        file_util::write_text_file(&galaxy_utils_x64.join("Galaxy64.dll"), "GALAXY64_BYTES")
            .unwrap();

        let game_root = temp_dir.join("GameRoot");
        let game_bin = game_root.join("bin").join("x64");
        file_util::ensure_dir(&game_bin).unwrap();
        file_util::write_text_file(&game_bin.join("Galaxy64.dll"), "ORIG_GALAXY").unwrap();

        let instance_dir = temp_dir.join("Instance0");

        let settings_path =
            GalaxyEmuTool::setup_galaxy_emu(&instance_dir, 0, "Player1", Some("french"), None)
                .unwrap();

        assert!(settings_path.exists());
        let read_json = file_util::read_text_file(&settings_path).unwrap();
        assert!(read_json.contains("\"language\": \"french\""));
        assert!(read_json.contains("\"galaxyid\": 14601386556348241"));
        assert!(instance_dir.join("NemirtingasGalaxyEmu.json").exists());

        let deployed =
            GalaxyEmuTool::deploy_dlls(&game_root, &instance_dir, &utils_dir, true).unwrap();
        assert_eq!(deployed.len(), 1);
        let deployed_dll = instance_dir.join("bin").join("x64").join("Galaxy64.dll");
        assert!(deployed_dll.exists());
        assert_eq!(
            file_util::read_text_file(&deployed_dll).unwrap(),
            "GALAXY64_BYTES"
        );

        let _ = std::fs::remove_dir_all(&temp_dir);
    }
}
