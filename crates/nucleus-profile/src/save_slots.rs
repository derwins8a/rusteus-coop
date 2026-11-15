//! Game save slots manifest, sync modes, and atomic persistence.

use nucleus_common::error::NucleusResult;
use serde::{Deserialize, Serialize};
use std::fs;
use std::path::{Path, PathBuf};

/// Mode for synchronizing save slots between host/instances and slot backup repositories.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Default)]
#[serde(rename_all = "snake_case")]
pub enum SaveSlotSyncMode {
    /// Full bidirectional sync: restores save on launch, backs up save on session exit.
    #[default]
    AutoBackupAndRestore,
    /// Restore save on launch for isolated play, but discard or do not overwrite slot on exit.
    RestoreOnlyGuest,
    /// Savegame management disabled for this game.
    Disabled,
}

/// Represents an individual savegame slot for a specific game.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct SaveSlot {
    /// Sanitized unique identifier slug (e.g. "p1_main", "coop_run").
    pub slot_id: String,
    /// User-visible display name.
    pub display_name: String,
    /// Game title recognized by Ludusavi.
    pub ludusavi_game_title: String,
    /// Total size in bytes of save files in this slot.
    #[serde(default)]
    pub total_bytes: u64,
    /// Number of tracked save files.
    #[serde(default)]
    pub file_count: usize,
    /// Unix timestamp when this slot was last backed up or restored.
    #[serde(default)]
    pub last_synced_unix: Option<u64>,
    /// Number of versioned backup snapshots retained.
    #[serde(default)]
    pub snapshot_count: usize,
    /// Provenance / creator hint (e.g. "Player 1", "Host PC Import", "Cloned from P1").
    #[serde(default)]
    pub source_hint: String,
}

impl SaveSlot {
    pub fn new(
        slot_id: impl Into<String>,
        display_name: impl Into<String>,
        game_title: impl Into<String>,
    ) -> Self {
        Self {
            slot_id: slot_id.into(),
            display_name: display_name.into(),
            ludusavi_game_title: game_title.into(),
            total_bytes: 0,
            file_count: 0,
            last_synced_unix: None,
            snapshot_count: 0,
            source_hint: "Player 1".to_string(),
        }
    }
}

/// Manifest cataloging all savegame slots and sync preferences for a specific game GUID.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, Default)]
pub struct GameSaveSlotsManifest {
    pub game_guid: String,
    #[serde(default)]
    pub ludusavi_title_override: Option<String>,
    #[serde(default)]
    pub sync_mode: SaveSlotSyncMode,
    #[serde(default)]
    pub slots: Vec<SaveSlot>,
}

impl GameSaveSlotsManifest {
    /// Sanitizes an arbitrary string into a clean filesystem and identifier slug.
    pub fn slugify(input: &str) -> String {
        let mut slug = String::new();
        let mut prev_dash = false;

        for c in input.chars() {
            if c.is_ascii_alphanumeric() {
                slug.push(c.to_ascii_lowercase());
                prev_dash = false;
            } else if (c == ' ' || c == '-' || c == '_') && !prev_dash && !slug.is_empty() {
                slug.push('-');
                prev_dash = true;
            }
        }

        let trimmed = slug.trim_end_matches('-');
        if trimmed.is_empty() {
            "default".to_string()
        } else {
            trimmed.to_string()
        }
    }

    /// Finds a slot by its unique slug ID.
    pub fn get_slot(&self, slot_id: &str) -> Option<&SaveSlot> {
        self.slots.iter().find(|s| s.slot_id == slot_id)
    }

    /// Finds a mutable slot by its unique slug ID.
    pub fn get_slot_mut(&mut self, slot_id: &str) -> Option<&mut SaveSlot> {
        self.slots.iter_mut().find(|s| s.slot_id == slot_id)
    }

    /// Adds a new slot to the manifest, generating a unique slug if collision occurs.
    pub fn create_slot(
        &mut self,
        display_name: &str,
        ludusavi_game_title: &str,
        source_hint: &str,
    ) -> &SaveSlot {
        let base_slug = Self::slugify(display_name);
        let mut unique_slug = base_slug.clone();
        let mut counter = 2;

        while self.slots.iter().any(|s| s.slot_id == unique_slug) {
            unique_slug = format!("{base_slug}-{counter}");
            counter += 1;
        }

        let new_slot = SaveSlot {
            slot_id: unique_slug,
            display_name: display_name.to_string(),
            ludusavi_game_title: ludusavi_game_title.to_string(),
            total_bytes: 0,
            file_count: 0,
            last_synced_unix: Some(
                std::time::SystemTime::now()
                    .duration_since(std::time::UNIX_EPOCH)
                    .unwrap_or_default()
                    .as_secs(),
            ),
            snapshot_count: 0,
            source_hint: source_hint.to_string(),
        };

        self.slots.push(new_slot);
        self.slots.last().unwrap()
    }

    /// Deletes a save slot by its ID.
    pub fn delete_slot(&mut self, slot_id: &str) -> bool {
        let prev_len = self.slots.len();
        self.slots.retain(|s| s.slot_id != slot_id);
        self.slots.len() < prev_len
    }

    /// Returns the manifest file path: `<saves_dir>/<game_guid>/manifest.json`.
    pub fn manifest_path(saves_dir: &Path, game_guid: &str) -> PathBuf {
        saves_dir.join(game_guid).join("manifest.json")
    }

    /// Loads the save slots manifest for a game, returning default if absent.
    pub fn load_from_dir(saves_dir: &Path, game_guid: &str) -> NucleusResult<Self> {
        let path = Self::manifest_path(saves_dir, game_guid);
        if !path.exists() {
            return Ok(Self {
                game_guid: game_guid.to_string(),
                ..Default::default()
            });
        }

        let content = fs::read_to_string(&path)?;
        let manifest = serde_json::from_str(&content)?;
        Ok(manifest)
    }

    /// Atomically saves the manifest to disk using a temporary file and rename.
    pub fn save_to_dir(&self, saves_dir: &Path) -> NucleusResult<()> {
        let final_path = Self::manifest_path(saves_dir, &self.game_guid);
        if let Some(parent) = final_path.parent() {
            fs::create_dir_all(parent)?;
        }

        let temp_path = final_path.with_extension("tmp");
        let content = serde_json::to_string_pretty(self)?;
        fs::write(&temp_path, content)?;

        if final_path.exists() {
            let _ = fs::remove_file(&final_path);
        }
        fs::rename(&temp_path, &final_path)?;

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_slugify() {
        assert_eq!(
            GameSaveSlotsManifest::slugify("Co-op Run #1 / P1"),
            "co-op-run-1-p1"
        );
        assert_eq!(
            GameSaveSlotsManifest::slugify("Left 4 Dead 2"),
            "left-4-dead-2"
        );
        assert_eq!(GameSaveSlotsManifest::slugify("!!!"), "default");
    }

    #[test]
    fn test_manifest_crud_and_atomic_save() {
        let temp_dir =
            std::env::temp_dir().join(format!("nucleus_saves_test_{}", std::process::id()));
        let guid = "test-game-guid-123";

        let mut manifest = GameSaveSlotsManifest {
            game_guid: guid.to_string(),
            sync_mode: SaveSlotSyncMode::AutoBackupAndRestore,
            ..Default::default()
        };

        manifest.create_slot("Main Campaign", "Game Title", "Host PC Import");
        manifest.create_slot("Main Campaign", "Game Title", "Cloned from P1"); // Duplicate should slugify to -2
        assert_eq!(manifest.slots.len(), 2);
        assert_eq!(manifest.slots[0].slot_id, "main-campaign");
        assert_eq!(manifest.slots[1].slot_id, "main-campaign-2");

        // Save atomically
        manifest.save_to_dir(&temp_dir).unwrap();

        // Reload
        let loaded = GameSaveSlotsManifest::load_from_dir(&temp_dir, guid).unwrap();
        assert_eq!(loaded.slots.len(), 2);
        assert_eq!(loaded.slots[0].display_name, "Main Campaign");

        // Delete
        manifest.delete_slot("main-campaign");
        assert_eq!(manifest.slots.len(), 1);

        let _ = fs::remove_dir_all(temp_dir);
    }
}
