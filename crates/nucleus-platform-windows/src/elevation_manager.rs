//! Windows implementation of `ElevationManager`.
//!
//! Uses `OpenProcessToken` / `TokenElevation` to check UAC elevation status,
//! and `ShellExecuteExW` with verb `"runas"` to prompt for elevation.

use nucleus_common::error::{NucleusError, NucleusResult};
use nucleus_platform::ElevationManager;
use std::path::Path;
use windows::Win32::Foundation::CloseHandle;
use windows::Win32::Security::{GetTokenInformation, TOKEN_ELEVATION, TOKEN_QUERY, TokenElevation};
use windows::Win32::System::Threading::{
    GetCurrentProcess, GetExitCodeProcess, INFINITE, OpenProcessToken, WaitForSingleObject,
};
use windows::Win32::UI::Shell::{SEE_MASK_NOCLOSEPROCESS, SHELLEXECUTEINFOW, ShellExecuteExW};
use windows::Win32::UI::WindowsAndMessaging::SW_SHOW;
use windows::core::{HSTRING, PCWSTR};

/// Concrete Windows elevation manager.
#[derive(Default, Debug, Clone)]
pub struct WindowsElevationManager;

impl WindowsElevationManager {
    /// Creates a new `WindowsElevationManager`.
    pub fn new() -> Self {
        Self
    }
}

impl ElevationManager for WindowsElevationManager {
    fn is_elevated(&self) -> bool {
        unsafe {
            let mut token = Default::default();
            if OpenProcessToken(GetCurrentProcess(), TOKEN_QUERY, &mut token).is_ok() {
                let mut elevation = TOKEN_ELEVATION::default();
                let mut return_length = 0u32;
                let size = std::mem::size_of::<TOKEN_ELEVATION>() as u32;
                let success = GetTokenInformation(
                    token,
                    TokenElevation,
                    Some(&mut elevation as *mut _ as *mut _),
                    size,
                    &mut return_length,
                );
                let _ = CloseHandle(token);
                if success.is_ok() {
                    return elevation.TokenIsElevated != 0;
                }
            }
            false
        }
    }

    fn run_elevated(
        &self,
        executable: &Path,
        arguments: &[&str],
        working_dir: Option<&Path>,
        wait: bool,
    ) -> NucleusResult<Option<u32>> {
        let verb = HSTRING::from("runas");
        let file = HSTRING::from(executable.as_os_str());
        let joined_args = arguments.join(" ");
        let params = HSTRING::from(joined_args);
        let dir = working_dir.map(|p| HSTRING::from(p.as_os_str()));

        let mut info = SHELLEXECUTEINFOW {
            cbSize: std::mem::size_of::<SHELLEXECUTEINFOW>() as u32,
            fMask: if wait {
                SEE_MASK_NOCLOSEPROCESS
            } else {
                Default::default()
            },
            hwnd: Default::default(),
            lpVerb: PCWSTR(verb.as_ptr()),
            lpFile: PCWSTR(file.as_ptr()),
            lpParameters: PCWSTR(params.as_ptr()),
            lpDirectory: dir.as_ref().map_or(PCWSTR::null(), |d| PCWSTR(d.as_ptr())),
            nShow: SW_SHOW.0,
            ..Default::default()
        };

        if let Err(e) = unsafe { ShellExecuteExW(&mut info) } {
            return Err(NucleusError::Platform(format!(
                "Failed to run elevated process '{}' (UAC prompt declined or failed): {}",
                executable.display(),
                e
            )));
        }

        if wait && !info.hProcess.is_invalid() {
            unsafe {
                let _ = WaitForSingleObject(info.hProcess, INFINITE);
                let mut exit_code = 0u32;
                let _ = GetExitCodeProcess(info.hProcess, &mut exit_code);
                let _ = CloseHandle(info.hProcess);
                return Ok(Some(exit_code));
            }
        }

        Ok(None)
    }

    fn ensure_symlink_support(&self) -> NucleusResult<bool> {
        // 1. Quick probe: test if unprivileged symlink creation works right now
        let temp_dir =
            std::env::temp_dir().join(format!("nucleus_sym_test_{}", std::process::id()));
        let test_src = temp_dir.join("src.tmp");
        let test_dst = temp_dir.join("dst.tmp");
        let _ = std::fs::create_dir_all(&temp_dir);
        let _ = std::fs::write(&test_src, b"symlink_probe");

        let probe_works = std::os::windows::fs::symlink_file(&test_src, &test_dst).is_ok();
        let _ = std::fs::remove_dir_all(&temp_dir);

        if probe_works {
            tracing::info!("Windows symlink support verified: unprivileged creation is available.");
            return Ok(true);
        }

        if self.is_elevated() {
            tracing::info!("Current process is running elevated: symlink privilege granted.");
            return Ok(true);
        }

        tracing::info!(
            "Unprivileged symlinks disabled. Invoking elevator.exe to enable Windows Developer Mode symlink support..."
        );

        let candidate_paths = [
            std::env::current_exe()
                .ok()
                .and_then(|p| p.parent().map(|d| d.join("elevator.exe"))),
            std::env::current_exe()
                .ok()
                .and_then(|p| p.parent().map(|d| d.join("utils").join("elevator.exe"))),
            Some(std::path::PathBuf::from("elevator.exe")),
            Some(std::path::PathBuf::from("utils").join("elevator.exe")),
            Some(std::path::PathBuf::from(
                r"C:\Games\NucleusCoop242\utils\elevator.exe",
            )),
            Some(std::path::PathBuf::from(
                r"C:\Games\NucleusCoop242\elevator.exe",
            )),
        ];

        for opt_path in candidate_paths.into_iter().flatten() {
            if opt_path.exists() {
                println!(
                    "[Elevation] Invoking elevator at {:?} to enable symlink support...",
                    opt_path
                );
                let res = self.run_elevated(&opt_path, &["enable-symlinks"], None, true);
                match res {
                    Ok(Some(0)) | Ok(None) => {
                        println!(
                            "[Elevation] Elevator successfully enabled Windows Developer Mode symlinks!"
                        );
                        return Ok(true);
                    }
                    Ok(Some(code)) => {
                        tracing::warn!(
                            "Elevator exited with code {}: UAC was declined or failed",
                            code
                        );
                    }
                    Err(e) => {
                        tracing::warn!("Elevator run failed: {}", e);
                    }
                }
                break;
            }
        }

        Ok(false)
    }
}
