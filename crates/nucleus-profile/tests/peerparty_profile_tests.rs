//! Unit tests for PeerParty settings and profile persistence.

use nucleus_profile::game_profile::GameProfile;
use nucleus_profile::settings::AppSettings;

#[test]
fn test_settings_peerparty_defaults() {
    let settings = AppSettings::default();
    assert!(settings.peerparty_enabled);
    assert_eq!(settings.peerparty_default_room, None);
    assert!(!settings.peerparty_public_sessions);
    assert!(!settings.peerparty_voice_chat);
    assert!(settings.peerparty_overlay);
    assert_eq!(settings.preferred_emulator, "Auto");
}

#[test]
fn test_settings_peerparty_serde_backward_compat() {
    // Legacy JSON without peerparty fields
    let legacy_json = r#"{
        "handlers_directory": "handlers",
        "theme": "dark"
    }"#;

    let settings: AppSettings = serde_json::from_str(legacy_json).expect("Deserialization failed");
    assert!(settings.peerparty_enabled);
    assert_eq!(settings.peerparty_default_room, None);
    assert!(!settings.peerparty_public_sessions);
    assert_eq!(settings.preferred_emulator, "Auto");
}

#[test]
fn test_game_profile_peerparty_fields() {
    let profile = GameProfile::new("game-1", "guid-1", "C:/Games/game.exe");
    assert_eq!(profile.preferred_emulator, "Auto");
    assert_eq!(profile.peerparty_room_override, None);

    let json = serde_json::to_string(&profile).expect("Serialization failed");
    let deserialized: GameProfile = serde_json::from_str(&json).expect("Deserialization failed");
    assert_eq!(deserialized.preferred_emulator, "Auto");
    assert_eq!(deserialized.peerparty_room_override, None);
}
