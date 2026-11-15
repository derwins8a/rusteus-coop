//! Platform-agnostic input hooking and window focus simulation operations.

use std::path::Path;

use nucleus_common::error::NucleusResult;
use nucleus_common::types::WindowHandle;

/// Platform-agnostic input hooking and window focus simulation operations.
pub trait InputHookManager: Send + Sync {
    /// Inject a hook DLL into a target process.
    fn inject_hook_dll(&self, pid: u32, dll_path: &Path) -> NucleusResult<()>;

    /// Send fake window focus messages to simulate active focus on a background window.
    fn send_fake_focus(&self, handle: WindowHandle) -> NucleusResult<()>;

    /// Block or unblock raw input messages for a process.
    fn block_raw_input(&self, pid: u32, block: bool) -> NucleusResult<()>;
}
