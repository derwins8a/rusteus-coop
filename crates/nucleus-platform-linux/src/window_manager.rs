//! Linux stub implementation of `WindowManager`.

#![cfg(target_os = "linux")]

use nucleus_common::error::{NucleusError, NucleusResult};
use nucleus_common::types::{Rect, WindowHandle, WindowStyle};
use nucleus_platform::WindowManager;

/// Linux window manager stub.
#[derive(Default)]
pub struct LinuxWindowManager;

impl LinuxWindowManager {
    /// Creates a new `LinuxWindowManager`.
    pub fn new() -> Self {
        Self
    }
}

impl WindowManager for LinuxWindowManager {
    fn find_window_by_title(
        &self,
        _title: &str,
        _partial: bool,
    ) -> NucleusResult<Option<WindowHandle>> {
        Err(NucleusError::Unsupported(
            "Window management not yet implemented on Linux".into(),
        ))
    }

    fn find_windows_by_process(&self, _pid: u32) -> NucleusResult<Vec<WindowHandle>> {
        Err(NucleusError::Unsupported(
            "Window discovery by process not yet implemented on Linux".into(),
        ))
    }

    fn set_window_position(&self, _handle: WindowHandle, _rect: &Rect) -> NucleusResult<()> {
        Err(NucleusError::Unsupported(
            "Window positioning not yet implemented on Linux".into(),
        ))
    }

    fn set_window_style(&self, _handle: WindowHandle, _style: &WindowStyle) -> NucleusResult<()> {
        Err(NucleusError::Unsupported(
            "Window styling not yet implemented on Linux".into(),
        ))
    }

    fn set_topmost(&self, _handle: WindowHandle, _topmost: bool) -> NucleusResult<()> {
        Err(NucleusError::Unsupported(
            "Topmost window flag not yet implemented on Linux".into(),
        ))
    }

    fn remove_borders(&self, _handle: WindowHandle) -> NucleusResult<()> {
        Err(NucleusError::Unsupported(
            "Border removal not yet implemented on Linux".into(),
        ))
    }

    fn get_window_rect(&self, _handle: WindowHandle) -> NucleusResult<Rect> {
        Err(NucleusError::Unsupported(
            "Window rect retrieval not yet implemented on Linux".into(),
        ))
    }

    fn set_foreground(&self, _handle: WindowHandle) -> NucleusResult<()> {
        Err(NucleusError::Unsupported(
            "Foreground focus not yet implemented on Linux".into(),
        ))
    }

    fn get_window_title(&self, _handle: WindowHandle) -> NucleusResult<String> {
        Err(NucleusError::Unsupported(
            "Window title retrieval not yet implemented on Linux".into(),
        ))
    }

    fn set_window_title(&self, _handle: WindowHandle, _title: &str) -> NucleusResult<()> {
        Err(NucleusError::Unsupported(
            "Window title modification not yet implemented on Linux".into(),
        ))
    }

    fn close_window(&self, _handle: WindowHandle) -> NucleusResult<()> {
        Err(NucleusError::Unsupported(
            "Window close not yet implemented on Linux".into(),
        ))
    }
}
