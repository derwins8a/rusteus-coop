use nucleus_common::error::NucleusResult;
use nucleus_common::file_util;
use std::path::{Path, PathBuf};

/// Configuration options for patching or generating `XInputPlus.ini`.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct XInputPlusIniOptions<'a> {
    /// Target binary architecture (sets `FileVersion=x64` or `FileVersion=x86`).
    pub is_64bit: bool,
    /// When true, enables DirectInput translation (`EnableX2Dinput=True`).
    pub enable_x2dinput: bool,
    /// Controller index mapping for `Controller1`, `Controller2`, etc.
    pub controller_indices: &'a [u32],
}

impl Default for XInputPlusIniOptions<'_> {
    fn default() -> Self {
        Self {
            is_64bit: false,
            enable_x2dinput: false,
            controller_indices: &[1],
        }
    }
}

/// Runtime hook options written to `ncoop.ini` for the Nucleus Custom DLL (`xinput1_3.dll`).
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CustomDllHookOptions {
    pub force_focus: bool,
    pub force_focus_window_name: String,
    pub window_x: i32,
    pub window_y: i32,
    pub res_width: i32,
    pub res_height: i32,
    pub fix_resolution: bool,
    pub fix_position: bool,
    pub clip_mouse: bool,
    pub reroute_input: bool,
    pub reroute_joystick_template: u32,
    pub enable_mkb_input: bool,
    pub is_keyboard_player: bool,
    pub block_input_events: bool,
    pub block_mouse_events: bool,
    pub block_keyboard_events: bool,
    pub xinput_enabled: bool,
    pub xinput_player_id: u32,
    pub dinput_enabled: bool,
    pub dinput_guid: String,
    pub dinput_force_disable: bool,
}

impl Default for CustomDllHookOptions {
    fn default() -> Self {
        Self {
            force_focus: true,
            force_focus_window_name: String::new(),
            window_x: 0,
            window_y: 0,
            res_width: 1920,
            res_height: 1080,
            fix_resolution: false,
            fix_position: false,
            clip_mouse: false,
            reroute_input: false,
            reroute_joystick_template: 0,
            enable_mkb_input: false,
            is_keyboard_player: false,
            block_input_events: false,
            block_mouse_events: false,
            block_keyboard_events: false,
            xinput_enabled: true,
            xinput_player_id: 0,
            dinput_enabled: false,
            dinput_guid: String::new(),
            dinput_force_disable: false,
        }
    }
}

/// Tool for copying and generating configuration files for XInput Plus wrapper DLLs
/// and Nucleus Custom DLL hooks.
#[derive(Debug, Default, Clone)]
pub struct XInputPlusTool;

impl XInputPlusTool {
    /// Generates a basic `XInputPlus.ini` configuration mapping the instance to a specified controller index.
    pub fn generate_ini(controller_index: u32) -> String {
        Self::patch_or_generate_xinput_plus_ini(
            None,
            &XInputPlusIniOptions {
                is_64bit: false,
                enable_x2dinput: true,
                controller_indices: &[controller_index],
            },
        )
    }

    /// Patches an existing `XInputPlus.ini` template or generates a complete one from scratch.
    /// Sets `EnableX2Dinput=True`, `FileVersion=x64` / `x86`, and maps `Controller1=...`, `Controller2=...`.
    pub fn patch_or_generate_xinput_plus_ini(
        template_content: Option<&str>,
        options: &XInputPlusIniOptions,
    ) -> String {
        let arch = if options.is_64bit { "x64" } else { "x86" };

        if let Some(template) = template_content {
            let mut lines = Vec::new();
            let mut in_device = false;
            let mut in_direct_input = false;
            let mut found_device = false;
            let mut found_direct_input = false;
            let mut found_file_version = false;

            for line in template.lines() {
                let trimmed = line.trim();

                if trimmed.starts_with('[') && trimmed.ends_with(']') {
                    in_device = trimmed.eq_ignore_ascii_case("[Device]");
                    in_direct_input = trimmed.eq_ignore_ascii_case("[DirectInput]");
                    if in_device {
                        found_device = true;
                    }
                    if in_direct_input {
                        found_direct_input = true;
                    }
                    lines.push(line.to_string());
                    continue;
                }

                if in_device {
                    let mut is_controller = false;
                    for (i, &idx) in options.controller_indices.iter().enumerate() {
                        let prefix = format!("Controller{}=", i + 1);
                        if trimmed.starts_with(&prefix) {
                            lines.push(format!("Controller{}={}", i + 1, idx));
                            is_controller = true;
                            break;
                        }
                    }
                    if !is_controller {
                        lines.push(line.to_string());
                    }
                } else if in_direct_input {
                    if trimmed.starts_with("EnableX2Dinput=") {
                        lines.push(format!(
                            "EnableX2Dinput={}",
                            if options.enable_x2dinput {
                                "True"
                            } else {
                                "False"
                            }
                        ));
                    } else {
                        lines.push(line.to_string());
                    }
                } else if trimmed.starts_with("FileVersion=") {
                    lines.push(format!("FileVersion={}", arch));
                    found_file_version = true;
                } else {
                    lines.push(line.to_string());
                }
            }

            if !found_file_version {
                lines.push(format!("FileVersion={}", arch));
            }

            if !found_direct_input && options.enable_x2dinput {
                lines.push(String::new());
                lines.push("[DirectInput]".to_string());
                lines.push("Enable=1".to_string());
                lines.push("EnableX2Dinput=True".to_string());
            }

            if !found_device {
                lines.push(String::new());
                lines.push("[Device]".to_string());
                for (i, &idx) in options.controller_indices.iter().enumerate() {
                    lines.push(format!("Controller{}={}", i + 1, idx));
                }
            }

            lines.join("\r\n")
        } else {
            let mut lines = Vec::new();
            lines.push("[Device]".to_string());
            for (i, &idx) in options.controller_indices.iter().enumerate() {
                lines.push(format!("Controller{}={}", i + 1, idx));
            }
            lines.push("PassThrough=0".to_string());
            lines.push(String::new());

            lines.push("[DirectInput]".to_string());
            lines.push("Enable=1".to_string());
            if options.enable_x2dinput {
                lines.push("EnableX2Dinput=True".to_string());
            }
            lines.push(String::new());

            lines.push("[Version]".to_string());
            lines.push(format!("FileVersion={}", arch));
            lines.push(String::new());

            lines.join("\r\n")
        }
    }

    /// Deploys XInput Plus wrapper DLLs into the game instance folder and configures `XInputPlus.ini`.
    pub fn setup_xinput_plus(
        instance_dir: &Path,
        utils_dir: &Path,
        is_64bit: bool,
        dll_targets: &[&str],
        use_old_dll: bool,
    ) -> NucleusResult<Vec<PathBuf>> {
        Self::setup_xinput_plus_with_controllers(
            instance_dir,
            utils_dir,
            is_64bit,
            dll_targets,
            use_old_dll,
            &[1],
        )
    }

    /// Extended setup function deploying XInput Plus wrapper DLLs and mapping multiple controller slots.
    pub fn setup_xinput_plus_with_controllers(
        instance_dir: &Path,
        utils_dir: &Path,
        is_64bit: bool,
        dll_targets: &[&str],
        use_old_dll: bool,
        controller_indices: &[u32],
    ) -> NucleusResult<Vec<PathBuf>> {
        file_util::ensure_dir(instance_dir)?;

        let mut base_dir = utils_dir.join("XInputPlus");
        if use_old_dll {
            base_dir = base_dir.join("old");
        }
        let arch_dir = if is_64bit {
            base_dir.join("x64")
        } else {
            base_dir.join("x86")
        };

        let mut deployed_files = Vec::new();
        let targets = if dll_targets.is_empty() {
            vec!["xinput1_3.dll"]
        } else {
            dll_targets.to_vec()
        };

        let mut has_dinput = false;
        for &dll_name in &targets {
            let lower = dll_name.to_ascii_lowercase();
            let src_file_name = if lower.starts_with("dinput8") {
                has_dinput = true;
                "Dinput8.dl_"
            } else if lower.starts_with("dinput") {
                has_dinput = true;
                "Dinput.dl_"
            } else {
                "xinput1_3.dl_"
            };

            let src_path = arch_dir.join(src_file_name);
            let dst_path = instance_dir.join(dll_name);

            if src_path.exists() {
                let _ = file_util::copy_file(&src_path, &dst_path, true);
                deployed_files.push(dst_path);
            }
        }

        // Deploy and patch XInputPlus.ini
        let template_path = base_dir.join("XInputPlus.ini");
        let template_content = if template_path.exists() {
            file_util::read_text_file(&template_path).ok()
        } else {
            None
        };

        let ini_options = XInputPlusIniOptions {
            is_64bit,
            enable_x2dinput: has_dinput,
            controller_indices,
        };
        let patched_ini =
            Self::patch_or_generate_xinput_plus_ini(template_content.as_deref(), &ini_options);
        let ini_path = instance_dir.join("XInputPlus.ini");
        file_util::write_text_file(&ini_path, &patched_ini)?;
        deployed_files.push(ini_path);

        Ok(deployed_files)
    }

    /// Generates genuine `ncoop.ini` hook configuration for the Nucleus Custom DLL mode.
    pub fn generate_ncoop_ini(options: &CustomDllHookOptions) -> String {
        format!(
            "[Options]\r\n\
            Log=0\r\n\
            FileLog=0\r\n\
            ForceFocus={}\r\n\
            Version=2\r\n\
            ForceFocusWindowRegex={}\r\n\
            ForceFocusWindowName={}\r\n\
            WindowX={}\r\n\
            WindowY={}\r\n\
            ResWidth={}\r\n\
            ResHeight={}\r\n\
            FixResolution={}\r\n\
            FixPosition={}\r\n\
            ClipMouse={}\r\n\
            RerouteInput={}\r\n\
            RerouteJoystickTemplate={}\r\n\
            EnableMKBInput={}\r\n\
            IsKeyboardPlayer={}\r\n\
            BlockInputEvents={}\r\n\
            BlockMouseEvents={}\r\n\
            BlockKeyboardEvents={}\r\n\
            XInputEnabled={}\r\n\
            XInputPlayerID={}\r\n\
            DInputEnabled={}\r\n\
            DInputGuid={}\r\n\
            DInputForceDisable={}\r\n",
            if options.force_focus { "1" } else { "0" },
            options.force_focus_window_name,
            options.force_focus_window_name,
            options.window_x,
            options.window_y,
            options.res_width,
            options.res_height,
            if options.fix_resolution {
                "True"
            } else {
                "False"
            },
            if options.fix_position {
                "True"
            } else {
                "False"
            },
            if options.clip_mouse { "True" } else { "False" },
            if options.reroute_input {
                "True"
            } else {
                "False"
            },
            options.reroute_joystick_template,
            if options.enable_mkb_input {
                "True"
            } else {
                "False"
            },
            if options.is_keyboard_player {
                "True"
            } else {
                "False"
            },
            if options.block_input_events {
                "True"
            } else {
                "False"
            },
            if options.block_mouse_events {
                "True"
            } else {
                "False"
            },
            if options.block_keyboard_events {
                "True"
            } else {
                "False"
            },
            if options.xinput_enabled {
                "True"
            } else {
                "False"
            },
            options.xinput_player_id,
            if options.dinput_enabled {
                "True"
            } else {
                "False"
            },
            options.dinput_guid,
            if options.dinput_force_disable {
                "True"
            } else {
                "False"
            },
        )
    }

    /// Deploys the Nucleus Custom DLL (`xinput1_3.dll`) and writes `ncoop.ini` to the instance directory.
    pub fn setup_custom_dll(
        instance_dir: &Path,
        utils_dir: &Path,
        is_64bit: bool,
        dll_names: &[&str],
        options: &CustomDllHookOptions,
    ) -> NucleusResult<Vec<PathBuf>> {
        file_util::ensure_dir(instance_dir)?;

        let custom_dll_dir = utils_dir.join("CustomDll");
        let arch_dir = if is_64bit {
            custom_dll_dir.join("x64")
        } else {
            custom_dll_dir.join("x86")
        };

        let mut deployed = Vec::new();
        let targets = if dll_names.is_empty() {
            vec!["xinput1_3.dll"]
        } else {
            dll_names.to_vec()
        };

        for &target_dll in &targets {
            let dest = instance_dir.join(target_dll);
            // Search arch dir, then root custom dll dir
            let src1 = arch_dir.join(target_dll);
            let src2 = custom_dll_dir.join(target_dll);
            let fallback_name = if is_64bit {
                "xinput1_3_x64.dll"
            } else {
                "xinput1_3.dll"
            };
            let src3 = arch_dir.join(fallback_name);
            let src4 = custom_dll_dir.join(fallback_name);

            let src_candidate = if src1.exists() {
                Some(src1)
            } else if src2.exists() {
                Some(src2)
            } else if src3.exists() {
                Some(src3)
            } else if src4.exists() {
                Some(src4)
            } else {
                None
            };

            if let Some(src) = src_candidate {
                let _ = file_util::copy_file(&src, &dest, true);
                deployed.push(dest);
            }
        }

        let ini_content = Self::generate_ncoop_ini(options);
        let ini_path = instance_dir.join("ncoop.ini");
        file_util::write_text_file(&ini_path, &ini_content)?;
        deployed.push(ini_path);

        Ok(deployed)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_xinput_plus_generate_ini() {
        let ini = XInputPlusTool::generate_ini(1);
        assert!(ini.contains("Controller1=1"));
        assert!(ini.contains("[DirectInput]"));
        assert!(ini.contains("EnableX2Dinput=True"));
        assert!(ini.contains("FileVersion=x86"));
    }

    #[test]
    fn test_xinput_plus_patch_template() {
        let template = "[Device]\r\nController1=99\r\nPassThrough=0\r\n\r\n[DirectInput]\r\nEnableX2Dinput=False\r\n\r\nFileVersion=x86\r\n";
        let options = XInputPlusIniOptions {
            is_64bit: true,
            enable_x2dinput: true,
            controller_indices: &[2, 3],
        };
        let patched = XInputPlusTool::patch_or_generate_xinput_plus_ini(Some(template), &options);
        assert!(patched.contains("Controller1=2"));
        assert!(patched.contains("EnableX2Dinput=True"));
        assert!(patched.contains("FileVersion=x64"));
    }

    #[test]
    fn test_xinput_plus_setup() {
        let temp_dir =
            std::env::temp_dir().join(format!("nucleus_xinput_test_{}", std::process::id()));
        let _ = std::fs::remove_dir_all(&temp_dir);
        let utils_dir = temp_dir.join("utils");
        let arch_dir = utils_dir.join("XInputPlus").join("x64");
        file_util::ensure_dir(&arch_dir).unwrap();

        // Create dummy source dl_
        let dummy_src = arch_dir.join("xinput1_3.dl_");
        file_util::write_text_file(&dummy_src, "dummy dll content").unwrap();

        let instance_dir = temp_dir.join("instance0");
        let deployed = XInputPlusTool::setup_xinput_plus(
            &instance_dir,
            &utils_dir,
            true,
            &["xinput1_3.dll"],
            false,
        )
        .unwrap();

        assert_eq!(deployed.len(), 2); // dll + ini
        assert!(deployed[0].exists());
        assert!(deployed[1].exists());

        let _ = std::fs::remove_dir_all(&temp_dir);
    }

    #[test]
    fn test_custom_dll_ncoop_ini() {
        let mut opts = CustomDllHookOptions::default();
        opts.force_focus = true;
        opts.force_focus_window_name = "GameWindow".to_string();
        opts.res_width = 1280;
        opts.res_height = 720;
        opts.clip_mouse = true;
        opts.reroute_input = true;

        let ini = XInputPlusTool::generate_ncoop_ini(&opts);
        assert!(ini.contains("ForceFocus=1"));
        assert!(ini.contains("ForceFocusWindowName=GameWindow"));
        assert!(ini.contains("ResWidth=1280"));
        assert!(ini.contains("ResHeight=720"));
        assert!(ini.contains("ClipMouse=True"));
        assert!(ini.contains("RerouteInput=True"));
    }
}
