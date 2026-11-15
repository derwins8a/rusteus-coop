//! # nucleus-platform-windows
//!
//! Windows-specific platform implementation for NucleusCoop using native Win32 APIs.

pub mod credentials;
#[cfg(windows)]
mod display_manager;
#[cfg(windows)]
mod elevation_manager;
#[cfg(windows)]
mod filesystem_manager;
#[cfg(windows)]
mod input_hook_manager;
#[cfg(windows)]
mod mutex_manager;
#[cfg(windows)]
mod network_manager;
#[cfg(windows)]
mod osd_manager;
#[cfg(windows)]
mod process_manager;
#[cfg(windows)]
mod registry_manager;
#[cfg(windows)]
mod taskbar_manager;
#[cfg(windows)]
mod user_account_manager;
#[cfg(windows)]
mod window_manager;

pub use credentials::WindowsDpapiCredentialStore;
#[cfg(windows)]
pub use display_manager::WindowsDisplayManager;
#[cfg(windows)]
pub use elevation_manager::WindowsElevationManager;
#[cfg(windows)]
pub use filesystem_manager::WindowsFilesystemManager;
#[cfg(windows)]
pub use input_hook_manager::WindowsInputHookManager;
#[cfg(windows)]
pub use mutex_manager::WindowsMutexManager;
#[cfg(windows)]
pub use network_manager::WindowsNetworkManager;
#[cfg(windows)]
pub use osd_manager::WindowsOsdManager;
#[cfg(windows)]
pub use process_manager::WindowsProcessManager;
#[cfg(windows)]
pub use registry_manager::WindowsRegistryManager;
#[cfg(windows)]
pub use taskbar_manager::WindowsTaskbarManager;
#[cfg(windows)]
pub use user_account_manager::WindowsUserAccountManager;
#[cfg(windows)]
pub use window_manager::WindowsWindowManager;

#[cfg(windows)]
use nucleus_platform::{
    DisplayManager, ElevationManager, FilesystemManager, InputHookManager, MutexManager,
    NetworkManager, OsdManager, PlatformServices, ProcessManager, RegistryManager, TaskbarManager,
    UserAccountManager, WindowManager,
};

/// Windows platform services provider implementing `PlatformServices`.
#[cfg(windows)]
pub struct WindowsPlatform {
    window_manager: WindowsWindowManager,
    display_manager: WindowsDisplayManager,
    process_manager: WindowsProcessManager,
    mutex_manager: WindowsMutexManager,
    registry_manager: WindowsRegistryManager,
    user_account_manager: WindowsUserAccountManager,
    network_manager: WindowsNetworkManager,
    filesystem_manager: WindowsFilesystemManager,
    input_hook_manager: WindowsInputHookManager,
    taskbar_manager: WindowsTaskbarManager,
    elevation_manager: WindowsElevationManager,
    osd_manager: WindowsOsdManager,
    credential_store: WindowsDpapiCredentialStore,
}

#[cfg(windows)]
impl WindowsPlatform {
    /// Creates a new `WindowsPlatform` services instance.
    pub fn new() -> Self {
        Self {
            window_manager: WindowsWindowManager::new(),
            display_manager: WindowsDisplayManager::new(),
            process_manager: WindowsProcessManager::new(),
            mutex_manager: WindowsMutexManager::new(),
            registry_manager: WindowsRegistryManager::new(),
            user_account_manager: WindowsUserAccountManager::new(),
            network_manager: WindowsNetworkManager::new(),
            filesystem_manager: WindowsFilesystemManager::new(),
            input_hook_manager: WindowsInputHookManager::new(),
            taskbar_manager: WindowsTaskbarManager::new(),
            elevation_manager: WindowsElevationManager::new(),
            osd_manager: WindowsOsdManager::new(),
            credential_store: WindowsDpapiCredentialStore::new(),
        }
    }
}

#[cfg(windows)]
impl Default for WindowsPlatform {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(windows)]
impl PlatformServices for WindowsPlatform {
    fn window_manager(&self) -> &dyn WindowManager {
        &self.window_manager
    }

    fn display_manager(&self) -> &dyn DisplayManager {
        &self.display_manager
    }

    fn process_manager(&self) -> &dyn ProcessManager {
        &self.process_manager
    }

    fn mutex_manager(&self) -> &dyn MutexManager {
        &self.mutex_manager
    }

    fn registry_manager(&self) -> Option<&dyn RegistryManager> {
        Some(&self.registry_manager)
    }

    fn user_account_manager(&self) -> Option<&dyn UserAccountManager> {
        Some(&self.user_account_manager)
    }

    fn network_manager(&self) -> &dyn NetworkManager {
        &self.network_manager
    }

    fn filesystem_manager(&self) -> &dyn FilesystemManager {
        &self.filesystem_manager
    }

    fn input_hook_manager(&self) -> &dyn InputHookManager {
        &self.input_hook_manager
    }

    fn taskbar_manager(&self) -> &dyn TaskbarManager {
        &self.taskbar_manager
    }

    fn elevation_manager(&self) -> &dyn ElevationManager {
        &self.elevation_manager
    }

    fn osd_manager(&self) -> &dyn OsdManager {
        &self.osd_manager
    }

    fn credential_store(&self) -> &dyn nucleus_platform::CredentialStore {
        &self.credential_store
    }
}

/// Fallback stub for non-Windows targets.
#[cfg(not(windows))]
pub struct WindowsPlatform;

#[cfg(not(windows))]
impl WindowsPlatform {
    /// Stub constructor for non-Windows platforms.
    pub fn new() -> Self {
        Self
    }
}

#[cfg(all(test, windows))]
mod tests {
    use super::*;

    #[test]
    fn test_windows_platform_creation() {
        let platform = WindowsPlatform::new();
        let displays = platform.display_manager().get_displays().unwrap();
        assert!(
            !displays.is_empty(),
            "At least one display must be detected"
        );

        let primary = platform.display_manager().get_primary_display().unwrap();
        assert!(primary.is_primary);
    }

    #[test]
    fn test_windows_filesystem_manager_hardlink() {
        let platform = WindowsPlatform::new();
        let fs_mgr = platform.filesystem_manager();

        let temp_dir = std::env::temp_dir().join(format!("nucleus_fs_test_{}", std::process::id()));
        let _ = std::fs::create_dir_all(&temp_dir);

        let src_file = temp_dir.join("source.txt");
        let dst_file = temp_dir.join("link.txt");

        std::fs::write(&src_file, "HardLinkContent").unwrap();
        fs_mgr.create_hardlink(&src_file, &dst_file).unwrap();

        assert!(dst_file.exists());
        let read_back = std::fs::read_to_string(&dst_file).unwrap();
        assert_eq!(read_back, "HardLinkContent");

        let _ = std::fs::remove_dir_all(&temp_dir);
    }

    #[test]
    fn test_windows_registry_key_exists() {
        let platform = WindowsPlatform::new();
        let reg_mgr = platform.registry_manager().unwrap();
        // HKCU\Software always exists on Windows
        assert!(reg_mgr.key_exists("HKEY_CURRENT_USER\\Software"));
    }
}
