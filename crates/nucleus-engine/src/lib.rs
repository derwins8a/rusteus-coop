//! # nucleus-engine
//!
//! Core orchestration engine for NucleusCoop.
//!
//! Coordinates game discovery, session lifecycle management, player device
//! assignment, script evaluation, and execution across multiple instances.

pub mod game_manager;
pub mod scanner;
pub mod session;

pub use game_manager::GameManager;
pub use nucleus_script::{HandlerConfig, HandlerOption};
pub use scanner::{DiskScanner, FoundGame};
pub use session::{GameSession, SessionStatus};
