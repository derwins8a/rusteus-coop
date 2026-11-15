//! Platform-agnostic user account management operations.

use std::path::PathBuf;

use nucleus_common::error::NucleusResult;
use nucleus_common::types::{ProcessConfig, ProcessHandle};

/// Platform-agnostic user account management operations (e.g. Nucleus user accounts for save isolation).
pub trait UserAccountManager: Send + Sync {
    /// Create a local user account.
    fn create_user(&self, username: &str, password: &str) -> NucleusResult<()>;

    /// Delete a local user account.
    fn delete_user(&self, username: &str) -> NucleusResult<()>;

    /// Check if a local user account exists.
    fn user_exists(&self, username: &str) -> NucleusResult<bool>;

    /// Launch a process under the credentials of a specific user.
    fn run_as_user(
        &self,
        username: &str,
        password: &str,
        config: &ProcessConfig,
    ) -> NucleusResult<ProcessHandle>;

    /// Retrieve the home/profile directory path for a user.
    fn get_user_profile_path(&self, username: &str) -> NucleusResult<PathBuf>;
}
