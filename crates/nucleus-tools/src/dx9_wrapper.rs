//! DirectX 9 wrapper and DLL loader configuration.

use nucleus_common::error::NucleusResult;
use nucleus_common::file_util;
use std::path::Path;

/// Configuration options for the DirectX 9 wrapper.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Dx9WrapperConfig {
    /// Force the rendering device into windowed mode.
    pub force_windowed: bool,
    /// Remove window borders when rendered.
    pub borderless: bool,
    /// Target width override.
    pub target_width: Option<u32>,
    /// Target height override.
    pub target_height: Option<u32>,
    /// Confine or clip cursor inside the instance window.
    pub clip_cursor: bool,
    /// Force vertical sync on/off.
    pub vsync: bool,
    /// Target frame rate limit (e.g. 60).
    pub fps_limit: Option<u32>,
    /// Optional chained forward DLL name (e.g. "d3d9_original.dll").
    pub forward_dll: Option<String>,
}

impl Default for Dx9WrapperConfig {
    fn default() -> Self {
        Self {
            force_windowed: true,
            borderless: true,
            target_width: None,
            target_height: None,
            clip_cursor: false,
            vsync: false,
            fps_limit: None,
            forward_dll: None,
        }
    }
}

/// Tool for configuring and deploying DirectX 9 wrappers to resolve fullscreen exclusivity issues,
/// hook window messages, and enable borderless windowed mode.
#[derive(Debug, Default, Clone)]
pub struct Dx9WrapperTool;

impl Dx9WrapperTool {
    /// Generates INI content for configuring the DirectX 9 wrapper.
    pub fn generate_ini(config: &Dx9WrapperConfig) -> String {
        let mut ini = String::new();
        ini.push_str("[Settings]\r\n");
        ini.push_str(&format!(
            "ForceWindowed={}\r\n",
            if config.force_windowed { "1" } else { "0" }
        ));
        ini.push_str(&format!(
            "Borderless={}\r\n",
            if config.borderless { "1" } else { "0" }
        ));
        ini.push_str(&format!(
            "ClipCursor={}\r\n",
            if config.clip_cursor { "1" } else { "0" }
        ));
        ini.push_str(&format!(
            "VSync={}\r\n",
            if config.vsync { "1" } else { "0" }
        ));

        if let Some(w) = config.target_width {
            ini.push_str(&format!("TargetWidth={}\r\n", w));
        }
        if let Some(h) = config.target_height {
            ini.push_str(&format!("TargetHeight={}\r\n", h));
        }
        if let Some(fps) = config.fps_limit {
            ini.push_str(&format!("FPSLimit={}\r\n", fps));
        }
        if let Some(ref forward) = config.forward_dll {
            ini.push_str(&format!("ForwardDLL={}\r\n", forward));
        }

        ini
    }

    /// Deploys `d3d9.ini` (and optionally copies `d3d9.dll`) to the target game directory.
    pub fn setup_wrapper(
        target_dir: &Path,
        config: &Dx9WrapperConfig,
        wrapper_dll_source: Option<&Path>,
    ) -> NucleusResult<()> {
        file_util::ensure_dir(target_dir)?;

        let ini_content = Self::generate_ini(config);
        let ini_path = target_dir.join("d3d9.ini");
        file_util::write_text_file(&ini_path, &ini_content)?;

        if let Some(src_dll) = wrapper_dll_source {
            if src_dll.exists() {
                let dest_dll = target_dir.join("d3d9.dll");
                file_util::copy_file(src_dll, &dest_dll, true)?;
            }
        }

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_dx9_wrapper_generate_ini() {
        let config = Dx9WrapperConfig {
            force_windowed: true,
            borderless: true,
            target_width: Some(1920),
            target_height: Some(1080),
            clip_cursor: true,
            vsync: false,
            fps_limit: Some(60),
            forward_dll: Some("d3d9_chain.dll".to_string()),
        };
        let ini = Dx9WrapperTool::generate_ini(&config);
        assert!(ini.contains("ForceWindowed=1"));
        assert!(ini.contains("Borderless=1"));
        assert!(ini.contains("ClipCursor=1"));
        assert!(ini.contains("TargetWidth=1920"));
        assert!(ini.contains("TargetHeight=1080"));
        assert!(ini.contains("FPSLimit=60"));
        assert!(ini.contains("ForwardDLL=d3d9_chain.dll"));
    }

    #[test]
    fn test_dx9_wrapper_setup() {
        let temp_dir =
            std::env::temp_dir().join(format!("nucleus_dx9_test_{}", std::process::id()));
        let config = Dx9WrapperConfig::default();
        Dx9WrapperTool::setup_wrapper(&temp_dir, &config, None).unwrap();

        let ini_file = temp_dir.join("d3d9.ini");
        assert!(ini_file.exists());
        let content = file_util::read_text_file(&ini_file).unwrap();
        assert!(content.contains("ForceWindowed=1"));

        let _ = std::fs::remove_dir_all(&temp_dir);
    }
}
