//! Platform-agnostic process lifecycle and control operations.

use nucleus_common::error::{NucleusError, NucleusResult};
use nucleus_common::types::{ProcessConfig, ProcessHandle, ProcessInfo, ProcessPriority};

/// Platform-agnostic process lifecycle and control operations.
pub trait ProcessManager: Send + Sync {
    /// Start a new process using the specified configuration.
    fn start_process(&self, config: &ProcessConfig) -> NucleusResult<ProcessHandle>;

    /// Start a new process using extended options (suspended creation, unicode environment).
    fn start_process_extended(&self, config: &ProcessConfig, suspended: bool) -> NucleusResult<ProcessHandle> {
        let _ = suspended;
        self.start_process(config)
    }

    /// Write bytes into a target process's virtual memory space.
    fn write_process_memory(&self, pid: u32, address: usize, data: &[u8]) -> NucleusResult<usize> {
        let _ = (pid, address, data);
        Err(NucleusError::Platform("WriteProcessMemory not supported on this platform".to_string()))
    }

    /// Terminate a process by process ID.
    fn kill_process(&self, pid: u32) -> NucleusResult<()>;

    /// Get detailed information about a process by its PID.
    fn get_process_info(&self, pid: u32) -> NucleusResult<ProcessInfo>;

    /// Check if a process is currently running.
    fn is_process_running(&self, pid: u32) -> bool;

    /// Set CPU affinity mask for a process.
    fn set_affinity(&self, pid: u32, mask: u64) -> NucleusResult<()>;

    /// Set process scheduling priority.
    fn set_priority(&self, pid: u32, priority: ProcessPriority) -> NucleusResult<()>;

    /// Find all processes matching an executable name.
    fn get_processes_by_name(&self, name: &str) -> NucleusResult<Vec<ProcessInfo>>;

    /// Wait for a process to exit within a specified timeout in milliseconds.
    ///
    /// Returns `true` if the process exited within the timeout, `false` if it timed out.
    fn wait_for_process(&self, pid: u32, timeout_ms: u32) -> NucleusResult<bool>;

    /// Suspend a process by process ID.
    fn suspend_process(&self, pid: u32) -> NucleusResult<()>;

    /// Resume a suspended process by process ID.
    fn resume_process(&self, pid: u32) -> NucleusResult<()>;
}
