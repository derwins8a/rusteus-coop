//! Native file and folder dialogs for NucleusCoop.
//!
//! Provides platform-native dialogs for selecting directories (such as handlers folder)
//! and executables (when manually adding games to library).

use std::path::PathBuf;

/// Opens a native folder selection dialog and returns the chosen directory path.
pub fn pick_folder(title: &str) -> Option<PathBuf> {
    #[cfg(windows)]
    {
        // Use PowerShell with -STA and System.Windows.Forms FolderBrowserDialog
        let script = format!(
            "[System.Reflection.Assembly]::LoadWithPartialName('System.Windows.Forms') | Out-Null; \
            $dlg = New-Object System.Windows.Forms.FolderBrowserDialog; \
            $dlg.Description = '{}'; \
            $dlg.ShowNewFolderButton = $true; \
            if ($dlg.ShowDialog() -eq [System.Windows.Forms.DialogResult]::OK) {{ \
                [Console]::Out.Write($dlg.SelectedPath) \
            }}",
            title.replace('\'', "''")
        );
        let output = std::process::Command::new("powershell")
            .args(["-NoProfile", "-NonInteractive", "-STA", "-Command", &script])
            .output()
            .ok()?;
        let path_str = String::from_utf8_lossy(&output.stdout).trim().to_string();
        if !path_str.is_empty() && std::path::Path::new(&path_str).exists() {
            return Some(PathBuf::from(path_str));
        }
    }

    #[cfg(not(windows))]
    {
        let _ = title;
    }

    None
}

/// Opens a native file selection dialog and returns the chosen file path.
pub fn pick_file(title: &str, filter: &str) -> Option<PathBuf> {
    #[cfg(windows)]
    {
        let script = format!(
            "[System.Reflection.Assembly]::LoadWithPartialName('System.Windows.Forms') | Out-Null; \
            $dlg = New-Object System.Windows.Forms.OpenFileDialog; \
            $dlg.Title = '{}'; \
            $dlg.Filter = '{}'; \
            if ($dlg.ShowDialog() -eq [System.Windows.Forms.DialogResult]::OK) {{ \
                [Console]::Out.Write($dlg.FileName) \
            }}",
            title.replace('\'', "''"),
            filter.replace('\'', "''")
        );
        let output = std::process::Command::new("powershell")
            .args(["-NoProfile", "-NonInteractive", "-STA", "-Command", &script])
            .output()
            .ok()?;
        let path_str = String::from_utf8_lossy(&output.stdout).trim().to_string();
        if !path_str.is_empty() && std::path::Path::new(&path_str).exists() {
            return Some(PathBuf::from(path_str));
        }
    }

    #[cfg(not(windows))]
    {
        let _ = (title, filter);
    }

    None
}

/// Opens a native file selection dialog asynchronously on a background blocking thread,
/// ensuring the main GPUI thread and UI rendering never freeze.
pub async fn pick_file_async(
    title: impl Into<String>,
    filter: impl Into<String>,
) -> Option<PathBuf> {
    let t = title.into();
    let f = filter.into();
    tokio::task::spawn_blocking(move || pick_file(&t, &f))
        .await
        .ok()
        .flatten()
}

/// Opens a native folder selection dialog asynchronously on a background blocking thread,
/// ensuring the main GPUI thread and UI rendering never freeze.
pub async fn pick_folder_async(title: impl Into<String>) -> Option<PathBuf> {
    let t = title.into();
    tokio::task::spawn_blocking(move || pick_folder(&t))
        .await
        .ok()
        .flatten()
}
