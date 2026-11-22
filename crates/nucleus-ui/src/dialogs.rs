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
            .args(["-NoProfile", "-STA", "-Command", &script])
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
        // Ensure filter conforms to .NET Windows Forms "Description|Pattern|..." format.
        let valid_filter = if filter.contains('|') {
            filter.to_string()
        } else {
            format!("Supported Files ({0})|{0}|All Files (*.*)|*.*", filter)
        };

        let script = format!(
            "[System.Reflection.Assembly]::LoadWithPartialName('System.Windows.Forms') | Out-Null; \
            $dlg = New-Object System.Windows.Forms.OpenFileDialog; \
            $dlg.Title = '{}'; \
            $dlg.Filter = '{}'; \
            if ($dlg.ShowDialog() -eq [System.Windows.Forms.DialogResult]::OK) {{ \
                [Console]::Out.Write($dlg.FileName) \
            }}",
            title.replace('\'', "''"),
            valid_filter.replace('\'', "''")
        );
        let output = std::process::Command::new("powershell")
            .args(["-NoProfile", "-STA", "-Command", &script])
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

/// Opens a native file selection dialog asynchronously on a dedicated background thread,
/// ensuring the main GPUI thread and UI rendering never freeze and no ambient Tokio runtime is required.
pub async fn pick_file_async(
    title: impl Into<String>,
    filter: impl Into<String>,
) -> Option<PathBuf> {
    let t = title.into();
    let f = filter.into();
    let (tx, rx) = futures::channel::oneshot::channel();
    std::thread::spawn(move || {
        let res = pick_file(&t, &f);
        let _ = tx.send(res);
    });
    rx.await.ok().flatten()
}

/// Opens a native folder selection dialog asynchronously on a dedicated background thread,
/// ensuring the main GPUI thread and UI rendering never freeze and no ambient Tokio runtime is required.
pub async fn pick_folder_async(title: impl Into<String>) -> Option<PathBuf> {
    let t = title.into();
    let (tx, rx) = futures::channel::oneshot::channel();
    std::thread::spawn(move || {
        let res = pick_folder(&t);
        let _ = tx.send(res);
    });
    rx.await.ok().flatten()
}
