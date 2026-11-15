//! Metadata and descriptive information for game handlers.

use serde::{Deserialize, Serialize};

/// Metadata identifying a game handler package or script.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, Default)]
pub struct GameMetaInfo {
    /// Unique handler identifier string.
    pub id: String,
    /// Human-readable game title.
    pub game_name: String,
    /// Script author or contributors.
    pub author: String,
    /// Semantic version of the handler script.
    pub version: String,
    /// Handler description and notes.
    pub description: String,
    /// Maximum supported player count.
    pub max_players: i32,
    /// Monotonically increasing handler revision number.
    pub handler_version: i32,
    /// URL or relative path to game box art / icon.
    pub icon_url: Option<String>,
}

impl GameMetaInfo {
    /// Creates a new `GameMetaInfo` with default values.
    pub fn new(id: impl Into<String>, game_name: impl Into<String>) -> Self {
        Self {
            id: id.into(),
            game_name: game_name.into(),
            ..Default::default()
        }
    }
}
