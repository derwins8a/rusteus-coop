//! Raw input handling abstractions and event definitions.

use nucleus_common::error::NucleusResult;
use nucleus_common::types::{InputDevice, WindowHandle};

/// Encapsulated raw input event data received from the operating system.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RawInputData {
    /// Native handle or identifier of the originating input device.
    pub device_handle: u64,
    /// Native window message or event code.
    pub message: u32,
    /// Raw payload bytes associated with the input report.
    pub payload: Vec<u8>,
}

/// Trait defining raw input registration, capture, and dispatching operations.
pub trait RawInputHandler: Send + Sync {
    /// Registers raw input devices for the specified target window.
    fn register(&mut self, window_handle: WindowHandle) -> NucleusResult<()>;

    /// Unregisters raw input devices previously hooked.
    fn unregister(&mut self) -> NucleusResult<()>;

    /// Processes an incoming raw input data report.
    fn process_input(&mut self, input: &RawInputData) -> NucleusResult<()>;

    /// Retrieves all enumerated raw input devices.
    fn get_devices(&self) -> Vec<InputDevice>;
}
