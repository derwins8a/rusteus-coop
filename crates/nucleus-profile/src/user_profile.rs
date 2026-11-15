//! User profile definitions and persistence.

use nucleus_common::error::NucleusResult;
use nucleus_common::file_util;
use serde::{Deserialize, Serialize};
use std::path::Path;

/// Represents an individual user profile within Nucleus Coop.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, Default)]
pub struct UserProfile {
    /// Unique profile identifier.
    pub id: String,
    /// Display name of the user profile.
    pub name: String,
    /// Optional file path or URL to user avatar icon.
    pub avatar_path: Option<String>,
    /// Preferred controller device GUID assigned to this user.
    pub default_gamepad_guid: Option<String>,
    /// Timestamp of profile creation in UTC seconds.
    pub created_at: u64,
}

impl UserProfile {
    /// Creates a new `UserProfile` with the specified name and identifier.
    pub fn new(id: impl Into<String>, name: impl Into<String>) -> Self {
        Self {
            id: id.into(),
            name: name.into(),
            ..Default::default()
        }
    }

    /// Loads a user profile from a JSON file.
    pub fn load_from_file(path: &Path) -> NucleusResult<Self> {
        let content = file_util::read_text_file(path)?;
        let profile = serde_json::from_str(&content)?;
        Ok(profile)
    }

    /// Saves the user profile to a JSON file.
    pub fn save_to_file(&self, path: &Path) -> NucleusResult<()> {
        let content = serde_json::to_string_pretty(self)?;
        file_util::write_text_file(path, &content)?;
        Ok(())
    }
}
