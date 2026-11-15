//! Download progress and confirmation dialog modal.
//!
//! Presents download progress bars, extraction status, change notes, and
//! confirmation prompts when downloading handler packages from the Hub.

/// Modal dialog component managing download status and progress display.
#[derive(Default)]
pub struct DownloadDialog {
    /// Handler package name being fetched.
    pub package_name: String,
    /// Download progress percentage (0.0 to 100.0).
    pub progress_percent: f32,
    /// Whether the download operation has completed.
    pub is_finished: bool,
}

impl DownloadDialog {
    /// Creates a new `DownloadDialog` for the specified package.
    pub fn new(package_name: impl Into<String>) -> Self {
        Self {
            package_name: package_name.into(),
            progress_percent: 0.0,
            is_finished: false,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_download_dialog() {
        let mut dialog = DownloadDialog::new("portal2-split");
        assert_eq!(dialog.package_name, "portal2-split");
        assert_eq!(dialog.progress_percent, 0.0);
        assert!(!dialog.is_finished);

        dialog.progress_percent = 100.0;
        dialog.is_finished = true;
        assert!(dialog.is_finished);
    }
}
