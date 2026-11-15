//! Elevation and UAC helper tool.
//!
//! Provides facilities to check elevation status, run commands elevated with UAC prompts,
//! and coordinate operations requiring administrative privileges.

use std::path::Path;
use nucleus_common::error::{NucleusError, NucleusResult};

/// Elevator tool for requesting and coordinating administrator elevation.
#[derive(Debug, Default, Clone)]
pub struct ElevatorTool;

impl ElevatorTool {
    /// Checks whether the current process is running with administrative elevation.
    pub fn is_elevated() -> bool {
        #[cfg(windows)]
        {
            use nucleus_platform::ElevationManager;
            nucleus_platform_windows::WindowsElevationManager::new().is_elevated()
        }
        #[cfg(not(windows))]
        {
            false
        }
    }

    /// Executes a command with elevated administrator privileges, triggering the UAC prompt if needed.
    pub fn run_elevated(
        executable: &Path,
        arguments: &[&str],
        working_dir: Option<&Path>,
        wait: bool,
    ) -> NucleusResult<Option<u32>> {
        #[cfg(windows)]
        {
            use nucleus_platform::ElevationManager;
            nucleus_platform_windows::WindowsElevationManager::new().run_elevated(
                executable,
                arguments,
                working_dir,
                wait,
            )
        }
        #[cfg(not(windows))]
        {
            let _ = (executable, arguments, working_dir, wait);
            Err(NucleusError::Unsupported("run_elevated is not supported on this platform".to_string()))
        }
    }

    /// Re-launches the current executable with elevated administrator privileges.
    pub fn restart_elevated(arguments: &[&str]) -> NucleusResult<()> {
        let current_exe = std::env::current_exe()
            .map_err(|e| NucleusError::Platform(format!("Failed to determine current exe: {}", e)))?;
        Self::run_elevated(&current_exe, arguments, None, false)?;
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_is_elevated_call() {
        // Checking elevation should not panic
        let _ = ElevatorTool::is_elevated();
    }
}
