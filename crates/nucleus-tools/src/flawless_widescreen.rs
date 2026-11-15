//! Flawless Widescreen integration, display detection overrides, and multi-instance module cloning.

use std::fs;
use std::path::{Path, PathBuf};
use nucleus_common::error::{NucleusError, NucleusResult};
use nucleus_common::file_util;

/// Tool for deploying Flawless Widescreen configurations, patching display detection overrides,
/// cloning game modules per-instance, and injecting instanced plugin entries into `FWS_Plugins.fws`.
#[derive(Debug, Default, Clone)]
pub struct FlawlessWidescreenTool;

impl FlawlessWidescreenTool {
    /// Configures legacy CFG plugin settings for a specific game instance to fix HUD scaling and FOV.
    pub fn configure_plugin(
        plugin_dir: &Path,
        game_name: &str,
        width: i32,
        height: i32,
    ) -> NucleusResult<()> {
        file_util::ensure_dir(plugin_dir)?;
        let config_file = plugin_dir.join(format!("{}.cfg", game_name.to_lowercase().replace(' ', "_")));
        let content = format!(
            "[Display]\r\nWidth={}\r\nHeight={}\r\nAspectRatio={:.4}\r\nFixHUD=1\r\nFixFOV=1\r\n",
            width,
            height,
            if height > 0 { width as f64 / height as f64 } else { 1.7778 }
        );
        file_util::write_text_file(&config_file, &content)?;
        Ok(())
    }

    /// Patches `settings.xml` in Flawless Widescreen utils to enable display detection override
    /// and specify target width and height for player instances.
    /// Creates `settings_NUCLEUS_BACKUP.xml` before modification if not already present.
    pub fn patch_settings_xml(
        settings_path: &Path,
        width: u32,
        height: u32,
    ) -> NucleusResult<()> {
        if let Some(parent) = settings_path.parent() {
            file_util::ensure_dir(parent)?;
        }

        let backup_path = if let Some(parent) = settings_path.parent() {
            parent.join("settings_NUCLEUS_BACKUP.xml")
        } else {
            PathBuf::from("settings_NUCLEUS_BACKUP.xml")
        };

        if settings_path.exists() && !backup_path.exists() {
            let _ = fs::copy(settings_path, &backup_path);
        }

        let existing_xml = if settings_path.exists() {
            file_util::read_text_file(settings_path).unwrap_or_default()
        } else {
            String::new()
        };

        let updated_xml = if existing_xml.is_empty() {
            format!(
                "<Configuration>\r\n  <DisplayDetectionSettings>\r\n    <CustomSettings Enabled=\"true\" Width=\"{}\" Height=\"{}\" />\r\n  </DisplayDetectionSettings>\r\n</Configuration>",
                width, height
            )
        } else if existing_xml.contains("<CustomSettings") {
            // Replace existing CustomSettings node
            let mut result = String::new();
            for line in existing_xml.lines() {
                if line.trim().starts_with("<CustomSettings") {
                    result.push_str(&format!(
                        "    <CustomSettings Enabled=\"true\" Width=\"{}\" Height=\"{}\" />\r\n",
                        width, height
                    ));
                } else {
                    result.push_str(line);
                    result.push_str("\r\n");
                }
            }
            result
        } else if let Some(pos) = existing_xml.find("</DisplayDetectionSettings>") {
            let (before, after) = existing_xml.split_at(pos);
            format!(
                "{}    <CustomSettings Enabled=\"true\" Width=\"{}\" Height=\"{}\" />\r\n{}",
                before, width, height, after
            )
        } else if let Some(pos) = existing_xml.find("</Configuration>") {
            let (before, after) = existing_xml.split_at(pos);
            format!(
                "{}  <DisplayDetectionSettings>\r\n    <CustomSettings Enabled=\"true\" Width=\"{}\" Height=\"{}\" />\r\n  </DisplayDetectionSettings>\r\n{}",
                before, width, height, after
            )
        } else {
            format!(
                "<Configuration>\r\n  <DisplayDetectionSettings>\r\n    <CustomSettings Enabled=\"true\" Width=\"{}\" Height=\"{}\" />\r\n  </DisplayDetectionSettings>\r\n</Configuration>",
                width, height
            )
        };

        file_util::write_text_file(settings_path, &updated_xml)?;
        Ok(())
    }

    /// Recursively copies directory tree from `src` to `dst`.
    fn copy_dir_all(src: &Path, dst: &Path) -> NucleusResult<()> {
        file_util::ensure_dir(dst)?;
        for entry in fs::read_dir(src)?.flatten() {
            let path = entry.path();
            let dest_path = dst.join(entry.file_name());
            if path.is_dir() {
                Self::copy_dir_all(&path, &dest_path)?;
            } else {
                let _ = fs::copy(&path, &dest_path);
            }
        }
        Ok(())
    }

    /// Clones the game's Flawless Widescreen module folder for a player instance,
    /// renames the Lua script, and rewrites `Process_WindowName` to target the specific instance window title.
    /// Also sets `Process_WindowName = "Removed"` in the original module script to prevent instance conflicts.
    pub fn clone_and_patch_module(
        modules_dir: &Path,
        game_module_name: &str,
        instance_index: usize,
        window_title: &str,
    ) -> NucleusResult<PathBuf> {
        let orig_folder = modules_dir.join(game_module_name);
        if !orig_folder.exists() {
            return Err(NucleusError::Other(format!(
                "Flawless Widescreen module directory not found: {}",
                orig_folder.display()
            )));
        }

        // Set Process_WindowName = "Removed" in original script
        let orig_script = orig_folder
            .join("Dependencies")
            .join("Scripts")
            .join(format!("{}.lua", game_module_name));
        if orig_script.exists() {
            if let Ok(content) = file_util::read_text_file(&orig_script) {
                let mut patched = Vec::new();
                for line in content.lines() {
                    if line.trim().starts_with("Process_WindowName = ") {
                        patched.push("Process_WindowName = \"Removed\"".to_string());
                    } else {
                        patched.push(line.to_string());
                    }
                }
                let _ = file_util::write_text_file(&orig_script, &patched.join("\r\n"));
            }
        }

        let new_folder_name = format!("{} - Nucleus Instance {}", game_module_name, instance_index + 1);
        let new_folder = modules_dir.join(&new_folder_name);

        if new_folder.exists() {
            let _ = fs::remove_dir_all(&new_folder);
        }
        Self::copy_dir_all(&orig_folder, &new_folder)?;

        // Rename Lua script and update Process_WindowName in cloned module
        let old_cloned_script = new_folder
            .join("Dependencies")
            .join("Scripts")
            .join(format!("{}.lua", game_module_name));
        let new_script_name = format!("{} - Nucleus Instance {}.lua", game_module_name, instance_index + 1);
        let new_cloned_script = new_folder
            .join("Dependencies")
            .join("Scripts")
            .join(new_script_name);

        if old_cloned_script.exists() {
            let _ = fs::rename(&old_cloned_script, &new_cloned_script);
        }

        if new_cloned_script.exists() {
            let content = file_util::read_text_file(&new_cloned_script)?;
            let mut patched_lines = Vec::new();
            let safe_title = window_title.replace('®', "%R");
            let target_line = format!(
                "Process_WindowName = \"Nucleus Instance {}({})\"",
                instance_index + 1,
                safe_title
            );

            for line in content.lines() {
                if line.trim().starts_with("Process_WindowName = ") {
                    patched_lines.push(target_line.clone());
                } else {
                    patched_lines.push(line.to_string());
                }
            }
            file_util::write_text_file(&new_cloned_script, &patched_lines.join("\r\n"))?;
        }

        Ok(new_folder)
    }

    /// Injects an instanced `<Module>` entry into the `FWS_Plugins.fws` XML file
    /// referencing the cloned module folder.
    pub fn inject_plugin_fws(
        fws_xml_path: &Path,
        game_module_name: &str,
        instance_index: usize,
    ) -> NucleusResult<()> {
        if !fws_xml_path.exists() {
            return Err(NucleusError::Other(format!(
                "FWS_Plugins.fws file not found at: {}",
                fws_xml_path.display()
            )));
        }

        let content = file_util::read_text_file(fws_xml_path)?;
        let instanced_namespace = format!("{} - Nucleus Instance {}", game_module_name, instance_index + 1);

        // Check if module is already present
        if content.contains(&format!("NameSpace=\"{}\"", instanced_namespace)) {
            return Ok(());
        }

        // Find original module block
        let pattern_start = format!("NameSpace=\"{}\"", game_module_name);
        let orig_pos = match content.find(&pattern_start) {
            Some(p) => p,
            None => {
                return Err(NucleusError::Other(format!(
                    "Module with NameSpace='{}' not found in {}",
                    game_module_name,
                    fws_xml_path.display()
                )));
            }
        };

        // Find the beginning of the <Module tag preceding NameSpace
        let module_start = match content[..orig_pos].rfind("<Module") {
            Some(p) => p,
            None => return Err(NucleusError::Other("Malformed <Module> tag in XML".to_string())),
        };

        // Find the end of this </Module> block
        let module_end = match content[orig_pos..].find("</Module>") {
            Some(p) => orig_pos + p + "</Module>".len(),
            None => return Err(NucleusError::Other("Closing </Module> tag not found in XML".to_string())),
        };

        let orig_block = &content[module_start..module_end];

        // Clone block and substitute NameSpace and FriendlyName
        let new_block = orig_block
            .replace(
                &format!("NameSpace=\"{}\"", game_module_name),
                &format!("NameSpace=\"{}\"", instanced_namespace),
            )
            .replace(
                &format!("<FriendlyName>{}</FriendlyName>", game_module_name),
                &format!("<FriendlyName>{} - Nucleus Instance {}</FriendlyName>", game_module_name, instance_index + 1),
            );

        // Insert new module before </Modules>
        let updated_content = if let Some(pos) = content.rfind("</Modules>") {
            let (before, after) = content.split_at(pos);
            format!("{}\r\n    {}\r\n  {}", before.trim_end(), new_block, after)
        } else {
            format!("{}\r\n{}", content, new_block)
        };

        file_util::write_text_file(fws_xml_path, &updated_content)?;
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_patch_settings_xml() {
        let temp_dir = std::env::temp_dir().join(format!("nucleus_fws_set_test_{}", std::process::id()));
        let settings = temp_dir.join("settings.xml");

        // Fresh creation
        FlawlessWidescreenTool::patch_settings_xml(&settings, 2560, 1440).unwrap();
        assert!(settings.exists());
        let read = file_util::read_text_file(&settings).unwrap();
        assert!(read.contains("Width=\"2560\""));
        assert!(read.contains("Height=\"1440\""));
        assert!(read.contains("Enabled=\"true\""));

        // Update existing
        FlawlessWidescreenTool::patch_settings_xml(&settings, 3840, 2160).unwrap();
        let read_updated = file_util::read_text_file(&settings).unwrap();
        assert!(read_updated.contains("Width=\"3840\""));
        assert!(read_updated.contains("Height=\"2160\""));
        assert!(temp_dir.join("settings_NUCLEUS_BACKUP.xml").exists());

        let _ = std::fs::remove_dir_all(&temp_dir);
    }

    #[test]
    fn test_clone_and_patch_module() {
        let temp_dir = std::env::temp_dir().join(format!("nucleus_fws_clone_test_{}", std::process::id()));
        let orig_module = temp_dir.join("DarkSoulsRemastered");
        let script_dir = orig_module.join("Dependencies").join("Scripts");
        file_util::ensure_dir(&script_dir).unwrap();

        let lua_file = script_dir.join("DarkSoulsRemastered.lua");
        file_util::write_text_file(
            &lua_file,
            "-- Flawless Widescreen Script\r\nProcess_WindowName = \"DARK SOULS\"\r\nprint('init')\r\n",
        ).unwrap();

        let cloned_dir = FlawlessWidescreenTool::clone_and_patch_module(
            &temp_dir,
            "DarkSoulsRemastered",
            0,
            "DARK SOULS®",
        ).unwrap();

        assert!(cloned_dir.exists());
        let cloned_script = cloned_dir
            .join("Dependencies")
            .join("Scripts")
            .join("DarkSoulsRemastered - Nucleus Instance 1.lua");
        assert!(cloned_script.exists());

        let cloned_content = file_util::read_text_file(&cloned_script).unwrap();
        assert!(cloned_content.contains("Process_WindowName = \"Nucleus Instance 1(DARK SOULS%R)\""));

        // Original script should be set to "Removed"
        let orig_content = file_util::read_text_file(&lua_file).unwrap();
        assert!(orig_content.contains("Process_WindowName = \"Removed\""));

        let _ = std::fs::remove_dir_all(&temp_dir);
    }

    #[test]
    fn test_inject_plugin_fws() {
        let temp_dir = std::env::temp_dir().join(format!("nucleus_fws_inject_test_{}", std::process::id()));
        file_util::ensure_dir(&temp_dir).unwrap();
        let fws_file = temp_dir.join("FWS_Plugins.fws");

        let xml = "<Plugin>\r\n  <Modules>\r\n    <Module NameSpace=\"DarkSoulsRemastered\">\r\n      <FriendlyName>DarkSoulsRemastered</FriendlyName>\r\n    </Module>\r\n  </Modules>\r\n</Plugin>";
        file_util::write_text_file(&fws_file, xml).unwrap();

        FlawlessWidescreenTool::inject_plugin_fws(&fws_file, "DarkSoulsRemastered", 0).unwrap();

        let read = file_util::read_text_file(&fws_file).unwrap();
        assert!(read.contains("NameSpace=\"DarkSoulsRemastered - Nucleus Instance 1\""));
        assert!(read.contains("<FriendlyName>DarkSoulsRemastered - Nucleus Instance 1</FriendlyName>"));

        let _ = std::fs::remove_dir_all(&temp_dir);
    }
}
