//! Community Hub integration client for Nucleus Coop.
//!
//! Provides asynchronous networking utilities to browse, search, query, and download
//! community-maintained game handlers and updates.

pub mod auth;
pub mod client;
pub mod ddp;
pub mod packager;
pub mod types;

pub use auth::{HubAuthCredentials, HubSessionToken, MeteorPasswordDigest};
pub use client::{HubClient, HubMetaSidecar};
pub use ddp::{DdpClient, DdpMethodMap};
pub use packager::{build_and_validate_nc_archive, PackageBuildInput};
pub use types::*;
