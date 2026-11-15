//! Display DPI awareness scaling and viewport resolution adjustment.

/// DPI scaling calculation mode matching Nucleus Co-op handler specifications.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum DpiMode {
    /// No scaling applied; passes direct pixel dimensions.
    #[default]
    None = 0,
    /// Scales width and height by system DPI factor.
    Scaled = 1,
    /// Scales width and height by the inverse (1 / DPI) factor.
    InvScaled = 2,
}

impl From<i32> for DpiMode {
    fn from(val: i32) -> Self {
        match val {
            1 => DpiMode::Scaled,
            2 => DpiMode::InvScaled,
            _ => DpiMode::None,
        }
    }
}

/// Tool for computing DPI-aware viewport dimensions and scaling adjustments.
#[derive(Debug, Default, Clone)]
pub struct DpiScalingTool;

impl DpiScalingTool {
    /// Adjusts viewport dimensions (width, height) according to the specified DPI mode and scale factor.
    pub fn calculate_viewport(
        width: i32,
        height: i32,
        dpi_scale: f64,
        mode: DpiMode,
    ) -> (i32, i32) {
        if dpi_scale <= 0.0 || (dpi_scale - 1.0).abs() < f64::EPSILON {
            return (width, height);
        }

        match mode {
            DpiMode::None => (width, height),
            DpiMode::Scaled => (
                (width as f64 * dpi_scale).round() as i32,
                (height as f64 * dpi_scale).round() as i32,
            ),
            DpiMode::InvScaled => (
                (width as f64 / dpi_scale).round() as i32,
                (height as f64 / dpi_scale).round() as i32,
            ),
        }
    }

    /// Converts Windows standard DPI dots (e.g. 96, 120, 144, 192) into a scale multiplier.
    pub fn dpi_to_scale_factor(dpi: u32) -> f64 {
        if dpi == 0 { 1.0 } else { dpi as f64 / 96.0 }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_dpi_scaling_modes() {
        let (w, h) = DpiScalingTool::calculate_viewport(1920, 1080, 1.5, DpiMode::None);
        assert_eq!((w, h), (1920, 1080));

        let (w, h) = DpiScalingTool::calculate_viewport(1920, 1080, 1.5, DpiMode::Scaled);
        assert_eq!((w, h), (2880, 1620));

        let (w, h) = DpiScalingTool::calculate_viewport(1920, 1080, 1.5, DpiMode::InvScaled);
        assert_eq!((w, h), (1280, 720));
    }

    #[test]
    fn test_dpi_to_scale_factor() {
        assert_eq!(DpiScalingTool::dpi_to_scale_factor(96), 1.0);
        assert_eq!(DpiScalingTool::dpi_to_scale_factor(144), 1.5);
        assert_eq!(DpiScalingTool::dpi_to_scale_factor(192), 2.0);
    }
}
