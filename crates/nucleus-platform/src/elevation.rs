//! Elevation management traits.
//!
//! Provides platform-agnostic checking and requesting of elevated (administrator/root) privileges.

use std::path::Path;
use nucleus_common::error::NucleusResult;

/// Trait defining privilege elevation inspection and elevated process execution.
pub trait ElevationManager: Send + Sync {
    /// Returns whether the current process is running with administrative / elevated privileges.
    fn is_elevated(&self) -> bool;

    /// Executes a command or process with elevated privileges (UAC prompt on Windows).
    ///
    /// If `wait` is true, waits for the elevated process to terminate and returns its exit code.
    fn run_elevated(
        &self,
        executable: &Path,
        arguments: &[&str],
        working_dir: Option<&Path>,
        wait: bool,
    ) -> NucleusResult<Option<u32>>;

    /// Verifies whether the OS supports creating symbolic links without Administrator privileges.
    /// If not, invokes the elevator helper to enable Developer Mode or grant necessary privileges.
    fn ensure_symlink_support(&self) -> NucleusResult<bool>;
}
