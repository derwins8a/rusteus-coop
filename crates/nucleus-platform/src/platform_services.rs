//! Unified platform services trait bundling all manager interfaces.

use crate::display::DisplayManager;
use crate::elevation::ElevationManager;
use crate::filesystem::FilesystemManager;
use crate::input_hook::InputHookManager;
use crate::mutex::MutexManager;
use crate::network::NetworkManager;
use crate::process::ProcessManager;
use crate::registry::RegistryManager;
use crate::taskbar::TaskbarManager;
use crate::user_account::UserAccountManager;
use crate::window::WindowManager;

/// Unified platform services trait bundling all manager interfaces.
pub trait PlatformServices: Send + Sync {
    /// Returns the window manager implementation.
    fn window_manager(&self) -> &dyn WindowManager;

    /// Returns the display manager implementation.
    fn display_manager(&self) -> &dyn DisplayManager;

    /// Returns the process manager implementation.
    fn process_manager(&self) -> &dyn ProcessManager;

    /// Returns the mutex manager implementation.
    fn mutex_manager(&self) -> &dyn MutexManager;

    /// Returns the registry manager implementation, or `None` on unsupported platforms (e.g. Linux).
    fn registry_manager(&self) -> Option<&dyn RegistryManager>;

    /// Returns the user account manager implementation, or `None` on unsupported platforms.
    fn user_account_manager(&self) -> Option<&dyn UserAccountManager>;

    /// Returns the network manager implementation.
    fn network_manager(&self) -> &dyn NetworkManager;

    /// Returns the filesystem manager implementation.
    fn filesystem_manager(&self) -> &dyn FilesystemManager;

    /// Returns the input hook manager implementation.
    fn input_hook_manager(&self) -> &dyn InputHookManager;

    /// Returns the taskbar manager implementation.
    fn taskbar_manager(&self) -> &dyn TaskbarManager;

    /// Returns the elevation manager implementation.
    fn elevation_manager(&self) -> &dyn ElevationManager;

    /// Returns the OSD manager implementation.
    fn osd_manager(&self) -> &dyn crate::osd::OsdManager;

    /// Returns the secure credential store implementation.
    fn credential_store(&self) -> &dyn crate::credentials::CredentialStore;
}
