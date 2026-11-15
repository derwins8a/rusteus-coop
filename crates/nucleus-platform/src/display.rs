//! Platform-agnostic display and monitor management operations.

use nucleus_common::error::NucleusResult;
use nucleus_common::types::Display;

/// Platform-agnostic display and monitor management operations.
pub trait DisplayManager: Send + Sync {
    /// Retrieve all connected displays.
    fn get_displays(&self) -> NucleusResult<Vec<Display>>;

    /// Retrieve the primary display.
    fn get_primary_display(&self) -> NucleusResult<Display>;

    /// Get the DPI scale factor or DPI value for a specific display by its identifier.
    fn get_dpi_for_display(&self, display_id: u32) -> NucleusResult<f64>;
}
