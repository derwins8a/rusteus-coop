//! Linux stub implementation of `DisplayManager`.

#![cfg(target_os = "linux")]

use nucleus_common::error::{NucleusError, NucleusResult};
use nucleus_common::types::Display;
use nucleus_platform::DisplayManager;

/// Linux display manager stub.
#[derive(Default)]
pub struct LinuxDisplayManager;

impl LinuxDisplayManager {
    /// Creates a new `LinuxDisplayManager`.
    pub fn new() -> Self {
        Self
    }
}

impl DisplayManager for LinuxDisplayManager {
    fn get_displays(&self) -> NucleusResult<Vec<Display>> {
        Err(NucleusError::Unsupported("Display enumeration not yet implemented on Linux".into()))
    }

    fn get_primary_display(&self) -> NucleusResult<Display> {
        Err(NucleusError::Unsupported("Primary display query not yet implemented on Linux".into()))
    }

    fn get_dpi_for_display(&self, _display_id: u32) -> NucleusResult<f64> {
        Err(NucleusError::Unsupported("DPI querying not yet implemented on Linux".into()))
    }
}
