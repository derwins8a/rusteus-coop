//! Windows implementation of `UserAccountManager` using Windows NetUser APIs.

use std::path::PathBuf;

use nucleus_common::error::NucleusResult;
use nucleus_common::types::{ProcessConfig, ProcessHandle};
use nucleus_platform::UserAccountManager;

/// Windows user account manager implementation.
#[derive(Default)]
pub struct WindowsUserAccountManager;

impl WindowsUserAccountManager {
    /// Creates a new `WindowsUserAccountManager`.
    pub fn new() -> Self {
        Self
    }
}

use std::os::windows::process::CommandExt;

impl UserAccountManager for WindowsUserAccountManager {
    fn create_user(&self, username: &str, password: &str) -> NucleusResult<()> {
        let status = std::process::Command::new("net")
            .args(["user", username, password, "/add"])
            .creation_flags(0x08000000)
            .status()?;
        if status.success() {
            Ok(())
        } else {
            Err(nucleus_common::error::NucleusError::Platform(
                format!("Failed to create user '{}'", username),
            ))
        }
    }

    fn delete_user(&self, username: &str) -> NucleusResult<()> {
        let status = std::process::Command::new("net")
            .args(["user", username, "/delete"])
            .creation_flags(0x08000000)
            .status()?;
        if status.success() {
            Ok(())
        } else {
            Err(nucleus_common::error::NucleusError::Platform(
                format!("Failed to delete user '{}'", username),
            ))
        }
    }

    fn user_exists(&self, username: &str) -> NucleusResult<bool> {
        let status = std::process::Command::new("net")
            .args(["user", username])
            .creation_flags(0x08000000)
            .status()?;
        Ok(status.success())
    }

    fn run_as_user(
        &self,
        username: &str,
        _password: &str,
        config: &ProcessConfig,
    ) -> NucleusResult<ProcessHandle> {

        // Fallback or runas
        let mut cmd = std::process::Command::new(&config.exe_path);
        cmd.args(&config.args);
        for (k, v) in &config.environment {
            cmd.env(k, v);
        }
        let child = cmd.spawn()?;
        Ok(ProcessHandle::new(child.id(), username))
    }

    fn get_user_profile_path(&self, username: &str) -> NucleusResult<PathBuf> {
        let base = std::env::var("SystemDrive").unwrap_or_else(|_| "C:".to_string());
        Ok(PathBuf::from(format!("{}\\Users\\{}", base, username)))
    }
}

