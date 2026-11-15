//! Process launching and process tracking.

use nucleus_common::error::{NucleusError, NucleusResult};
use std::path::Path;
use std::process::Command;

/// Responsible for executing game binaries with custom arguments, environment, and working directory.
pub struct ProcessLauncher;

impl ProcessLauncher {
    /// Launches an executable process and returns the spawned process ID.
    pub fn launch<I, K, V>(
        executable: &Path,
        args: &[String],
        working_dir: &Path,
        environment: I,
    ) -> NucleusResult<u32>
    where
        I: IntoIterator<Item = (K, V)>,
        K: AsRef<std::ffi::OsStr>,
        V: AsRef<std::ffi::OsStr>,
    {
        let mut cmd = Command::new(executable);
        cmd.args(args);
        cmd.current_dir(working_dir);

        for (k, v) in environment {
            cmd.env(k, v);
        }

        let child = cmd.spawn().map_err(|e| {
            NucleusError::Process(format!("Failed to spawn {:?}: {}", executable, e))
        })?;

        Ok(child.id())
    }

    /// Launches an executable process with optional working directory and returns the spawned process ID.
    pub fn launch_process<I, K, V>(
        executable: &Path,
        args: &[String],
        working_dir: Option<&Path>,
        environment: I,
    ) -> NucleusResult<u32>
    where
        I: IntoIterator<Item = (K, V)>,
        K: AsRef<std::ffi::OsStr>,
        V: AsRef<std::ffi::OsStr>,
    {
        let fallback = executable.parent().unwrap_or_else(|| Path::new("."));
        let wd = working_dir.unwrap_or(fallback);
        Self::launch(executable, args, wd, environment)
    }

    /// Terminates a process by its process ID.
    pub fn terminate_process(pid: u32) -> NucleusResult<()> {
        Self::terminate(pid)
    }

    /// Terminates a process by its process ID.
    pub fn terminate(pid: u32) -> NucleusResult<()> {
        #[cfg(windows)]
        {
            let status = Command::new("taskkill")
                .args(["/T", "/F", "/PID", &pid.to_string()])
                .output()
                .map_err(|e| NucleusError::Process(e.to_string()))?;

            if !status.status.success() {
                // If it already exited, that's acceptable
                return Ok(());
            }
        }

        #[cfg(not(windows))]
        {
            let _ = Command::new("kill").args(["-9", &pid.to_string()]).status();
        }

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_process_launch_and_terminate() {
        #[cfg(windows)]
        let (prog, args) = (
            "cmd.exe",
            vec!["/c".to_string(), "timeout /t 5".to_string()],
        );
        #[cfg(not(windows))]
        let (prog, args) = ("sleep", vec!["5".to_string()]);

        let cur_dir = std::env::current_dir().unwrap();
        let pid = ProcessLauncher::launch(
            Path::new(prog),
            &args,
            &cur_dir,
            std::iter::empty::<(&str, &str)>(),
        )
        .unwrap();
        assert!(pid > 0);

        ProcessLauncher::terminate(pid).unwrap();
    }
}
