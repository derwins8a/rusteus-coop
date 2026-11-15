use std::time::{Duration, Instant};
use nucleus_ui::big_picture::input::{
    GamepadInputDispatcher, StickDirection, map_key_to_couch_action_name,
};
use nucleus_ui::actions::{
    CancelAction, ConfirmAction, ContextAction1, ContextAction2, NavigateDown, NavigateLeft,
    NavigateRight, NavigateUp, NextTab, PrevTab, QuickLaunch, ToggleBigPicture,
};
use nucleus_ui::theme;

#[test]
fn test_stick_deadzone_and_repeat_rate() {
    let mut dispatcher = GamepadInputDispatcher::new(0.5); // 0.5 deadzone
    let now = Instant::now();

    // 0.3 deflection -> deadzone suppresses it
    assert_eq!(dispatcher.poll_stick(0.3, 0.0, now), None);

    // 0.8 deflection -> initial trigger
    assert_eq!(dispatcher.poll_stick(0.8, 0.0, now), Some(StickDirection::Right));

    // Polled 50ms later -> debounce suppresses repeat
    assert_eq!(dispatcher.poll_stick(0.8, 0.0, now + Duration::from_millis(50)), None);

    // Polled 250ms later -> repeat fires
    assert_eq!(dispatcher.poll_stick(0.8, 0.0, now + Duration::from_millis(250)), Some(StickDirection::Right));
}

#[test]
fn test_stick_all_directions_and_subsequent_repeat_rate() {
    let mut dispatcher = GamepadInputDispatcher::new(0.5);
    let now = Instant::now();

    // Test Up (+Y)
    assert_eq!(dispatcher.poll_stick(0.0, 0.8, now), Some(StickDirection::Up));
    // Release
    assert_eq!(dispatcher.poll_stick(0.0, 0.0, now + Duration::from_millis(10)), None);

    // Test Down (-Y)
    assert_eq!(dispatcher.poll_stick(0.0, -0.8, now + Duration::from_millis(20)), Some(StickDirection::Down));
    // Release
    assert_eq!(dispatcher.poll_stick(0.0, 0.0, now + Duration::from_millis(30)), None);

    // Test Left (-X)
    assert_eq!(dispatcher.poll_stick(-0.8, 0.0, now + Duration::from_millis(40)), Some(StickDirection::Left));
    // Release
    assert_eq!(dispatcher.poll_stick(0.0, 0.0, now + Duration::from_millis(50)), None);

    // Test Right (+X) with initial repeat (200ms) and subsequent repeat (100ms)
    let t0 = now + Duration::from_millis(60);
    assert_eq!(dispatcher.poll_stick(0.9, 0.0, t0), Some(StickDirection::Right));

    // 150ms after initial -> no repeat yet
    assert_eq!(dispatcher.poll_stick(0.9, 0.0, t0 + Duration::from_millis(150)), None);

    // 210ms after initial -> first repeat fires (>= 200ms)
    assert_eq!(dispatcher.poll_stick(0.9, 0.0, t0 + Duration::from_millis(210)), Some(StickDirection::Right));

    // 50ms after first repeat -> no repeat yet (100ms interval)
    assert_eq!(dispatcher.poll_stick(0.9, 0.0, t0 + Duration::from_millis(260)), None);

    // 110ms after first repeat -> second repeat fires (>= 100ms)
    assert_eq!(dispatcher.poll_stick(0.9, 0.0, t0 + Duration::from_millis(320)), Some(StickDirection::Right));
}

#[test]
fn test_direction_immediate_switch() {
    let mut dispatcher = GamepadInputDispatcher::new(0.5);
    let now = Instant::now();

    // Deflect Right
    assert_eq!(dispatcher.poll_stick(0.8, 0.0, now), Some(StickDirection::Right));

    // Flick directly to Down at 50ms without waiting for debounce
    assert_eq!(dispatcher.poll_stick(0.0, -0.8, now + Duration::from_millis(50)), Some(StickDirection::Down));
}

#[test]
fn test_couch_action_structs_exist() {
    let _ = NavigateUp;
    let _ = NavigateDown;
    let _ = NavigateLeft;
    let _ = NavigateRight;
    let _ = ConfirmAction;
    let _ = CancelAction;
    let _ = NextTab;
    let _ = PrevTab;
    let _ = ContextAction1;
    let _ = ContextAction2;
    let _ = QuickLaunch;
    let _ = ToggleBigPicture;
}

#[test]
fn test_button_edge_triggered() {
    let mut dispatcher = GamepadInputDispatcher::new(0.5);
    let now = Instant::now();

    // Press South (A) -> triggers ConfirmAction
    let act = dispatcher.poll_button(gilrs::Button::South, true, now);
    assert!(act.is_some());

    // Holding South (A) 50ms later -> does NOT repeat
    let act_held = dispatcher.poll_button(gilrs::Button::South, true, now + Duration::from_millis(50));
    assert!(act_held.is_none());

    // Releasing South
    let act_rel = dispatcher.poll_button(gilrs::Button::South, false, now + Duration::from_millis(100));
    assert!(act_rel.is_none());

    // Pressing South again -> triggers ConfirmAction again
    let act_again = dispatcher.poll_button(gilrs::Button::South, true, now + Duration::from_millis(150));
    assert!(act_again.is_some());
}

#[test]
fn test_focus_glow_theme_utility() {
    let glow = theme::focus_glow();
    // Accent with 0.35 opacity
    assert!((glow.a - 0.35).abs() < 0.01);
}

#[test]
fn test_dpad_repeat_rate() {
    let mut dispatcher = GamepadInputDispatcher::new(0.5);
    let now = Instant::now();

    // D-pad Right pressed -> initial trigger
    assert_eq!(
        dispatcher.poll_dpad(false, false, false, true, now),
        Some(StickDirection::Right)
    );

    // Polled 50ms later -> debounce suppresses repeat
    assert_eq!(
        dispatcher.poll_dpad(false, false, false, true, now + Duration::from_millis(50)),
        None
    );

    // Polled 210ms later -> initial repeat fires (>= 200ms)
    assert_eq!(
        dispatcher.poll_dpad(false, false, false, true, now + Duration::from_millis(210)),
        Some(StickDirection::Right)
    );

    // Polled 50ms after repeat -> debounce suppresses
    assert_eq!(
        dispatcher.poll_dpad(false, false, false, true, now + Duration::from_millis(260)),
        None
    );

    // Polled 110ms after first repeat -> second repeat fires (>= 100ms)
    assert_eq!(
        dispatcher.poll_dpad(false, false, false, true, now + Duration::from_millis(320)),
        Some(StickDirection::Right)
    );
}

#[test]
fn test_interleaved_neutral_stick_and_held_dpad() {
    let mut dispatcher = GamepadInputDispatcher::new(0.5);
    let now = Instant::now();

    // Frame 0: Stick is neutral, D-pad Right is held
    assert_eq!(dispatcher.poll_stick(0.0, 0.0, now), None);
    assert_eq!(
        dispatcher.poll_dpad(false, false, false, true, now),
        Some(StickDirection::Right)
    );

    // Frame 1 (50ms later): Stick is neutral, D-pad Right is STILL held
    // Crucial check: poll_stick(0.0, 0.0) MUST NOT wipe D-pad repeat state!
    assert_eq!(
        dispatcher.poll_stick(0.0, 0.0, now + Duration::from_millis(50)),
        None
    );
    assert_eq!(
        dispatcher.poll_dpad(false, false, false, true, now + Duration::from_millis(50)),
        None
    );

    // Frame 2 (210ms later): Stick is neutral, D-pad Right is STILL held
    // Repeat should fire after 200ms initial repeat delay has elapsed
    assert_eq!(
        dispatcher.poll_stick(0.0, 0.0, now + Duration::from_millis(210)),
        None
    );
    assert_eq!(
        dispatcher.poll_dpad(false, false, false, true, now + Duration::from_millis(210)),
        Some(StickDirection::Right)
    );
}

#[test]
fn test_dpad_release_resets_state() {
    let mut dispatcher = GamepadInputDispatcher::new(0.5);
    let now = Instant::now();

    // Hold D-pad Up
    assert_eq!(
        dispatcher.poll_dpad(true, false, false, false, now),
        Some(StickDirection::Up)
    );

    // Release all D-pad directions
    assert_eq!(
        dispatcher.poll_dpad(false, false, false, false, now + Duration::from_millis(50)),
        None
    );

    // Re-press D-pad Up at 60ms -> fires immediately because state was reset on release
    assert_eq!(
        dispatcher.poll_dpad(true, false, false, false, now + Duration::from_millis(60)),
        Some(StickDirection::Up)
    );
}

#[test]
fn test_mode_button_toggles_big_picture_select_does_not() {
    let mut dispatcher = GamepadInputDispatcher::new(0.5);
    let now = Instant::now();

    // Select button must NOT trigger ToggleBigPicture (reserved for OSD combo)
    assert!(dispatcher
        .poll_button(gilrs::Button::Select, true, now)
        .is_none());

    // Mode button DOES trigger ToggleBigPicture
    assert!(dispatcher
        .poll_button(gilrs::Button::Mode, true, now + Duration::from_millis(50))
        .is_some());
}

#[test]
fn test_back_start_combo_hold_triggers_toggle_osd() {
    let mut dispatcher = GamepadInputDispatcher::new(0.5);
    let now = Instant::now();

    // 1. Neither held -> None
    assert!(dispatcher.poll_combo_osd(false, false, now).is_none());

    // 2. Only Back held -> None
    assert!(dispatcher.poll_combo_osd(true, false, now).is_none());

    // 3. Only Start held -> None
    assert!(dispatcher.poll_combo_osd(false, true, now).is_none());

    // 4. Both held at t=0 -> None (timer started)
    assert!(dispatcher.poll_combo_osd(true, true, now).is_none());

    // 5. Both held at t=1000ms -> None (< 1500ms)
    assert!(dispatcher
        .poll_combo_osd(true, true, now + Duration::from_millis(1000))
        .is_none());

    // 6. Both held at t=1500ms -> Some(ToggleOsd)
    let action = dispatcher.poll_combo_osd(true, true, now + Duration::from_millis(1500));
    assert!(action.is_some());

    // 7. Both still held at t=2000ms -> None (suppressed until release, fires once per hold)
    assert!(dispatcher
        .poll_combo_osd(true, true, now + Duration::from_millis(2000))
        .is_none());

    // 8. Release
    assert!(dispatcher
        .poll_combo_osd(false, false, now + Duration::from_millis(2100))
        .is_none());

    // 9. New hold at t=3000ms -> timer restarts
    assert!(dispatcher
        .poll_combo_osd(true, true, now + Duration::from_millis(3000))
        .is_none());
    assert!(dispatcher
        .poll_combo_osd(true, true, now + Duration::from_millis(4500))
        .is_some());
}

#[test]
fn test_map_key_to_couch_action_name_navigation() {
    // Arrow keys
    assert_eq!(map_key_to_couch_action_name("arrowup", false, false), Some("NavigateUp"));
    assert_eq!(map_key_to_couch_action_name("ArrowUp", false, false), Some("NavigateUp"));
    assert_eq!(map_key_to_couch_action_name("up", false, false), Some("NavigateUp"));
    assert_eq!(map_key_to_couch_action_name("UP", false, false), Some("NavigateUp"));
    assert_eq!(map_key_to_couch_action_name("w", false, false), Some("NavigateUp"));
    assert_eq!(map_key_to_couch_action_name("W", false, false), Some("NavigateUp"));

    assert_eq!(map_key_to_couch_action_name("arrowdown", false, false), Some("NavigateDown"));
    assert_eq!(map_key_to_couch_action_name("ArrowDown", false, false), Some("NavigateDown"));
    assert_eq!(map_key_to_couch_action_name("down", false, false), Some("NavigateDown"));
    assert_eq!(map_key_to_couch_action_name("DOWN", false, false), Some("NavigateDown"));
    assert_eq!(map_key_to_couch_action_name("s", false, false), Some("NavigateDown"));
    assert_eq!(map_key_to_couch_action_name("S", false, false), Some("NavigateDown"));

    assert_eq!(map_key_to_couch_action_name("arrowleft", false, false), Some("NavigateLeft"));
    assert_eq!(map_key_to_couch_action_name("ArrowLeft", false, false), Some("NavigateLeft"));
    assert_eq!(map_key_to_couch_action_name("left", false, false), Some("NavigateLeft"));
    assert_eq!(map_key_to_couch_action_name("LEFT", false, false), Some("NavigateLeft"));
    assert_eq!(map_key_to_couch_action_name("a", false, false), Some("NavigateLeft"));
    assert_eq!(map_key_to_couch_action_name("A", false, false), Some("NavigateLeft"));

    assert_eq!(map_key_to_couch_action_name("arrowright", false, false), Some("NavigateRight"));
    assert_eq!(map_key_to_couch_action_name("ArrowRight", false, false), Some("NavigateRight"));
    assert_eq!(map_key_to_couch_action_name("right", false, false), Some("NavigateRight"));
    assert_eq!(map_key_to_couch_action_name("RIGHT", false, false), Some("NavigateRight"));
    assert_eq!(map_key_to_couch_action_name("d", false, false), Some("NavigateRight"));
    assert_eq!(map_key_to_couch_action_name("D", false, false), Some("NavigateRight"));
}

#[test]
fn test_map_key_to_couch_action_name_confirm_cancel() {
    assert_eq!(map_key_to_couch_action_name("enter", false, false), Some("Confirm"));
    assert_eq!(map_key_to_couch_action_name("Enter", false, false), Some("Confirm"));
    assert_eq!(map_key_to_couch_action_name("space", false, false), Some("Confirm"));
    assert_eq!(map_key_to_couch_action_name("Space", false, false), Some("Confirm"));
    assert_eq!(map_key_to_couch_action_name("return", false, false), Some("Confirm"));

    assert_eq!(map_key_to_couch_action_name("escape", false, false), Some("Cancel"));
    assert_eq!(map_key_to_couch_action_name("Escape", false, false), Some("Cancel"));
    assert_eq!(map_key_to_couch_action_name("backspace", false, false), Some("Cancel"));
    assert_eq!(map_key_to_couch_action_name("Backspace", false, false), Some("Cancel"));
    assert_eq!(map_key_to_couch_action_name("b", false, false), Some("Cancel"));
    assert_eq!(map_key_to_couch_action_name("B", false, false), Some("Cancel"));
}

#[test]
fn test_map_key_to_couch_action_name_tabs_and_context() {
    // NextTab / PrevTab via Tab
    assert_eq!(map_key_to_couch_action_name("tab", false, false), Some("NextTab"));
    assert_eq!(map_key_to_couch_action_name("Tab", false, false), Some("NextTab"));
    assert_eq!(map_key_to_couch_action_name("tab", false, true), Some("PrevTab"));
    assert_eq!(map_key_to_couch_action_name("Tab", false, true), Some("PrevTab"));

    // Bumper aliases Q and E
    assert_eq!(map_key_to_couch_action_name("e", false, false), Some("NextTab"));
    assert_eq!(map_key_to_couch_action_name("E", false, false), Some("NextTab"));
    assert_eq!(map_key_to_couch_action_name("q", false, false), Some("PrevTab"));
    assert_eq!(map_key_to_couch_action_name("Q", false, false), Some("PrevTab"));

    // Context actions X and Y
    assert_eq!(map_key_to_couch_action_name("x", false, false), Some("Context1"));
    assert_eq!(map_key_to_couch_action_name("X", false, false), Some("Context1"));
    assert_eq!(map_key_to_couch_action_name("y", false, false), Some("Context2"));
    assert_eq!(map_key_to_couch_action_name("Y", false, false), Some("Context2"));
}

#[test]
fn test_map_key_to_couch_action_name_shortcuts_and_combos() {
    // Big Picture toggle: F11
    assert_eq!(map_key_to_couch_action_name("f11", false, false), Some("ToggleBigPicture"));
    assert_eq!(map_key_to_couch_action_name("F11", false, false), Some("ToggleBigPicture"));

    // QuickLaunch: F5
    assert_eq!(map_key_to_couch_action_name("f5", false, false), Some("QuickLaunch"));
    assert_eq!(map_key_to_couch_action_name("F5", false, false), Some("QuickLaunch"));

    // OSD: F10, Ctrl+Shift+O, Ctrl+Escape
    assert_eq!(map_key_to_couch_action_name("f10", false, false), Some("ToggleOsd"));
    assert_eq!(map_key_to_couch_action_name("F10", false, false), Some("ToggleOsd"));
    assert_eq!(map_key_to_couch_action_name("o", true, true), Some("ToggleOsd"));
    assert_eq!(map_key_to_couch_action_name("O", true, true), Some("ToggleOsd"));
    assert_eq!(map_key_to_couch_action_name("escape", true, false), Some("ToggleOsd"));

    // Suppressed modifier combinations
    assert_eq!(map_key_to_couch_action_name("o", true, false), None); // Ctrl+O alone does not trigger OSD
    assert_eq!(map_key_to_couch_action_name("o", false, true), None); // Shift+O alone does not trigger OSD
    assert_eq!(map_key_to_couch_action_name("w", true, false), None); // Ctrl+W does not navigate Up
    assert_eq!(map_key_to_couch_action_name("s", true, false), None); // Ctrl+S does not navigate Down
    assert_eq!(map_key_to_couch_action_name("enter", true, false), None);

    // Unhandled keys return None without panicking
    assert_eq!(map_key_to_couch_action_name("z", false, false), None);
    assert_eq!(map_key_to_couch_action_name("1", false, false), None);
    assert_eq!(map_key_to_couch_action_name("alt", false, false), None);
    assert_eq!(map_key_to_couch_action_name("control", false, false), None);
}

#[test]
fn spatial_rect_geometry_and_overlap_helpers() {
    use nucleus_ui::big_picture::spatial::SpatialRect;

    let a = SpatialRect::new(10.0, 20.0, 100.0, 40.0);
    assert!((a.cx() - 60.0).abs() < 1e-4);
    assert!((a.cy() - 40.0).abs() < 1e-4);
    assert!((a.right() - 110.0).abs() < 1e-4);
    assert!((a.bottom() - 60.0).abs() < 1e-4);

    // Overlapping in both axes
    let b = SpatialRect::new(60.0, 40.0, 80.0, 50.0);
    assert!((a.overlap_x(b) - 50.0).abs() < 1e-4);
    assert!((a.overlap_y(b) - 20.0).abs() < 1e-4);
    assert!(a.intersects_axis_x(b));
    assert!(a.intersects_axis_y(b));

    // Touching edge -> 0 overlap
    let c = SpatialRect::new(110.0, 20.0, 50.0, 40.0);
    assert!((a.overlap_x(c) - 0.0).abs() < 1e-4);
    assert!(!a.intersects_axis_x(c));
    assert!((a.overlap_y(c) - 40.0).abs() < 1e-4);
    assert!(a.intersects_axis_y(c));

    // Disjoint in both axes
    let d = SpatialRect::new(200.0, 200.0, 50.0, 50.0);
    assert!(!a.intersects_axis_x(d));
    assert!(!a.intersects_axis_y(d));
}

#[test]
fn spatial_bounds_registry_merges_live_over_fallback_for_active_targets_only() {
    use nucleus_ui::big_picture::spatial::{
        SpatialBoundsRegistry, SpatialNode, SpatialRect, SpatialTarget,
    };

    let registry = SpatialBoundsRegistry::new();
    let live_slot_rect = SpatialRect::new(40.0, 350.0, 320.0, 150.0);
    let stale_unassigned_rect = SpatialRect::new(40.0, 520.0, 200.0, 48.0);

    registry.record(SpatialTarget::PlayerSlot(0), live_slot_rect);
    registry.record(SpatialTarget::UnassignedItem(0), stale_unassigned_rect);

    assert_eq!(registry.resolve(SpatialTarget::PlayerSlot(0)), Some(live_slot_rect));

    let fallback = vec![
        SpatialNode {
            target: SpatialTarget::PlayerSlot(0),
            rect: SpatialRect::new(0.0, 0.0, 100.0, 100.0),
        },
        SpatialNode {
            target: SpatialTarget::LaunchButton,
            rect: SpatialRect::new(32.0, 620.0, 340.0, 48.0),
        },
    ];

    let merged = registry.into_nodes(fallback);
    assert_eq!(merged.len(), 2);
    assert_eq!(merged[0].rect, live_slot_rect);
    assert_eq!(merged[1].target, SpatialTarget::LaunchButton);
    assert!(merged
        .iter()
        .all(|n| n.target != SpatialTarget::UnassignedItem(0)));

    registry.clear();
    assert_eq!(registry.resolve(SpatialTarget::PlayerSlot(0)), None);
}

#[test]
fn spatial_setup_2x2_grid_navigation() {
    use nucleus_ui::big_picture::spatial::{
        SetupLayoutSpec, SpatialDirection, SpatialTarget, build_setup_layout_rects,
        resolve_spatial_move,
    };

    let spec = SetupLayoutSpec {
        has_km_row: true,
        has_raw_kb: true,
        has_raw_mouse: true,
        player_count: 4,
        unassigned_count: 0,
        ignored_count: 0,
        profile_count: 2,
        can_launch: true,
    };
    let nodes = build_setup_layout_rects(&spec);

    assert_eq!(
        resolve_spatial_move(SpatialTarget::PlayerSlot(0), SpatialDirection::Right, &nodes),
        Some(SpatialTarget::PlayerSlot(1))
    );
    assert_eq!(
        resolve_spatial_move(SpatialTarget::PlayerSlot(1), SpatialDirection::Left, &nodes),
        Some(SpatialTarget::PlayerSlot(0))
    );
    assert_eq!(
        resolve_spatial_move(SpatialTarget::PlayerSlot(0), SpatialDirection::Down, &nodes),
        Some(SpatialTarget::PlayerSlot(2))
    );
    assert_eq!(
        resolve_spatial_move(SpatialTarget::PlayerSlot(2), SpatialDirection::Up, &nodes),
        Some(SpatialTarget::PlayerSlot(0))
    );
    assert_eq!(
        resolve_spatial_move(SpatialTarget::PlayerSlot(1), SpatialDirection::Down, &nodes),
        Some(SpatialTarget::PlayerSlot(3))
    );
    assert_eq!(
        resolve_spatial_move(SpatialTarget::PlayerSlot(3), SpatialDirection::Up, &nodes),
        Some(SpatialTarget::PlayerSlot(1))
    );
    assert_eq!(
        resolve_spatial_move(SpatialTarget::PlayerSlot(2), SpatialDirection::Right, &nodes),
        Some(SpatialTarget::PlayerSlot(3))
    );
}

#[test]
fn spatial_setup_empty_tray_skipping_and_non_empty_tray_stops() {
    use nucleus_ui::big_picture::spatial::{
        SetupLayoutSpec, SpatialDirection, SpatialTarget, build_setup_layout_rects,
        resolve_spatial_move,
    };

    // Empty trays -> Down from bottom slot lands directly on LaunchButton
    let empty_trays = SetupLayoutSpec {
        has_km_row: true,
        has_raw_kb: false,
        has_raw_mouse: false,
        player_count: 4,
        unassigned_count: 0,
        ignored_count: 0,
        profile_count: 1,
        can_launch: true,
    };
    let nodes_empty = build_setup_layout_rects(&empty_trays);
    assert_eq!(
        resolve_spatial_move(SpatialTarget::PlayerSlot(2), SpatialDirection::Down, &nodes_empty),
        Some(SpatialTarget::LaunchButton)
    );
    assert_eq!(
        resolve_spatial_move(SpatialTarget::LaunchButton, SpatialDirection::Up, &nodes_empty),
        Some(SpatialTarget::PlayerSlot(2))
    );

    // Non-empty trays -> Down stops at UnassignedItem(0) -> IgnoredItem(0) -> LaunchButton
    let with_trays = SetupLayoutSpec {
        has_km_row: true,
        has_raw_kb: false,
        has_raw_mouse: false,
        player_count: 4,
        unassigned_count: 2,
        ignored_count: 1,
        profile_count: 1,
        can_launch: true,
    };
    let nodes_trays = build_setup_layout_rects(&with_trays);
    assert_eq!(
        resolve_spatial_move(SpatialTarget::PlayerSlot(2), SpatialDirection::Down, &nodes_trays),
        Some(SpatialTarget::UnassignedItem(0))
    );
    assert_eq!(
        resolve_spatial_move(SpatialTarget::UnassignedItem(0), SpatialDirection::Down, &nodes_trays),
        Some(SpatialTarget::IgnoredItem(0))
    );
    assert_eq!(
        resolve_spatial_move(SpatialTarget::IgnoredItem(0), SpatialDirection::Down, &nodes_trays),
        Some(SpatialTarget::LaunchButton)
    );
}

#[test]
fn spatial_setup_two_column_cross_jump() {
    use nucleus_ui::big_picture::spatial::{
        SetupLayoutSpec, SpatialDirection, SpatialNode, SpatialRect, SpatialTarget,
        build_setup_layout_rects, resolve_spatial_move,
    };

    let spec = SetupLayoutSpec {
        has_km_row: true,
        has_raw_kb: true,
        has_raw_mouse: true,
        player_count: 4,
        unassigned_count: 0,
        ignored_count: 0,
        profile_count: 2,
        can_launch: true,
    };
    let nodes = build_setup_layout_rects(&spec);

    // Right from rightmost top-row slot jumps across divider into right column
    let right_from_p1 = resolve_spatial_move(
        SpatialTarget::PlayerSlot(1),
        SpatialDirection::Right,
        &nodes,
    );
    assert!(
        matches!(
            right_from_p1,
            Some(SpatialTarget::ProfileSaveNew | SpatialTarget::ProfileChip(_))
        ),
        "expected Right from PlayerSlot(1) to enter profiles column, got {:?}",
        right_from_p1
    );

    // Left from right column jumps back to left column near same Y band
    let left_back = resolve_spatial_move(
        right_from_p1.unwrap(),
        SpatialDirection::Left,
        &nodes,
    );
    assert!(
        matches!(left_back, Some(SpatialTarget::PlayerSlot(1 | 3))),
        "expected Left from right column to return to rightmost PlayerSlot, got {:?}",
        left_back
    );

    // Right from bottom-right slot (PlayerSlot(3)) jumps to ProfileChip(0)
    let right_from_p3 = resolve_spatial_move(
        SpatialTarget::PlayerSlot(3),
        SpatialDirection::Right,
        &nodes,
    );
    assert_eq!(right_from_p3, Some(SpatialTarget::ProfileChip(0)));
    assert_eq!(
        resolve_spatial_move(SpatialTarget::ProfileChip(0), SpatialDirection::Left, &nodes),
        Some(SpatialTarget::PlayerSlot(3))
    );

    // Single-player with wide LaunchButton below: Right from PlayerSlot(0) still goes to ProfileSaveNew
    let custom_nodes = vec![
        SpatialNode {
            target: SpatialTarget::PlayerSlot(0),
            rect: SpatialRect::new(32.0, 340.0, 358.0, 160.0),
        },
        SpatialNode {
            target: SpatialTarget::LaunchButton,
            rect: SpatialRect::new(32.0, 680.0, 728.0, 44.0),
        },
        SpatialNode {
            target: SpatialTarget::ProfileSaveNew,
            rect: SpatialRect::new(1040.0, 520.0, 192.0, 40.0),
        },
    ];
    assert_eq!(
        resolve_spatial_move(SpatialTarget::PlayerSlot(0), SpatialDirection::Right, &custom_nodes),
        Some(SpatialTarget::ProfileSaveNew)
    );
}

#[test]
fn spatial_setup_3_player_down_from_top_right_hits_row1_before_launch() {
    use nucleus_ui::big_picture::spatial::{
        SetupLayoutSpec, SpatialDirection, SpatialTarget, build_setup_layout_rects,
        resolve_spatial_move,
    };

    let spec = SetupLayoutSpec {
        has_km_row: true,
        has_raw_kb: false,
        has_raw_mouse: false,
        player_count: 3,
        unassigned_count: 0,
        ignored_count: 0,
        profile_count: 0,
        can_launch: true,
    };
    let nodes = build_setup_layout_rects(&spec);
    assert_eq!(
        resolve_spatial_move(SpatialTarget::PlayerSlot(1), SpatialDirection::Down, &nodes),
        Some(SpatialTarget::PlayerSlot(2))
    );
}

#[test]
fn spatial_edge_clamp_returns_none() {
    use nucleus_ui::big_picture::spatial::{
        SetupLayoutSpec, SpatialDirection, SpatialTarget, build_carousel_layout_rects,
        build_hub_layout_rects, build_setup_layout_rects, resolve_spatial_move,
    };

    let setup_nodes = build_setup_layout_rects(&SetupLayoutSpec {
        has_km_row: true,
        has_raw_kb: false,
        has_raw_mouse: false,
        player_count: 2,
        unassigned_count: 0,
        ignored_count: 0,
        profile_count: 1,
        can_launch: true,
    });
    assert_eq!(
        resolve_spatial_move(SpatialTarget::LayoutPrev, SpatialDirection::Up, &setup_nodes),
        None
    );
    assert_eq!(
        resolve_spatial_move(SpatialTarget::LayoutNext, SpatialDirection::Up, &setup_nodes),
        None
    );
    assert_eq!(
        resolve_spatial_move(SpatialTarget::PlayerSlot(0), SpatialDirection::Left, &setup_nodes),
        None
    );
    assert_eq!(
        resolve_spatial_move(SpatialTarget::LaunchButton, SpatialDirection::Down, &setup_nodes),
        None
    );

    let carousel_nodes = build_carousel_layout_rects(3, 0, 3, false);
    assert_eq!(
        resolve_spatial_move(SpatialTarget::HeaderTab(0), SpatialDirection::Up, &carousel_nodes),
        None
    );

    let hub_nodes = build_hub_layout_rects(4, 6, 3);
    assert_eq!(
        resolve_spatial_move(SpatialTarget::HubSearch, SpatialDirection::Up, &hub_nodes),
        None
    );
}

#[test]
fn spatial_carousel_action_bar_cards_and_missing_exe_cta() {
    use nucleus_ui::big_picture::spatial::{
        SpatialDirection, SpatialTarget, build_carousel_layout_rects, resolve_spatial_move,
    };

    let nodes = build_carousel_layout_rects(3, 0, 3, true);

    // Up from cards reaches closest horizontal action-bar pill
    assert_eq!(
        resolve_spatial_move(SpatialTarget::CarouselCard(0), SpatialDirection::Up, &nodes),
        Some(SpatialTarget::CarouselSearch)
    );
    assert_eq!(
        resolve_spatial_move(SpatialTarget::CarouselCard(1), SpatialDirection::Up, &nodes),
        Some(SpatialTarget::CarouselFilter)
    );
    assert_eq!(
        resolve_spatial_move(SpatialTarget::CarouselCard(2), SpatialDirection::Up, &nodes),
        Some(SpatialTarget::CarouselSort)
    );

    // Down from action-bar pills returns to cards
    assert_eq!(
        resolve_spatial_move(SpatialTarget::CarouselSearch, SpatialDirection::Down, &nodes),
        Some(SpatialTarget::CarouselCard(0))
    );
    assert_eq!(
        resolve_spatial_move(SpatialTarget::CarouselFilter, SpatialDirection::Down, &nodes),
        Some(SpatialTarget::CarouselCard(1))
    );
    assert_eq!(
        resolve_spatial_move(SpatialTarget::CarouselSort, SpatialDirection::Down, &nodes),
        Some(SpatialTarget::CarouselCard(2))
    );

    // Horizontal navigation across action bar & cards
    assert_eq!(
        resolve_spatial_move(SpatialTarget::CarouselSearch, SpatialDirection::Right, &nodes),
        Some(SpatialTarget::CarouselFilter)
    );
    assert_eq!(
        resolve_spatial_move(SpatialTarget::CarouselFilter, SpatialDirection::Right, &nodes),
        Some(SpatialTarget::CarouselSort)
    );
    assert_eq!(
        resolve_spatial_move(SpatialTarget::CarouselCard(0), SpatialDirection::Right, &nodes),
        Some(SpatialTarget::CarouselCard(1))
    );

    // Down from center card reaches Missing Exe CTA, Up returns to center card
    assert_eq!(
        resolve_spatial_move(SpatialTarget::CarouselCard(1), SpatialDirection::Down, &nodes),
        Some(SpatialTarget::CarouselMissingExeCta)
    );
    assert_eq!(
        resolve_spatial_move(SpatialTarget::CarouselMissingExeCta, SpatialDirection::Up, &nodes),
        Some(SpatialTarget::CarouselCard(1))
    );
}

#[test]
fn spatial_carousel_popover_navigation() {
    use nucleus_ui::big_picture::spatial::{
        SpatialDirection, SpatialTarget, build_carousel_layout_rects, resolve_spatial_move,
    };

    let nodes = build_carousel_layout_rects(3, 4, 0, false);
    assert_eq!(
        resolve_spatial_move(SpatialTarget::CarouselFilter, SpatialDirection::Down, &nodes),
        Some(SpatialTarget::CarouselPopoverItem(0))
    );
    assert_eq!(
        resolve_spatial_move(SpatialTarget::CarouselPopoverItem(0), SpatialDirection::Down, &nodes),
        Some(SpatialTarget::CarouselPopoverItem(1))
    );
    assert_eq!(
        resolve_spatial_move(SpatialTarget::CarouselPopoverItem(1), SpatialDirection::Down, &nodes),
        Some(SpatialTarget::CarouselPopoverItem(2))
    );
    assert_eq!(
        resolve_spatial_move(SpatialTarget::CarouselPopoverItem(2), SpatialDirection::Up, &nodes),
        Some(SpatialTarget::CarouselPopoverItem(1))
    );
    assert_eq!(
        resolve_spatial_move(SpatialTarget::CarouselPopoverItem(0), SpatialDirection::Up, &nodes),
        Some(SpatialTarget::CarouselFilter)
    );
}

#[test]
fn spatial_hub_3_column_grid_navigation() {
    use nucleus_ui::big_picture::spatial::{
        SpatialDirection, SpatialTarget, build_hub_layout_rects, resolve_spatial_move,
    };

    let nodes = build_hub_layout_rects(4, 6, 3);

    // HubSearch -> Down -> HubCategory -> Down -> HubCard
    let cat = resolve_spatial_move(SpatialTarget::HubSearch, SpatialDirection::Down, &nodes)
        .expect("Down from HubSearch should hit a HubCategory");
    assert!(matches!(cat, SpatialTarget::HubCategory(_)));
    assert_eq!(
        resolve_spatial_move(cat, SpatialDirection::Up, &nodes),
        Some(SpatialTarget::HubSearch)
    );

    // Category row horizontal nav
    assert_eq!(
        resolve_spatial_move(SpatialTarget::HubCategory(0), SpatialDirection::Right, &nodes),
        Some(SpatialTarget::HubCategory(1))
    );

    // Category -> Down -> Row 0 card, Row 0 -> Down -> Row 1 card
    let card_from_cat = resolve_spatial_move(
        SpatialTarget::HubCategory(0),
        SpatialDirection::Down,
        &nodes,
    );
    assert_eq!(card_from_cat, Some(SpatialTarget::HubCard(0)));

    assert_eq!(
        resolve_spatial_move(SpatialTarget::HubCard(0), SpatialDirection::Right, &nodes),
        Some(SpatialTarget::HubCard(1))
    );
    assert_eq!(
        resolve_spatial_move(SpatialTarget::HubCard(1), SpatialDirection::Right, &nodes),
        Some(SpatialTarget::HubCard(2))
    );
    assert_eq!(
        resolve_spatial_move(SpatialTarget::HubCard(0), SpatialDirection::Down, &nodes),
        Some(SpatialTarget::HubCard(3))
    );
    assert_eq!(
        resolve_spatial_move(SpatialTarget::HubCard(1), SpatialDirection::Down, &nodes),
        Some(SpatialTarget::HubCard(4))
    );
    assert_eq!(
        resolve_spatial_move(SpatialTarget::HubCard(2), SpatialDirection::Down, &nodes),
        Some(SpatialTarget::HubCard(5))
    );
    assert_eq!(
        resolve_spatial_move(SpatialTarget::HubCard(4), SpatialDirection::Up, &nodes),
        Some(SpatialTarget::HubCard(1))
    );
    assert_eq!(
        resolve_spatial_move(SpatialTarget::HubCard(4), SpatialDirection::Left, &nodes),
        Some(SpatialTarget::HubCard(3))
    );
}

#[test]
fn spatial_nav_graph_wrapper_matches_resolve_spatial_move() {
    use nucleus_ui::big_picture::spatial::{
        SetupLayoutSpec, SpatialDirection, SpatialNavGraph, SpatialTarget, build_setup_layout_rects,
        resolve_spatial_move,
    };

    let nodes = build_setup_layout_rects(&SetupLayoutSpec {
        has_km_row: true,
        has_raw_kb: false,
        has_raw_mouse: false,
        player_count: 4,
        unassigned_count: 0,
        ignored_count: 0,
        profile_count: 1,
        can_launch: true,
    });
    let expected = resolve_spatial_move(SpatialTarget::PlayerSlot(0), SpatialDirection::Down, &nodes);
    let graph = SpatialNavGraph::new(nodes);
    assert_eq!(
        graph.find_best_neighbor(SpatialTarget::PlayerSlot(0), SpatialDirection::Down),
        expected
    );
}
