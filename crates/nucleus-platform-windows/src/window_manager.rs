//! Windows implementation of `WindowManager` using Win32 APIs.

use windows::Win32::Foundation::{BOOL, HWND, LPARAM, WPARAM};
use windows::Win32::UI::WindowsAndMessaging::{
    ClipCursor, EnumWindows, FindWindowW, GW_OWNER, GWL_EXSTYLE, GWL_STYLE, GetDesktopWindow,
    GetWindow, GetWindowLongW, GetWindowRect, GetWindowTextLengthW, GetWindowTextW,
    GetWindowThreadProcessId, HWND_NOTOPMOST, HWND_TOPMOST, IsIconic, IsWindowVisible,
    PostMessageW, SW_MINIMIZE, SW_RESTORE, SWP_FRAMECHANGED, SWP_NOACTIVATE, SWP_NOMOVE,
    SWP_NOSIZE, SWP_NOZORDER, SWP_SHOWWINDOW, SendMessageW, SetForegroundWindow, SetWindowLongW,
    SetWindowPos, SetWindowTextW, ShowWindow, WM_CLOSE, WM_COMMAND, WM_EXITSIZEMOVE, WM_SIZE,
    WS_BORDER, WS_CAPTION, WS_DLGFRAME, WS_EX_CLIENTEDGE, WS_EX_DLGMODALFRAME, WS_EX_STATICEDGE,
    WS_EX_WINDOWEDGE, WS_MAXIMIZEBOX, WS_MINIMIZEBOX, WS_SYSMENU, WS_THICKFRAME,
};
use windows::core::HSTRING;

use nucleus_common::error::{NucleusError, NucleusResult};
use nucleus_common::types::{Rect, WindowHandle, WindowStyle};
use nucleus_platform::WindowManager;

/// Windows Win32 window manager implementation.
#[derive(Default)]
pub struct WindowsWindowManager;

impl WindowsWindowManager {
    /// Creates a new `WindowsWindowManager`.
    pub fn new() -> Self {
        Self
    }

    fn to_hwnd(handle: WindowHandle) -> HWND {
        HWND(handle.0 as *mut std::ffi::c_void)
    }

    fn from_hwnd(hwnd: HWND) -> WindowHandle {
        WindowHandle(hwnd.0 as u64)
    }

    fn get_text(hwnd: HWND) -> String {
        unsafe {
            let len = GetWindowTextLengthW(hwnd);
            if len == 0 {
                return String::new();
            }
            let mut buf = vec![0u16; (len + 1) as usize];
            let read_len = GetWindowTextW(hwnd, &mut buf);
            String::from_utf16_lossy(&buf[..read_len as usize])
        }
    }
}

impl WindowManager for WindowsWindowManager {
    fn find_window_by_title(
        &self,
        title: &str,
        partial: bool,
    ) -> NucleusResult<Option<WindowHandle>> {
        struct SearchData<'a> {
            target: &'a str,
            partial: bool,
            found: Option<WindowHandle>,
        }

        let mut data = SearchData {
            target: title,
            partial,
            found: None,
        };

        unsafe extern "system" fn enum_proc(hwnd: HWND, lparam: LPARAM) -> BOOL {
            let data = unsafe { &mut *(lparam.0 as *mut SearchData) };
            if unsafe { IsWindowVisible(hwnd).as_bool() } {
                let win_title = WindowsWindowManager::get_text(hwnd);

                if !win_title.is_empty() {
                    let matched = if data.partial {
                        win_title
                            .to_lowercase()
                            .contains(&data.target.to_lowercase())
                    } else {
                        win_title.eq_ignore_ascii_case(data.target)
                    };
                    if matched {
                        data.found = Some(WindowsWindowManager::from_hwnd(hwnd));
                        return BOOL(0); // stop enumeration
                    }
                }
            }
            BOOL(1)
        }

        unsafe {
            let _ = EnumWindows(Some(enum_proc), LPARAM(&mut data as *mut _ as isize));
        }

        Ok(data.found)
    }

    fn find_windows_by_process(&self, pid: u32) -> NucleusResult<Vec<WindowHandle>> {
        struct SearchData {
            target_pid: u32,
            results: Vec<WindowHandle>,
        }

        let mut data = SearchData {
            target_pid: pid,
            results: Vec::new(),
        };

        unsafe extern "system" fn enum_proc(hwnd: HWND, lparam: LPARAM) -> BOOL {
            let data = unsafe { &mut *(lparam.0 as *mut SearchData) };
            let mut win_pid = 0u32;
            unsafe {
                GetWindowThreadProcessId(hwnd, Some(&mut win_pid));
            }
            if win_pid == data.target_pid && unsafe { IsWindowVisible(hwnd).as_bool() } {
                let owner = unsafe { GetWindow(hwnd, GW_OWNER) };
                let is_unowned = match owner {
                    Ok(h) => h.0.is_null(),
                    Err(_) => true,
                };
                if is_unowned {
                    let title = WindowsWindowManager::get_text(hwnd);
                    if !title.contains("ProtoInput") && !title.contains("Proto Input") {
                        data.results.push(WindowsWindowManager::from_hwnd(hwnd));
                    }
                }
            }
            BOOL(1)
        }

        unsafe {
            let _ = EnumWindows(Some(enum_proc), LPARAM(&mut data as *mut _ as isize));
        }

        Ok(data.results)
    }

    fn set_window_position(&self, handle: WindowHandle, rect: &Rect) -> NucleusResult<()> {
        self.set_window_position_ext(handle, rect, false)
    }

    fn set_window_position_ext(
        &self,
        handle: WindowHandle,
        rect: &Rect,
        not_top_most: bool,
    ) -> NucleusResult<()> {
        let hwnd = Self::to_hwnd(handle);
        unsafe {
            if IsIconic(hwnd).as_bool() {
                let _ = ShowWindow(hwnd, SW_RESTORE);
            }

            // Parity: If not_top_most is requested, use HWND_NOTOPMOST and SWP_NOZORDER
            // rather than forcing topmost unconditionally.
            let (z_order, flags) = if not_top_most {
                (
                    HWND_NOTOPMOST,
                    SWP_FRAMECHANGED | SWP_NOACTIVATE | SWP_SHOWWINDOW | SWP_NOZORDER,
                )
            } else {
                (
                    HWND_TOPMOST,
                    SWP_FRAMECHANGED | SWP_NOACTIVATE | SWP_SHOWWINDOW,
                )
            };

            SetWindowPos(
                hwnd,
                z_order,
                rect.x,
                rect.y,
                rect.width,
                rect.height,
                flags,
            )
            .map_err(|e| NucleusError::Platform(e.to_string()))?;

            // Notify game window and Direct3D runtime of the viewport resize
            let lparam = LPARAM(((rect.height as isize) << 16) | (rect.width as isize & 0xFFFF));
            let _ = PostMessageW(hwnd, WM_SIZE, WPARAM(0), lparam);
            let _ = PostMessageW(hwnd, WM_EXITSIZEMOVE, WPARAM(0), LPARAM(0));
            let _ = windows::Win32::Graphics::Gdi::RedrawWindow(
                hwnd,
                None,
                None,
                windows::Win32::Graphics::Gdi::RDW_INVALIDATE
                    | windows::Win32::Graphics::Gdi::RDW_UPDATENOW
                    | windows::Win32::Graphics::Gdi::RDW_FRAME
                    | windows::Win32::Graphics::Gdi::RDW_ALLCHILDREN,
            );
        }
        Ok(())
    }

    fn minimize_all(&self) -> NucleusResult<()> {
        unsafe {
            // Parity with C# User32Util.MinimizeEverything: Send MIN_ALL (419) to Shell_TrayWnd and Desktop
            if let Ok(tray) = FindWindowW(windows::core::w!("Shell_TrayWnd"), None) {
                let _ = SendMessageW(tray, WM_COMMAND, WPARAM(419), LPARAM(0));
            }
            let desktop = GetDesktopWindow();
            let _ = SendMessageW(desktop, WM_COMMAND, WPARAM(419), LPARAM(0));
        }
        Ok(())
    }

    fn apply_custom_window_styles(
        &self,
        handle: WindowHandle,
        style_values: &[String],
        ext_style_values: &[String],
    ) -> NucleusResult<()> {
        let hwnd = Self::to_hwnd(handle);
        unsafe {
            if !style_values.is_empty() {
                let mut current_style = GetWindowLongW(hwnd, GWL_STYLE) as u32;
                for val in style_values {
                    let trimmed = val.trim();
                    if let Some(rest) = trimmed.strip_prefix('~') {
                        let hex_str = rest.trim_start_matches("0x").trim_start_matches("0X");
                        if let Ok(mask) = u32::from_str_radix(hex_str, 16) {
                            current_style &= !mask;
                        }
                    } else {
                        let hex_str = trimmed.trim_start_matches("0x").trim_start_matches("0X");
                        if let Ok(mask) = u32::from_str_radix(hex_str, 16) {
                            current_style |= mask;
                        }
                    }
                }
                SetWindowLongW(hwnd, GWL_STYLE, current_style as i32);
            }

            if !ext_style_values.is_empty() {
                let mut current_ex_style = GetWindowLongW(hwnd, GWL_EXSTYLE) as u32;
                for val in ext_style_values {
                    let trimmed = val.trim();
                    if let Some(rest) = trimmed.strip_prefix('~') {
                        let hex_str = rest.trim_start_matches("0x").trim_start_matches("0X");
                        if let Ok(mask) = u32::from_str_radix(hex_str, 16) {
                            current_ex_style &= !mask;
                        }
                    } else {
                        let hex_str = trimmed.trim_start_matches("0x").trim_start_matches("0X");
                        if let Ok(mask) = u32::from_str_radix(hex_str, 16) {
                            current_ex_style |= mask;
                        }
                    }
                }
                SetWindowLongW(hwnd, GWL_EXSTYLE, current_ex_style as i32);
            }

            let _ = SetWindowPos(
                hwnd,
                None,
                0,
                0,
                0,
                0,
                SWP_NOMOVE | SWP_NOSIZE | SWP_NOZORDER | SWP_FRAMECHANGED,
            );
        }
        Ok(())
    }

    fn set_window_style(&self, handle: WindowHandle, style: &WindowStyle) -> NucleusResult<()> {
        let hwnd = Self::to_hwnd(handle);
        unsafe {
            let current_style = GetWindowLongW(hwnd, GWL_STYLE);
            let mut new_style = current_style;

            if !style.borders {
                new_style &= !(WS_BORDER.0 as i32 | WS_THICKFRAME.0 as i32 | WS_DLGFRAME.0 as i32);
            }
            if !style.title_bar {
                new_style &= !(WS_CAPTION.0 as i32 | WS_SYSMENU.0 as i32);
            }
            if !style.minimize_box {
                new_style &= !(WS_MINIMIZEBOX.0 as i32);
            }
            if !style.maximize_box {
                new_style &= !(WS_MAXIMIZEBOX.0 as i32);
            }

            SetWindowLongW(hwnd, GWL_STYLE, new_style);
            let _ = SetWindowPos(
                hwnd,
                None,
                0,
                0,
                0,
                0,
                SWP_NOMOVE | SWP_NOSIZE | SWP_NOZORDER | SWP_FRAMECHANGED,
            );
        }

        if style.topmost {
            self.set_topmost(handle, true)?;
        }

        Ok(())
    }

    fn set_topmost(&self, handle: WindowHandle, topmost: bool) -> NucleusResult<()> {
        let hwnd = Self::to_hwnd(handle);
        let z_order = if topmost {
            HWND_TOPMOST
        } else {
            HWND_NOTOPMOST
        };
        unsafe {
            SetWindowPos(
                hwnd,
                z_order,
                0,
                0,
                0,
                0,
                SWP_NOMOVE | SWP_NOSIZE | SWP_NOACTIVATE,
            )
            .map_err(|e| NucleusError::Platform(e.to_string()))?;
        }
        Ok(())
    }

    fn remove_borders(&self, handle: WindowHandle) -> NucleusResult<()> {
        let hwnd = Self::to_hwnd(handle);
        unsafe {
            let mut style = GetWindowLongW(hwnd, GWL_STYLE);
            style &= !(WS_CAPTION.0 as i32
                | WS_THICKFRAME.0 as i32
                | WS_MINIMIZEBOX.0 as i32
                | WS_MAXIMIZEBOX.0 as i32
                | WS_SYSMENU.0 as i32
                | WS_BORDER.0 as i32
                | WS_DLGFRAME.0 as i32);
            SetWindowLongW(hwnd, GWL_STYLE, style);

            let mut ex_style = GetWindowLongW(hwnd, GWL_EXSTYLE);
            ex_style &= !(WS_EX_DLGMODALFRAME.0 as i32
                | WS_EX_CLIENTEDGE.0 as i32
                | WS_EX_STATICEDGE.0 as i32
                | WS_EX_WINDOWEDGE.0 as i32);
            SetWindowLongW(hwnd, GWL_EXSTYLE, ex_style);

            let _ = SetWindowPos(
                hwnd,
                None,
                0,
                0,
                0,
                0,
                SWP_NOMOVE | SWP_NOSIZE | SWP_NOZORDER | SWP_FRAMECHANGED | SWP_SHOWWINDOW,
            );
        }
        Ok(())
    }

    fn get_window_rect(&self, handle: WindowHandle) -> NucleusResult<Rect> {
        let hwnd = Self::to_hwnd(handle);
        let mut r = windows::Win32::Foundation::RECT::default();
        unsafe {
            GetWindowRect(hwnd, &mut r).map_err(|e| NucleusError::Platform(e.to_string()))?;
        }
        Ok(Rect {
            x: r.left,
            y: r.top,
            width: r.right - r.left,
            height: r.bottom - r.top,
        })
    }

    fn set_foreground(&self, handle: WindowHandle) -> NucleusResult<()> {
        let hwnd = Self::to_hwnd(handle);
        unsafe {
            let _ = SetForegroundWindow(hwnd);
        }
        Ok(())
    }

    fn get_window_title(&self, handle: WindowHandle) -> NucleusResult<String> {
        let hwnd = Self::to_hwnd(handle);
        Ok(Self::get_text(hwnd))
    }

    fn set_window_title(&self, handle: WindowHandle, title: &str) -> NucleusResult<()> {
        let hwnd = Self::to_hwnd(handle);
        let hstring = HSTRING::from(title);
        unsafe {
            SetWindowTextW(hwnd, &hstring).map_err(|e| NucleusError::Platform(e.to_string()))?;
        }
        Ok(())
    }

    fn close_window(&self, handle: WindowHandle) -> NucleusResult<()> {
        let hwnd = Self::to_hwnd(handle);
        unsafe {
            let _ = PostMessageW(hwnd, WM_CLOSE, WPARAM(0), LPARAM(0));
        }
        Ok(())
    }

    fn minimize_window(&self, handle: WindowHandle) -> NucleusResult<()> {
        let hwnd = Self::to_hwnd(handle);
        unsafe {
            let _ = ShowWindow(hwnd, SW_MINIMIZE);
        }
        Ok(())
    }

    fn restore_window(&self, handle: WindowHandle) -> NucleusResult<()> {
        let hwnd = Self::to_hwnd(handle);
        unsafe {
            let _ = ShowWindow(hwnd, SW_RESTORE);
        }
        Ok(())
    }

    fn is_minimized(&self, handle: WindowHandle) -> NucleusResult<bool> {
        let hwnd = Self::to_hwnd(handle);
        let iconic = unsafe { IsIconic(hwnd).as_bool() };
        Ok(iconic)
    }

    fn unclip_cursor(&self) -> NucleusResult<()> {
        unsafe {
            let _ = ClipCursor(None);
        }
        Ok(())
    }

    fn configure_overlay_window(&self, handle: WindowHandle, rect: &Rect) -> NucleusResult<()> {
        if !handle.is_valid() {
            return Err(NucleusError::Platform(
                "Invalid window handle for overlay configuration".into(),
            ));
        }
        let hwnd = Self::to_hwnd(handle);
        unsafe {
            // Apply WS_EX_TOPMOST | WS_EX_TOOLWINDOW | WS_EX_LAYERED to extended style
            let ex_style = GetWindowLongW(hwnd, GWL_EXSTYLE);
            let target_ex_style = ex_style
                | (windows::Win32::UI::WindowsAndMessaging::WS_EX_TOPMOST.0 as i32)
                | (windows::Win32::UI::WindowsAndMessaging::WS_EX_TOOLWINDOW.0 as i32)
                | (windows::Win32::UI::WindowsAndMessaging::WS_EX_LAYERED.0 as i32);
            SetWindowLongW(hwnd, GWL_EXSTYLE, target_ex_style);

            // Extend DWM glass/blur margins into entire client area (-1)
            let margins = windows::Win32::UI::Controls::MARGINS {
                cxLeftWidth: -1,
                cxRightWidth: -1,
                cyTopHeight: -1,
                cyBottomHeight: -1,
            };
            let _ = windows::Win32::Graphics::Dwm::DwmExtendFrameIntoClientArea(hwnd, &margins);

            // Position and size the overlay window on the target monitor
            let _ = SetWindowPos(
                hwnd,
                HWND_TOPMOST,
                rect.x,
                rect.y,
                rect.width,
                rect.height,
                SWP_SHOWWINDOW | SWP_FRAMECHANGED,
            );

            // Release any cursor clipping that games or ProtoInput applied
            let _ = ClipCursor(None);
        }
        Ok(())
    }

    fn get_monitor_bounds_for_rect(&self, rect: &Rect) -> NucleusResult<Rect> {
        let win_rect = windows::Win32::Foundation::RECT {
            left: rect.x,
            top: rect.y,
            right: rect.x + rect.width,
            bottom: rect.y + rect.height,
        };
        unsafe {
            let hmon = windows::Win32::Graphics::Gdi::MonitorFromRect(
                &win_rect,
                windows::Win32::Graphics::Gdi::MONITOR_DEFAULTTONEAREST,
            );
            if !hmon.is_invalid() {
                let mut mi = windows::Win32::Graphics::Gdi::MONITORINFO {
                    cbSize: std::mem::size_of::<windows::Win32::Graphics::Gdi::MONITORINFO>()
                        as u32,
                    ..Default::default()
                };
                let ok = windows::Win32::Graphics::Gdi::GetMonitorInfoW(hmon, &mut mi);
                if ok.as_bool() {
                    return Ok(Rect::new(
                        mi.rcMonitor.left,
                        mi.rcMonitor.top,
                        mi.rcMonitor.right - mi.rcMonitor.left,
                        mi.rcMonitor.bottom - mi.rcMonitor.top,
                    ));
                }
            }
        }
        Ok(Rect::new(0, 0, 1920, 1080))
    }

    fn restore_game_focus(&self, game_handle: WindowHandle) -> NucleusResult<()> {
        if !game_handle.is_valid() {
            return Ok(());
        }
        let hwnd = Self::to_hwnd(game_handle);
        unsafe {
            let _ = windows::Win32::UI::WindowsAndMessaging::BringWindowToTop(hwnd);
            let _ = SetForegroundWindow(hwnd);
        }
        Ok(())
    }
}

#[cfg(all(test, windows))]
mod tests {
    use super::*;

    #[test]
    fn test_window_manager_creation_and_minimize_all() {
        let mgr = WindowsWindowManager::new();
        // minimize_all should execute safely without crashing
        let res = mgr.minimize_all();
        assert!(res.is_ok());
    }

    #[test]
    fn test_custom_window_styles_parsing() {
        let mgr = WindowsWindowManager::new();
        // Testing on invalid window handle should return Ok (graceful failure) or not panic
        let res = mgr.apply_custom_window_styles(
            WindowHandle::INVALID,
            &["~0x00C00000".to_string(), "0x80000".to_string()],
            &["~0x00000200".to_string()],
        );
        assert!(res.is_ok());
    }
}
