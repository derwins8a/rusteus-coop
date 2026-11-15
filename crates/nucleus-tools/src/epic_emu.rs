//! Nemirtingas Epic Games offline emulator configuration and DLL deployment.

use std::fs;
use std::path::{Path, PathBuf};
use serde::{Deserialize, Serialize};
use nucleus_common::error::NucleusResult;
use nucleus_common::file_util;

/// Mapping between common language names and ISO language codes supported by Epic Games Store and EOS.
pub fn get_epic_language(lang: &str) -> String {
    let clean = lang.trim().to_ascii_lowercase();
    match clean.as_str() {
        "arabic" | "ar" => "ar".to_string(),
        "brazilian" | "portuguese-brazil" | "pt-br" | "pt_br" => "pt-BR".to_string(),
        "bulgarian" | "bg" => "bg".to_string(),
        "chinese" | "zh" => "zh".to_string(),
        "czech" | "cs" => "cs".to_string(),
        "danish" | "da" => "da".to_string(),
        "dutch" | "nl" => "nl".to_string(),
        "english" | "en" | "en_us" | "en-us" => "en".to_string(),
        "finnish" | "fi" => "fi".to_string(),
        "french" | "fr" => "fr".to_string(),
        "german" | "de" => "de".to_string(),
        "greek" | "el" => "el".to_string(),
        "hungarian" | "hu" => "hu".to_string(),
        "italian" | "it" => "it".to_string(),
        "japanese" | "ja" => "ja".to_string(),
        "koreana" | "korean" | "ko" => "ko".to_string(),
        "norwegian" | "no" => "no".to_string(),
        "polish" | "pl" => "pl".to_string(),
        "portuguese" | "pt" => "pt".to_string(),
        "romanian" | "ro" => "ro".to_string(),
        "russian" | "ru" => "ru".to_string(),
        "spanish" | "es" => "es".to_string(),
        "swedish" | "sv" => "sv".to_string(),
        "thai" | "th" => "th".to_string(),
        "turkish" | "tr" => "tr".to_string(),
        "ukrainian" | "uk" => "uk".to_string(),
        other => {
            if other.is_empty() {
                "en".to_string()
            } else {
                lang.to_string()
            }
        }
    }
}

/// JSON configuration format for `NemirtingasEpicEmu.json`.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct EpicEmuConfig {
    pub enable_overlay: bool,
    pub epicid: String,
    pub disable_online_networking: bool,
    pub enable_lan: bool,
    pub savepath: String,
    pub unlock_dlcs: bool,
    pub language: String,
    pub username: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub app_name: Option<String>,
}

impl Default for EpicEmuConfig {
    fn default() -> Self {
        Self {
            enable_overlay: false,
            epicid: "0000000000000000000000000player1".to_string(),
            disable_online_networking: false,
            enable_lan: true,
            savepath: "appdata".to_string(),
            unlock_dlcs: true,
            language: "en".to_string(),
            username: "Player1".to_string(),
            app_name: None,
        }
    }
}

/// Tool for configuring and deploying Nemirtingas Epic Games emulator settings and EOSSDK DLLs
/// to enable multi-instance offline play on Epic Games Store titles.
#[derive(Debug, Default, Clone)]
pub struct EpicEmuTool;

impl EpicEmuTool {
    /// Generates the raw JSON string for `NemirtingasEpicEmu.json`.
    pub fn generate_json(config: &EpicEmuConfig) -> String {
        serde_json::to_string_pretty(config).unwrap_or_else(|_| "{}".to_string())
    }

    /// Recursively deploys Nemirtingas Epic Emulator DLLs (`EOSSDK-Win32-Shipping.dll` / `EOSSDK-Win64-Shipping.dll`)
    /// into the instance directory matching locations found in `game_root_dir`.
    pub fn deploy_dlls(
        game_root_dir: &Path,
        instance_dir: &Path,
        utils_dir: &Path,
        is_64bit: bool,
    ) -> NucleusResult<Vec<PathBuf>> {
        file_util::ensure_dir(instance_dir)?;
        let mut deployed = Vec::new();
        let epic_utils_dir = utils_dir.join("NemirtingasEpicEmu");

        let x86_src = epic_utils_dir.join("x86").join("EOSSDK-Win32-Shipping.dll");
        let x86_fallback = epic_utils_dir.join("EOSSDK-Win32-Shipping.dll");
        let x64_src = epic_utils_dir.join("x64").join("EOSSDK-Win64-Shipping.dll");
        let x64_fallback = epic_utils_dir.join("EOSSDK-Win64-Shipping.dll");

        let src_86 = if x86_src.exists() { Some(x86_src) } else if x86_fallback.exists() { Some(x86_fallback) } else { None };
        let src_64 = if x64_src.exists() { Some(x64_src) } else if x64_fallback.exists() { Some(x64_fallback) } else { None };

        let mut found_any = false;
        if game_root_dir.exists() {
            Self::scan_and_deploy_recursive(game_root_dir, game_root_dir, instance_dir, &src_86, &src_64, &mut deployed, &mut found_any)?;
        }

        // If no existing EOSSDK DLL was found in game tree, deploy standard DLL into instance root
        if !found_any {
            if is_64bit {
                if let Some(src) = &src_64 {
                    let dest = instance_dir.join("EOSSDK-Win64-Shipping.dll");
                    let _ = file_util::copy_file(src, &dest, true);
                    deployed.push(dest);
                }
            } else if let Some(src) = &src_86 {
                let dest = instance_dir.join("EOSSDK-Win32-Shipping.dll");
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
                Self::scan_and_deploy_recursive(&path, root_dir, instance_dir, src_86, src_64, deployed, found_any)?;
            } else if let Some(file_name) = path.file_name().and_then(|n| n.to_str()) {
                let lower = file_name.to_ascii_lowercase();
                let is_x64 = lower == "eossdk-win64-shipping.dll";
                let is_x86 = lower == "eossdk-win32-shipping.dll";

                if is_x64 || is_x86 {
                    *found_any = true;
                    let rel_folder = path.parent().and_then(|p| p.strip_prefix(root_dir).ok()).unwrap_or(Path::new(""));
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

    /// Sets up Nemirtingas Epic Games emulator settings (`NemirtingasEpicEmu.json`) in the game instance directory,
    /// writing to both `nepice_settings/NemirtingasEpicEmu.json` and instance root.
    pub fn setup_epic_emu(
        game_instance_dir: &Path,
        app_name: &str,
        user_id: &str,
        username: &str,
        language: Option<&str>,
    ) -> NucleusResult<PathBuf> {
        let config = EpicEmuConfig {
            enable_overlay: false,
            epicid: user_id.to_string(),
            disable_online_networking: false,
            enable_lan: true,
            savepath: "appdata".to_string(),
            unlock_dlcs: true,
            language: get_epic_language(language.unwrap_or("English")),
            username: username.to_string(),
            app_name: if app_name.is_empty() { None } else { Some(app_name.to_string()) },
        };

        let json_content = Self::generate_json(&config);

        // nepice_settings folder
        let settings_dir = game_instance_dir.join("nepice_settings");
        file_util::ensure_dir(&settings_dir)?;
        let settings_path = settings_dir.join("NemirtingasEpicEmu.json");
        file_util::write_text_file(&settings_path, &json_content)?;

        // Root of instance folder (legacy support)
        let root_path = game_instance_dir.join("NemirtingasEpicEmu.json");
        file_util::write_text_file(&root_path, &json_content)?;

        Ok(settings_path)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_epic_language_mapping() {
        assert_eq!(get_epic_language("English"), "en");
        assert_eq!(get_epic_language("French"), "fr");
        assert_eq!(get_epic_language("German"), "de");
        assert_eq!(get_epic_language("Brazilian"), "pt-BR");
        assert_eq!(get_epic_language("pt-BR"), "pt-BR");
        assert_eq!(get_epic_language("es"), "es");
    }

    #[test]
    fn test_epic_emu_generate_json() {
        let config = EpicEmuConfig {
            enable_overlay: false,
            epicid: "0000000000000000000000000player1".to_string(),
            disable_online_networking: false,
            enable_lan: true,
            savepath: "appdata".to_string(),
            unlock_dlcs: true,
            language: "en".to_string(),
            username: "Player1".to_string(),
            app_name: Some("Borderlands3".to_string()),
        };
        let json = EpicEmuTool::generate_json(&config);
        assert!(json.contains("\"epicid\": \"0000000000000000000000000player1\""));
        assert!(json.contains("\"username\": \"Player1\""));
        assert!(json.contains("\"language\": \"en\""));
        assert!(json.contains("\"enable_lan\": true"));
        assert!(json.contains("\"app_name\": \"Borderlands3\""));
    }

    #[test]
    fn test_epic_emu_setup_and_deploy() {
        let temp_dir = std::env::temp_dir().join(format!("nucleus_epic_test_{}", std::process::id()));
        let _ = std::fs::remove_dir_all(&temp_dir);

        let utils_dir = temp_dir.join("utils");
        let epic_utils_x64 = utils_dir.join("NemirtingasEpicEmu").join("x64");
        file_util::ensure_dir(&epic_utils_x64).unwrap();
        file_util::write_text_file(&epic_utils_x64.join("EOSSDK-Win64-Shipping.dll"), "EOS64_BYTES").unwrap();

        let game_root = temp_dir.join("GameRoot");
        let game_bin = game_root.join("Engine").join("Binaries").join("Win64");
        file_util::ensure_dir(&game_bin).unwrap();
        file_util::write_text_file(&game_bin.join("EOSSDK-Win64-Shipping.dll"), "ORIG_EOS").unwrap();

        let instance_dir = temp_dir.join("Instance0");

        let settings_path = EpicEmuTool::setup_epic_emu(
            &instance_dir,
            "Borderlands3",
            "0000000000000000000000000player1",
            "Player1",
            Some("German"),
        ).unwrap();

        assert!(settings_path.exists());
        let read_json = file_util::read_text_file(&settings_path).unwrap();
        assert!(read_json.contains("\"language\": \"de\""));
        assert!(instance_dir.join("NemirtingasEpicEmu.json").exists());

        let deployed = EpicEmuTool::deploy_dlls(&game_root, &instance_dir, &utils_dir, true).unwrap();
        assert_eq!(deployed.len(), 1);
        let deployed_dll = instance_dir.join("Engine").join("Binaries").join("Win64").join("EOSSDK-Win64-Shipping.dll");
        assert!(deployed_dll.exists());
        assert_eq!(file_util::read_text_file(&deployed_dll).unwrap(), "EOS64_BYTES");

        let _ = std::fs::remove_dir_all(&temp_dir);
    }
}
