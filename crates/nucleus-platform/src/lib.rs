//! # nucleus-platform
//!
//! Platform abstraction layer for NucleusCoop.
//!
//! This crate defines pure trait interfaces representing all platform-specific operations
//! (window management, display enumeration, process control, mutex manipulation, registry access,
//! user account management, networking, filesystem links, input hooking, and taskbar control).
//!
//! Concrete implementations are provided by `nucleus-platform-windows` and `nucleus-platform-linux`.

pub mod credentials;
pub mod display;
pub mod dll_manager;
pub mod elevation;
pub mod filesystem;
pub mod input_hook;
pub mod mutex;
pub mod network;
pub mod osd;
pub mod platform_services;
pub mod process;
pub mod registry;
pub mod taskbar;
pub mod user_account;
pub mod window;

// Re-export all traits at crate root
pub use credentials::{CredentialStore, InMemoryCredentialStore};
pub use display::DisplayManager;
pub use dll_manager::{DllInjectionManager, ProcessMemoryManager};
pub use elevation::ElevationManager;
pub use filesystem::FilesystemManager;
pub use input_hook::InputHookManager;
pub use mutex::MutexManager;
pub use network::NetworkManager;
pub use osd::OsdManager;
pub use platform_services::PlatformServices;
pub use process::ProcessManager;
pub use registry::RegistryManager;
pub use taskbar::TaskbarManager;
pub use user_account::UserAccountManager;
pub use window::WindowManager;
