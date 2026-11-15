use nucleus_engine::session::{ActiveSessionSlotSnapshot, ActiveSessionSnapshot};
use nucleus_ui::big_picture::overlay::{
    InGameOsdState, OsdActivePlayer, OsdSubView, format_duration_hhmmss,
};

#[test]
fn test_sparse_active_instances_tile_mapping() {
    let active_player_slots = vec![0, 2]; // Player 1 dropped out
    let layout_tile_count = 2;
    assert_eq!(active_player_slots.len(), layout_tile_count);

    // Verify ordinal mapping: Tile 0 -> Player 0, Tile 1 -> Player 2
    let tile_map: Vec<(usize, usize)> = active_player_slots
        .iter()
        .enumerate()
        .map(|(tile_idx, &player_idx)| (tile_idx, player_idx))
        .collect();
    assert_eq!(tile_map, vec![(0, 0), (1, 2)]);
}

#[test]
fn test_osd_duration_formatting() {
    assert_eq!(format_duration_hhmmss(0), "00:00:00");
    assert_eq!(format_duration_hhmmss(59), "00:00:59");
    assert_eq!(format_duration_hhmmss(60), "00:01:00");
    assert_eq!(format_duration_hhmmss(768), "00:12:48");
    assert_eq!(format_duration_hhmmss(3665), "01:01:05");
}

#[test]
fn test_active_session_snapshot_elapsed_seconds() {
    let snap = ActiveSessionSnapshot {
        game_guid: "game-123".to_string(),
        game_name: "Left 4 Dead 2".to_string(),
        layout: "1x2 (Stacked)".to_string(),
        max_players: 4,
        is_frozen: false,
        elapsed_seconds: 768,
        slots: vec![
            ActiveSessionSlotSnapshot {
                slot_idx: 0,
                nickname: "Player 1".to_string(),
                device_assignment: "Gamepad #0".to_string(),
                pid: Some(101),
                hwnd: Some(201),
            },
            ActiveSessionSlotSnapshot {
                slot_idx: 1,
                nickname: "Player 2".to_string(),
                device_assignment: "Gamepad #1".to_string(),
                pid: Some(102),
                hwnd: Some(202),
            },
        ],
    };

    let mut osd = InGameOsdState::new();
    assert_eq!(osd.elapsed_seconds, 0);

    osd.sync_from_snapshot(&snap);
    assert_eq!(osd.elapsed_seconds, 768);
    assert_eq!(osd.active_players.len(), 2);
    assert_eq!(osd.current_layout, "1x2 (Stacked)");
    assert_eq!(format_duration_hhmmss(osd.elapsed_seconds), "00:12:48");
}

#[test]
fn test_osd_sub_views_transitions() {
    let mut osd = InGameOsdState::new();
    assert_eq!(osd.sub_view, OsdSubView::Main);

    // Open Player Devices
    osd.open_player_devices();
    assert_eq!(osd.sub_view, OsdSubView::PlayerDevices);

    // Cancel back to Main
    assert!(osd.cancel_submodal());
    assert_eq!(osd.sub_view, OsdSubView::Main);

    // Open Game Guide
    osd.open_game_guide();
    assert_eq!(osd.sub_view, OsdSubView::GameGuide);

    // Explicit back to main
    osd.back_to_main();
    assert_eq!(osd.sub_view, OsdSubView::Main);
}

#[test]
fn test_drop_in_player_lowest_vacant_slot_allocation() {
    let mut osd = InGameOsdState::new();
    osd.max_players = 4;
    // Slots 0 and 2 are active (Slot 1 was dropped out)
    osd.active_players = vec![
        OsdActivePlayer {
            slot_idx: 0,
            nickname: "Player 1".to_string(),
            device_label: "Gamepad #0".to_string(),
        },
        OsdActivePlayer {
            slot_idx: 2,
            nickname: "Player 3".to_string(),
            device_label: "Gamepad #2".to_string(),
        },
    ];

    let used: std::collections::HashSet<usize> =
        osd.active_players.iter().map(|p| p.slot_idx).collect();
    let next_slot = (0..osd.max_players)
        .find(|idx| !used.contains(idx))
        .unwrap_or(osd.active_players.len());

    // Slot 1 is the lowest vacant slot
    assert_eq!(next_slot, 1);
}

#[test]
fn test_drop_out_player_survivors_retention() {
    let mut osd = InGameOsdState::new();
    osd.active_players = vec![
        OsdActivePlayer {
            slot_idx: 0,
            nickname: "Player 1".to_string(),
            device_label: "Gamepad #0".to_string(),
        },
        OsdActivePlayer {
            slot_idx: 1,
            nickname: "Player 2".to_string(),
            device_label: "Gamepad #1".to_string(),
        },
        OsdActivePlayer {
            slot_idx: 2,
            nickname: "Player 3".to_string(),
            device_label: "Gamepad #2".to_string(),
        },
    ];

    // Dropping out slot 1
    let drop_out_target = 1;
    osd.active_players.retain(|p| p.slot_idx != drop_out_target);

    assert_eq!(osd.active_players.len(), 2);
    assert_eq!(osd.active_players[0].slot_idx, 0);
    assert_eq!(osd.active_players[1].slot_idx, 2);
}
