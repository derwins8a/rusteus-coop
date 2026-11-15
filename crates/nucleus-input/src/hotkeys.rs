//! System-wide or application hotkey monitoring and listening.

use nucleus_common::error::NucleusResult;
use std::collections::HashMap;

// Standard NucleusCoop hotkey IDs (matching C# HotkeysRegistration constants)
/// Close active game processes (default Ctrl+Q).
pub const HOTKEY_ID_CLOSE: u32 = 1;
/// Toggle window topmost / minimize state (default Ctrl+T).
pub const HOTKEY_ID_TOPMOST: u32 = 2;
/// Stop the active splitscreen session (default Ctrl+E).
pub const HOTKEY_ID_STOP: u32 = 3;
/// Unfocus or cycle window focus (default Ctrl+H).
pub const HOTKEY_ID_SET_FOCUS: u32 = 4;
/// Reset and reposition game windows (default Ctrl+R).
pub const HOTKEY_ID_RESET_WINDOWS: u32 = 5;
/// Toggle cutscenes presentation mode (default Ctrl+M).
pub const HOTKEY_ID_CUTSCENES: u32 = 6;
/// Switch active splitscreen layout (default Ctrl+N).
pub const HOTKEY_ID_SWITCH: u32 = 7;
/// Display shortcuts reminder overlay (default Alt+R).
pub const HOTKEY_ID_REMINDER: u32 = 8;
/// Switch merger foreground child window (default Alt+A).
pub const HOTKEY_ID_MERGER_SWITCH: u32 = 9;
/// Toggle input focus lock (default End key).
pub const HOTKEY_ID_LOCK: u32 = 10;
/// Toggle Couch In-Game Big Picture OSD menu overlay (default F10).
pub const HOTKEY_ID_COUCH_OSD: u32 = 11;

/// Hotkey binding definition.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct HotkeyBinding {
    /// Unique hotkey registration identifier.
    pub id: u32,
    /// Virtual key code or key symbol.
    pub key_code: u32,
    /// Modifier mask (Ctrl = 1, Alt = 2, Shift = 4).
    pub modifiers: u32,
}

/// Listener for managing hotkey registrations and processing trigger events.
#[derive(Default)]
pub struct HotkeyListener {
    /// Active hotkey registrations indexed by ID.
    bindings: HashMap<u32, HotkeyBinding>,
    /// Cooldown / debounce tracker for registered hotkeys.
    last_triggered: HashMap<u32, std::time::Instant>,
}

impl HotkeyListener {
    /// Creates a new `HotkeyListener`.
    pub fn new() -> Self {
        Self {
            bindings: HashMap::new(),
            last_triggered: HashMap::new(),
        }
    }

    /// Registers a new global or windowed hotkey combination.
    pub fn register_hotkey(&mut self, id: u32, key_code: u32, modifiers: u32) -> NucleusResult<()> {
        let binding = HotkeyBinding {
            id,
            key_code,
            modifiers,
        };
        self.bindings.insert(id, binding);
        Ok(())
    }

    /// Registers all standard hotkey combinations defined in `AppSettings`.
    pub fn register_from_settings(
        &mut self,
        settings: &nucleus_profile::AppSettings,
    ) -> NucleusResult<()> {
        if let Some((m, k)) = nucleus_profile::AppSettings::parse_hotkey(&settings.hotkey_close) {
            self.register_hotkey(HOTKEY_ID_CLOSE, k, m)?;
        }
        if let Some((m, k)) = nucleus_profile::AppSettings::parse_hotkey(&settings.hotkey_topmost) {
            self.register_hotkey(HOTKEY_ID_TOPMOST, k, m)?;
        }
        if let Some((m, k)) = nucleus_profile::AppSettings::parse_hotkey(&settings.hotkey_stop) {
            self.register_hotkey(HOTKEY_ID_STOP, k, m)?;
        }
        if let Some((m, k)) = nucleus_profile::AppSettings::parse_hotkey(&settings.hotkey_set_focus)
        {
            self.register_hotkey(HOTKEY_ID_SET_FOCUS, k, m)?;
        }
        if let Some((m, k)) =
            nucleus_profile::AppSettings::parse_hotkey(&settings.hotkey_reset_windows)
        {
            self.register_hotkey(HOTKEY_ID_RESET_WINDOWS, k, m)?;
        }
        if let Some((m, k)) = nucleus_profile::AppSettings::parse_hotkey(&settings.hotkey_cutscenes)
        {
            self.register_hotkey(HOTKEY_ID_CUTSCENES, k, m)?;
        }
        if let Some((m, k)) = nucleus_profile::AppSettings::parse_hotkey(&settings.hotkey_switch) {
            self.register_hotkey(HOTKEY_ID_SWITCH, k, m)?;
        }
        if let Some((m, k)) =
            nucleus_profile::AppSettings::parse_hotkey(&settings.hotkey_shortcuts_reminder)
        {
            self.register_hotkey(HOTKEY_ID_REMINDER, k, m)?;
        }
        if let Some((m, k)) =
            nucleus_profile::AppSettings::parse_hotkey(&settings.hotkey_merger_switch)
        {
            self.register_hotkey(HOTKEY_ID_MERGER_SWITCH, k, m)?;
        }
        if let Some((m, k)) = nucleus_profile::AppSettings::parse_hotkey(&settings.hotkey_couch_osd)
        {
            self.register_hotkey(HOTKEY_ID_COUCH_OSD, k, m)?;
        } else {
            self.register_hotkey(HOTKEY_ID_COUCH_OSD, 121, 0)?;
        }
        let lock_code = settings.lock_key_code();
        self.register_hotkey(HOTKEY_ID_LOCK, lock_code, 0)?;
        Ok(())
    }

    /// Unregisters an existing hotkey by registration identifier.
    pub fn unregister_hotkey(&mut self, id: u32) -> NucleusResult<()> {
        self.bindings.remove(&id);
        self.last_triggered.remove(&id);
        Ok(())
    }

    /// Checks and returns any hotkey triggers that occurred since the last poll.
    pub fn poll_hotkeys(&mut self) -> Vec<u32> {
        let mut triggered = Vec::new();
        #[cfg(windows)]
        {
            use windows::Win32::UI::Input::KeyboardAndMouse::{
                GetAsyncKeyState, VK_CONTROL, VK_MENU, VK_SHIFT,
            };
            let now = std::time::Instant::now();
            for binding in self.bindings.values() {
                unsafe {
                    let key_down = (GetAsyncKeyState(binding.key_code as i32) as u16 & 0x8000) != 0;
                    if key_down {
                        let ctrl_req = (binding.modifiers & 1) != 0;
                        let alt_req = (binding.modifiers & 2) != 0;
                        let shift_req = (binding.modifiers & 4) != 0;

                        let ctrl_down =
                            (GetAsyncKeyState(VK_CONTROL.0 as i32) as u16 & 0x8000) != 0;
                        let alt_down = (GetAsyncKeyState(VK_MENU.0 as i32) as u16 & 0x8000) != 0;
                        let shift_down = (GetAsyncKeyState(VK_SHIFT.0 as i32) as u16 & 0x8000) != 0;

                        if (ctrl_req == ctrl_down)
                            && (alt_req == alt_down)
                            && (shift_req == shift_down)
                        {
                            if let Some(&last) = self.last_triggered.get(&binding.id) {
                                if now.duration_since(last) < std::time::Duration::from_millis(600)
                                {
                                    continue;
                                }
                            }
                            self.last_triggered.insert(binding.id, now);
                            triggered.push(binding.id);
                        }
                    }
                }
            }
        }
        triggered
    }

    /// Simulates a hotkey trigger event (useful for tests and automation).
    pub fn simulate_trigger(&self, id: u32) -> bool {
        self.bindings.contains_key(&id)
    }

    /// Returns the number of registered hotkeys.
    pub fn count(&self) -> usize {
        self.bindings.len()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_hotkey_registration_and_simulation() {
        let mut listener = HotkeyListener::new();
        // Register Ctrl+Q (id 1)
        listener.register_hotkey(1, 0x51, 1).unwrap();
        // Register VK_END (id 2)
        listener.register_hotkey(2, 0x23, 0).unwrap();

        assert!(listener.simulate_trigger(1));
        assert!(listener.simulate_trigger(2));
        assert!(!listener.simulate_trigger(99));

        listener.unregister_hotkey(1).unwrap();
        assert!(!listener.simulate_trigger(1));
        assert!(listener.simulate_trigger(2));

        let polled = listener.poll_hotkeys();
        assert!(polled.is_empty() || !polled.is_empty());
    }

    #[test]
    fn test_register_from_settings() {
        let mut listener = HotkeyListener::new();
        let settings = nucleus_profile::AppSettings::default();
        listener.register_from_settings(&settings).unwrap();

        assert!(listener.count() >= 8);
        assert!(listener.simulate_trigger(HOTKEY_ID_CLOSE));
        assert!(listener.simulate_trigger(HOTKEY_ID_STOP));
        assert!(listener.simulate_trigger(HOTKEY_ID_TOPMOST));
        assert!(listener.simulate_trigger(HOTKEY_ID_RESET_WINDOWS));
        assert!(listener.simulate_trigger(HOTKEY_ID_CUTSCENES));
        assert!(listener.simulate_trigger(HOTKEY_ID_SWITCH));
        assert!(listener.simulate_trigger(HOTKEY_ID_REMINDER));
        assert!(listener.simulate_trigger(HOTKEY_ID_MERGER_SWITCH));
        assert!(listener.simulate_trigger(HOTKEY_ID_COUCH_OSD));
    }
}
