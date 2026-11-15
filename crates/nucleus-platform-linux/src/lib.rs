//! # nucleus-platform-linux
//!
//! Linux platform implementation for NucleusCoop.
//!
//! Provides stub implementations returning `Err(NucleusError::Unsupported(...))`
//! for operations not yet implemented on Linux.

#[cfg(target_os = "linux")]
mod display_manager;
#[cfg(target_os = "linux")]
mod elevation_manager;
#[cfg(target_os = "linux")]
mod filesystem_manager;
#[cfg(target_os = "linux")]
mod input_hook_manager;
#[cfg(target_os = "linux")]
mod mutex_manager;
#[cfg(target_os = "linux")]
mod network_manager;
#[cfg(target_os = "linux")]
mod osd_manager;
#[cfg(target_os = "linux")]
mod process_manager;
#[cfg(target_os = "linux")]
mod taskbar_manager;
#[cfg(target_os = "linux")]
mod window_manager;

#[cfg(target_os = "linux")]
pub use display_manager::LinuxDisplayManager;
#[cfg(target_os = "linux")]
pub use elevation_manager::LinuxElevationManager;
#[cfg(target_os = "linux")]
pub use filesystem_manager::LinuxFilesystemManager;
#[cfg(target_os = "linux")]
pub use input_hook_manager::LinuxInputHookManager;
#[cfg(target_os = "linux")]
pub use mutex_manager::LinuxMutexManager;
#[cfg(target_os = "linux")]
pub use network_manager::LinuxNetworkManager;
#[cfg(target_os = "linux")]
pub use osd_manager::LinuxOsdManager;
#[cfg(target_os = "linux")]
pub use process_manager::LinuxProcessManager;
#[cfg(target_os = "linux")]
pub use taskbar_manager::LinuxTaskbarManager;
#[cfg(target_os = "linux")]
pub use window_manager::LinuxWindowManager;

#[cfg(target_os = "linux")]
use nucleus_platform::{
    DisplayManager, ElevationManager, FilesystemManager, InputHookManager, MutexManager, NetworkManager,
    OsdManager, PlatformServices, ProcessManager, RegistryManager, TaskbarManager, UserAccountManager,
    WindowManager,
};

/// Linux platform services provider.
#[cfg(target_os = "linux")]
pub struct LinuxPlatform {
    window_manager: LinuxWindowManager,
    display_manager: LinuxDisplayManager,
    process_manager: LinuxProcessManager,
    mutex_manager: LinuxMutexManager,
    network_manager: LinuxNetworkManager,
    filesystem_manager: LinuxFilesystemManager,
    input_hook_manager: LinuxInputHookManager,
    taskbar_manager: LinuxTaskbarManager,
    elevation_manager: LinuxElevationManager,
    osd_manager: LinuxOsdManager,
    credential_store: nucleus_platform::InMemoryCredentialStore,
}

#[cfg(target_os = "linux")]
impl LinuxPlatform {
    /// Creates a new `LinuxPlatform` services instance.
    pub fn new() -> Self {
        Self {
            window_manager: LinuxWindowManager::new(),
            display_manager: LinuxDisplayManager::new(),
            process_manager: LinuxProcessManager::new(),
            mutex_manager: LinuxMutexManager::new(),
            network_manager: LinuxNetworkManager::new(),
            filesystem_manager: LinuxFilesystemManager::new(),
            input_hook_manager: LinuxInputHookManager::new(),
            taskbar_manager: LinuxTaskbarManager::new(),
            elevation_manager: LinuxElevationManager::new(),
            osd_manager: LinuxOsdManager::new(),
            credential_store: nucleus_platform::InMemoryCredentialStore::new(),
        }
    }
}

#[cfg(target_os = "linux")]
impl Default for LinuxPlatform {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(target_os = "linux")]
impl PlatformServices for LinuxPlatform {
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
        None
    }

    fn user_account_manager(&self) -> Option<&dyn UserAccountManager> {
        None
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

/// Fallback stub for non-Linux targets.
#[cfg(not(target_os = "linux"))]
pub struct LinuxPlatform;

#[cfg(not(target_os = "linux"))]
impl LinuxPlatform {
    /// Stub constructor for non-Linux platforms.
    pub fn new() -> Self {
        Self
    }
}
