//! Windows implementation of `TaskbarManager` for toggling taskbar visibility.

use nucleus_common::error::NucleusResult;
use nucleus_platform::TaskbarManager;

/// Windows taskbar visibility manager.
#[derive(Default)]
pub struct WindowsTaskbarManager;

impl WindowsTaskbarManager {
    /// Creates a new `WindowsTaskbarManager`.
    pub fn new() -> Self {
        Self
    }
}

use windows::core::w;
use windows::Win32::Foundation::{HWND, LPARAM};
use windows::Win32::UI::Shell::{SHAppBarMessage, ABM_SETSTATE, APPBARDATA};
use windows::Win32::UI::WindowsAndMessaging::{
    FindWindowExW, FindWindowW, ShowWindow, SW_HIDE, SW_SHOW,
};

impl TaskbarManager for WindowsTaskbarManager {
    fn hide(&self) -> NucleusResult<()> {
        unsafe {
            // 1. Hide primary taskbar
            if let Ok(hwnd) = FindWindowW(w!("Shell_TrayWnd"), None) {
                if !hwnd.is_invalid() {
                    let _ = ShowWindow(hwnd, SW_HIDE);
                }
            }

            // 2. Hide all secondary taskbars across multi-monitor setups
            let mut sec_hwnd = FindWindowW(w!("Shell_SecondaryTrayWnd"), None).unwrap_or_default();
            while !sec_hwnd.is_invalid() && !sec_hwnd.0.is_null() {
                let _ = ShowWindow(sec_hwnd, SW_HIDE);
                sec_hwnd = FindWindowExW(
                    HWND::default(),
                    sec_hwnd,
                    w!("Shell_SecondaryTrayWnd"),
                    None,
                )
                .unwrap_or_default();
            }

            // 3. Set AppBar message to ABS_AUTOHIDE (1) so Explorer stays behind borderless fullscreen windows
            let mut abd = APPBARDATA {
                cbSize: std::mem::size_of::<APPBARDATA>() as u32,
                lParam: LPARAM(1),
                ..Default::default()
            };
            let _ = SHAppBarMessage(ABM_SETSTATE, &mut abd);
        }
        Ok(())
    }

    fn show(&self) -> NucleusResult<()> {
        unsafe {
            // 1. Restore primary taskbar
            if let Ok(hwnd) = FindWindowW(w!("Shell_TrayWnd"), None) {
                if !hwnd.is_invalid() {
                    let _ = ShowWindow(hwnd, SW_SHOW);
                }
            }

            // 2. Restore all secondary taskbars
            let mut sec_hwnd = FindWindowW(w!("Shell_SecondaryTrayWnd"), None).unwrap_or_default();
            while !sec_hwnd.is_invalid() && !sec_hwnd.0.is_null() {
                let _ = ShowWindow(sec_hwnd, SW_SHOW);
                sec_hwnd = FindWindowExW(
                    HWND::default(),
                    sec_hwnd,
                    w!("Shell_SecondaryTrayWnd"),
                    None,
                )
                .unwrap_or_default();
            }

            // 3. Restore AppBar message to normal non-autohide state (0)
            let mut abd = APPBARDATA {
                cbSize: std::mem::size_of::<APPBARDATA>() as u32,
                lParam: LPARAM(0),
                ..Default::default()
            };
            let _ = SHAppBarMessage(ABM_SETSTATE, &mut abd);
        }
        Ok(())
    }
}


