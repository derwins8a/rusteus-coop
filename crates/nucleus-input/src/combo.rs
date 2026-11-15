//! Gamepad combination detector for universal overlay and couch shortcuts.
//!
//! Provides continuous hold detection for controller button combos (such as
//! Back + Start held for 1.5 seconds) to trigger overlays, emergency stops, or
//! couch mode navigation without interfering with normal in-game button taps.

use std::time::{Duration, Instant};

pub use crate::gamepad::DeviceFingerprint;

/// Configuration for continuous hold button combination detection.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct HoldComboConfig {
    /// Duration that the combo buttons must be continuously held down.
    pub hold_duration: Duration,
}

impl Default for HoldComboConfig {
    fn default() -> Self {
        Self {
            hold_duration: Duration::from_millis(1500),
        }
    }
}

/// Detects a held button combination over a continuous duration.
#[derive(Debug, Clone)]
pub struct ComboDetector {
    config: HoldComboConfig,
    held_since: Option<Instant>,
    button1_down: bool,
    button2_down: bool,
}

impl ComboDetector {
    /// Creates a new `ComboDetector` with the specified configuration.
    pub fn new(config: HoldComboConfig) -> Self {
        Self {
            config,
            held_since: None,
            button1_down: false,
            button2_down: false,
        }
    }

    /// Updates the state of the two combo buttons at the given timestamp.
    ///
    /// If both buttons are pressed simultaneously and were not previously being held,
    /// the hold timer starts at `now`. If either button is released, the timer resets.
    pub fn update_buttons(&mut self, btn1: bool, btn2: bool, now: Instant) {
        self.button1_down = btn1;
        self.button2_down = btn2;

        if btn1 && btn2 {
            if self.held_since.is_none() {
                self.held_since = Some(now);
            }
        } else {
            self.held_since = None;
        }
    }

    /// Checks if the combo has been held continuously for at least the configured duration.
    pub fn is_triggered(&self, now: Instant) -> bool {
        match self.held_since {
            Some(start) => now.saturating_duration_since(start) >= self.config.hold_duration,
            None => false,
        }
    }

    /// Resets the hold timer and internal button states.
    pub fn reset(&mut self) {
        self.held_since = None;
        self.button1_down = false;
        self.button2_down = false;
    }

    /// Returns whether the combo is currently being held.
    pub fn is_holding(&self) -> bool {
        self.held_since.is_some()
    }

    /// Returns how long the combo has been held continuously, or `Duration::ZERO` if not held.
    pub fn held_duration(&self, now: Instant) -> Duration {
        match self.held_since {
            Some(start) => now.saturating_duration_since(start),
            None => Duration::ZERO,
        }
    }

    /// Returns the normalized progress of the hold combo from 0.0 to 1.0.
    pub fn hold_progress(&self, now: Instant) -> f32 {
        if self.config.hold_duration.is_zero() {
            return if self.is_holding() { 1.0 } else { 0.0 };
        }
        let elapsed = self.held_duration(now);
        (elapsed.as_secs_f32() / self.config.hold_duration.as_secs_f32()).clamp(0.0, 1.0)
    }

    /// Returns the hold combo configuration.
    pub fn config(&self) -> &HoldComboConfig {
        &self.config
    }

    /// Updates button states based on a `gilrs::Event`.
    ///
    /// Recognizes `gilrs::Button::Select` (Back / View) and `gilrs::Button::Start` (Start / Menu).
    pub fn handle_gilrs_event(&mut self, event: &gilrs::Event, now: Instant) {
        match event.event {
            gilrs::EventType::ButtonPressed(button, _) => {
                if button == gilrs::Button::Select {
                    self.button1_down = true;
                    self.update_buttons(true, self.button2_down, now);
                } else if button == gilrs::Button::Start {
                    self.button2_down = true;
                    self.update_buttons(self.button1_down, true, now);
                }
            }
            gilrs::EventType::ButtonReleased(button, _) => {
                if button == gilrs::Button::Select {
                    self.button1_down = false;
                    self.update_buttons(false, self.button2_down, now);
                } else if button == gilrs::Button::Start {
                    self.button2_down = false;
                    self.update_buttons(self.button1_down, false, now);
                }
            }
            gilrs::EventType::Disconnected | gilrs::EventType::Dropped => {
                self.reset();
            }
            _ => {}
        }
    }

    /// Updates state from a `gilrs::Gamepad` instance directly.
    pub fn update_from_gamepad(&mut self, gamepad: &gilrs::Gamepad, now: Instant) {
        let btn1 = gamepad.is_pressed(gilrs::Button::Select);
        let btn2 = gamepad.is_pressed(gilrs::Button::Start);
        self.update_buttons(btn1, btn2, now);
    }
}

impl Default for ComboDetector {
    fn default() -> Self {
        Self::new(HoldComboConfig::default())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_hold_progress_and_reset() {
        let mut detector = ComboDetector::new(HoldComboConfig {
            hold_duration: Duration::from_millis(1000),
        });
        let start = Instant::now();

        assert_eq!(detector.hold_progress(start), 0.0);
        assert!(!detector.is_holding());

        detector.update_buttons(true, true, start);
        assert!(detector.is_holding());
        assert_eq!(
            detector.held_duration(start + Duration::from_millis(500)),
            Duration::from_millis(500)
        );
        assert!((detector.hold_progress(start + Duration::from_millis(500)) - 0.5).abs() < 1e-4);

        detector.reset();
        assert!(!detector.is_holding());
        assert_eq!(
            detector.held_duration(start + Duration::from_millis(500)),
            Duration::ZERO
        );
    }
}
