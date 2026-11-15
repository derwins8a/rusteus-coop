//! Game starter utilities for handling secondary launcher processes and launch coordination.

use std::collections::HashMap;
use std::path::{Path, PathBuf};
use std::process::Command;
use std::time::Duration;
use nucleus_common::error::{NucleusError, NucleusResult};

/// Specification for launching a game or helper instance.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct LaunchSpec {
    /// Absolute path to executable.
    pub executable: PathBuf,
    /// Command line arguments.
    pub arguments: Vec<String>,
    /// Working directory for execution.
    pub working_dir: Option<PathBuf>,
    /// Custom environment variables.
    pub environment: HashMap<String, String>,
    /// Optional delay before starting the instance in milliseconds.
    pub launch_delay_ms: u64,
}

impl LaunchSpec {
    /// Creates a new launch spec with executable and arguments.
    pub fn new(executable: impl Into<PathBuf>, arguments: Vec<String>) -> Self {
        Self {
            executable: executable.into(),
            arguments,
            working_dir: None,
            environment: HashMap::new(),
            launch_delay_ms: 0,
        }
    }

    /// Sets the working directory.
    pub fn with_working_dir(mut self, dir: impl Into<PathBuf>) -> Self {
        self.working_dir = Some(dir.into());
        self
    }

    /// Adds an environment variable.
    pub fn with_env(mut self, key: impl Into<String>, val: impl Into<String>) -> Self {
        self.environment.insert(key.into(), val.into());
        self
    }

    /// Sets launch delay in milliseconds.
    pub fn with_delay(mut self, delay_ms: u64) -> Self {
        self.launch_delay_ms = delay_ms;
        self
    }
}

/// Tool for coordinating execution when games use intermediate launchers or bootstrap executables
/// before spawning the actual game window.
#[derive(Debug, Default, Clone)]
pub struct GameStarterTool;

impl GameStarterTool {
    /// Builds a `std::process::Command` from the given launch specification.
    pub fn build_command(spec: &LaunchSpec) -> Command {
        let mut cmd = Command::new(&spec.executable);
        cmd.args(&spec.arguments);

        if let Some(ref wd) = spec.working_dir {
            cmd.current_dir(wd);
        } else if let Some(parent) = spec.executable.parent() {
            cmd.current_dir(parent);
        }

        for (k, v) in &spec.environment {
            cmd.env(k, v);
        }

        cmd
    }

    /// Formats a full command line string from an executable path and list of arguments.
    pub fn format_command_line(executable: &Path, args: &[String]) -> String {
        let mut parts = Vec::new();
        let exe_str = executable.to_string_lossy();
        if exe_str.contains(' ') {
            parts.push(format!("\"{}\"", exe_str));
        } else {
            parts.push(exe_str.to_string());
        }

        for arg in args {
            if arg.contains(' ') && !arg.starts_with('"') {
                parts.push(format!("\"{}\"", arg));
            } else {
                parts.push(arg.clone());
            }
        }

        parts.join(" ")
    }

    /// Calculates staggered launch delays for a sequence of instances.
    pub fn calculate_staggered_delays(num_instances: usize, base_delay_ms: u64) -> Vec<u64> {
        (0..num_instances)
            .map(|i| (i as u64) * base_delay_ms)
            .collect()
    }

    /// Sleeps for the specified launch delay if non-zero.
    pub fn apply_delay(delay_ms: u64) {
        if delay_ms > 0 {
            std::thread::sleep(Duration::from_millis(delay_ms));
        }
    }

    /// Launches the process described by the specification and returns its spawned PID.
    pub fn spawn_process(spec: &LaunchSpec) -> NucleusResult<u32> {
        Self::apply_delay(spec.launch_delay_ms);

        let mut cmd = Self::build_command(spec);
        let child = cmd.spawn().map_err(|e| {
            NucleusError::Process(format!("Failed to spawn {}: {}", spec.executable.display(), e))
        })?;

        Ok(child.id())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_format_command_line() {
        let exe = Path::new("C:\\Games\\My Game\\game.exe");
        let args = vec!["-windowed".to_string(), "-player 1".to_string()];
        let cmd_line = GameStarterTool::format_command_line(exe, &args);

        assert_eq!(cmd_line, "\"C:\\Games\\My Game\\game.exe\" -windowed \"-player 1\"");
    }

    #[test]
    fn test_staggered_delays() {
        let delays = GameStarterTool::calculate_staggered_delays(4, 2500);
        assert_eq!(delays, vec![0, 2500, 5000, 7500]);
    }

    #[test]
    fn test_build_command() {
        let spec = LaunchSpec::new("test.exe", vec!["--arg1".to_string()])
            .with_env("TEST_VAR", "VALUE");
        let cmd = GameStarterTool::build_command(&spec);
        assert_eq!(cmd.get_program(), "test.exe");
    }
}
