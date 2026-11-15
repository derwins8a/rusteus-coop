//! Standalone Elevator binary (`elevator.exe`).
//!
//! Checks if running with administrator privileges. If not, requests elevation via Windows UAC
//! (`ShellExecuteExW` with verb `"runas"`) to execute target commands or tools requiring elevation.

use nucleus_platform::ElevationManager;
use std::env;
use std::process::{Command, exit};

fn main() {
    let args: Vec<String> = env::args().skip(1).collect();
    if args.is_empty() {
        println!("NucleusCoop Elevator Tool");
        println!("Usage: elevator <command> [args...]");
        println!();
        println!("If not already running as Administrator, this triggers the Windows UAC dialog");
        println!("and executes the target command with elevated privileges.");
        exit(0);
    }

    #[cfg(windows)]
    let elev_mgr = nucleus_platform_windows::WindowsElevationManager::new();

    #[cfg(not(windows))]
    let elev_mgr = nucleus_platform_linux::LinuxElevationManager::new();

    if !elev_mgr.is_elevated() {
        let current_exe = match env::current_exe() {
            Ok(p) => p,
            Err(e) => {
                eprintln!(
                    "[Elevator] Failed to determine current executable path: {}",
                    e
                );
                exit(1);
            }
        };

        let arg_refs: Vec<&str> = args.iter().map(|s| s.as_str()).collect();
        match elev_mgr.run_elevated(&current_exe, &arg_refs, None, true) {
            Ok(Some(code)) => exit(code as i32),
            Ok(None) => exit(0),
            Err(e) => {
                eprintln!("[Elevator] UAC elevation request failed: {}", e);
                exit(1);
            }
        }
    }

    // Process is running with elevated administrator rights:
    if args.len() == 1 && args[0].eq_ignore_ascii_case("enable-symlinks") {
        #[cfg(windows)]
        {
            use winreg::RegKey;
            use winreg::enums::*;
            let hklm = RegKey::predef(HKEY_LOCAL_MACHINE);
            let key_path = r"SOFTWARE\Microsoft\Windows\CurrentVersion\AppModelUnlock";
            match hklm.create_subkey(key_path) {
                Ok((key, _)) => {
                    if let Err(e) = key.set_value("AllowDevelopmentWithoutDevLicense", &1u32) {
                        eprintln!(
                            "[Elevator] Failed to set AllowDevelopmentWithoutDevLicense: {}",
                            e
                        );
                        exit(1);
                    }
                    println!("[Elevator] Developer Mode symlink support enabled in registry.");
                    exit(0);
                }
                Err(e) => {
                    eprintln!(
                        "[Elevator] Failed to create or open AppModelUnlock registry key: {}",
                        e
                    );
                    exit(1);
                }
            }
        }
        #[cfg(not(windows))]
        {
            println!("[Elevator] enable-symlinks is only required on Windows.");
            exit(0);
        }
    }

    // Spawn and execute the target command
    let target = &args[0];
    let target_args = &args[1..];
    let mut cmd = Command::new(target);
    cmd.args(target_args);

    match cmd.status() {
        Ok(status) => exit(status.code().unwrap_or(0)),
        Err(e) => {
            eprintln!("[Elevator] Failed to execute target '{}': {}", target, e);
            exit(1);
        }
    }
}
