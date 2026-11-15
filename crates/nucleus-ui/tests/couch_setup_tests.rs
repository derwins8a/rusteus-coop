use nucleus_common::types::DeviceType;
use nucleus_input::gamepad::GamepadDevice;
use nucleus_ui::big_picture::setup::{
    AVAILABLE_LAYOUTS, CouchSetupState, PartitionLayoutKind, create_keyboard_mouse_device,
    is_keyboard_device,
};
use nucleus_ui::models::DeviceItem;

fn make_test_device(id: usize, name: &str, guid: &str, connected: bool) -> GamepadDevice {
    GamepadDevice {
        id,
        name: name.to_string(),
        guid: guid.to_string(),
        device_type: DeviceType::XInputGamepad,
        is_connected: connected,
    }
}

#[test]
fn test_auto_assignment_and_swap_motion() {
    let mut state = CouchSetupState::new();
    let ignored = Vec::new();

    // 1 device -> Fullscreen
    let devs1 = vec![make_test_device(0, "Controller 1", "GUID-1", true)];
    state.auto_assign(&devs1, &ignored);
    assert_eq!(state.slots.len(), 1);
    assert_eq!(state.selected_layout, "Fullscreen (1 Player)");
    assert_eq!(state.slots[0].player_index, 0);
    assert_eq!(state.slots[0].device_id, Some(0));
    assert_eq!(state.slots[0].device_name, "Controller 1");
    assert_eq!(state.slots[0].device_guid, "GUID-1");

    // 2 devices -> 2x1 (Side-by-Side)
    let devs2 = vec![
        make_test_device(0, "Xbox Controller 1", "GUID-1", true),
        make_test_device(1, "PlayStation Controller", "GUID-2", true),
    ];
    state.auto_assign(&devs2, &ignored);
    assert_eq!(state.slots.len(), 2);
    assert_eq!(state.selected_layout, "2x1 (Side-by-Side)");
    assert_eq!(state.slots[0].device_name, "Xbox Controller 1");
    assert_eq!(state.slots[1].device_name, "PlayStation Controller");

    // 3 devices -> Top 1 / Bottom 2
    let devs3 = vec![
        make_test_device(0, "P1 Pad", "GUID-1", true),
        make_test_device(1, "P2 Pad", "GUID-2", true),
        make_test_device(2, "P3 Pad", "GUID-3", true),
    ];
    state.auto_assign(&devs3, &ignored);
    assert_eq!(state.slots.len(), 3);
    assert_eq!(state.selected_layout, "Top 1 / Bottom 2");

    // 4 devices -> 2x2 (Grid)
    let devs4 = vec![
        make_test_device(0, "P1 Pad", "GUID-1", true),
        make_test_device(1, "P2 Pad", "GUID-2", true),
        make_test_device(2, "P3 Pad", "GUID-3", true),
        make_test_device(3, "P4 Pad", "GUID-4", true),
    ];
    state.auto_assign(&devs4, &ignored);
    assert_eq!(state.slots.len(), 4);
    assert_eq!(state.selected_layout, "2x2 (Grid)");

    // Test Grab & Swap
    state.grab_slot(0);
    assert_eq!(state.grabbed_slot, Some(0));
    assert!(state.slots[0].is_grabbed);
    assert!(!state.slots[1].is_grabbed);

    // Swap slot 0 with slot 1
    state.drop_swap_target(1);
    assert_eq!(state.grabbed_slot, None);
    assert!(!state.slots[0].is_grabbed);
    assert!(!state.slots[1].is_grabbed);

    // Slot 0 now has P2 Pad, Slot 1 has P1 Pad
    assert_eq!(state.slots[0].player_index, 0);
    assert_eq!(state.slots[0].device_name, "P2 Pad");
    assert_eq!(state.slots[0].device_guid, "GUID-2");
    assert_eq!(state.slots[1].player_index, 1);
    assert_eq!(state.slots[1].device_name, "P1 Pad");
    assert_eq!(state.slots[1].device_guid, "GUID-1");

    // Test cancel grab
    state.grab_slot(2);
    assert_eq!(state.grabbed_slot, Some(2));
    assert!(state.slots[2].is_grabbed);
    state.cancel_grab();
    assert_eq!(state.grabbed_slot, None);
    assert!(!state.slots[2].is_grabbed);
}

#[test]
fn test_remove_slot_moves_to_unassigned_tray() {
    let mut state = CouchSetupState::new();
    let devs = vec![
        make_test_device(0, "P1 Pad", "GUID-1", true),
        make_test_device(1, "P2 Pad", "GUID-2", true),
    ];
    state.auto_assign(&devs, &[]);
    state.focused_slot = 1;

    // Remove slot 0
    state.remove_slot(0);
    assert_eq!(state.slots.len(), 1);
    assert_eq!(state.unassigned_tray.len(), 1);
    assert_eq!(state.unassigned_tray[0].guid, "GUID-1");

    // Remaining slot has player_index re-indexed to 0
    assert_eq!(state.slots[0].player_index, 0);
    assert_eq!(state.slots[0].device_guid, "GUID-2");

    // Focused slot clamped
    assert_eq!(state.focused_slot, 0);
}

#[test]
fn test_reclaim_unassigned_slot() {
    let mut state = CouchSetupState::new();
    let devs = vec![
        make_test_device(0, "P1 Pad", "GUID-1", true),
        make_test_device(1, "P2 Pad", "GUID-2", true),
    ];
    state.auto_assign(&devs, &[]);
    state.remove_slot(0);
    assert_eq!(state.slots.len(), 1);
    assert_eq!(state.unassigned_tray.len(), 1);

    // Reclaim from tray index 0
    state.reclaim_unassigned(0);
    assert_eq!(state.unassigned_tray.len(), 0);
    assert_eq!(state.slots.len(), 2);
    assert_eq!(state.slots[1].player_index, 1);
    assert_eq!(state.slots[1].device_guid, "GUID-1");
}

#[test]
fn test_ignore_device_filters_and_persists() {
    let mut state = CouchSetupState::new();
    let mut ignored = vec!["IGNORED-GUID".to_string()];

    let devs = vec![
        make_test_device(0, "Valid Pad", "VALID-1", true),
        make_test_device(1, "Phantom Stick", "IGNORED-GUID", true),
        make_test_device(2, "Disconnected Pad", "DISC-1", false),
    ];

    // auto_assign should ignore IGNORED-GUID and disconnected
    state.auto_assign(&devs, &ignored);
    assert_eq!(state.slots.len(), 1);
    assert_eq!(state.slots[0].device_guid, "VALID-1");

    // Now dynamically ignore the remaining device
    state.ignore_device("VALID-1", &mut ignored);
    assert!(ignored.contains(&"VALID-1".to_string()));
    assert_eq!(state.slots.len(), 0);
}

#[test]
fn test_layout_cycling() {
    let mut state = CouchSetupState::new();
    state.selected_layout = "Fullscreen (1 Player)".to_string();

    state.cycle_layout(true);
    assert_eq!(state.selected_layout, "2x1 (Side-by-Side)");

    state.cycle_layout(true);
    assert_eq!(state.selected_layout, "1x2 (Stacked)");

    state.cycle_layout(true);
    assert_eq!(state.selected_layout, "2x2 (Grid)");

    state.cycle_layout(true);
    assert_eq!(state.selected_layout, "Top 1 / Bottom 2");

    state.cycle_layout(true);
    assert_eq!(state.selected_layout, "Top 2 / Bottom 1");

    state.cycle_layout(true);
    assert_eq!(state.selected_layout, "3x2 (Grid 6)");

    state.cycle_layout(true);
    assert_eq!(state.selected_layout, "4x2 (Grid 8)");

    state.cycle_layout(true);
    assert_eq!(state.selected_layout, "4x4 (Grid 16)");

    state.cycle_layout(true);
    assert_eq!(state.selected_layout, "Multi-Monitor");

    // Wraps back to start
    state.cycle_layout(true);
    assert_eq!(state.selected_layout, "Fullscreen (1 Player)");

    // Backwards cycle
    state.cycle_layout(false);
    assert_eq!(state.selected_layout, "Multi-Monitor");
}

#[test]
fn test_slot_navigation() {
    let mut state = CouchSetupState::new();
    let devs = vec![
        make_test_device(0, "P1", "G1", true),
        make_test_device(1, "P2", "G2", true),
        make_test_device(2, "P3", "G3", true),
    ];
    state.auto_assign(&devs, &[]);
    assert_eq!(state.focused_slot, 0);

    state.navigate_slot_right();
    assert_eq!(state.focused_slot, 1);

    state.navigate_slot_right();
    assert_eq!(state.focused_slot, 2);

    // Wrap to 0
    state.navigate_slot_right();
    assert_eq!(state.focused_slot, 0);

    // Wrap to 2
    state.navigate_slot_left();
    assert_eq!(state.focused_slot, 2);

    state.navigate_slot_left();
    assert_eq!(state.focused_slot, 1);
}

#[test]
fn test_auto_assign_from_items() {
    let mut state = CouchSetupState::new();
    let items = vec![
        DeviceItem::new(0, "Item Pad 1", "ITEM-1", true),
        DeviceItem::new(1, "Item Pad 2", "ITEM-2", true),
    ];
    state.auto_assign_from_items(&items, &[]);
    assert_eq!(state.slots.len(), 2);
    assert_eq!(state.slots[0].device_name, "Item Pad 1");
    assert_eq!(state.slots[1].device_name, "Item Pad 2");
}

#[test]
fn test_partition_layout_kind_mappings() {
    // Assert all available couch layouts map to expected partition geometry and slot counts
    assert_eq!(
        PartitionLayoutKind::from_layout_name("Fullscreen (1 Player)"),
        PartitionLayoutKind::Fullscreen
    );
    assert_eq!(PartitionLayoutKind::Fullscreen.slot_count(), 1);

    assert_eq!(
        PartitionLayoutKind::from_layout_name("2x1 (Side-by-Side)"),
        PartitionLayoutKind::SideBySide
    );
    assert_eq!(PartitionLayoutKind::SideBySide.slot_count(), 2);

    assert_eq!(
        PartitionLayoutKind::from_layout_name("1x2 (Stacked)"),
        PartitionLayoutKind::Stacked
    );
    assert_eq!(PartitionLayoutKind::Stacked.slot_count(), 2);

    assert_eq!(
        PartitionLayoutKind::from_layout_name("2x2 (Grid)"),
        PartitionLayoutKind::Grid
    );
    assert_eq!(PartitionLayoutKind::Grid.slot_count(), 4);

    assert_eq!(
        PartitionLayoutKind::from_layout_name("Top 1 / Bottom 2"),
        PartitionLayoutKind::TopOneBottomTwo
    );
    assert_eq!(PartitionLayoutKind::TopOneBottomTwo.slot_count(), 3);

    assert_eq!(
        PartitionLayoutKind::from_layout_name("Top 2 / Bottom 1"),
        PartitionLayoutKind::TopTwoBottomOne
    );
    assert_eq!(PartitionLayoutKind::TopTwoBottomOne.slot_count(), 3);

    // Verify all entries in AVAILABLE_LAYOUTS resolve correctly
    for layout in AVAILABLE_LAYOUTS {
        let kind = PartitionLayoutKind::from_layout_name(layout);
        assert!(kind.slot_count() >= 1 && kind.slot_count() <= 16);
    }
}

#[test]
fn test_is_keyboard_device_and_icons() {
    assert!(is_keyboard_device("USB Keyboard", "GUID-KBD-1"));
    assert!(is_keyboard_device("Keyboard & Mouse", "GUID-2"));
    assert!(is_keyboard_device("Generic Controller", "usb_keyboard_device"));
    assert!(is_keyboard_device("keyboard", "guid-none"));

    assert!(!is_keyboard_device("Xbox Wireless Controller", "GUID-XB-1"));
    assert!(!is_keyboard_device("PlayStation 5 DualSense", "GUID-PS-2"));
    assert!(!is_keyboard_device("Nintendo Switch Pro Controller", "GUID-NSW-3"));
}

#[test]
fn test_partition_slots_match_layout_and_swap_reflections() {
    let mut state = CouchSetupState::new();
    let devs = vec![
        make_test_device(0, "Xbox Controller", "GUID-XB", true),
        make_test_device(1, "USB Keyboard", "GUID-KBD", true),
        make_test_device(2, "PlayStation Pad", "GUID-PS", true),
    ];
    state.auto_assign(&devs, &[]);
    assert_eq!(state.selected_layout, "Top 1 / Bottom 2");
    assert_eq!(state.partition_layout_kind(), PartitionLayoutKind::TopOneBottomTwo);

    // Partition 0 (Top full): Player 1 with Xbox Controller
    let p0 = state.partition_slot(0).expect("slot 0 must exist");
    assert_eq!(p0.player_index, 0);
    assert_eq!(p0.device_name, "Xbox Controller");
    assert!(!is_keyboard_device(&p0.device_name, &p0.device_guid));

    // Partition 1 (Bottom Left): Player 2 with USB Keyboard
    let p1 = state.partition_slot(1).expect("slot 1 must exist");
    assert_eq!(p1.player_index, 1);
    assert_eq!(p1.device_name, "USB Keyboard");
    assert!(is_keyboard_device(&p1.device_name, &p1.device_guid));

    // Partition 2 (Bottom Right): Player 3 with PlayStation Pad
    let p2 = state.partition_slot(2).expect("slot 2 must exist");
    assert_eq!(p2.player_index, 2);
    assert_eq!(p2.device_name, "PlayStation Pad");
    assert!(!is_keyboard_device(&p2.device_name, &p2.device_guid));

    // Perform slot swap: Grab slot 0 and swap with slot 1
    state.grab_slot(0);
    state.drop_swap_target(1);

    // Verify swap reflects in partition slots:
    // Partition 0 (Top full) now displays USB Keyboard assigned to Player 1
    let p0_swapped = state.partition_slot(0).expect("slot 0 must exist");
    assert_eq!(p0_swapped.player_index, 0);
    assert_eq!(p0_swapped.device_name, "USB Keyboard");
    assert!(is_keyboard_device(&p0_swapped.device_name, &p0_swapped.device_guid));

    // Partition 1 (Bottom Left) now displays Xbox Controller assigned to Player 2
    let p1_swapped = state.partition_slot(1).expect("slot 1 must exist");
    assert_eq!(p1_swapped.player_index, 1);
    assert_eq!(p1_swapped.device_name, "Xbox Controller");
    assert!(!is_keyboard_device(&p1_swapped.device_name, &p1_swapped.device_guid));

    // Partition 2 remains unchanged
    let p2_swapped = state.partition_slot(2).expect("slot 2 must exist");
    assert_eq!(p2_swapped.player_index, 2);
    assert_eq!(p2_swapped.device_name, "PlayStation Pad");
}

#[test]
fn test_partition_unassigned_slots_representation() {
    let mut state = CouchSetupState::new();
    let devs = vec![
        make_test_device(0, "P1 Pad", "GUID-1", true),
        make_test_device(1, "P2 Pad", "GUID-2", true),
    ];
    state.auto_assign(&devs, &[]);
    // Manually set layout to 2x2 Grid (which requires 4 partitions)
    state.selected_layout = "2x2 (Grid)".to_string();
    assert_eq!(state.partition_layout_kind(), PartitionLayoutKind::Grid);
    assert_eq!(state.partition_layout_kind().slot_count(), 4);

    // Slot 0 and 1 are assigned
    assert!(state.partition_slot(0).is_some());
    assert_eq!(state.partition_slot(0).unwrap().device_name, "P1 Pad");
    assert!(state.partition_slot(1).is_some());
    assert_eq!(state.partition_slot(1).unwrap().device_name, "P2 Pad");

    // Slots 2 and 3 are unassigned placeholders in the 2x2 grid
    assert!(state.partition_slot(2).is_none());
    assert!(state.partition_slot(3).is_none());
}

#[test]
fn test_restore_device_removes_from_ignored_and_restores_to_slots() {
    let mut state = CouchSetupState::new();
    let mut ignored = Vec::new();

    let dev1 = make_test_device(0, "Xbox Controller", "GUID-XBOX", true);
    let dev2 = make_test_device(1, "DualSense Controller", "GUID-PS5", true);

    state.auto_assign(&[dev1.clone(), dev2.clone()], &ignored);
    assert_eq!(state.slots.len(), 2);

    // 1. Ignore device 2
    state.ignore_device("GUID-PS5", &mut ignored);
    assert_eq!(state.slots.len(), 1);
    assert_eq!(state.slots[0].device_guid, "GUID-XBOX");
    assert!(ignored.contains(&"GUID-PS5".to_string()));

    // 2. Restore device 2
    state.restore_device(&dev2, &mut ignored);
    assert!(!ignored.contains(&"GUID-PS5".to_string()));
    assert_eq!(state.slots.len(), 2);
    assert_eq!(state.slots[1].device_guid, "GUID-PS5");
    assert_eq!(state.slots[1].player_index, 1);
    assert_eq!(state.slots[1].device_name, "DualSense Controller");
    assert!(state.slots[1].is_connected);

    // 3. Re-restoring an already restored slot does not duplicate it
    state.restore_device(&dev2, &mut ignored);
    assert_eq!(state.slots.len(), 2);
}

#[test]
fn test_restore_device_clears_from_unassigned_tray_if_present() {
    let mut state = CouchSetupState::new();
    let mut ignored = vec!["GUID-TRAY".to_string()];

    let dev = make_test_device(0, "Tray Controller", "GUID-TRAY", true);
    state.unassigned_tray.push(dev.clone());
    assert_eq!(state.unassigned_tray.len(), 1);

    state.restore_device(&dev, &mut ignored);
    assert_eq!(state.unassigned_tray.len(), 0);
    assert_eq!(state.slots.len(), 1);
    assert_eq!(state.slots[0].device_guid, "GUID-TRAY");
    assert!(!ignored.contains(&"GUID-TRAY".to_string()));
}

#[test]
fn test_restore_device_from_item() {
    let mut state = CouchSetupState::new();
    let mut ignored = vec!["GUID-ITEM".to_string()];
    let item = DeviceItem::new(2, "Switch Pro Controller", "GUID-ITEM", true);

    state.restore_device_from_item(&item, &mut ignored);
    assert!(!ignored.contains(&"GUID-ITEM".to_string()));
    assert_eq!(state.slots.len(), 1);
    assert_eq!(state.slots[0].device_id, Some(2));
    assert_eq!(state.slots[0].device_name, "Switch Pro Controller");
    assert_eq!(state.slots[0].device_guid, "GUID-ITEM");
    assert!(state.slots[0].is_connected);
}

#[test]
fn test_create_keyboard_mouse_device_properties() {
    let dev = create_keyboard_mouse_device();
    assert_eq!(dev.id, 9999);
    assert_eq!(dev.name, "Keyboard & Mouse");
    assert_eq!(dev.guid, "keyboard-mouse-0");
    assert_eq!(dev.device_type, DeviceType::Keyboard);
    assert!(dev.is_connected);
    assert!(is_keyboard_device(&dev.name, &dev.guid));
}

#[test]
fn test_auto_assign_with_keyboard_options() {
    let mut state = CouchSetupState::new();
    let ignored = Vec::new();

    // 0 gamepads + supports_keyboard = false -> 0 slots
    state.auto_assign_with_options(&[], &ignored, false);
    assert_eq!(state.slots.len(), 0);
    assert_eq!(state.selected_layout, "Fullscreen (1 Player)");

    // 0 gamepads + supports_keyboard = true -> 1 slot (Keyboard & Mouse)
    state.auto_assign_with_options(&[], &ignored, true);
    assert_eq!(state.slots.len(), 1);
    assert_eq!(state.slots[0].player_index, 0);
    assert_eq!(state.slots[0].device_id, Some(9999));
    assert_eq!(state.slots[0].device_name, "Keyboard & Mouse");
    assert_eq!(state.slots[0].device_guid, "keyboard-mouse-0");
    assert!(is_keyboard_device(&state.slots[0].device_name, &state.slots[0].device_guid));
    assert_eq!(state.selected_layout, "Fullscreen (1 Player)");

    // 1 gamepad + supports_keyboard = true -> 2 slots: Gamepad + Keyboard & Mouse
    let devs1 = vec![make_test_device(0, "Xbox Controller", "GUID-XB1", true)];
    state.auto_assign_with_options(&devs1, &ignored, true);
    assert_eq!(state.slots.len(), 2);
    assert_eq!(state.slots[0].device_name, "Xbox Controller");
    assert_eq!(state.slots[1].device_name, "Keyboard & Mouse");
    assert_eq!(state.selected_layout, "2x1 (Side-by-Side)");

    // 2 gamepads + supports_keyboard = true -> 3 slots: 2 Gamepads + Keyboard & Mouse
    let devs2 = vec![
        make_test_device(0, "Xbox Controller 1", "GUID-XB1", true),
        make_test_device(1, "PlayStation Pad", "GUID-PS1", true),
    ];
    state.auto_assign_with_options(&devs2, &ignored, true);
    assert_eq!(state.slots.len(), 3);
    assert_eq!(state.slots[0].device_name, "Xbox Controller 1");
    assert_eq!(state.slots[1].device_name, "PlayStation Pad");
    assert_eq!(state.slots[2].device_name, "Keyboard & Mouse");
    assert_eq!(state.selected_layout, "Top 1 / Bottom 2");

    // Ignored keyboard: if "keyboard-mouse-0" is in ignored_guids, should not be included
    let ignored_kbm = vec!["keyboard-mouse-0".to_string()];
    state.auto_assign_with_options(&devs1, &ignored_kbm, true);
    assert_eq!(state.slots.len(), 1);
    assert_eq!(state.slots[0].device_name, "Xbox Controller");

    // auto_assign_from_items_with_options
    let items = vec![DeviceItem::new(10, "USB Gamepad", "GUID-USB", true)];
    state.auto_assign_from_items_with_options(&items, &[], true);
    assert_eq!(state.slots.len(), 2);
    assert_eq!(state.slots[0].device_name, "USB Gamepad");
    assert_eq!(state.slots[1].device_name, "Keyboard & Mouse");
}

#[test]
fn test_keyboard_mouse_swap_remove_ignore_restore() {
    let mut state = CouchSetupState::new();
    let mut ignored = Vec::new();
    let devs = vec![make_test_device(0, "Xbox Pad", "GUID-XB", true)];

    // 1. Auto-assign with keyboard enabled
    state.auto_assign_with_options(&devs, &ignored, true);
    assert_eq!(state.slots.len(), 2);
    assert_eq!(state.slots[0].device_name, "Xbox Pad");
    assert_eq!(state.slots[1].device_name, "Keyboard & Mouse");

    // 2. Swap slots: Grab slot 1 (Keyboard & Mouse) and drop on slot 0 (Xbox Pad)
    state.grab_slot(1);
    assert_eq!(state.grabbed_slot, Some(1));
    state.drop_swap_target(0);
    assert_eq!(state.slots[0].device_name, "Keyboard & Mouse");
    assert_eq!(state.slots[0].device_guid, "keyboard-mouse-0");
    assert_eq!(state.slots[1].device_name, "Xbox Pad");
    assert_eq!(state.slots[1].device_guid, "GUID-XB");

    // 3. Remove slot 0 (Keyboard & Mouse)
    state.remove_slot(0);
    assert_eq!(state.slots.len(), 1);
    assert_eq!(state.slots[0].device_name, "Xbox Pad");
    assert_eq!(state.unassigned_tray.len(), 1);
    assert_eq!(state.unassigned_tray[0].name, "Keyboard & Mouse");
    assert_eq!(state.unassigned_tray[0].guid, "keyboard-mouse-0");
    assert_eq!(state.unassigned_tray[0].device_type, DeviceType::Keyboard);

    // 4. Reclaim from unassigned tray
    state.reclaim_unassigned(0);
    assert_eq!(state.unassigned_tray.len(), 0);
    assert_eq!(state.slots.len(), 2);
    assert_eq!(state.slots[1].device_name, "Keyboard & Mouse");

    // 5. Ignore Keyboard & Mouse
    state.ignore_device("keyboard-mouse-0", &mut ignored);
    assert!(ignored.contains(&"keyboard-mouse-0".to_string()));
    assert_eq!(state.slots.len(), 1);
    assert_eq!(state.slots[0].device_name, "Xbox Pad");

    // 6. Restore Keyboard & Mouse
    let kbm_dev = create_keyboard_mouse_device();
    state.restore_device(&kbm_dev, &mut ignored);
    assert!(!ignored.contains(&"keyboard-mouse-0".to_string()));
    assert_eq!(state.slots.len(), 2);
    assert_eq!(state.slots[1].device_name, "Keyboard & Mouse");
    assert_eq!(state.slots[1].device_guid, "keyboard-mouse-0");
}

#[test]
fn test_raw_kbm_merge_and_split() {
    let mut state = CouchSetupState::new();

    // 1. Add individual keyboard and individual mouse as separate slots
    state.add_raw_keyboard(0);
    assert_eq!(state.slots.len(), 1);
    assert!(state.slots[0].has_raw_keyboard());
    assert!(!state.slots[0].has_raw_mouse());

    // Deselect focused slot to create a second slot
    state.focused_slot = 999;
    state.add_raw_mouse(0);
    assert_eq!(state.slots.len(), 2);
    assert!(state.slots[1].has_raw_mouse());
    assert!(!state.slots[1].has_raw_keyboard());

    // 2. Grab slot 1 (Mouse) and drop on slot 0 (Keyboard) -> Merge!
    state.grab_slot(1);
    state.drop_swap_target(0);

    // Verify slots merged into 1 composite slot
    assert_eq!(state.slots.len(), 1);
    assert!(state.slots[0].is_complete_raw_kbm());
    assert_eq!(state.slots[0].device_name, "Keyboard #1 + Mouse #1");

    // 3. Split composite slot back into 2 separate slots
    state.split_raw_kbm(0);
    assert_eq!(state.slots.len(), 2);
    assert!(state.slots[0].has_raw_keyboard());
    assert!(!state.slots[0].has_raw_mouse());
    assert!(state.slots[1].has_raw_mouse());
    assert!(!state.slots[1].has_raw_keyboard());
}

#[test]
fn test_extended_grid_layouts_and_partition_previews() {
    let mut state = CouchSetupState::new();
    let ignored = Vec::new();

    // 6 devices -> 3x2 (Grid 6)
    let devs6: Vec<_> = (0..6)
        .map(|i| make_test_device(i, &format!("Pad {}", i), &format!("GUID-{}", i), true))
        .collect();
    state.auto_assign(&devs6, &ignored);
    assert_eq!(state.slots.len(), 6);
    assert_eq!(state.selected_layout, "3x2 (Grid 6)");
    assert_eq!(state.partition_layout_kind(), PartitionLayoutKind::Grid6);
    assert_eq!(state.partition_layout_kind().slot_count(), 6);

    // 8 devices -> 4x2 (Grid 8)
    let devs8: Vec<_> = (0..8)
        .map(|i| make_test_device(i, &format!("Pad {}", i), &format!("GUID-{}", i), true))
        .collect();
    state.auto_assign(&devs8, &ignored);
    assert_eq!(state.slots.len(), 8);
    assert_eq!(state.selected_layout, "4x2 (Grid 8)");
    assert_eq!(state.partition_layout_kind(), PartitionLayoutKind::Grid8);
    assert_eq!(state.partition_layout_kind().slot_count(), 8);

    // 16 devices -> 4x4 (Grid 16)
    let devs16: Vec<_> = (0..16)
        .map(|i| make_test_device(i, &format!("Pad {}", i), &format!("GUID-{}", i), true))
        .collect();
    state.auto_assign(&devs16, &ignored);
    assert_eq!(state.slots.len(), 16);
    assert_eq!(state.selected_layout, "4x4 (Grid 16)");
    assert_eq!(state.partition_layout_kind(), PartitionLayoutKind::Grid16);
    assert_eq!(state.partition_layout_kind().slot_count(), 16);

    // Test PartitionLayoutKind resolution
    assert_eq!(PartitionLayoutKind::from_layout_name("Multi-Monitor"), PartitionLayoutKind::MultiMonitor);
    assert_eq!(PartitionLayoutKind::from_layout_name("Multi-Monitor").slot_count(), 2);
}

#[test]
fn test_spatial_nav_geometric_distance() {
    // 2D distance calculation in direction cone
    let p_current: (f32, f32) = (100.0, 100.0);
    let p_down: (f32, f32) = (100.0, 200.0);
    let p_right: (f32, f32) = (200.0, 100.0);

    let dy_down = p_down.1 - p_current.1;
    let dx_down = (p_down.0 - p_current.0).abs();
    assert!(dy_down > 0.0 && dx_down < dy_down); // In downward cone

    let dx_right = p_right.0 - p_current.0;
    let dy_right = (p_right.1 - p_current.1).abs();
    assert!(dx_right > 0.0 && dy_right < dx_right); // In rightward cone

    use nucleus_ui::big_picture::spatial::{
        SpatialDirection, SpatialRect, calculate_cone_distance, summoner_glow_color,
        weighted_cone_distance,
    };

    let origin = SpatialRect::new(100.0, 100.0, 40.0, 40.0);
    let down_cardinal = SpatialRect::new(100.0, 200.0, 40.0, 40.0);
    let down_diagonal = SpatialRect::new(200.0, 200.0, 40.0, 40.0);
    let behind = SpatialRect::new(100.0, 50.0, 40.0, 40.0);

    // Cardinal neighbor directly down is inside cone
    let (in_cone, dist_cardinal) = calculate_cone_distance(origin, down_cardinal, SpatialDirection::Down);
    assert!(in_cone);
    assert!((dist_cardinal - 100.0).abs() < 1e-3);

    // Diagonal neighbor is also in forward half-plane but penalized by angle
    let (in_cone_diag, dist_diag) = calculate_cone_distance(origin, down_diagonal, SpatialDirection::Down);
    assert!(in_cone_diag);
    assert!(dist_diag > dist_cardinal * 2.0);

    // Candidate behind origin is excluded
    let (in_cone_behind, _) = calculate_cone_distance(origin, behind, SpatialDirection::Down);
    assert!(!in_cone_behind);
    assert_eq!(weighted_cone_distance(origin, behind, SpatialDirection::Down), f32::MAX);

    // Summoner colors: P1 emerald, P2 blue, P3 amber, P4 rose
    let p1_glow = summoner_glow_color(0);
    let p2_glow = summoner_glow_color(1);
    let p3_glow = summoner_glow_color(2);
    let p4_glow = summoner_glow_color(3);

    assert_ne!(p1_glow, p2_glow);
    assert_ne!(p2_glow, p3_glow);
    assert_ne!(p3_glow, p4_glow);
    assert_eq!(summoner_glow_color(4), p1_glow); // wrap-around
}

#[test]
fn test_setup_stepper_state_machine() {
    #[derive(Debug, PartialEq, Eq, Clone, Copy)]
    enum Step {
        Players,
        ScreenLayout,
        GameOptions,
        Launch,
    }

    let current = Step::Players;
    let next = match current {
        Step::Players => Step::ScreenLayout,
        Step::ScreenLayout => Step::GameOptions,
        Step::GameOptions => Step::Launch,
        Step::Launch => Step::Launch,
    };
    assert_eq!(next, Step::ScreenLayout);
}

#[test]
fn test_couch_setup_state_step_transitions() {
    use nucleus_ui::big_picture::CouchSetupStep;

    let mut state = CouchSetupState::new();
    assert_eq!(state.current_step, CouchSetupStep::Players);
    assert!(!state.is_step_completed(CouchSetupStep::Players));

    state.next_step();
    assert_eq!(state.current_step, CouchSetupStep::ScreenLayout);

    state.next_step();
    assert_eq!(state.current_step, CouchSetupStep::GameOptions);

    state.next_step();
    assert_eq!(state.current_step, CouchSetupStep::Launch);

    // Capped at Launch
    state.next_step();
    assert_eq!(state.current_step, CouchSetupStep::Launch);

    state.prev_step();
    assert_eq!(state.current_step, CouchSetupStep::GameOptions);

    state.set_step(CouchSetupStep::Players);
    assert_eq!(state.current_step, CouchSetupStep::Players);
}
