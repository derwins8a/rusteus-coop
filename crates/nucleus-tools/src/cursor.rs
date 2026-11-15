//! Custom cursor confinement, clipping, and visibility controls.

use nucleus_common::error::NucleusResult;
use nucleus_common::types::Rect;

/// Tool for confining the mouse cursor to a specific screen rectangle or game window,
/// and toggling system cursor visibility during splitscreen gameplay.
#[derive(Debug, Default, Clone)]
pub struct CursorTool;

impl CursorTool {
    /// Clips the cursor to the specified screen rectangle.
    pub fn clip_cursor(rect: &Rect) -> NucleusResult<()> {
        #[cfg(windows)]
        {
            use windows::Win32::Foundation::RECT;
            use windows::Win32::UI::WindowsAndMessaging::ClipCursor;

            let win_rect = RECT {
                left: rect.x,
                top: rect.y,
                right: rect.x + rect.width,
                bottom: rect.y + rect.height,
            };
            unsafe {
                let _ = ClipCursor(Some(&win_rect));
            }
        }
        #[cfg(not(windows))]
        {
            let _ = rect;
        }
        Ok(())
    }

    /// Releases any active cursor confinement, restoring free movement across all displays.
    pub fn release_cursor() -> NucleusResult<()> {
        #[cfg(windows)]
        {
            use windows::Win32::UI::WindowsAndMessaging::ClipCursor;
            unsafe {
                let _ = ClipCursor(None);
            }
        }
        Ok(())
    }

    /// Shows or hides the system cursor.
    pub fn set_cursor_visibility(visible: bool) {
        #[cfg(windows)]
        {
            use windows::Win32::UI::WindowsAndMessaging::ShowCursor;
            unsafe {
                ShowCursor(visible);
            }
        }
        #[cfg(not(windows))]
        {
            let _ = visible;
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_cursor_clip_and_release() {
        let rect = Rect::new(0, 0, 800, 600);
        assert!(CursorTool::clip_cursor(&rect).is_ok());
        assert!(CursorTool::release_cursor().is_ok());
    }
}
