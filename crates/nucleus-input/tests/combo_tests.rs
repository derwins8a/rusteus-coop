use std::time::{Duration, Instant};
use nucleus_input::combo::{ComboDetector, HoldComboConfig};
use nucleus_input::gamepad::GamepadDevice;
use nucleus_common::types::DeviceType;

#[test]
fn test_combo_requires_full_hold_duration() {
    let config = HoldComboConfig {
        hold_duration: Duration::from_millis(1500),
    };
    let mut detector = ComboDetector::new(config);
    let start = Instant::now();

    // Tap Back + Start for 500ms -> should not trigger
    detector.update_buttons(true, true, start);
    assert!(!detector.is_triggered(start + Duration::from_millis(500)));

    // Releasing resets the timer
    detector.update_buttons(false, true, start + Duration::from_millis(600));
    assert!(!detector.is_triggered(start + Duration::from_millis(600)));

    // Full 1500ms continuous hold -> triggers
    let hold_start = start + Duration::from_millis(1000);
    detector.update_buttons(true, true, hold_start);
    assert!(detector.is_triggered(hold_start + Duration::from_millis(1500)));
}

#[test]
fn test_device_fingerprint_matches_across_reconnect() {
    let dev1 = GamepadDevice {
        id: 0,
        name: "Xbox Wireless Controller".to_string(),
        guid: "030000005e040000120b000000000000".to_string(),
        device_type: DeviceType::XInputGamepad,
        is_connected: true,
    };
    let dev2 = GamepadDevice {
        id: 2, // different ID assigned by OS on replug
        name: "Xbox Wireless Controller".to_string(),
        guid: "030000005e040000120b000000000000".to_string(),
        device_type: DeviceType::XInputGamepad,
        is_connected: true,
    };
    assert_eq!(dev1.fingerprint(), dev2.fingerprint());
}

#[test]
fn test_disconnect_clears_hold_state() {
    let mut detector = ComboDetector::new(HoldComboConfig {
        hold_duration: Duration::from_millis(1500),
    });
    let start = Instant::now();

    // Start holding Back + Start
    detector.update_buttons(true, true, start);
    assert!(detector.is_holding());

    // Explicit reset clears hold state
    detector.reset();
    assert!(!detector.is_holding());
    assert!(!detector.is_triggered(start + Duration::from_millis(1500)));

    // Re-engage combo hold
    let hold_start = start + Duration::from_millis(2000);
    detector.update_buttons(true, true, hold_start);
    assert!(detector.is_holding());

    // Disconnect event arrives via handle_gilrs_event
    let event = gilrs::Event::new(
        unsafe { std::mem::transmute::<usize, gilrs::GamepadId>(0) },
        gilrs::EventType::Disconnected,
    );
    detector.handle_gilrs_event(&event, hold_start + Duration::from_millis(500));

    // Must be completely reset and not trigger even after 1500ms
    assert!(!detector.is_holding());
    assert!(!detector.is_triggered(hold_start + Duration::from_millis(2000)));
}

#[test]
fn test_couch_osd_hotkey_registration_and_simulation() {
    use nucleus_input::hotkeys::{HotkeyListener, HOTKEY_ID_COUCH_OSD};
    use nucleus_profile::AppSettings;

    // 1. Default settings with "F10"
    let mut listener = HotkeyListener::new();
    let mut settings = AppSettings::default();
    assert_eq!(settings.hotkey_couch_osd, "F10");
    listener
        .register_from_settings(&settings)
        .expect("Registration with default F10 should succeed");
    assert!(listener.simulate_trigger(HOTKEY_ID_COUCH_OSD));

    // 2. Custom hotkey with "Ctrl+Shift+O"
    let mut listener2 = HotkeyListener::new();
    settings.hotkey_couch_osd = "Ctrl+Shift+O".to_string();
    listener2
        .register_from_settings(&settings)
        .expect("Registration with Ctrl+Shift+O should succeed");
    assert!(listener2.simulate_trigger(HOTKEY_ID_COUCH_OSD));

    // 3. Fallback when invalid key string is provided (falls back to F10 / 121, modifier 0)
    let mut listener3 = HotkeyListener::new();
    settings.hotkey_couch_osd = "InvalidKeyCombination???".to_string();
    listener3
        .register_from_settings(&settings)
        .expect("Registration fallback should succeed");
    assert!(listener3.simulate_trigger(HOTKEY_ID_COUCH_OSD));
}
