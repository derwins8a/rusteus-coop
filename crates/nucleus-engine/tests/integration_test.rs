use nucleus_common::file_util;
use nucleus_engine::GameManager;
use nucleus_input::InputManager;
use nucleus_platform::PlatformServices;
use nucleus_script::HandlerConfig;
use std::path::PathBuf;
use std::sync::Arc;

#[derive(Default)]
struct TestPlatform;

impl PlatformServices for TestPlatform {
    fn window_manager(&self) -> &dyn nucleus_platform::WindowManager {
        panic!("window_manager not needed in test")
    }
    fn process_manager(&self) -> &dyn nucleus_platform::ProcessManager {
        panic!("process_manager not needed in test")
    }
    fn display_manager(&self) -> &dyn nucleus_platform::DisplayManager {
        panic!("display_manager not needed in test")
    }
    fn input_hook_manager(&self) -> &dyn nucleus_platform::InputHookManager {
        panic!("input_hook_manager not needed in test")
    }
    fn filesystem_manager(&self) -> &dyn nucleus_platform::FilesystemManager {
        panic!("filesystem_manager not needed in test")
    }
    fn registry_manager(&self) -> Option<&dyn nucleus_platform::RegistryManager> {
        None
    }
    fn taskbar_manager(&self) -> &dyn nucleus_platform::TaskbarManager {
        panic!("taskbar_manager not needed in test")
    }
    fn mutex_manager(&self) -> &dyn nucleus_platform::MutexManager {
        panic!("mutex_manager not needed in test")
    }
    fn user_account_manager(&self) -> Option<&dyn nucleus_platform::UserAccountManager> {
        None
    }
    fn network_manager(&self) -> &dyn nucleus_platform::NetworkManager {
        panic!("network_manager not needed in test")
    }
    fn elevation_manager(&self) -> &dyn nucleus_platform::ElevationManager {
        panic!("elevation_manager not needed in test")
    }
    fn osd_manager(&self) -> &dyn nucleus_platform::OsdManager {
        panic!("osd_manager not needed in test")
    }
}

#[test]
fn test_end_to_end_game_discovery_and_profile_flow() {
    let temp_dir = std::env::temp_dir().join(format!("nucleus_e2e_{}", std::process::id()));
    let game_dir = temp_dir
        .join("SteamLibrary")
        .join("steamapps")
        .join("common")
        .join("Portal 2");
    file_util::ensure_dir(&game_dir).unwrap();

    let exe_path = game_dir.join("portal2.exe");
    file_util::write_text_file(&exe_path, "MZ dummy binary").unwrap();

    let platform = Arc::new(TestPlatform::default());
    let mut gm = GameManager::new(platform.clone()).unwrap();

    // Register test handler
    let mut config = HandlerConfig::default();
    config.guid = "portal2-test".to_string();
    config.game_name = "Portal 2".to_string();
    config.executable_name = "portal2.exe".to_string();
    config.max_players = 2;
    config.supports_keyboard = true;

    let handler_js = r#"
        Game.GUID = "portal2-test";
        Game.GameName = "Portal 2";
        Game.ExecutableName = "portal2.exe";
        Game.MaxPlayers = 2;
        Game.Play = function() {
            Context.Log("Preparing instance for: " + Context.Nickname);
        };
    "#;

    gm.register_handler(config, Some(handler_js.to_string()));

    // 1. Add game via executable detection
    let detected_guid = gm.add_game(&exe_path).unwrap();
    assert_eq!(detected_guid, Some("portal2-test".to_string()));

    // 2. Verify registered profile
    let profiles = gm.profiles();
    assert!(profiles.contains_key("portal2-test"));
    let prof = profiles.get("portal2-test").unwrap();
    assert_eq!(prof.handler_guid, "portal2-test");
    assert_eq!(PathBuf::from(&prof.executable_path), exe_path);

    // 3. Verify InputManager integration
    let input_mgr = InputManager::new(Some(platform)).unwrap();
    assert!(!input_mgr.lock().is_locked());
    input_mgr.lock().toggle();
    assert!(input_mgr.lock().is_locked());

    // Cleanup
    let _ = std::fs::remove_dir_all(&temp_dir);
}
