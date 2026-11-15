//! Primary input coordinator managing gamepads, raw input, hotkeys, and lock states.

use crate::gamepad::GamepadManager;
use crate::hotkeys::HotkeyListener;
use crate::lock::InputLock;
use crate::raw_input::RawInputHandler;
use nucleus_common::error::NucleusResult;
use nucleus_platform::PlatformServices;
use std::sync::Arc;

/// Central input manager orchestrating gamepads, raw input handlers, hotkeys, and locks.
pub struct InputManager {
    /// Gamepad subsystem manager.
    gamepad: GamepadManager,
    /// Optional platform raw input handler.
    raw_input: Option<Box<dyn RawInputHandler>>,
    /// Global hotkey listener.
    hotkeys: HotkeyListener,
    /// Focus lock coordinator.
    lock: InputLock,
    /// Reference to platform services.
    #[allow(dead_code)]
    platform: Option<Arc<dyn PlatformServices>>,
}

impl InputManager {
    /// Creates a new `InputManager` with the specified platform services.
    pub fn new(platform: Option<Arc<dyn PlatformServices>>) -> NucleusResult<Self> {
        let lock = InputLock::new();
        let mut hotkeys = HotkeyListener::new();
        // Register default toggle key (VK_END = 0x23)
        let _ = hotkeys.register_hotkey(lock.toggle_key(), lock.toggle_key(), 0);
        // Also register Ctrl+Q (0x51 with modifier 1) as alternative convenience hotkey
        let _ = hotkeys.register_hotkey(0x51, 0x51, 1);

        Ok(Self {
            gamepad: GamepadManager::new()?,
            raw_input: None,
            hotkeys,
            lock,
            platform,
        })
    }

    /// Configures hotkeys and input lock toggle key based on persistent `AppSettings`.
    pub fn register_settings(&mut self, settings: &nucleus_profile::AppSettings) -> NucleusResult<()> {
        let toggle_key = settings.lock_key_code();
        self.lock.set_toggle_key(toggle_key);
        let _ = self.hotkeys.register_hotkey(toggle_key, toggle_key, 0);
        self.hotkeys.register_from_settings(settings)?;
        Ok(())
    }

    /// Sets the platform-specific raw input handler implementation.
    pub fn set_raw_input_handler(&mut self, handler: Box<dyn RawInputHandler>) {
        self.raw_input = Some(handler);
    }

    /// Performs periodic input polling and state updates, returning any triggered hotkey IDs.
    pub fn update(&mut self) -> NucleusResult<Vec<u32>> {
        self.gamepad.poll_events()?;
        let triggered = self.hotkeys.poll_hotkeys();
        let mut result = Vec::new();

        for hotkey_id in triggered {
            if hotkey_id == crate::HOTKEY_ID_LOCK || hotkey_id == self.lock.toggle_key() {
                self.lock.toggle_with_osd(self.platform.as_deref().map(|p| p.osd_manager()));
                result.push(hotkey_id);
            } else if self.lock.is_locked() {
                // Parity with C# HotkeyListener.cs:161:
                // Show "Unlock Inputs First (Press <Key> Key)" and suppress other actions
                if let Some(ref p) = self.platform {
                    let key_name = match self.lock.toggle_key() {
                        0x23 => "End",
                        0x24 => "Home",
                        0x2E => "Delete",
                        0x6A => "Multiply",
                        0x70 => "F1",
                        0x71 => "F2",
                        0x72 => "F3",
                        0x73 => "F4",
                        0x74 => "F5",
                        0x75 => "F6",
                        0x76 => "F7",
                        0x77 => "F8",
                        0x78 => "F9",
                        0x79 => "F10",
                        0x7A => "F11",
                        0x7B => "F12",
                        0x60 => "Numpad 0",
                        0x61 => "Numpad 1",
                        _ => "Lock",
                    };
                    p.osd_manager().show(1600, &format!("Unlock Inputs First (Press {} Key)", key_name));
                }
            } else {
                result.push(hotkey_id);
            }
        }
        Ok(result)
    }

    /// Returns a reference to the gamepad manager.
    pub fn gamepad(&self) -> &GamepadManager {
        &self.gamepad
    }

    /// Returns a mutable reference to the gamepad manager.
    pub fn gamepad_mut(&mut self) -> &mut GamepadManager {
        &mut self.gamepad
    }

    /// Returns a reference to the hotkey listener.
    pub fn hotkeys(&self) -> &HotkeyListener {
        &self.hotkeys
    }

    /// Returns a mutable reference to the hotkey listener.
    pub fn hotkeys_mut(&mut self) -> &mut HotkeyListener {
        &mut self.hotkeys
    }

    /// Returns a reference to the input lock state manager.
    pub fn lock(&self) -> &InputLock {
        &self.lock
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_input_manager_lifecycle() {
        let mut mgr = InputManager::new(None).expect("Failed to create InputManager");

        assert!(!mgr.lock().is_locked());
        mgr.lock().set_locked(true);
        assert!(mgr.lock().is_locked());

        mgr.update().expect("Update should succeed");
    }

    #[test]
    fn test_input_manager_register_settings() {
        let mut mgr = InputManager::new(None).expect("Failed to create InputManager");
        let settings = nucleus_profile::AppSettings::default();
        mgr.register_settings(&settings).expect("Register settings should succeed");
        assert!(mgr.hotkeys().count() >= 8);
    }
}
