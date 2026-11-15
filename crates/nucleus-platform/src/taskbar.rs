//! Platform-agnostic taskbar visibility control operations.

use nucleus_common::error::NucleusResult;

/// Platform-agnostic taskbar visibility control operations.
pub trait TaskbarManager: Send + Sync {
    /// Hide the system taskbar/dock.
    fn hide(&self) -> NucleusResult<()>;

    /// Show the system taskbar/dock.
    fn show(&self) -> NucleusResult<()>;
}
