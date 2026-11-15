//! Platform-agnostic On-Screen Display (OSD) notification operations.

/// Platform-agnostic On-Screen Display (OSD) management operations.
pub trait OsdManager: Send + Sync {
    /// Displays an on-screen notification for `duration_ms` with the specified text.
    fn show(&self, duration_ms: u32, text: &str);

    /// Immediately hides the currently active OSD notification.
    fn hide(&self);

    /// Returns the currently active OSD notification text, if visible.
    fn current_message(&self) -> Option<String>;

    /// Sets the text and border color for the OSD overlay (e.g. "255,255,255" or "#ffffff").
    fn set_color(&self, color: &str);
}
