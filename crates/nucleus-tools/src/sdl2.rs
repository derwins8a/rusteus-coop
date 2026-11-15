use std::path::{Path, PathBuf};
use nucleus_common::error::NucleusResult;
use nucleus_common::file_util;

/// Tool for generating SDL2 game controller environment variables and mapping files
/// to route distinct physical controllers to separate instances.
#[derive(Debug, Default, Clone)]
pub struct Sdl2Tool;

impl Sdl2Tool {
    /// Generates the contents of `SDL2.ini` specifying assigned gamepad indices.
    /// If no gamepad is assigned (e.g. keyboard player), it writes "-1".
    pub fn generate_ini(gamepad_id: Option<i32>, guest_gamepad_ids: &[i32]) -> String {
        match gamepad_id {
            Some(id) => {
                let mut parts = vec![id.to_string()];
                for &guest_id in guest_gamepad_ids {
                    parts.push(guest_id.to_string());
                }
                parts.join(" ")
            }
            None => "-1".to_string(),
        }
    }

    /// Sets up SDL2 for the instance directory by writing `SDL2.ini` and copying `SDL2.dll`
    /// to specified destination paths.
    pub fn setup_sdl2(
        instance_dir: &Path,
        gamepad_id: Option<i32>,
        guest_gamepad_ids: &[i32],
        utils_dir: &Path,
        custom_sdl_paths: &[&str],
    ) -> NucleusResult<PathBuf> {
        file_util::ensure_dir(instance_dir)?;

        // Write SDL2.ini
        let ini_content = Self::generate_ini(gamepad_id, guest_gamepad_ids);
        let ini_path = instance_dir.join("SDL2.ini");
        file_util::write_text_file(&ini_path, &ini_content)?;

        // Copy custom SDL2 dlls if requested
        let src_sdl2 = utils_dir.join("SDL2").join("SDL2.dll");
        if src_sdl2.exists() {
            if custom_sdl_paths.is_empty() {
                let dst = instance_dir.join("SDL2.dll");
                let _ = file_util::copy_file(&src_sdl2, &dst, true);
            } else {
                for &rel_path in custom_sdl_paths {
                    let dst = instance_dir.join(rel_path);
                    if let Some(parent) = dst.parent() {
                        let _ = file_util::ensure_dir(parent);
                    }
                    let _ = file_util::copy_file(&src_sdl2, &dst, true);
                }
            }
        }

        Ok(ini_path)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_sdl2_generate_ini() {
        assert_eq!(Sdl2Tool::generate_ini(None, &[]), "-1");
        assert_eq!(Sdl2Tool::generate_ini(Some(0), &[]), "0");
        assert_eq!(Sdl2Tool::generate_ini(Some(1), &[2, 3]), "1 2 3");
    }

    #[test]
    fn test_sdl2_setup() {
        let temp_dir = std::env::temp_dir().join(format!("nucleus_sdl2_test_{}", std::process::id()));
        let _ = std::fs::remove_dir_all(&temp_dir);
        let utils_dir = temp_dir.join("utils");
        let instance_dir = temp_dir.join("instance0");

        let ini_path = Sdl2Tool::setup_sdl2(
            &instance_dir,
            Some(2),
            &[3],
            &utils_dir,
            &[],
        ).unwrap();

        assert!(ini_path.exists());
        let content = file_util::read_text_file(&ini_path).unwrap();
        assert_eq!(content.trim(), "2 3");

        let _ = std::fs::remove_dir_all(&temp_dir);
    }
}

