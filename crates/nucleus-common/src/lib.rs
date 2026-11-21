//! # nucleus-common
//!
//! Foundational crate for NucleusCoop providing shared types, error handling,
//! logging initialization, global path definitions, and utility helpers.

pub mod dll_injection;
pub mod error;
pub mod file_util;
pub mod globals;
pub mod logging;
pub mod process;
pub mod types;

// Re-export key error types
pub use error::{NucleusError, NucleusResult};

// Re-export logging utilities
pub use logging::{init_logging, init_logging_with_file};

// Re-export global runtime context
pub use globals::Globals;

// Re-export process utilities
pub use process::{get_exe_architecture, is_64bit_executable};

// Re-export DLL injection configuration types
pub use dll_injection::{
    DllInjectionConfig, InjectionMethod, InjectionTiming, IpcChannelKind, TargetArchitecture,
    TargetInstanceFilter,
};

// Re-export shared types
pub use types::{
    DeviceType, Display, InputDevice, IntVector2, NetworkInterface, ProcessConfig, ProcessData,
    ProcessHandle, ProcessInfo, ProcessPriority, Rect, RegistryBackup, RegistryValueBackup,
    SearchMode, UserInfo, WindowHandle, WindowStyle,
};
