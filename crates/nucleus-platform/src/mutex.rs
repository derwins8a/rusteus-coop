//! Platform-agnostic mutex manipulation operations.

use nucleus_common::error::NucleusResult;

/// Platform-agnostic mutex manipulation operations (used to allow multi-instance game execution).
pub trait MutexManager: Send + Sync {
    /// Close/kill a named mutex belonging to a target process.
    ///
    /// Returns `true` if a mutex handle was found and closed.
    fn kill_mutex(&self, pid: u32, mutex_name: &str, mutex_type: &str) -> NucleusResult<bool>;

    /// Close/kill a named mutex belonging to a target process with extended matching options (partial search).
    fn kill_mutex_ext(
        &self,
        pid: u32,
        mutex_name: &str,
        mutex_type: &str,
        partial: bool,
    ) -> NucleusResult<bool> {
        let _ = partial;
        self.kill_mutex(pid, mutex_name, mutex_type)
    }

    /// Rename an existing mutex in a target process to prevent collision.
    ///
    /// Returns `true` if successfully renamed.
    fn rename_mutex(&self, pid: u32, old_name: &str, new_name: &str) -> NucleusResult<bool>;

    /// Find named mutexes held by a process (exact or partial matching).
    fn find_mutex(&self, pid: u32, name: &str, partial: bool) -> NucleusResult<Vec<String>>;
}
