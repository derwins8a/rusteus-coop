use nucleus_common::error::NucleusResult;
use nucleus_common::file_util;
use std::path::{Path, PathBuf};

/// Tool for configuring DevReorder (dinput8.dll wrapper) to selectively expose or hide
/// DirectInput devices per game instance.
#[derive(Debug, Default, Clone)]
pub struct DevReorderTool;

impl DevReorderTool {
    /// Generates the contents of `devreorder.ini` given the visible gamepad GUID and the list
    /// of all participating gamepad GUIDs.
    pub fn generate_ini(visible_guid: &str, all_guids: &[&str]) -> String {
        let mut lines = Vec::new();
        lines.push("[order]".to_string());

        let trimmed_visible = visible_guid.trim().trim_matches(|c| c == '{' || c == '}');
        if !trimmed_visible.is_empty() {
            lines.push(format!("{{{}}}", trimmed_visible));
        }
        lines.push(String::new());

        lines.push("[hidden]".to_string());
        for &guid in all_guids {
            let trimmed = guid.trim().trim_matches(|c| c == '{' || c == '}');
            if !trimmed.is_empty() && !trimmed.eq_ignore_ascii_case(trimmed_visible) {
                lines.push(format!("{{{}}}", trimmed));
            }
        }
        lines.push(String::new());

        lines.join("\r\n")
    }

    /// Sets up DevReorder for an instance directory, deploying `devreorder.ini` and optionally `dinput8.dll`.
    pub fn setup_dev_reorder(
        instance_dir: &Path,
        visible_guid: &str,
        all_guids: &[&str],
        utils_dir: &Path,
        is_64bit: bool,
        copy_dll: bool,
    ) -> NucleusResult<PathBuf> {
        file_util::ensure_dir(instance_dir)?;

        if copy_dll {
            let arch_folder = if is_64bit { "x64" } else { "x86" };
            let src_dll = utils_dir
                .join("devreorder")
                .join(arch_folder)
                .join("dinput8.dll");
            let dst_dll = instance_dir.join("dinput8.dll");
            if src_dll.exists() {
                file_util::copy_file(&src_dll, &dst_dll, true)?;
            }
        }

        let ini_content = Self::generate_ini(visible_guid, all_guids);
        let ini_path = instance_dir.join("devreorder.ini");
        file_util::write_text_file(&ini_path, &ini_content)?;

        Ok(ini_path)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_devreorder_generate_ini() {
        let visible = "12345678-1234-1234-1234-123456789abc";
        let all = [
            "12345678-1234-1234-1234-123456789abc",
            "87654321-4321-4321-4321-cba987654321",
            "abcdef12-3456-7890-abcd-ef1234567890",
        ];

        let ini = DevReorderTool::generate_ini(visible, &all);
        assert!(ini.contains("[order]\r\n{12345678-1234-1234-1234-123456789abc}"));
        assert!(ini.contains("[hidden]"));
        assert!(ini.contains("{87654321-4321-4321-4321-cba987654321}"));
        assert!(ini.contains("{abcdef12-3456-7890-abcd-ef1234567890}"));
        assert!(!ini.contains("[hidden]\r\n{12345678-1234-1234-1234-123456789abc}"));
    }

    #[test]
    fn test_devreorder_setup() {
        let temp_dir =
            std::env::temp_dir().join(format!("nucleus_devreorder_test_{}", std::process::id()));
        let _ = std::fs::remove_dir_all(&temp_dir);
        let utils_dir = temp_dir.join("utils");
        let instance_dir = temp_dir.join("instance0");

        let visible = "00000000-0000-0000-0000-000000000001";
        let all = [visible, "00000000-0000-0000-0000-000000000002"];

        let ini_path = DevReorderTool::setup_dev_reorder(
            &instance_dir,
            visible,
            &all,
            &utils_dir,
            true,
            false,
        )
        .unwrap();

        assert!(ini_path.exists());
        let read_content = file_util::read_text_file(&ini_path).unwrap();
        assert!(read_content.contains("{00000000-0000-0000-0000-000000000001}"));
        assert!(read_content.contains("{00000000-0000-0000-0000-000000000002}"));

        let _ = std::fs::remove_dir_all(&temp_dir);
    }
}
