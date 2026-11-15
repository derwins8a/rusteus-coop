//! Linux stub implementation of `InputHookManager`.

#![cfg(target_os = "linux")]

use std::path::Path;

use nucleus_common::error::{NucleusError, NucleusResult};
use nucleus_common::types::WindowHandle;
use nucleus_platform::InputHookManager;

/// Linux input hooking and focus injection stub.
#[derive(Default)]
pub struct LinuxInputHookManager;

impl LinuxInputHookManager {
    /// Creates a new `LinuxInputHookManager`.
    pub fn new() -> Self {
        Self
    }
}

impl InputHookManager for LinuxInputHookManager {
    fn inject_hook_dll(&self, _pid: u32, _dll_path: &Path) -> NucleusResult<()> {
        Err(NucleusError::Unsupported("Windows DLL injection not supported on Linux".into()))
    }

    fn send_fake_focus(&self, _handle: WindowHandle) -> NucleusResult<()> {
        Err(NucleusError::Unsupported("Fake focus not yet implemented on Linux".into()))
    }

    fn block_raw_input(&self, _pid: u32, _block: bool) -> NucleusResult<()> {
        Err(NucleusError::Unsupported("Raw input blocking not yet implemented on Linux".into()))
    }
}
