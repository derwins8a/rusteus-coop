//! # nucleus-input
//!
//! Input management layer for NucleusCoop.
//!
//! Provides gamepad discovery and state monitoring via `gilrs`, raw input
//! device abstractions, global hotkey capture, and player focus locking.

pub mod combo;
pub mod gamepad;
pub mod hotkeys;
pub mod lock;
pub mod manager;
pub mod raw_input;

pub use combo::{ComboDetector, HoldComboConfig};
pub use gamepad::{DeviceFingerprint, GamepadDevice, GamepadManager};
pub use hotkeys::{
    HotkeyBinding, HotkeyListener, HOTKEY_ID_CLOSE, HOTKEY_ID_COUCH_OSD, HOTKEY_ID_CUTSCENES,
    HOTKEY_ID_LOCK, HOTKEY_ID_MERGER_SWITCH, HOTKEY_ID_REMINDER, HOTKEY_ID_RESET_WINDOWS,
    HOTKEY_ID_SET_FOCUS, HOTKEY_ID_STOP, HOTKEY_ID_SWITCH, HOTKEY_ID_TOPMOST,
};
pub use lock::{InputLock, DEFAULT_LOCK_TOGGLE_KEY};
pub use manager::InputManager;
pub use raw_input::{RawInputData, RawInputHandler};
