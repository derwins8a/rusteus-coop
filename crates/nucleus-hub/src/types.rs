//! API data types and response models for the Nucleus Coop Community Hub.
//!
//! Maps SplitScreen.Me Hub REST endpoints (https://hub.splitscreen.me/) and CDN packages.

use serde::{Deserialize, Serialize};

/// Helper to extract an integer from a JSON Value that might be a number or string.
pub fn value_to_i32(val: &Option<serde_json::Value>) -> Option<i32> {
    match val {
        Some(serde_json::Value::Number(n)) => n.as_i64().map(|v| v as i32),
        Some(serde_json::Value::String(s)) => s.parse::<i32>().ok(),
        _ => None,
    }
}

/// Helper to extract a u64 from a JSON Value that might be a number or string.
pub fn value_to_u64(val: &Option<serde_json::Value>) -> Option<u64> {
    match val {
        Some(serde_json::Value::Number(n)) => n.as_u64(),
        Some(serde_json::Value::String(s)) => s.parse::<u64>().ok(),
        _ => None,
    }
}

/// Top-level response container for endpoints returning `{"Handlers": [...]}`.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, Default)]
pub struct HubHandlersResponse {
    #[serde(rename = "Handlers", default)]
    pub handlers: Vec<HubHandler>,
}

/// Full game handler record directly returned from SplitScreen.Me Community Hub API.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, Default)]
#[serde(default)]
pub struct HubHandler {
    /// Unique handler identifier on the hub.
    #[serde(rename = "_id")]
    pub id: String,
    /// Owner/uploader user ID.
    pub owner: Option<String>,
    /// Author / maintainer username.
    #[serde(rename = "ownerName")]
    pub owner_name: Option<String>,
    /// Handler description and launch instructions.
    pub description: Option<String>,
    /// Specific handler title.
    pub title: Option<String>,
    /// Target game display name.
    #[serde(rename = "gameName")]
    pub game_name: Option<String>,
    /// Game summary or synopsis from IGDB.
    #[serde(rename = "gameDescription")]
    pub game_description: Option<String>,
    /// IGDB cover image hash (e.g. "co1x7i").
    #[serde(rename = "gameCover")]
    pub game_cover: Option<String>,
    /// IGDB game identifier.
    #[serde(rename = "gameId")]
    pub game_id: Option<serde_json::Value>,
    /// IGDB game website URL.
    #[serde(rename = "gameUrl")]
    pub game_url: Option<String>,
    /// Creation date timestamp.
    #[serde(rename = "createdAt")]
    pub created_at: Option<String>,
    /// Last updated timestamp.
    #[serde(rename = "updatedAt")]
    pub updated_at: Option<String>,
    /// Community star rating score.
    pub stars: Option<serde_json::Value>,
    /// Total download count.
    #[serde(rename = "downloadCount")]
    pub download_count: Option<serde_json::Value>,
    /// Whether handler is officially verified.
    pub verified: Option<bool>,
    /// Whether handler is private.
    pub private: Option<bool>,
    /// Total number of comments.
    #[serde(rename = "commentCount")]
    pub comment_count: Option<serde_json::Value>,
    /// Current package version number (e.g. 2).
    #[serde(rename = "currentVersion")]
    pub current_version: Option<serde_json::Value>,
    /// ID of the latest package package archive for download.
    #[serde(rename = "currentPackage")]
    pub current_package: Option<String>,
    /// Maximum supported players.
    #[serde(rename = "maxPlayers")]
    pub max_players: Option<serde_json::Value>,
    /// Controller support flag.
    #[serde(rename = "playableControllers")]
    pub playable_controllers: Option<bool>,
    /// Mouse & keyboard support flag.
    #[serde(rename = "playableMouseKeyboard")]
    pub playable_mouse_keyboard: Option<bool>,
    /// Multiple mice and keyboards support flag.
    #[serde(rename = "playableMultiMouseKeyboard")]
    pub playable_multi_mouse_keyboard: Option<bool>,
    /// Game genres.
    pub genres: Option<Vec<String>>,
}

impl HubHandler {
    /// Returns the primary display title (falling back to game_name or ID).
    pub fn display_title(&self) -> &str {
        if let Some(ref t) = self.title {
            if !t.trim().is_empty() {
                return t.trim();
            }
        }
        if let Some(ref g) = self.game_name {
            if !g.trim().is_empty() {
                return g.trim();
            }
        }
        &self.id
    }

    /// Returns the author/maintainer username.
    pub fn author(&self) -> &str {
        self.owner_name.as_deref().unwrap_or("Community")
    }

    /// Returns the current version number, defaulting to 1.
    pub fn version_number(&self) -> i32 {
        value_to_i32(&self.current_version).unwrap_or(1)
    }

    /// Returns the maximum player count, defaulting to 4.
    pub fn max_players_count(&self) -> i32 {
        value_to_i32(&self.max_players).unwrap_or(4)
    }

    /// Returns the download count.
    pub fn downloads(&self) -> u64 {
        value_to_u64(&self.download_count).unwrap_or(0)
    }

    /// Returns the star rating score.
    pub fn rating(&self) -> u32 {
        value_to_i32(&self.stars)
            .map(|s| s.max(0) as u32)
            .unwrap_or(0)
    }

    /// Returns the total comments count.
    pub fn comments(&self) -> u32 {
        value_to_i32(&self.comment_count)
            .map(|c| c.max(0) as u32)
            .unwrap_or(0)
    }

    /// Returns the primary cover image URL.
    pub fn cover_url(&self) -> Option<String> {
        self.cover_image_url("cover_big")
    }

    /// Extracts categorized community tags (device support, capacity, genres).
    pub fn extracted_tags(&self) -> Vec<String> {
        let mut tags = Vec::new();
        if self.playable_controllers.unwrap_or(false) {
            tags.push("Controller support".to_string());
        }
        if self.playable_mouse_keyboard.unwrap_or(false) {
            tags.push("Keyboard/mouse support".to_string());
        }
        if self.playable_multi_mouse_keyboard.unwrap_or(false) {
            tags.push("Multiple keyboards/mice support".to_string());
        }
        let mp = self.max_players_count();
        if mp >= 16 {
            tags.push("16+ players".to_string());
        }
        if mp >= 8 {
            tags.push("8+ players".to_string());
        }
        if mp >= 4 {
            tags.push("4+ players".to_string());
        }
        if mp >= 3 {
            tags.push("3+ players".to_string());
        }

        let standard_genres = [
            "Adventure",
            "Real Time Strategy",
            "Role-playing (RPG)",
            "Shooter",
            "Simulator",
            "Sport",
            "Strategy",
            "Tactical",
            "Turn-based strategy",
            "Arcade",
            "Fighting",
            "Hack and slash",
            "Indie",
            "MOBA",
            "Music",
            "Platform",
        ];

        if let Some(ref g_list) = self.genres {
            for g in g_list {
                for std_g in &standard_genres {
                    if g.eq_ignore_ascii_case(std_g)
                        || g.to_lowercase().contains(&std_g.to_lowercase())
                    {
                        if !tags.contains(&std_g.to_string()) {
                            tags.push(std_g.to_string());
                        }
                    }
                }
            }
        }

        let combined = format!(
            "{} {} {}",
            self.title.as_deref().unwrap_or(""),
            self.game_name.as_deref().unwrap_or(""),
            self.description.as_deref().unwrap_or("")
        )
        .to_lowercase();

        for std_g in &standard_genres {
            let key = if std_g.contains("RPG") {
                "rpg"
            } else {
                &std_g.to_lowercase()
            };
            if combined.contains(key) && !tags.contains(&std_g.to_string()) {
                tags.push(std_g.to_string());
            }
        }

        tags
    }

    /// Constructs the direct CDN package download URL for this handler's current release.
    pub fn cdn_download_url(&self) -> Option<String> {
        let pkg_id = self.current_package.as_ref()?;
        let ver = self.version_number();
        Some(format!(
            "https://hub.splitscreen.me/cdn/storage/packages/{}/original/handler-{}-v{}.nc?download=true",
            pkg_id, self.id, ver
        ))
    }

    /// Constructs the IGDB cover image URL with the specified resolution (e.g. "cover_big", "thumb", "720p").
    pub fn cover_image_url(&self, size: &str) -> Option<String> {
        let cover = self.game_cover.as_ref()?;
        Some(format!(
            "https://images.igdb.com/igdb/image/upload/t_{}/{}.jpg",
            size, cover
        ))
    }

    /// Converts this handler to a `HubHandlerSummary` for backward compatibility.
    pub fn to_summary(&self) -> HubHandlerSummary {
        HubHandlerSummary {
            id: self.id.clone(),
            title: self.display_title().to_string(),
            author: self.author().to_string(),
            version: format!("v{}", self.version_number()),
            description: self.description.clone().unwrap_or_default(),
            download_count: self.downloads(),
            rating: self.rating(),
            updated_at: self.updated_at.clone().unwrap_or_default(),
            current_package: self.current_package.clone(),
            max_players: self.max_players_count(),
            game_name: self.game_name.clone().unwrap_or_default(),
            game_cover: self.game_cover.clone(),
        }
    }
}

/// Response container for `/api/v1/packages/:handler_id`.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, Default)]
pub struct HubPackagesResponse {
    #[serde(rename = "packages", alias = "Packages", default)]
    pub packages: Vec<HubPackage>,
}

/// Response container for `/api/v1/package/:package_id`.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, Default)]
pub struct HubPackageResponse {
    #[serde(rename = "package", alias = "Package", default)]
    pub package: Option<HubPackage>,
}

/// Package record in `/api/v1/packages/:handler_id`.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, Default)]
#[serde(default)]
pub struct HubPackage {
    #[serde(rename = "_id")]
    pub id: String,
    pub size: Option<u64>,
    #[serde(rename = "type")]
    pub mime_type: Option<String>,
    pub name: Option<String>,
    pub meta: Option<HubPackageMeta>,
}

/// Metadata embedded in a package release.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, Default)]
#[serde(default)]
pub struct HubPackageMeta {
    #[serde(rename = "releaseDescription")]
    pub release_description: Option<String>,
    #[serde(rename = "handlerId")]
    pub handler_id: Option<String>,
    #[serde(rename = "handlerVersion")]
    pub handler_version: Option<serde_json::Value>,
    #[serde(rename = "jsContent")]
    pub js_content: Option<String>,
    #[serde(rename = "ownerId")]
    pub owner_id: Option<String>,
    #[serde(rename = "releaseDate")]
    pub release_date: Option<String>,
    pub verified: Option<bool>,
    pub downloads: Option<serde_json::Value>,
}

/// Response container for `/api/v1/screenshots/:handler_id`.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, Default)]
pub struct HubScreenshotsResponse {
    #[serde(default)]
    pub screenshots: Vec<HubScreenshot>,
}

/// Individual IGDB screenshot descriptor.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, Default)]
#[serde(default)]
pub struct HubScreenshot {
    pub id: u64,
    pub game: Option<u64>,
    pub url: String,
    #[serde(rename = "image_id")]
    pub image_id: String,
    pub width: Option<u32>,
    pub height: Option<u32>,
}

/// Summary of a game handler available in the community hub (compatible format).
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, Default)]
pub struct HubHandlerSummary {
    /// Unique handler ID.
    pub id: String,
    /// Game title or handler title.
    pub title: String,
    /// Author username.
    pub author: String,
    /// Current handler version string.
    pub version: String,
    /// Short description or summary.
    pub description: String,
    /// Number of downloads recorded.
    pub download_count: u64,
    /// Star rating or score.
    pub rating: u32,
    /// Date of last update.
    pub updated_at: String,
    /// Package ID if available.
    pub current_package: Option<String>,
    /// Maximum players supported.
    pub max_players: i32,
    /// Game name.
    pub game_name: String,
    /// IGDB game cover hash.
    pub game_cover: Option<String>,
}

/// Detailed information and metadata for a specific handler package (compatible format).
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, Default)]
pub struct HubHandlerDetail {
    /// Unique handler ID.
    pub id: String,
    /// Associated game information.
    pub game: HubGameInfo,
    /// Author username.
    pub author: String,
    /// Current handler version string.
    pub current_version: String,
    /// Extended markdown or text description.
    pub description: String,
    /// Full version history.
    pub versions: Vec<HubVersionInfo>,
    /// User comments and feedback.
    pub comments: Vec<HubComment>,
    /// Download URL for the latest handler archive.
    pub download_url: String,
    /// Package ID.
    pub current_package: Option<String>,
}

/// Game metadata associated with a handler on the hub.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, Default)]
pub struct HubGameInfo {
    /// Game title.
    pub name: String,
    /// Steam App ID if applicable.
    pub steam_id: Option<String>,
    /// Developer or publisher name.
    pub publisher: Option<String>,
    /// URL to banner or header image.
    pub banner_url: Option<String>,
    /// URL to icon image.
    pub icon_url: Option<String>,
}

/// Version entry in handler release history.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, Default)]
pub struct HubVersionInfo {
    /// Semantic version identifier.
    pub version: String,
    /// Changelog notes for this release.
    pub changelog: String,
    /// Release date in ISO 8601.
    pub release_date: String,
    /// Direct download URL for this specific version.
    pub download_url: String,
}

/// Response container for `/api/v1/comments/:handler_id`.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, Default)]
pub struct HubCommentsResponse {
    #[serde(rename = "Comments", alias = "comments", default)]
    pub comments: Vec<HubComment>,
}

/// Community user comment on a handler.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, Default)]
pub struct HubComment {
    /// Unique comment ID.
    #[serde(rename = "_id", alias = "id", default)]
    pub id: String,
    /// Author username.
    #[serde(rename = "ownerName", alias = "author", default)]
    pub author: String,
    /// Owner ID.
    #[serde(rename = "owner", default)]
    pub owner: Option<String>,
    /// Reply target comment ID if this is a reply.
    #[serde(rename = "replyTo", default)]
    pub reply_to: Option<String>,
    /// Comment text body.
    #[serde(default)]
    pub content: String,
    /// Date comment was posted.
    #[serde(rename = "createdAt", alias = "created_at", default)]
    pub created_at: String,
}

/// Reason category for reporting a handler for moderation.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ReportReason {
    BrokenHandler,
    MaliciousOrUnsafe,
    OutdatedOrWrongGame,
    SpamOrInappropriate,
    Other,
}

impl std::fmt::Display for ReportReason {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::BrokenHandler => write!(f, "Broken / crashes game"),
            Self::MaliciousOrUnsafe => write!(f, "Malicious or unsafe executable/script"),
            Self::OutdatedOrWrongGame => write!(f, "Outdated or incorrect game"),
            Self::SpamOrInappropriate => write!(f, "Spam or inappropriate content"),
            Self::Other => write!(f, "Other issue"),
        }
    }
}

/// Request payload to post a new comment or reply to an existing comment.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct CommentSubmission {
    pub handler_id: String,
    pub content: String,
    pub reply_to: Option<String>,
}

/// Request payload to package and upload a new handler or release.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UploadHandlerRequest {
    /// Handler ID if updating an existing handler, or None if publishing a new handler.
    pub handler_id: Option<String>,
    pub game_name: String,
    pub title: String,
    pub description: String,
    pub archive_bytes: Vec<u8>,
    pub release_notes: String,
}

/// Result returned after uploading a handler.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, Default)]
pub struct UploadedHandlerResult {
    pub handler_id: String,
    pub package_id: String,
    pub version: i32,
    pub message: String,
}

/// Result returned after toggling or giving hotness to a handler.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, Default)]
pub struct HotnessVoteResult {
    pub handler_id: String,
    pub total_hotness: i32,
    pub voted: bool,
}
