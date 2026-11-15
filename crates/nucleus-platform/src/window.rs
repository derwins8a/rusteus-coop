//! Platform-agnostic window management operations.

use nucleus_common::error::NucleusResult;
use nucleus_common::types::{Rect, WindowHandle, WindowStyle};

/// Platform-agnostic window management operations.
pub trait WindowManager: Send + Sync {
    /// Find a window by its title (exact or partial match).
    fn find_window_by_title(
        &self,
        title: &str,
        partial: bool,
    ) -> NucleusResult<Option<WindowHandle>>;

    /// Find all windows belonging to a process.
    fn find_windows_by_process(&self, pid: u32) -> NucleusResult<Vec<WindowHandle>>;

    /// Set window position and size.
    fn set_window_position(&self, handle: WindowHandle, rect: &Rect) -> NucleusResult<()>;

    /// Set window position and size with topmost control (not_top_most guard).
    fn set_window_position_ext(
        &self,
        handle: WindowHandle,
        rect: &Rect,
        not_top_most: bool,
    ) -> NucleusResult<()> {
        let _ = not_top_most;
        self.set_window_position(handle, rect)
    }

    /// Minimize all windows across the desktop (MinimizeEverything).
    fn minimize_all(&self) -> NucleusResult<()> {
        Ok(())
    }

    /// Apply custom window style and extended style masks (WindowStyleValues / ExtWindowStyleValues).
    fn apply_custom_window_styles(
        &self,
        handle: WindowHandle,
        style_values: &[String],
        ext_style_values: &[String],
    ) -> NucleusResult<()> {
        let _ = (handle, style_values, ext_style_values);
        Ok(())
    }

    /// Set window style flags.
    fn set_window_style(&self, handle: WindowHandle, style: &WindowStyle) -> NucleusResult<()>;

    /// Set or remove topmost flag.
    fn set_topmost(&self, handle: WindowHandle, topmost: bool) -> NucleusResult<()>;

    /// Remove window borders/decorations.
    fn remove_borders(&self, handle: WindowHandle) -> NucleusResult<()>;

    /// Get current window rectangle.
    fn get_window_rect(&self, handle: WindowHandle) -> NucleusResult<Rect>;

    /// Bring window to foreground.
    fn set_foreground(&self, handle: WindowHandle) -> NucleusResult<()>;

    /// Get window title text.
    fn get_window_title(&self, handle: WindowHandle) -> NucleusResult<String>;

    /// Set window title text.
    fn set_window_title(&self, handle: WindowHandle, title: &str) -> NucleusResult<()>;

    /// Send a close message to the window.
    fn close_window(&self, handle: WindowHandle) -> NucleusResult<()>;

    /// Minimize the window.
    fn minimize_window(&self, _handle: WindowHandle) -> NucleusResult<()> {
        Ok(())
    }

    /// Restore the window from minimized state.
    fn restore_window(&self, _handle: WindowHandle) -> NucleusResult<()> {
        Ok(())
    }

    /// Checks if the window is currently minimized (iconic).
    fn is_minimized(&self, _handle: WindowHandle) -> NucleusResult<bool> {
        Ok(false)
    }

    /// Unclip the mouse cursor if it was confined to a window or rect.
    fn unclip_cursor(&self) -> NucleusResult<()> {
        Ok(())
    }

    /// Configure an out-of-process transparent topmost DWM overlay window.
    ///
    /// Applies `WS_EX_TOPMOST | WS_EX_TOOLWINDOW | WS_EX_LAYERED`, sets position/size
    /// to the target display rect, extends the DWM frame into the client area,
    /// and unclips the mouse cursor.
    fn configure_overlay_window(&self, _handle: WindowHandle, _rect: &Rect) -> NucleusResult<()> {
        Ok(())
    }

    /// Resolves the physical monitor bounding rect enclosing the provided viewport rect.
    fn get_monitor_bounds_for_rect(&self, _rect: &Rect) -> NucleusResult<Rect> {
        Ok(Rect::new(0, 0, 1920, 1080))
    }

    /// Restores input focus to a running game instance window and brings it to the top.
    fn restore_game_focus(&self, _game_handle: WindowHandle) -> NucleusResult<()> {
        Ok(())
    }
}
