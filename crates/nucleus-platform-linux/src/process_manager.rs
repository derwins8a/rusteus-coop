//! Linux stub implementation of `ProcessManager`.

#![cfg(target_os = "linux")]

use nucleus_common::error::{NucleusError, NucleusResult};
use nucleus_common::types::{ProcessConfig, ProcessHandle, ProcessInfo, ProcessPriority};
use nucleus_platform::ProcessManager;

/// Linux process management stub.
#[derive(Default)]
pub struct LinuxProcessManager;

impl LinuxProcessManager {
    /// Creates a new `LinuxProcessManager`.
    pub fn new() -> Self {
        Self
    }
}

impl ProcessManager for LinuxProcessManager {
    fn start_process(&self, _config: &ProcessConfig) -> NucleusResult<ProcessHandle> {
        Err(NucleusError::Unsupported("Process spawning not yet implemented on Linux".into()))
    }

    fn kill_process(&self, _pid: u32) -> NucleusResult<()> {
        Err(NucleusError::Unsupported("Process termination not yet implemented on Linux".into()))
    }

    fn get_process_info(&self, _pid: u32) -> NucleusResult<ProcessInfo> {
        Err(NucleusError::Unsupported("Process info querying not yet implemented on Linux".into()))
    }

    fn is_process_running(&self, _pid: u32) -> bool {
        false
    }

    fn set_affinity(&self, _pid: u32, _mask: u64) -> NucleusResult<()> {
        Err(NucleusError::Unsupported("CPU affinity setting not yet implemented on Linux".into()))
    }

    fn set_priority(&self, _pid: u32, _priority: ProcessPriority) -> NucleusResult<()> {
        Err(NucleusError::Unsupported("Process priority setting not yet implemented on Linux".into()))
    }

    fn get_processes_by_name(&self, _name: &str) -> NucleusResult<Vec<ProcessInfo>> {
        Err(NucleusError::Unsupported("Process search not yet implemented on Linux".into()))
    }

    fn wait_for_process(&self, _pid: u32, _timeout_ms: u32) -> NucleusResult<bool> {
        Err(NucleusError::Unsupported("Process waiting not yet implemented on Linux".into()))
    }

    fn suspend_process(&self, _pid: u32) -> NucleusResult<()> {
        Err(NucleusError::Unsupported("Process suspension not yet implemented on Linux".into()))
    }

    fn resume_process(&self, _pid: u32) -> NucleusResult<()> {
        Err(NucleusError::Unsupported("Process resumption not yet implemented on Linux".into()))
    }
}
