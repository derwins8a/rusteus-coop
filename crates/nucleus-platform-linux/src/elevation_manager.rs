//! Linux implementation of `ElevationManager`.

use nucleus_common::error::{NucleusError, NucleusResult};
use nucleus_platform::ElevationManager;
use std::path::Path;

/// Concrete Linux elevation manager.
#[derive(Default, Debug, Clone)]
pub struct LinuxElevationManager;

impl LinuxElevationManager {
    /// Creates a new `LinuxElevationManager`.
    pub fn new() -> Self {
        Self
    }
}

impl ElevationManager for LinuxElevationManager {
    fn is_elevated(&self) -> bool {
        #[cfg(target_os = "linux")]
        {
            unsafe { libc::geteuid() == 0 }
        }
        #[cfg(not(target_os = "linux"))]
        {
            false
        }
    }

    fn run_elevated(
        &self,
        executable: &Path,
        arguments: &[&str],
        working_dir: Option<&Path>,
        wait: bool,
    ) -> NucleusResult<Option<u32>> {
        let mut cmd = std::process::Command::new("pkexec");
        cmd.arg(executable);
        cmd.args(arguments);
        if let Some(dir) = working_dir {
            cmd.current_dir(dir);
        }

        if wait {
            let status = cmd
                .status()
                .map_err(|e| NucleusError::Platform(format!("Failed to execute pkexec: {}", e)))?;
            Ok(status.code().map(|c| c as u32))
        } else {
            cmd.spawn()
                .map_err(|e| NucleusError::Platform(format!("Failed to spawn pkexec: {}", e)))?;
            Ok(None)
        }
    }

    fn ensure_symlink_support(&self) -> NucleusResult<bool> {
        Ok(true)
    }
}
