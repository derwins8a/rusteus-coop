use nucleus_engine::session::GameSession;

#[test]
fn test_session_mock_creation() {
    let session = GameSession::mock_two_player_session();

    assert!(session.is_running);
    assert_eq!(session.players.len(), 2);
    assert_eq!(session.players[0].player_index, 0);
    assert_eq!(session.players[0].nickname, "Player 1");
    assert_eq!(session.players[0].device_id, 0);

    assert_eq!(session.players[1].player_index, 1);
    assert_eq!(session.players[1].nickname, "Player 2");
    assert_eq!(session.players[1].device_id, 1);

    assert!(!session.is_frozen());
}

#[test]
fn test_session_freeze_and_controller_reassignment() {
    let mut session = GameSession::mock_two_player_session();
    assert!(!session.is_frozen());

    // Freeze all instances
    session.freeze_all().expect("freeze_all should succeed");
    assert!(session.is_frozen());

    // Unfreeze all instances
    session.unfreeze_all().expect("unfreeze_all should succeed");
    assert!(!session.is_frozen());

    // Reassign controller for slot 0 to device ID 3
    session
        .reassign_controller(0, 3)
        .expect("reassign_controller for slot 0 should succeed");
    assert_eq!(session.players[0].device_id, 3);

    // Reassign controller for slot 1 to device ID 4
    session
        .reassign_controller(1, 4)
        .expect("reassign_controller for slot 1 should succeed");
    assert_eq!(session.players[1].device_id, 4);

    // Re-enforce window positions
    session
        .re_enforce_windows()
        .expect("re_enforce_windows should succeed");

    // Clean session stop
    session.stop().expect("stop should succeed");
    assert!(!session.is_running);
}

#[test]
fn test_launch_countdown_osd_format() {
    let player_idx = 1; // Player 2
    let remaining_secs = 24;
    let msg = format!("Launching Player {} in {}s...", player_idx + 1, remaining_secs);
    assert_eq!(msg, "Launching Player 2 in 24s...");
    assert!(msg.starts_with("Launching Player 2"));
    assert!(msg.ends_with("in 24s..."));
}
