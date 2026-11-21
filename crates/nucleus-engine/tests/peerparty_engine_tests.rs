//! Unit and integration tests for PeerParty session coordination in nucleus-engine.

use nucleus_engine::session::GameSession;
use nucleus_handler::game_handler::GameHandler;
use nucleus_profile::game_profile::GameProfile;
use nucleus_script::handler_config::HandlerConfig;
use std::path::PathBuf;

#[test]
fn test_session_peerparty_status() {
    let profile = GameProfile::new("test_game", "guid_1", "C:/Games/game.exe");
    let mut session = GameSession::new("session_1", "guid_1", profile);

    let mut config = HandlerConfig::default();
    config.use_peerparty = true;
    config.peerparty_room = Some("TEST_ROOM_123".to_string());

    let handler = GameHandler::new(
        config,
        PathBuf::from("C:/Games"),
        PathBuf::from("C:/Games/game.exe"),
        PathBuf::from("C:/Games/staging"),
        2,
    );
    session.set_handler(handler);

    assert!(session.is_peerparty_active());
    assert_eq!(session.peerparty_room_code(), Some("TEST_ROOM_123"));

    session.stop().unwrap();
    assert!(!session.is_running);
}

#[test]
fn test_session_toggle_peerparty_active() {
    let profile = GameProfile::new("test_game", "guid_1", "C:/Games/game.exe");
    let mut session = GameSession::new("session_1", "guid_1", profile);

    let config = HandlerConfig::default();
    let handler = GameHandler::new(
        config,
        PathBuf::from("C:/Games"),
        PathBuf::from("C:/Games/game.exe"),
        PathBuf::from("C:/Games/staging"),
        2,
    );
    session.set_handler(handler);

    assert!(!session.is_peerparty_active());
    session.set_peerparty_active(true);
    assert!(session.is_peerparty_active());
}
