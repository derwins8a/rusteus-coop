//! Windows-specific implementation of the OSD (On-Screen Display) manager.

use std::sync::{mpsc, Arc, Mutex};
use std::time::{Duration, Instant};
use nucleus_platform::OsdManager;
use tracing::{info, warn};
use windows::core::w;
use windows::Win32::Foundation::{COLORREF, HWND, LPARAM, LRESULT, RECT, WPARAM};
use windows::Win32::Graphics::Gdi::{
    BeginPaint, CreateFontW, CreatePen, CreateSolidBrush, DeleteObject, DrawTextW, EndPaint,
    FillRect, InvalidateRect, RoundRect, SelectObject, SetBkMode, SetTextColor, UpdateWindow, DT_CENTER,
    DT_SINGLELINE, DT_VCENTER, FW_BOLD, PAINTSTRUCT, PS_SOLID, TRANSPARENT,
};
use windows::Win32::UI::WindowsAndMessaging::{
    BringWindowToTop, CreateWindowExW, DefWindowProcW, DestroyWindow, DispatchMessageW,
    GetClientRect, GetSystemMetrics, PeekMessageW, RegisterClassW, SetLayeredWindowAttributes,
    SetWindowPos, ShowWindow, TranslateMessage, HWND_TOP, HWND_TOPMOST, LWA_ALPHA, LWA_COLORKEY, MSG, PM_REMOVE,
    SM_CXSCREEN, SM_CYSCREEN, SWP_NOACTIVATE, SWP_NOMOVE, SWP_NOSIZE, SWP_SHOWWINDOW, SW_HIDE,
    SW_SHOWNOACTIVATE, WNDCLASSW, WS_EX_LAYERED, WS_EX_NOACTIVATE, WS_EX_TOOLWINDOW,
    WS_EX_TOPMOST, WS_POPUP, WM_ERASEBKGND, WM_NCHITTEST, WM_PAINT,
};

struct OsdSharedData {
    text: Vec<u16>,
    color: (u8, u8, u8),
}

static OSD_SHARED: Mutex<OsdSharedData> = Mutex::new(OsdSharedData {
    text: Vec::new(),
    color: (255, 255, 255),
});

enum OsdWindowCommand {
    Show {
        text: String,
        duration: Duration,
        color_rgb: (u8, u8, u8),
    },
    Hide,
}

unsafe extern "system" fn osd_wndproc(
    hwnd: HWND,
    msg: u32,
    wparam: WPARAM,
    lparam: LPARAM,
) -> LRESULT {
    unsafe {
        match msg {
            WM_NCHITTEST => LRESULT(-1), // HTTRANSPARENT: mouse events pass straight through
            WM_PAINT => {
                let mut ps = PAINTSTRUCT::default();
                let hdc = BeginPaint(hwnd, &mut ps);
                if hdc.0.is_null() {
                    return LRESULT(0);
                }
                let mut rect = RECT::default();
                let _ = GetClientRect(hwnd, &mut rect);

                let (text_utf16, (r, g, b)) = match OSD_SHARED.lock() {
                    Ok(guard) => (guard.text.clone(), guard.color),
                    Err(poisoned) => {
                        let guard = poisoned.into_inner();
                        (guard.text.clone(), guard.color)
                    }
                };

                // Clear background with colorkey (0x00010101) so corners outside RoundRect are completely transparent
                let colorkey_brush = CreateSolidBrush(COLORREF(0x00010101));
                let _ = FillRect(hdc, &rect, colorkey_brush);
                let _ = DeleteObject(colorkey_brush);

                let bg_brush = CreateSolidBrush(COLORREF(0x00141414));
                let border_pen = CreatePen(
                    PS_SOLID,
                    2,
                    COLORREF((r as u32) | ((g as u32) << 8) | ((b as u32) << 16)),
                );

                let old_brush = SelectObject(hdc, bg_brush);
                let old_pen = SelectObject(hdc, border_pen);

                let _ = RoundRect(hdc, rect.left, rect.top, rect.right, rect.bottom, 16, 16);

                SelectObject(hdc, old_brush);
                SelectObject(hdc, old_pen);
                let _ = DeleteObject(bg_brush);
                let _ = DeleteObject(border_pen);

                if !text_utf16.is_empty() {
                    SetBkMode(hdc, TRANSPARENT);
                    SetTextColor(
                        hdc,
                        COLORREF((r as u32) | ((g as u32) << 8) | ((b as u32) << 16)),
                    );

                    let font = CreateFontW(
                        24,
                        0,
                        0,
                        0,
                        FW_BOLD.0 as i32,
                        0,
                        0,
                        0,
                        0,
                        0,
                        0,
                        0,
                        0,
                        w!("Segoe UI"),
                    );
                    let old_font = SelectObject(hdc, font);

                    let mut wide_buf = text_utf16;
                    let _ = DrawTextW(
                        hdc,
                        &mut wide_buf,
                        &mut rect,
                        DT_CENTER | DT_VCENTER | DT_SINGLELINE,
                    );

                    SelectObject(hdc, old_font);
                    let _ = DeleteObject(font);
                }

                let _ = EndPaint(hwnd, &ps);
                LRESULT(0)
            }
            WM_ERASEBKGND => LRESULT(1),
            _ => DefWindowProcW(hwnd, msg, wparam, lparam),
        }
    }
}

fn parse_color_rgb(s: &str) -> (u8, u8, u8) {
    let parts: Vec<&str> = s.split(',').map(|p| p.trim()).collect();
    if parts.len() == 3 {
        let r = parts[0].parse::<u8>().unwrap_or(255);
        let g = parts[1].parse::<u8>().unwrap_or(255);
        let b = parts[2].parse::<u8>().unwrap_or(255);
        (r, g, b)
    } else {
        (255, 255, 255)
    }
}

fn run_osd_event_loop(rx: mpsc::Receiver<OsdWindowCommand>) {
    unsafe {
        let class_name = w!("NucleusCoop_OSD_Overlay");
        let hmodule = windows::Win32::System::LibraryLoader::GetModuleHandleW(None).unwrap_or_default();
        let hinstance: windows::Win32::Foundation::HINSTANCE = hmodule.into();
        let wnd_class = WNDCLASSW {
            lpfnWndProc: Some(osd_wndproc),
            hInstance: hinstance,
            lpszClassName: class_name,
            ..Default::default()
        };
        let _ = RegisterClassW(&wnd_class);

        let screen_w = GetSystemMetrics(SM_CXSCREEN);
        let _screen_h = GetSystemMetrics(SM_CYSCREEN);
        let win_w = 640;
        let win_h = 64;
        let pos_x = (screen_w - win_w) / 2;
        let pos_y = 48; // Positioned near the top-center of the screen

        let hwnd = match CreateWindowExW(
            WS_EX_TOPMOST | WS_EX_TOOLWINDOW | WS_EX_LAYERED | WS_EX_NOACTIVATE,
            class_name,
            w!("NucleusCoop OSD"),
            WS_POPUP,
            pos_x,
            pos_y,
            win_w,
            win_h,
            None,
            None,
            Some(&hinstance),
            None,
        ) {
            Ok(h) => h,
            Err(e) => {
                warn!("[OSD] CreateWindowExW failed (running headless or non-GUI?): {}", e);
                // Drain channel so caller doesn't block or leak
                while let Ok(_) = rx.recv() {}
                return;
            }
        };

        // 92% opacity (235/255) matching C# WPF_OSD with colorkey transparency for rounded corners
        let _ = SetLayeredWindowAttributes(hwnd, COLORREF(0x00010101), 235, LWA_COLORKEY | LWA_ALPHA);

        let mut active_deadline: Option<Instant> = None;

        loop {
            // Process any pending Win32 messages
            let mut msg = MSG::default();
            while PeekMessageW(&mut msg, None, 0, 0, PM_REMOVE).as_bool() {
                let _ = TranslateMessage(&msg);
                DispatchMessageW(&msg);
            }

            let timeout = if let Some(deadline) = active_deadline {
                let now = Instant::now();
                if now >= deadline {
                    let _ = ShowWindow(hwnd, SW_HIDE);
                    active_deadline = None;
                    Duration::from_millis(500)
                } else {
                    deadline.duration_since(now).min(Duration::from_millis(16))
                }
            } else {
                Duration::from_millis(500)
            };

            match rx.recv_timeout(timeout) {
                Ok(OsdWindowCommand::Show { text, duration, color_rgb }) => {
                    let text_len = text.chars().count() as i32;
                    let cur_w = (text_len * 14 + 100).clamp(480, screen_w - 40);
                    let cur_x = (screen_w - cur_w) / 2;
                    {
                        let mut guard = OSD_SHARED.lock().unwrap();
                        guard.text = text.encode_utf16().collect();
                        guard.color = color_rgb;
                    }
                    active_deadline = Some(Instant::now() + duration);
                    let _ = ShowWindow(hwnd, SW_SHOWNOACTIVATE);
                    let _ = SetWindowPos(
                        hwnd,
                        HWND_TOPMOST,
                        cur_x,
                        pos_y,
                        cur_w,
                        win_h,
                        SWP_NOACTIVATE | SWP_SHOWWINDOW,
                    );
                    let _ = SetWindowPos(
                        hwnd,
                        HWND_TOP,
                        0,
                        0,
                        0,
                        0,
                        SWP_NOACTIVATE | SWP_NOMOVE | SWP_NOSIZE,
                    );
                    let _ = BringWindowToTop(hwnd);
                    let _ = InvalidateRect(hwnd, None, true);
                    let _ = UpdateWindow(hwnd);
                }
                Ok(OsdWindowCommand::Hide) => {
                    active_deadline = None;
                    let _ = ShowWindow(hwnd, SW_HIDE);
                }
                Err(mpsc::RecvTimeoutError::Timeout) => {
                    if let Some(deadline) = active_deadline {
                        if Instant::now() >= deadline {
                            let _ = ShowWindow(hwnd, SW_HIDE);
                            active_deadline = None;
                        } else {
                            // Continuously re-assert TOPMOST to prevent occlusion by borderless fullscreen games
                            let _ = SetWindowPos(
                                hwnd,
                                HWND_TOPMOST,
                                0,
                                0,
                                0,
                                0,
                                SWP_NOACTIVATE | SWP_NOMOVE | SWP_NOSIZE | SWP_SHOWWINDOW,
                            );
                            let _ = SetWindowPos(
                                hwnd,
                                HWND_TOP,
                                0,
                                0,
                                0,
                                0,
                                SWP_NOACTIVATE | SWP_NOMOVE | SWP_NOSIZE,
                            );
                            let _ = BringWindowToTop(hwnd);
                        }
                    }
                }
                Err(mpsc::RecvTimeoutError::Disconnected) => {
                    break;
                }
            }
        }

        let _ = DestroyWindow(hwnd);
    }
}

/// Windows implementation of the `OsdManager` trait.
pub struct WindowsOsdManager {
    current: Arc<Mutex<Option<(String, Instant, Duration)>>>,
    color: Arc<Mutex<String>>,
    tx: Mutex<Option<mpsc::Sender<OsdWindowCommand>>>,
}

impl WindowsOsdManager {
    /// Creates a new `WindowsOsdManager`.
    pub fn new() -> Self {
        let (tx, rx) = mpsc::channel::<OsdWindowCommand>();
        std::thread::Builder::new()
            .name("nucleus-osd-overlay".to_string())
            .spawn(move || {
                run_osd_event_loop(rx);
            })
            .ok();

        Self {
            current: Arc::new(Mutex::new(None)),
            color: Arc::new(Mutex::new("255,255,255".to_string())),
            tx: Mutex::new(Some(tx)),
        }
    }
}

impl Default for WindowsOsdManager {
    fn default() -> Self {
        Self::new()
    }
}

impl OsdManager for WindowsOsdManager {
    fn show(&self, duration_ms: u32, text: &str) {
        info!("[OSD] {} (displaying for {}ms)", text, duration_ms);
        let duration = Duration::from_millis(duration_ms as u64);
        let color_str = match self.color.lock() {
            Ok(c) => c.clone(),
            Err(p) => p.into_inner().clone(),
        };
        let color_rgb = parse_color_rgb(&color_str);

        if let Ok(mut cur) = self.current.lock() {
            *cur = Some((text.to_string(), Instant::now(), duration));
        }

        if let Ok(guard) = self.tx.lock() {
            if let Some(ref tx) = *guard {
                let _ = tx.send(OsdWindowCommand::Show {
                    text: text.to_string(),
                    duration,
                    color_rgb,
                });
            }
        }
    }

    fn hide(&self) {
        if let Ok(mut cur) = self.current.lock() {
            *cur = None;
        }
        if let Ok(guard) = self.tx.lock() {
            if let Some(ref tx) = *guard {
                let _ = tx.send(OsdWindowCommand::Hide);
            }
        }
    }

    fn current_message(&self) -> Option<String> {
        if let Ok(mut cur) = self.current.lock() {
            if let Some((msg, start, dur)) = cur.as_ref() {
                if start.elapsed() < *dur {
                    return Some(msg.clone());
                } else {
                    *cur = None;
                }
            }
        }
        None
    }

    fn set_color(&self, color: &str) {
        if let Ok(mut c) = self.color.lock() {
            *c = color.to_string();
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_windows_osd_show_and_expire() {
        let osd = WindowsOsdManager::new();
        assert_eq!(osd.current_message(), None);

        osd.show(100, "Starting Left 4 Dead 2 instance for Player1 as Player #1");
        assert_eq!(
            osd.current_message(),
            Some("Starting Left 4 Dead 2 instance for Player1 as Player #1".to_string())
        );

        osd.hide();
        assert_eq!(osd.current_message(), None);

        osd.show(50, "Inputs Locked");
        assert_eq!(osd.current_message(), Some("Inputs Locked".to_string()));
        std::thread::sleep(Duration::from_millis(60));
        assert_eq!(osd.current_message(), None);
    }
}
