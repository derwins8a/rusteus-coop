//! Linux stub implementation of `TaskbarManager`.

#![cfg(target_os = "linux")]

use nucleus_common::error::{NucleusError, NucleusResult};
use nucleus_platform::TaskbarManager;

/// Linux taskbar / dock visibility manager stub.
#[derive(Default)]
pub struct LinuxTaskbarManager;

impl LinuxTaskbarManager {
    /// Creates a new `LinuxTaskbarManager`.
    pub fn new() -> Self {
        Self
    }
}

impl TaskbarManager for LinuxTaskbarManager {
    fn hide(&self) -> NucleusResult<()> {
        Err(NucleusError::Unsupported(
            "Taskbar toggle not supported on Linux".into(),
        ))
    }

    fn show(&self) -> NucleusResult<()> {
        Err(NucleusError::Unsupported(
            "Taskbar toggle not supported on Linux".into(),
        ))
    }
}
