//! DirectInput device blocker configuration and deployment.

use std::path::Path;
use nucleus_common::error::NucleusResult;
use nucleus_common::file_util;

/// Tool for deploying and configuring DirectInput blocking modules to prevent
/// dual-input detection on games that read both XInput and DirectInput concurrently.
#[derive(Debug, Default, Clone)]
pub struct DinputBlockerTool;

impl DinputBlockerTool {
    /// Generates INI content for configuring DirectInput device blocking.
    pub fn generate_ini(block_all: bool, allowed_guids: &[&str]) -> String {
        let mut ini = String::new();
        ini.push_str("[Settings]\r\n");
        ini.push_str(&format!("BlockAll={}\r\n", if block_all { "1" } else { "0" }));
        ini.push_str("\r\n[AllowedDevices]\r\n");
        for (idx, guid) in allowed_guids.iter().enumerate() {
            ini.push_str(&format!("Device{}={}\r\n", idx + 1, guid));
        }
        ini
    }

    /// Deploys `dinputblocker.ini` into the specified target game directory.
    pub fn setup_blocker(
        target_dir: &Path,
        block_all: bool,
        allowed_guids: &[&str],
    ) -> NucleusResult<()> {
        file_util::ensure_dir(target_dir)?;
        let ini_content = Self::generate_ini(block_all, allowed_guids);
        let ini_path = target_dir.join("dinputblocker.ini");
        file_util::write_text_file(&ini_path, &ini_content)?;
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_dinput_blocker_generate_ini() {
        let ini = DinputBlockerTool::generate_ini(true, &["{GUID-1234}"]);
        assert!(ini.contains("BlockAll=1"));
        assert!(ini.contains("Device1={GUID-1234}"));
    }

    #[test]
    fn test_setup_blocker() {
        let temp_dir = std::env::temp_dir().join(format!("nucleus_dinput_test_{}", std::process::id()));
        DinputBlockerTool::setup_blocker(&temp_dir, false, &["{DEV-1}", "{DEV-2}"]).unwrap();

        let ini_path = temp_dir.join("dinputblocker.ini");
        assert!(ini_path.exists());
        let content = file_util::read_text_file(&ini_path).unwrap();
        assert!(content.contains("BlockAll=0"));
        assert!(content.contains("Device1={DEV-1}"));
        assert!(content.contains("Device2={DEV-2}"));

        let _ = std::fs::remove_dir_all(&temp_dir);
    }
}
