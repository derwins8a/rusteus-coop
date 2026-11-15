use nucleus_common::types::{Rect, WindowHandle};
use nucleus_platform::WindowManager;
use nucleus_platform_windows::WindowsWindowManager;

#[test]
fn test_get_monitor_bounds_for_rect() {
    let mgr = WindowsWindowManager::new();
    let viewport = Rect::new(100, 100, 800, 600);
    let monitor_bounds = mgr
        .get_monitor_bounds_for_rect(&viewport)
        .expect("should resolve monitor bounds");

    // Physical monitor bounds should have valid non-zero dimensions
    assert!(monitor_bounds.width > 0, "monitor width must be positive");
    assert!(monitor_bounds.height > 0, "monitor height must be positive");
}

#[test]
fn test_configure_overlay_window_invalid_handle() {
    let mgr = WindowsWindowManager::new();
    let rect = Rect::new(0, 0, 1920, 1080);
    // Configuring an invalid window handle should return an error
    let res = mgr.configure_overlay_window(WindowHandle::INVALID, &rect);
    assert!(res.is_err(), "configuring invalid window handle should fail");
}

#[test]
fn test_restore_game_focus_invalid_handle() {
    let mgr = WindowsWindowManager::new();
    let res = mgr.restore_game_focus(WindowHandle::INVALID);
    // Checking that calling on invalid handle doesn't panic
    let _ = res;
}

#[cfg(windows)]
#[test]
fn test_configure_overlay_window_real_window() {
    use windows::core::PCWSTR;
    use windows::Win32::Foundation::{HWND, LPARAM, LRESULT, WPARAM};
    use windows::Win32::UI::WindowsAndMessaging::{
        CreateWindowExW, DefWindowProcW, DestroyWindow, GetWindowLongW, RegisterClassW, GWL_EXSTYLE,
        WNDCLASSW, WS_EX_LAYERED, WS_EX_TOOLWINDOW, WS_EX_TOPMOST, WS_POPUP,
    };

    unsafe extern "system" fn test_wndproc(
        hwnd: HWND,
        msg: u32,
        wparam: WPARAM,
        lparam: LPARAM,
    ) -> LRESULT {
        unsafe { DefWindowProcW(hwnd, msg, wparam, lparam) }
    }

    let mgr = WindowsWindowManager::new();

    let class_name: Vec<u16> = "NucleusTestOverlayClass\0".encode_utf16().collect();
    let wnd_class = WNDCLASSW {
        lpfnWndProc: Some(test_wndproc),
        lpszClassName: PCWSTR(class_name.as_ptr()),
        ..Default::default()
    };
    unsafe {
        let _ = RegisterClassW(&wnd_class);
        let hwnd = CreateWindowExW(
            Default::default(),
            PCWSTR(class_name.as_ptr()),
            PCWSTR(std::ptr::null()),
            WS_POPUP,
            0,
            0,
            200,
            200,
            None,
            None,
            None,
            None,
        );
        if let Ok(valid_hwnd) = hwnd {
            let handle = WindowHandle(valid_hwnd.0 as u64);
            let rect = Rect::new(10, 10, 500, 400);
            let config_res = mgr.configure_overlay_window(handle, &rect);
            assert!(
                config_res.is_ok(),
                "configure_overlay_window should succeed on valid window"
            );

            let ex_style = GetWindowLongW(valid_hwnd, GWL_EXSTYLE);
            assert_ne!(
                ex_style & (WS_EX_TOPMOST.0 as i32),
                0,
                "WS_EX_TOPMOST must be set"
            );
            assert_ne!(
                ex_style & (WS_EX_TOOLWINDOW.0 as i32),
                0,
                "WS_EX_TOOLWINDOW must be set"
            );
            assert_ne!(
                ex_style & (WS_EX_LAYERED.0 as i32),
                0,
                "WS_EX_LAYERED must be set"
            );

            let focus_res = mgr.restore_game_focus(handle);
            assert!(focus_res.is_ok(), "restore_game_focus should succeed");

            let _ = DestroyWindow(valid_hwnd);
        }
    }
}
