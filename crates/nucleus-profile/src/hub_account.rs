//! Non-secret Community Hub account profile metadata and preferences.

use std::path::Path;
use serde::{Deserialize, Serialize};
use nucleus_common::error::NucleusResult;
use nucleus_common::file_util;

/// Non-secret summary of the authenticated Splitscreen.Me Hub account.
///
/// NOTE: Passwords and raw session/resume tokens are NEVER stored in this struct.
/// Sensitive credentials remain secured inside `CredentialStore` (e.g. DPAPI).
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct HubAccountSummary {
    /// Remote user ID assigned by the Hub.
    pub user_id: String,
    /// Public username/display name of the user.
    pub username: String,
    /// Masked or hint email (optional).
    pub email_hint: Option<String>,
    /// Avatar image URL (optional).
    pub avatar_url: Option<String>,
    /// Token expiration timestamp in UTC milliseconds (optional).
    pub token_expires_unix_ms: Option<i64>,
    /// Hub base URL this session belongs to.
    pub hub_base_url: String,
}

impl HubAccountSummary {
    /// Creates a new `HubAccountSummary`.
    pub fn new(user_id: impl Into<String>, username: impl Into<String>, hub_base_url: impl Into<String>) -> Self {
        Self {
            user_id: user_id.into(),
            username: username.into(),
            email_hint: None,
            avatar_url: None,
            token_expires_unix_ms: None,
            hub_base_url: hub_base_url.into(),
        }
    }

    /// Loads the saved `HubAccountSummary` from the specified JSON file.
    pub fn load_from_file(path: &Path) -> NucleusResult<Self> {
        let content = file_util::read_text_file(path)?;
        let summary = serde_json::from_str(&content)?;
        Ok(summary)
    }

    /// Saves the `HubAccountSummary` to the specified JSON file.
    pub fn save_to_file(&self, path: &Path) -> NucleusResult<()> {
        let json = serde_json::to_string_pretty(self)?;
        file_util::write_text_file(path, &json)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_hub_account_summary_roundtrip() {
        let temp_dir = std::env::temp_dir().join(format!("nucleus_hub_acc_test_{}", std::process::id()));
        let _ = std::fs::create_dir_all(&temp_dir);
        let path = temp_dir.join("hub_account.json");

        let mut summary = HubAccountSummary::new("user_xyz123", "GamerPro", "https://hub.splitscreen.me");
        summary.email_hint = Some("g***@example.com".to_string());
        summary.token_expires_unix_ms = Some(1775000000000);

        summary.save_to_file(&path).unwrap();
        let loaded = HubAccountSummary::load_from_file(&path).unwrap();
        assert_eq!(summary, loaded);

        let _ = std::fs::remove_dir_all(&temp_dir);
    }
}
