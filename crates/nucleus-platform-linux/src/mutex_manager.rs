//! Linux stub implementation of `MutexManager`.

#![cfg(target_os = "linux")]

use nucleus_common::error::{NucleusError, NucleusResult};
use nucleus_platform::MutexManager;

/// Linux mutex management stub.
#[derive(Default)]
pub struct LinuxMutexManager;

impl LinuxMutexManager {
    /// Creates a new `LinuxMutexManager`.
    pub fn new() -> Self {
        Self
    }
}

impl MutexManager for LinuxMutexManager {
    fn kill_mutex(&self, _pid: u32, _mutex_name: &str, _mutex_type: &str) -> NucleusResult<bool> {
        Err(NucleusError::Unsupported("Named mutex killing not supported on Linux".into()))
    }

    fn rename_mutex(&self, _pid: u32, _old_name: &str, _new_name: &str) -> NucleusResult<bool> {
        Err(NucleusError::Unsupported("Named mutex renaming not supported on Linux".into()))
    }

    fn find_mutex(&self, _pid: u32, _name: &str, _partial: bool) -> NucleusResult<Vec<String>> {
        Err(NucleusError::Unsupported("Mutex discovery not supported on Linux".into()))
    }
}
