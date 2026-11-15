use nucleus_ui::big_picture::overlay::{
    DropInStep, DropOutStep, InGameOsdState, OsdAction, OsdActivePlayer, player_slot_tint,
    summoner_focus_color, valid_layouts_for_player_count,
};

#[test]
fn test_in_game_osd_state_navigation() {
    let mut osd = InGameOsdState::new();
    assert!(!osd.is_visible);
    assert_eq!(osd.selected_index, 0);
    assert_eq!(OsdAction::count(), 8);

    // Open OSD
    osd.open();
    assert!(osd.is_visible);
    assert_eq!(osd.selected_index, 0);

    // Navigate down through 8 items (indices 0..=7)
    for expected in 1..=7 {
        osd.move_down();
        assert_eq!(osd.selected_index, expected);
    }

    // Wrap around downwards: 7 -> 0
    osd.move_down();
    assert_eq!(osd.selected_index, 0);

    // Wrap around upwards: 0 -> 7
    osd.move_up();
    assert_eq!(osd.selected_index, 7);
    osd.move_up();
    assert_eq!(osd.selected_index, 6);

    // Close OSD
    osd.close();
    assert!(!osd.is_visible);

    // Toggle visibility
    osd.toggle();
    assert!(osd.is_visible);
    osd.toggle();
    assert!(!osd.is_visible);
}

#[test]
fn test_in_game_osd_action_selection() {
    let mut osd = InGameOsdState::new();

    osd.select(0);
    assert_eq!(osd.selected_action(), OsdAction::ResumeGame);

    osd.select(1);
    assert_eq!(osd.selected_action(), OsdAction::ToggleFreeze);

    osd.select(2);
    assert_eq!(osd.selected_action(), OsdAction::ReassignControllers);

    osd.select(3);
    assert_eq!(osd.selected_action(), OsdAction::DropInPlayer);

    osd.select(4);
    assert_eq!(osd.selected_action(), OsdAction::DropOutPlayer);

    osd.select(5);
    assert_eq!(osd.selected_action(), OsdAction::ReEnforceWindows);

    osd.select(6);
    assert_eq!(osd.selected_action(), OsdAction::ToggleInputLock);

    osd.select(7);
    assert_eq!(osd.selected_action(), OsdAction::StopSession);

    // Out of bounds clamps to index 7
    osd.select(99);
    assert_eq!(osd.selected_index, 7);
    assert_eq!(osd.selected_action(), OsdAction::StopSession);
}

#[test]
fn test_in_game_osd_reassignment_state() {
    let mut osd = InGameOsdState::new();
    assert!(!osd.reassign_modal_open);

    // Open reassign modal for slot 1
    osd.open_reassign(1);
    assert!(osd.reassign_modal_open);
    assert_eq!(osd.reassign_slot_idx, 1);
    assert_eq!(osd.reassign_new_device_id, None);

    // Select new device ID 3
    osd.reassign_new_device_id = Some(3);

    // Confirm reassignment
    let res = osd.confirm_reassign();
    assert_eq!(res, Some((1, 3)));
    assert!(!osd.reassign_modal_open);
    assert_eq!(osd.reassign_new_device_id, None);

    // Confirming when closed yields None
    assert_eq!(osd.confirm_reassign(), None);

    // Reopen and cancel
    osd.open_reassign(0);
    assert!(osd.reassign_modal_open);
    osd.close_reassign();
    assert!(!osd.reassign_modal_open);

    // Closing OSD also dismisses reassign modal
    osd.open();
    osd.open_reassign(0);
    assert!(osd.is_visible);
    assert!(osd.reassign_modal_open);
    osd.close();
    assert!(!osd.is_visible);
    assert!(!osd.reassign_modal_open);
}

#[test]
fn test_in_game_osd_summoner_slot() {
    let mut osd = InGameOsdState::new();
    assert_eq!(osd.summoner_slot, None);

    osd.summoner_slot = Some(2);
    assert_eq!(osd.summoner_slot, Some(2));

    osd.close();
    assert_eq!(osd.summoner_slot, None);
}

#[test]
fn test_summoner_focus_and_slot_colors() {
    let host = summoner_focus_color(None);
    let p1 = summoner_focus_color(Some(0));
    let p2 = summoner_focus_color(Some(1));
    let p3 = summoner_focus_color(Some(2));
    let p4 = summoner_focus_color(Some(3));

    // P1 and host default match primary accent
    assert_eq!(host, p1);
    // Distinct slot colors for distinct players
    assert_ne!(p1, p2);
    assert_ne!(p2, p3);
    assert_ne!(p3, p4);

    // Slot tint checks
    assert_eq!(player_slot_tint(0), p1);
    assert_eq!(player_slot_tint(1), p2);
    assert_eq!(player_slot_tint(2), p3);
    assert_eq!(player_slot_tint(3), p4);
    // Modulo 4 wrap around
    assert_eq!(player_slot_tint(4), player_slot_tint(0));
}

#[test]
fn test_main_window_osd_summoner_lifecycle() {
    use std::sync::{Arc, Mutex};
    let calls = Arc::new(Mutex::new(Vec::<(bool, Option<usize>)>::new()));
    let calls_clone = calls.clone();

    let mut win = nucleus_ui::MainWindow::new(2, vec!["Game 1".to_string()]);
    win.session_active = true;

    win = win.with_osd_topmost_handler(move |visible, summoner| {
        calls_clone.lock().unwrap().push((visible, summoner));
    });

    // Player 2 (slot 1) summons overlay
    win.set_osd_visible_for_player(Some(1), true);
    assert!(win.in_game_osd.is_visible);
    assert_eq!(win.in_game_osd.summoner_slot, Some(1));
    assert_eq!(win.in_game_osd.reassign_slot_idx, 1);
    assert_eq!(*calls.lock().unwrap(), vec![(true, Some(1))]);

    // Close overlay
    win.set_osd_visible_for_player(Some(1), false);
    assert!(!win.in_game_osd.is_visible);
    assert_eq!(win.in_game_osd.summoner_slot, None);
    assert_eq!(*calls.lock().unwrap(), vec![(true, Some(1)), (false, Some(1))]);
}

#[test]
fn test_valid_layouts_for_player_count_matches_canonical_setup() {
    assert_eq!(
        valid_layouts_for_player_count(1),
        vec!["Fullscreen (1 Player)"]
    );
    assert_eq!(
        valid_layouts_for_player_count(2),
        vec!["2x1 (Side-by-Side)", "1x2 (Stacked)", "Multi-Monitor"]
    );
    assert_eq!(
        valid_layouts_for_player_count(3),
        vec![
            "Top 1 / Bottom 2",
            "Top 2 / Bottom 1",
            "2x2 (Grid)",
            "Multi-Monitor",
        ]
    );
    assert_eq!(
        valid_layouts_for_player_count(4),
        vec![
            "2x2 (Grid)",
            "2x1 (Side-by-Side)",
            "1x2 (Stacked)",
            "Multi-Monitor",
        ]
    );
}

#[test]
fn test_osd_drop_in_two_step_flow_and_max_capacity_guard() {
    let mut osd = InGameOsdState::new();
    osd.open();

    // Capacity guard when active_count >= max_players
    assert!(!osd.open_drop_in(4, 4));
    assert_eq!(osd.drop_in_step, DropInStep::None);
    assert!(osd.action_error.is_some());

    // Valid capacity -> opens Step 1 PickController and clears action_error
    assert!(osd.open_drop_in(2, 4));
    assert_eq!(osd.action_error, None);
    assert_eq!(osd.drop_in_step, DropInStep::PickController { focused_idx: 0 });

    // Select device -> advances to Step 2 SelectLayout
    osd.select_drop_in_device(
        Some(2),
        "Device #2: Controller".to_string(),
        "Gamepad #2 (Controller)".to_string(),
    );
    assert!(matches!(
        osd.drop_in_step,
        DropInStep::SelectLayout {
            chosen_device_id: Some(2),
            ..
        }
    ));

    // Cancel step-back: SelectLayout -> PickController -> None -> outer close
    assert!(osd.cancel_submodal());
    assert_eq!(osd.drop_in_step, DropInStep::PickController { focused_idx: 0 });
    assert!(osd.cancel_submodal());
    assert_eq!(osd.drop_in_step, DropInStep::None);
    assert!(!osd.cancel_submodal());
}

#[test]
fn test_osd_drop_out_two_step_flow_non_summoner_and_single_player_guard() {
    let mut osd = InGameOsdState::new();
    osd.open();
    osd.active_players = vec![OsdActivePlayer {
        slot_idx: 0,
        nickname: "P1".to_string(),
        device_label: "Gamepad #0".to_string(),
    }];

    // Single active player guard keeps OSD on main menu with explanatory error
    assert!(!osd.open_drop_out(Some(0)));
    assert_eq!(osd.drop_out_step, DropOutStep::None);
    assert_eq!(
        osd.action_error.as_deref(),
        Some("Only 1 active player — use End Game Session to exit.")
    );

    // Three active non-contiguous slots [0, 2, 3], summoned by slot 3
    osd.active_players = vec![
        OsdActivePlayer {
            slot_idx: 0,
            nickname: "P1".to_string(),
            device_label: "Gamepad #0".to_string(),
        },
        OsdActivePlayer {
            slot_idx: 2,
            nickname: "P3".to_string(),
            device_label: "Gamepad #2".to_string(),
        },
        OsdActivePlayer {
            slot_idx: 3,
            nickname: "P4".to_string(),
            device_label: "Keyboard & Mouse".to_string(),
        },
    ];
    assert!(osd.open_drop_out(Some(3)));
    assert_eq!(osd.action_error, None);
    // Summoner slot 3 is index 2 inside active_players
    assert_eq!(osd.drop_out_step, DropOutStep::PickSlot { focused_idx: 2 });

    // Any player slot (e.g. non-summoner middle slot 2) can be selected
    osd.select_drop_out_slot(2);
    assert_eq!(
        osd.drop_out_step,
        DropOutStep::SelectLayout {
            target_slot_idx: 2,
            focused_layout_idx: 0,
        }
    );

    // Cancel steps back to PickSlot focused on slot 2 (index 1) -> None -> false
    assert!(osd.cancel_submodal());
    assert_eq!(osd.drop_out_step, DropOutStep::PickSlot { focused_idx: 1 });
    assert!(osd.cancel_submodal());
    assert_eq!(osd.drop_out_step, DropOutStep::None);
    assert!(!osd.cancel_submodal());
}

#[test]
fn test_callbacks_builder_wires_version_install_and_drop_in_out() {
    let cbs = nucleus_ui::MainWindowCallbacks::new()
        .with_hub_version_installer(|id, pkg, ver, sz, _cb| {
            let _ = (id, pkg, ver, sz);
            None
        })
        .with_drop_in_handler(|assign, layout| {
            assert_eq!(assign, "Virtual Gamepad");
            assert_eq!(layout, "Top 1 / Bottom 2");
            Ok(2)
        })
        .with_drop_out_handler(|slot, layout| {
            assert_eq!(slot, 1);
            assert_eq!(layout, "Fullscreen (1 Player)");
            Ok(1)
        })
        .with_active_session_querier(|| None);

    assert!(cbs.on_install_handler_version.is_some());
    assert_eq!(
        (cbs.on_drop_in_player.as_ref().unwrap())("Virtual Gamepad", "Top 1 / Bottom 2"),
        Ok(2)
    );
    assert_eq!(
        (cbs.on_drop_out_player.as_ref().unwrap())(1, "Fullscreen (1 Player)"),
        Ok(1)
    );
    assert!((cbs.on_query_active_session.as_ref().unwrap())().is_none());
}
