use nucleus_common::types::DeviceType;
use nucleus_input::gamepad::{DeviceFingerprint, GamepadDevice};
use nucleus_ui::big_picture::setup::{CouchSetupState, DisconnectStatus};
use std::time::Duration;

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
fn test_disconnect_holds_slot_and_expires_cleanly() {
    let mut state = CouchSetupState::new();
    let devs = vec![
        make_test_device(0, "Xbox Controller", "GUID-P1", true),
        make_test_device(1, "PlayStation Controller", "GUID-P2", true),
    ];
    state.auto_assign(&devs, &[]);
    assert_eq!(state.slots.len(), 2);
    assert_eq!(state.slots[1].status, DisconnectStatus::Connected);
    assert!(state.slots[1].is_connected);

    // Disconnect Player 2
    state.handle_device_disconnect_by_id(1);

    // Verify slot is NOT removed and transitions to ReplugPending with 10s
    assert_eq!(state.slots.len(), 2);
    assert!(!state.slots[1].is_connected);
    assert_eq!(
        state.slots[1].status,
        DisconnectStatus::ReplugPending {
            remaining_ms: 10_000,
            total_ms: 10_000,
        }
    );

    // Verify active disconnect banner is returned
    let banner = state.active_disconnect_banner();
    assert!(banner.is_some());
    let (idx, slot, rem, tot) = banner.unwrap();
    assert_eq!(idx, 1);
    assert_eq!(slot.device_guid, "GUID-P2");
    assert_eq!(rem, 10_000);
    assert_eq!(tot, 10_000);

    // Advance time by 10 seconds -> Countdown expires
    state.tick_disconnect(Duration::from_secs(10));

    // Verify slot remains preserved
    assert_eq!(state.slots.len(), 2);
    assert!(!state.slots[1].is_connected);
    assert_eq!(state.slots[1].status, DisconnectStatus::DisconnectedSilent);

    // Verify banner is dismissed
    assert!(state.active_disconnect_banner().is_none());
}

#[test]
fn test_reconnect_during_countdown_restores_connected() {
    let mut state = CouchSetupState::new();
    let devs = vec![make_test_device(0, "Xbox Controller", "GUID-P1", true)];
    state.auto_assign(&devs, &[]);

    // Disconnect
    state.handle_device_disconnect(0);
    assert_eq!(
        state.slots[0].status,
        DisconnectStatus::ReplugPending {
            remaining_ms: 10_000,
            total_ms: 10_000,
        }
    );

    // 3 seconds pass
    state.tick_disconnect(Duration::from_secs(3));
    if let DisconnectStatus::ReplugPending { remaining_ms, .. } = state.slots[0].status {
        assert_eq!(remaining_ms, 7_000);
    } else {
        panic!("Expected ReplugPending with 7000ms remaining");
    }

    // Player replugs (possibly assigned a new OS dev_id e.g. 5)
    let reconnected = state.handle_device_reconnect(5, "GUID-P1", "Xbox Controller", None);
    assert!(reconnected);

    assert_eq!(state.slots[0].device_id, Some(5));
    assert!(state.slots[0].is_connected);
    assert_eq!(state.slots[0].status, DisconnectStatus::Connected);
    assert!(state.active_disconnect_banner().is_none());
}

#[test]
fn test_reconnect_after_expiry_restores_connected() {
    let mut state = CouchSetupState::new();
    let devs = vec![make_test_device(0, "Xbox Controller", "GUID-P1", true)];
    state.auto_assign(&devs, &[]);

    state.handle_device_disconnect_by_id(0);

    // 12 seconds pass (expires 10s countdown)
    state.tick_disconnect(Duration::from_secs(12));
    assert_eq!(state.slots[0].status, DisconnectStatus::DisconnectedSilent);

    // Reconnect after expiry using fingerprint
    let fp = DeviceFingerprint::new("GUID-P1", "Xbox Controller", &DeviceType::XInputGamepad);
    let reconnected =
        state.handle_device_reconnect(10, "GUID-P1-DIFFERENT", "Xbox Controller", Some(&fp));
    assert!(reconnected);

    assert_eq!(state.slots[0].device_id, Some(10));
    assert!(state.slots[0].is_connected);
    assert_eq!(state.slots[0].status, DisconnectStatus::Connected);
    assert!(state.active_disconnect_banner().is_none());
}

#[test]
fn test_tick_disconnect_decrements_progressively() {
    let mut state = CouchSetupState::new();
    let devs = vec![make_test_device(0, "Pad", "GUID-1", true)];
    state.auto_assign(&devs, &[]);

    state.handle_device_disconnect(0);

    // 1s
    state.tick_disconnect(Duration::from_millis(1000));
    assert_eq!(
        state.slots[0].status,
        DisconnectStatus::ReplugPending {
            remaining_ms: 9000,
            total_ms: 10_000,
        }
    );

    // 2.5s more
    state.tick_disconnect(Duration::from_millis(2500));
    assert_eq!(
        state.slots[0].status,
        DisconnectStatus::ReplugPending {
            remaining_ms: 6500,
            total_ms: 10_000,
        }
    );

    // 6.5s more (hits 0)
    state.tick_disconnect(Duration::from_millis(6500));
    assert_eq!(state.slots[0].status, DisconnectStatus::DisconnectedSilent);
}

#[test]
fn test_main_window_sync_devices_disconnect_and_reconnect() {
    use nucleus_ui::models::DeviceItem;
    use nucleus_ui::window::MainWindow;

    let mut window = MainWindow::new(0, vec![]);
    let dev1 = DeviceItem::new(1, "Xbox Controller", "GUID-X1", true);
    let dev2 = DeviceItem::new(2, "PS5 Controller", "GUID-P2", true);

    // Initially 2 devices connected
    window.sync_devices(vec![dev1.clone(), dev2.clone()]);
    window
        .couch_setup_state
        .auto_assign_from_items(&window.detected_devices_list, &[]);
    assert_eq!(window.couch_setup_state.slots.len(), 2);
    assert_eq!(
        window.couch_setup_state.slots[0].status,
        DisconnectStatus::Connected
    );
    assert_eq!(
        window.couch_setup_state.slots[1].status,
        DisconnectStatus::Connected
    );

    // dev2 disconnects (only dev1 present in new device list)
    window.sync_devices(vec![dev1.clone()]);
    assert_eq!(window.couch_setup_state.slots.len(), 2);
    assert_eq!(
        window.couch_setup_state.slots[0].status,
        DisconnectStatus::Connected
    );
    assert_eq!(
        window.couch_setup_state.slots[1].status,
        DisconnectStatus::ReplugPending {
            remaining_ms: 10_000,
            total_ms: 10_000,
        }
    );

    // Tick 3 seconds
    window.tick_disconnect_resilience_state(Duration::from_secs(3));
    if let DisconnectStatus::ReplugPending { remaining_ms, .. } =
        window.couch_setup_state.slots[1].status
    {
        assert_eq!(remaining_ms, 7_000);
    } else {
        panic!("Expected ReplugPending with 7000ms remaining");
    }

    // dev2 reconnected
    window.sync_devices(vec![dev1.clone(), dev2.clone()]);
    assert_eq!(
        window.couch_setup_state.slots[1].status,
        DisconnectStatus::Connected
    );
    assert!(
        window
            .couch_setup_state
            .active_disconnect_banner()
            .is_none()
    );
}
