//! Error types and result definitions for NucleusCoop.

use thiserror::Error;

/// Core error types for NucleusCoop operations.
#[derive(Error, Debug)]
pub enum NucleusError {
    /// IO error during file or stream operations.
    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),

    /// Serialization or deserialization error with JSON.
    #[error("JSON error: {0}")]
    Json(#[from] serde_json::Error),

    /// Script execution or parsing error.
    #[error("Script error: {0}")]
    Script(String),

    /// Platform-specific operation error.
    #[error("Platform error: {0}")]
    Platform(String),

    /// Handler execution or setup error.
    #[error("Handler error: {0}")]
    Handler(String),

    /// Operation or feature unsupported on the current platform.
    #[error("Feature not supported on this platform: {0}")]
    Unsupported(String),

    /// Process creation, manipulation, or termination error.
    #[error("Process error: {0}")]
    Process(String),

    /// Network or socket operation error.
    #[error("Network error: {0}")]
    Network(String),

    /// Configuration parsing or validation error.
    #[error("Configuration error: {0}")]
    Config(String),

    /// Authentication or authorization error.
    #[error("Authentication required or expired: {0}")]
    Auth(String),

    /// Extension, plugin, or add-on error.
    #[error("Plugin error: {0}")]
    Plugin(String),

    /// DLL injection or remote thread execution error.
    #[error("DLL injection error: {0}")]
    DllInjection(String),

    /// Portable Executable (PE) inspection or architecture verification error.
    #[error("PE inspection error: {0}")]
    PeInspection(String),

    /// Inter-process communication (IPC) channel error.
    #[error("IPC error: {0}")]
    Ipc(String),

    /// Generic error with an explanatory message.
    #[error("{0}")]
    Other(String),
}

/// A specialized Result type for NucleusCoop operations.
pub type NucleusResult<T> = Result<T, NucleusError>;
