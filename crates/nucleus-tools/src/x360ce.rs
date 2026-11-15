use std::path::{Path, PathBuf};
use nucleus_common::error::NucleusResult;
use nucleus_common::file_util;

/// Tool for generating x360ce.ini configuration files and deploying xinput / dinput
/// wrappers to remap DirectInput controllers to virtual Xbox 360 gamepads.
#[derive(Debug, Default, Clone)]
pub struct X360ceTool;

impl X360ceTool {
    /// Formats a raw gamepad GUID into the clean string format required by x360ce (e.g. `IG_...`).
    pub fn format_guid(guid: &str) -> String {
        let clean = guid
            .trim()
            .trim_matches(|c| c == '{' || c == '}')
            .replace('-', "");
        if clean.is_empty() {
            String::new()
        } else {
            format!("IG_{}", clean)
        }
    }

    /// Generates a fresh `x360ce.ini` file for one or multiple players on a single instance.
    /// Maps slots `[PAD1]` through `[PAD4]`. If fewer GUIDs are provided than 4, remaining PAD slots are left empty.
    pub fn generate_ini_multi(player_guids: &[&str], xbox_one_fix: bool) -> String {
        let mut lines = Vec::new();
        lines.push("[Options]".to_string());
        lines.push("UseInitBeep=1".to_string());
        lines.push("Log=0".to_string());
        lines.push("Console=0".to_string());
        lines.push("DebugOutput=0".to_string());
        lines.push(String::new());

        lines.push("[Mappings]".to_string());
        for slot in 1..=4 {
            let mapped = player_guids
                .get(slot - 1)
                .map(|g| Self::format_guid(g))
                .unwrap_or_default();
            lines.push(format!("PAD{}={}", slot, mapped));
        }
        lines.push(String::new());

        if xbox_one_fix {
            lines.push("[InputHook]".to_string());
            lines.push("HookLL=0".to_string());
            lines.push("HookCOM=1".to_string());
            lines.push("HookSA=0".to_string());
            lines.push("HookWT=0".to_string());
            lines.push("HOOKDI=1".to_string());
            lines.push("HOOKPIDVID=1".to_string());
            lines.push("HookName=0".to_string());
            lines.push("HookMode=0".to_string());
            lines.push(String::new());
        }

        lines.join("\r\n")
    }

    /// Generates an `x360ce.ini` content assigning the primary PAD1 to the specified player gamepad GUID.
    pub fn generate_ini(player_guid: &str, xbox_one_fix: bool) -> String {
        Self::generate_ini_multi(&[player_guid], xbox_one_fix)
    }

    /// Updates existing `x360ce.ini` content (such as a template copied from Instance0 or utils),
    /// replacing or inserting `PAD1=`, `PAD2=`, `PAD3=`, `PAD4=` lines under `[Mappings]` with the given `player_guids`.
    pub fn update_ini_mappings(ini_content: &str, player_guids: &[&str]) -> String {
        let mut output_lines = Vec::new();
        let mut in_mappings = false;
        let mut found_mappings = false;
        let mut handled_pads = [false; 4];

        for line in ini_content.lines() {
            let trimmed = line.trim();
            if trimmed.starts_with('[') && trimmed.ends_with(']') {
                if in_mappings {
                    for slot in 1..=4 {
                        if !handled_pads[slot - 1] {
                            let mapped = player_guids
                                .get(slot - 1)
                                .map(|g| Self::format_guid(g))
                                .unwrap_or_default();
                            output_lines.push(format!("PAD{}={}", slot, mapped));
                            handled_pads[slot - 1] = true;
                        }
                    }
                }
                in_mappings = trimmed.eq_ignore_ascii_case("[Mappings]");
                if in_mappings {
                    found_mappings = true;
                }
                output_lines.push(line.to_string());
                continue;
            }

            if in_mappings {
                let mut replaced = false;
                for slot in 1..=4 {
                    let prefix = format!("PAD{}=", slot);
                    if trimmed.starts_with(&prefix) {
                        let mapped = player_guids
                            .get(slot - 1)
                            .map(|g| Self::format_guid(g))
                            .unwrap_or_default();
                        output_lines.push(format!("PAD{}={}", slot, mapped));
                        handled_pads[slot - 1] = true;
                        replaced = true;
                        break;
                    }
                }
                if !replaced {
                    output_lines.push(line.to_string());
                }
            } else {
                output_lines.push(line.to_string());
            }
        }

        if in_mappings {
            for slot in 1..=4 {
                if !handled_pads[slot - 1] {
                    let mapped = player_guids
                        .get(slot - 1)
                        .map(|g| Self::format_guid(g))
                        .unwrap_or_default();
                    output_lines.push(format!("PAD{}={}", slot, mapped));
                    handled_pads[slot - 1] = true;
                }
            }
        }

        if !found_mappings {
            output_lines.push(String::new());
            output_lines.push("[Mappings]".to_string());
            for slot in 1..=4 {
                let mapped = player_guids
                    .get(slot - 1)
                    .map(|g| Self::format_guid(g))
                    .unwrap_or_default();
                output_lines.push(format!("PAD{}={}", slot, mapped));
            }
        }

        output_lines.join("\r\n")
    }

    /// Extended setup function supporting multi-player per instance (`PlayersPerInstance > 1`),
    /// carrying over `x360ce.ini` and DLLs from `Instance0` for `instance_index > 0`,
    /// and handling `dinput8.dll` / `dinput8_x64.dll` renaming.
    pub fn setup_x360ce_multi(
        instance_dir: &Path,
        player_guids: &[&str],
        utils_dir: &Path,
        is_64bit: bool,
        dll_names: &[&str],
        xbox_one_fix: bool,
        instance_index: usize,
        instance0_dir: Option<&Path>,
    ) -> NucleusResult<PathBuf> {
        file_util::ensure_dir(instance_dir)?;

        let x360_src_dir = utils_dir.join("x360ce");
        let target_dlls = if dll_names.is_empty() {
            vec!["xinput1_3.dll"]
        } else {
            dll_names.to_vec()
        };

        for &target_dll in &target_dlls {
            let dest = instance_dir.join(target_dll);
            let mut copied = false;

            // Carry over DLL from Instance0 if available for instance_index > 0
            if instance_index > 0 {
                if let Some(inst0) = instance0_dir {
                    let src0 = inst0.join(target_dll);
                    if src0.exists() {
                        let _ = file_util::copy_file(&src0, &dest, true);
                        copied = true;
                    }
                }
            }

            if !copied {
                let is_dinput = target_dll.to_ascii_lowercase().starts_with("dinput");
                let src_dll_name = if is_dinput {
                    if is_64bit {
                        "dinput8_x64.dll"
                    } else {
                        "dinput8.dll"
                    }
                } else if is_64bit {
                    "xinput1_3_x64.dll"
                } else {
                    "xinput1_3.dll"
                };

                let src_dll = x360_src_dir.join(src_dll_name);
                if src_dll.exists() {
                    let _ = file_util::copy_file(&src_dll, &dest, true);
                }
            }
        }

        // Setup x360ce.ini
        let ini_path = instance_dir.join("x360ce.ini");
        let ini_content = if instance_index > 0 {
            if let Some(inst0) = instance0_dir {
                let inst0_ini = inst0.join("x360ce.ini");
                if inst0_ini.exists() {
                    if let Ok(content) = file_util::read_text_file(&inst0_ini) {
                        Self::update_ini_mappings(&content, player_guids)
                    } else {
                        Self::generate_ini_multi(player_guids, xbox_one_fix)
                    }
                } else {
                    Self::generate_ini_multi(player_guids, xbox_one_fix)
                }
            } else {
                Self::generate_ini_multi(player_guids, xbox_one_fix)
            }
        } else {
            let template_ini = x360_src_dir.join("x360ce.ini");
            if template_ini.exists() {
                if let Ok(content) = file_util::read_text_file(&template_ini) {
                    Self::update_ini_mappings(&content, player_guids)
                } else {
                    Self::generate_ini_multi(player_guids, xbox_one_fix)
                }
            } else {
                Self::generate_ini_multi(player_guids, xbox_one_fix)
            }
        };

        file_util::write_text_file(&ini_path, &ini_content)?;
        Ok(ini_path)
    }

    /// Deploys x360ce files and writes `x360ce.ini` to the instance directory.
    pub fn setup_x360ce(
        instance_dir: &Path,
        player_guid: &str,
        utils_dir: &Path,
        is_64bit: bool,
        dll_names: &[&str],
        xbox_one_fix: bool,
    ) -> NucleusResult<PathBuf> {
        Self::setup_x360ce_multi(
            instance_dir,
            &[player_guid],
            utils_dir,
            is_64bit,
            dll_names,
            xbox_one_fix,
            0,
            None,
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_x360ce_generate_ini() {
        let guid = "12345678-abcd-1234-cdef-1234567890ab";
        let ini = X360ceTool::generate_ini(guid, false);
        assert!(ini.contains("PAD1=IG_12345678abcd1234cdef1234567890ab"));
        assert!(ini.contains("PAD2="));
        assert!(ini.contains("PAD3="));
        assert!(ini.contains("PAD4="));
        assert!(!ini.contains("[InputHook]"));

        let ini_fixed = X360ceTool::generate_ini(guid, true);
        assert!(ini_fixed.contains("[InputHook]"));
        assert!(ini_fixed.contains("HOOKDI=1"));
    }

    #[test]
    fn test_x360ce_multi_pad() {
        let guids = [
            "11111111-1111-1111-1111-111111111111",
            "22222222-2222-2222-2222-222222222222",
        ];
        let ini = X360ceTool::generate_ini_multi(&guids, false);
        assert!(ini.contains("PAD1=IG_11111111111111111111111111111111"));
        assert!(ini.contains("PAD2=IG_22222222222222222222222222222222"));
        assert!(ini.contains("PAD3="));
        assert!(ini.contains("PAD4="));
    }

    #[test]
    fn test_x360ce_update_ini_mappings() {
        let base_ini = "[Options]\r\nLog=0\r\n\r\n[Mappings]\r\nPAD1=IG_OLD1\r\nPAD2=IG_OLD2\r\nPAD3=\r\nPAD4=\r\n";
        let new_guids = ["33333333-3333-3333-3333-333333333333"];
        let updated = X360ceTool::update_ini_mappings(base_ini, &new_guids);
        assert!(updated.contains("PAD1=IG_33333333333333333333333333333333"));
        assert!(updated.contains("PAD2="));
        assert!(updated.contains("PAD3="));
        assert!(updated.contains("PAD4="));
    }

    #[test]
    fn test_x360ce_setup_multi_and_copy_instance0() {
        let temp_dir = std::env::temp_dir().join(format!("nucleus_x360ce_multi_test_{}", std::process::id()));
        let _ = std::fs::remove_dir_all(&temp_dir);
        let utils_dir = temp_dir.join("utils");
        let x360_utils = utils_dir.join("x360ce");
        file_util::ensure_dir(&x360_utils).unwrap();

        // Create dummy source DLLs
        file_util::write_text_file(&x360_utils.join("xinput1_3_x64.dll"), "xinput64").unwrap();
        file_util::write_text_file(&x360_utils.join("dinput8_x64.dll"), "dinput8_64").unwrap();

        let inst0 = temp_dir.join("Instance0");
        let inst1 = temp_dir.join("Instance1");

        // Setup Instance0 with 2 players
        let ini0 = X360ceTool::setup_x360ce_multi(
            &inst0,
            &["11111111-1111-1111-1111-111111111111", "22222222-2222-2222-2222-222222222222"],
            &utils_dir,
            true,
            &["dinput8.dll"],
            true,
            0,
            None,
        ).unwrap();
        assert!(ini0.exists());
        assert!(inst0.join("dinput8.dll").exists());
        let dinput_content = file_util::read_text_file(&inst0.join("dinput8.dll")).unwrap();
        assert_eq!(dinput_content, "dinput8_64");

        // Setup Instance1 carrying over from Instance0 with players 3 & 4
        let ini1 = X360ceTool::setup_x360ce_multi(
            &inst1,
            &["33333333-3333-3333-3333-333333333333", "44444444-4444-4444-4444-444444444444"],
            &utils_dir,
            true,
            &["dinput8.dll"],
            true,
            1,
            Some(&inst0),
        ).unwrap();
        assert!(ini1.exists());
        assert!(inst1.join("dinput8.dll").exists());

        let read_ini1 = file_util::read_text_file(&ini1).unwrap();
        assert!(read_ini1.contains("PAD1=IG_33333333333333333333333333333333"));
        assert!(read_ini1.contains("PAD2=IG_44444444444444444444444444444444"));

        let _ = std::fs::remove_dir_all(&temp_dir);
    }
}
