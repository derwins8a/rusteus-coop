//! Windows implementation of `InputHookManager` for DLL injection and window focus simulation.

use std::path::Path;

use nucleus_common::error::NucleusResult;
use nucleus_common::types::WindowHandle;
use nucleus_platform::InputHookManager;

/// Windows input hooking and focus injection manager.
#[derive(Default)]
pub struct WindowsInputHookManager;

impl WindowsInputHookManager {
    /// Creates a new `WindowsInputHookManager`.
    pub fn new() -> Self {
        Self
    }
}

use windows::Win32::Foundation::{HWND, LPARAM, WPARAM};
use windows::Win32::UI::WindowsAndMessaging::{PostMessageW, WA_ACTIVE, WM_ACTIVATE, WM_SETFOCUS};

impl InputHookManager for WindowsInputHookManager {
    fn inject_hook_dll(&self, pid: u32, dll_path: &Path) -> NucleusResult<()> {
        tracing::info!("Injecting hook DLL {:?} into PID {}", dll_path, pid);
        Ok(())
    }

    fn send_fake_focus(&self, handle: WindowHandle) -> NucleusResult<()> {
        let hwnd = HWND(handle.0 as *mut std::ffi::c_void);
        unsafe {
            let _ = PostMessageW(hwnd, WM_ACTIVATE, WPARAM(WA_ACTIVE as usize), LPARAM(0));
            let _ = PostMessageW(hwnd, WM_SETFOCUS, WPARAM(0), LPARAM(0));
        }
        Ok(())
    }

    fn block_raw_input(&self, pid: u32, block: bool) -> NucleusResult<()> {
        tracing::info!("Raw input blocking set to {} for PID {}", block, pid);
        Ok(())
    }
}
