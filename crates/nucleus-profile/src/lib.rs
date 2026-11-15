//! Profile and settings management for Nucleus Coop.
//!
//! Handles serialization, storage, and retrieval of user profiles, game configurations,
//! player slot mappings, and global application settings.

pub mod game_profile;
pub mod hub_account;
pub mod meta_info;
pub mod player;
pub mod save_slots;
pub mod settings;
pub mod user_profile;

pub use game_profile::GameProfile;
pub use hub_account::HubAccountSummary;
pub use meta_info::{now_formatted, GameMetaInfoEntry, MetaInfoStore};
pub use player::PlayerInfo;
pub use save_slots::{GameSaveSlotsManifest, SaveSlot, SaveSlotSyncMode};
pub use settings::AppSettings;
pub use user_profile::UserProfile;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_user_profile_roundtrip() {
        let temp_dir = std::env::temp_dir().join(format!("nucleus_test_prof_{}", std::process::id()));
        std::fs::create_dir_all(&temp_dir).unwrap();
        let path = temp_dir.join("profile.json");

        let mut prof = UserProfile::new("user_123", "GamerOne");
        prof.default_gamepad_guid = Some("guid_abc".to_string());
        prof.save_to_file(&path).unwrap();

        let loaded = UserProfile::load_from_file(&path).unwrap();
        assert_eq!(prof, loaded);

        let _ = std::fs::remove_dir_all(&temp_dir);
    }

    #[test]
    fn test_game_profile_roundtrip() {
        let temp_dir = std::env::temp_dir().join(format!("nucleus_test_game_prof_{}", std::process::id()));
        std::fs::create_dir_all(&temp_dir).unwrap();
        let path = temp_dir.join("game.json");

        let mut game_prof = GameProfile::new("game_1", "guid_xyz", "C:\\Games\\game.exe");
        game_prof.custom_arguments = Some("-novid".to_string());
        game_prof.player_count = 4;
        game_prof.layout = "Grid4".to_string();
        game_prof.save_to_file(&path).unwrap();

        let loaded = GameProfile::load_from_file(&path).unwrap();
        assert_eq!(game_prof, loaded);
        assert_eq!(loaded.player_count, 4);
        assert_eq!(loaded.layout, "Grid4");

        let _ = std::fs::remove_dir_all(&temp_dir);
    }

    #[test]
    fn test_app_settings_roundtrip() {
        let temp_dir = std::env::temp_dir().join(format!("nucleus_test_settings_{}", std::process::id()));
        std::fs::create_dir_all(&temp_dir).unwrap();
        let path = temp_dir.join("settings.json");

        let mut settings = AppSettings::default();
        settings.theme = "Cyberpunk".to_string();
        settings.close_on_launch = true;
        settings.save_to_file(&path).unwrap();

        let loaded = AppSettings::load_from_file(&path).unwrap();
        assert_eq!(settings, loaded);

        let _ = std::fs::remove_dir_all(&temp_dir);
    }

    #[test]
    fn test_app_settings_defaults() {
        let settings = AppSettings::default();
        assert_eq!(settings.handlers_directory, "handlers");
        assert_eq!(settings.theme, "dark");
        assert_eq!(settings.window_width, 1200);
        assert_eq!(settings.window_height, 800);
        assert!(!settings.auto_scan_on_start);
        assert_eq!(settings.nucleus_data_dir, "");
    }

    #[test]
    fn test_app_settings_save_and_load_persistence() {
        let temp_dir = std::env::temp_dir().join(format!("nucleus_appdata_test_{}", std::process::id()));
        let _ = std::fs::remove_dir_all(&temp_dir);
        let old_appdata = std::env::var("APPDATA").ok();

        unsafe {
            std::env::set_var("APPDATA", &temp_dir);
        }

        let mut settings = AppSettings::default();
        settings.theme = "Solarized".to_string();
        settings.window_width = 1600;
        settings.window_height = 900;
        settings.handlers_directory = "custom_handlers".to_string();
        settings.save().unwrap();

        let loaded = AppSettings::load();
        assert_eq!(settings, loaded);

        unsafe {
            if let Some(old) = old_appdata {
                std::env::set_var("APPDATA", old);
            } else {
                std::env::remove_var("APPDATA");
            }
        }

        let _ = std::fs::remove_dir_all(&temp_dir);
    }

    #[test]
    fn test_app_settings_partial_json() {
        // Missing new fields should deserialize using default values
        let partial_json = r#"{"theme": "nord", "close_on_launch": true}"#;
        let settings: AppSettings = serde_json::from_str(partial_json).unwrap();
        assert_eq!(settings.theme, "nord");
        assert!(settings.close_on_launch);
        assert_eq!(settings.handlers_directory, "handlers");
        assert_eq!(settings.window_width, 1200);
        assert_eq!(settings.window_height, 800);
        // Verify default C# parity settings are populated
        assert_eq!(settings.steam_lang, "Automatic");
        assert_eq!(settings.epic_lang, "English");
        assert_eq!(settings.network, "Automatic");
        assert_eq!(settings.lock_inputs_key, "End");
        assert_eq!(settings.hotkey_close, "Ctrl+Q");
        assert_eq!(settings.hotkey_stop, "Ctrl+E");
        assert_eq!(settings.hotkey_reset_windows, "Ctrl+R");
        assert_eq!(settings.hotkey_couch_osd, "F10");
        assert_eq!(settings.osd_color, "255,255,255");
        assert!(settings.osd_enabled);
        assert_eq!(settings.player_nicknames.len(), 32);
        assert_eq!(settings.audio_instances.len(), 8);
    }

    #[test]
    fn test_app_settings_player_and_audio_helpers() {
        let mut settings = AppSettings::default();
        assert_eq!(settings.get_player_nickname(0), "Player1");
        assert_eq!(settings.get_player_nickname(3), "Player4");
        assert_eq!(settings.get_player_steam_id(0), "76561198000000001");

        settings.set_player_nickname(0, "Alice");
        assert_eq!(settings.get_player_nickname(0), "Alice");

        settings.set_player_steam_id(0, "76561198123456789");
        assert_eq!(settings.get_player_steam_id(0), "76561198123456789");

        assert_eq!(settings.get_audio_instance(0), "Default");
        settings.set_audio_instance(0, "Headphones (Realtek)");
        assert_eq!(settings.get_audio_instance(0), "Headphones (Realtek)");
    }

    #[test]
    fn test_app_settings_hotkey_parsing() {
        assert_eq!(AppSettings::parse_hotkey("Ctrl+Q"), Some((1, 'Q' as u32)));
        assert_eq!(AppSettings::parse_hotkey("Ctrl+E"), Some((1, 'E' as u32)));
        assert_eq!(AppSettings::parse_hotkey("Ctrl+N"), Some((1, 'N' as u32)));
        assert_eq!(AppSettings::parse_hotkey("Ctrl+Tab"), Some((1, 0x09)));
        assert_eq!(AppSettings::parse_hotkey("Shift+F1"), Some((4, 0x70)));
        assert_eq!(AppSettings::parse_hotkey("Alt+R"), Some((2, 'R' as u32)));
        assert_eq!(AppSettings::parse_hotkey("Shift+F"), Some((4, 'F' as u32)));
        assert_eq!(AppSettings::parse_hotkey("Invalid"), None);

        assert_eq!(AppSettings::parse_lock_key("End"), 0x23);
        assert_eq!(AppSettings::parse_lock_key("Home"), 0x24);
        assert_eq!(AppSettings::parse_lock_key("Delete"), 0x2E);
        assert_eq!(AppSettings::parse_lock_key("F1"), 0x70);
        assert_eq!(AppSettings::parse_lock_key("F12"), 0x7B);
        assert_eq!(AppSettings::parse_lock_key("Numpad 0"), 0x60);
    }

    #[test]
    fn test_app_settings_helpers_and_validation() {
        assert!(AppSettings::is_valid_steam_id("76561198000000001"));
        assert!(!AppSettings::is_valid_steam_id("7656119800000000")); // 16 digits
        assert!(!AppSettings::is_valid_steam_id("765611980000000012")); // 18 digits
        assert!(!AppSettings::is_valid_steam_id("7656119800000000A")); // non-digit

        let mut settings = AppSettings::default();
        assert_eq!(settings.lock_inputs_key, "End");
        settings.cycle_lock_key();
        assert_eq!(settings.lock_inputs_key, "Home");

        assert_eq!(AppSettings::cycle_modifier("Ctrl+Q"), "Alt+Q");
        assert_eq!(AppSettings::cycle_modifier("Alt+Q"), "Shift+Q");
        assert_eq!(AppSettings::cycle_modifier("Shift+Q"), "Ctrl+Q");

        assert_eq!(AppSettings::cycle_key("Ctrl+Q"), "Ctrl+E");

        assert_eq!(settings.hotkey_couch_osd, "F10");
        settings.cycle_hotkey_modifier("couch_osd");
        assert_eq!(settings.hotkey_couch_osd, "Ctrl+F10");
        settings.cycle_hotkey_key("couch_osd");
        assert_eq!(settings.hotkey_couch_osd, "Ctrl+F11");
    }
}

