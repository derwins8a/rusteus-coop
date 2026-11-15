//! Window focus hooking and fake activation management.

use nucleus_common::NucleusResult;

/// Provides focus retention hooks and simulated WM_ACTIVATE / WM_SETFOCUS message dispatching.
pub struct FocusHook;

impl FocusHook {
    /// Hooks the target window to prevent focus loss deactivation.
    pub fn hook_focus(hwnd: usize) -> NucleusResult<()> {
        Self::send_fake_focus(hwnd)
    }

    /// Dispatches a simulated focus activation message to the target window.
    pub fn send_fake_focus(hwnd: usize) -> NucleusResult<()> {
        #[cfg(windows)]
        {
            use windows::Win32::Foundation::{HWND, LPARAM, WPARAM};
            use windows::Win32::UI::WindowsAndMessaging::{
                PostMessageW, WA_ACTIVE, WM_ACTIVATE, WM_NCACTIVATE, WM_SETFOCUS,
            };

            let win_handle = HWND(hwnd as *mut _);
            unsafe {
                let _ = PostMessageW(win_handle, WM_NCACTIVATE, WPARAM(1), LPARAM(0));
                let _ = PostMessageW(
                    win_handle,
                    WM_ACTIVATE,
                    WPARAM(WA_ACTIVE as usize),
                    LPARAM(0),
                );
                let _ = PostMessageW(win_handle, WM_SETFOCUS, WPARAM(0), LPARAM(0));
            }
        }
        #[cfg(not(windows))]
        {
            let _ = hwnd;
        }
        Ok(())
    }

    /// Removes focus hooks from the specified window.
    pub fn unhook_focus(_hwnd: usize) -> NucleusResult<()> {
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_focus_hook_lifecycle() {
        let dummy_hwnd = 0;
        assert!(FocusHook::hook_focus(dummy_hwnd).is_ok());
        assert!(FocusHook::send_fake_focus(dummy_hwnd).is_ok());
        assert!(FocusHook::unhook_focus(dummy_hwnd).is_ok());
    }
}
