//! Disk scanning progress and game discovery view.
//!
//! Interactive UI for initiating disk searches, watching directory traversal progress,
//! and reviewing discovered game executables for import into the library.

use nucleus_engine::scanner::FoundGame;

/// View component displaying disk scan progress and discovered results.
#[derive(Default)]
pub struct DiskScannerView {
    /// Currently scanned path or directory description.
    pub current_path: String,
    /// List of games discovered during the scan.
    pub found_games: Vec<FoundGame>,
    /// Whether scanning is actively in progress.
    pub is_scanning: bool,
}

impl DiskScannerView {
    /// Creates a new `DiskScannerView`.
    pub fn new() -> Self {
        Self::default()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_disk_scanner_view() {
        let mut view = DiskScannerView::new();
        assert!(!view.is_scanning);
        assert!(view.found_games.is_empty());

        view.is_scanning = true;
        view.current_path = "C:\\Games".to_string();
        assert!(view.is_scanning);
        assert_eq!(view.current_path, "C:\\Games");
    }
}
