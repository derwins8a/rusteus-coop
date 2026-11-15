//! Logging initialization and configuration.

use std::fs::OpenOptions;
use std::path::Path;
use std::sync::Mutex;
use tracing_subscriber::prelude::*;
use tracing_subscriber::{fmt, EnvFilter};

use crate::error::NucleusResult;

/// Initializes tracing subscriber with environment filter and formatted console output.
pub fn init_logging() {
    let filter = EnvFilter::try_from_default_env()
        .unwrap_or_else(|_| EnvFilter::new("info"));

    let _ = fmt()
        .with_env_filter(filter)
        .try_init();
}

/// Initializes tracing subscriber with formatted output to both standard output and a log file.
pub fn init_logging_with_file(log_path: &Path) -> NucleusResult<()> {
    if let Some(parent) = log_path.parent() {
        std::fs::create_dir_all(parent)?;
    }

    let file = OpenOptions::new()
        .create(true)
        .write(true)
        .truncate(true)
        .open(log_path)?;

    let filter = EnvFilter::try_from_default_env()
        .unwrap_or_else(|_| EnvFilter::new("info"));

    let stdout_layer = fmt::layer();
    let file_layer = fmt::layer()
        .with_writer(Mutex::new(file))
        .with_ansi(false);

    let _ = tracing_subscriber::registry()
        .with(filter)
        .with(stdout_layer)
        .with(file_layer)
        .try_init();

    Ok(())
}
