use windows::Win32::Foundation::{BOOL, LPARAM, RECT};
use windows::Win32::Graphics::Gdi::{
    EnumDisplayMonitors, GetMonitorInfoW, HDC, HMONITOR, MONITORINFOEXW,
};

const MONITORINFOF_PRIMARY: u32 = 1;


use nucleus_common::error::{NucleusError, NucleusResult};
use nucleus_common::types::{Display, Rect};
use nucleus_platform::DisplayManager;

/// Windows display and monitor manager implementation.
#[derive(Default)]
pub struct WindowsDisplayManager;

impl WindowsDisplayManager {
    /// Creates a new `WindowsDisplayManager`.
    pub fn new() -> Self {
        Self
    }
}

impl DisplayManager for WindowsDisplayManager {
    fn get_displays(&self) -> NucleusResult<Vec<Display>> {
        struct EnumData {
            displays: Vec<Display>,
            counter: u32,
        }

        let mut data = EnumData {
            displays: Vec::new(),
            counter: 0,
        };

        unsafe extern "system" fn monitor_proc(
            hmonitor: HMONITOR,
            _hdc: HDC,
            _lprect: *mut RECT,
            lparam: LPARAM,
        ) -> BOOL {
            let data = unsafe { &mut *(lparam.0 as *mut EnumData) };
            let mut info = MONITORINFOEXW::default();
            info.monitorInfo.cbSize = std::mem::size_of::<MONITORINFOEXW>() as u32;

            if unsafe { GetMonitorInfoW(hmonitor, &mut info.monitorInfo as *mut _ as *mut _) }.as_bool() {
                let r = info.monitorInfo.rcMonitor;
                let is_primary = (info.monitorInfo.dwFlags & MONITORINFOF_PRIMARY) != 0;
                let name = String::from_utf16_lossy(&info.szDevice)
                    .trim_matches(char::from(0))
                    .to_string();

                data.displays.push(Display {
                    id: data.counter,
                    name: if name.is_empty() { format!("Display {}", data.counter) } else { name },
                    bounds: Rect {
                        x: r.left,
                        y: r.top,
                        width: r.right - r.left,
                        height: r.bottom - r.top,
                    },
                    is_primary,
                    scale_factor: 1.0,
                });
                data.counter += 1;
            }
            BOOL(1)
        }

        unsafe {
            let _ = EnumDisplayMonitors(None, None, Some(monitor_proc), LPARAM(&mut data as *mut _ as isize));
        }

        if data.displays.is_empty() {
            // Fallback default display
            data.displays.push(Display {
                id: 0,
                name: "Primary Display".to_string(),
                bounds: Rect {
                    x: 0,
                    y: 0,
                    width: 1920,
                    height: 1080,
                },
                is_primary: true,
                scale_factor: 1.0,
            });
        }

        Ok(data.displays)
    }

    fn get_primary_display(&self) -> NucleusResult<Display> {
        let displays = self.get_displays()?;
        for d in &displays {
            if d.is_primary {
                return Ok(d.clone());
            }
        }
        displays
            .into_iter()
            .next()
            .ok_or_else(|| NucleusError::Platform("No displays detected".to_string()))
    }

    fn get_dpi_for_display(&self, _display_id: u32) -> NucleusResult<f64> {
        Ok(1.0)
    }
}

