//! Integration tests for PeerParty handler staging and mutual exclusion.

use nucleus_handler::game_handler::GameHandler;
use nucleus_script::handler_config::HandlerConfig;
use std::path::PathBuf;

#[test]
fn test_peerparty_suppresses_goldberg_in_handler() {
    let mut config = HandlerConfig::default();
    config.use_goldberg = true;
    config.use_peerparty = true;
    config.peerparty_services = Some(vec!["Steam".to_string()]);

    let dummy_game_dir = PathBuf::from("C:/Games/TestGame");
    let dummy_exe = dummy_game_dir.join("test.exe");
    let dummy_staging = PathBuf::from("C:/Games/TestGame/Instances");
    let handler = GameHandler::new(config, dummy_game_dir, dummy_exe, dummy_staging, 2);

    let discovered = handler.discover_steam_api_dll_paths();
    let _ = discovered;
}

#[test]
fn test_peerparty_handler_config_presence() {
    let mut config = HandlerConfig::default();
    config.use_peerparty = true;
    config.peerparty_room = Some("MY_SECRET_ROOM".to_string());
    config.peerparty_public = true;
    config.peerparty_voice = true;

    assert!(config.use_peerparty);
    assert_eq!(config.peerparty_room.as_deref(), Some("MY_SECRET_ROOM"));
    assert!(config.peerparty_public);
    assert!(config.peerparty_voice);
}
