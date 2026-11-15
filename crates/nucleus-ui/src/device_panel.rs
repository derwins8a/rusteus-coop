//! Device assignment and gamepad indicator panel component.
//!
//! Visualizes active gamepads, mice, and keyboards with live input activity
//! indicators, battery status, and drag targets for player assignment slots.

/// Panel component presenting connected input hardware.
#[derive(Default)]
pub struct DevicePanelComponent {
    /// Number of detected controller devices.
    pub detected_devices: usize,
}

impl DevicePanelComponent {
    /// Creates a new `DevicePanelComponent`.
    pub fn new() -> Self {
        Self::default()
    }
}
