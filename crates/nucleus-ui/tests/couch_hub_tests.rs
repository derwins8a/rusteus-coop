use nucleus_ui::big_picture::hub::{
    CouchHubState, CouchOptionItem, CouchOptionsModalState, PathResolution,
    resolve_game_path_or_couch_prompt,
};
use std::path::PathBuf;

#[test]
fn test_couch_hub_fallback_prompts_desktop_mode() {
    // None -> PromptDesktopMode
    let res_none = resolve_game_path_or_couch_prompt(None);
    assert_eq!(res_none, PathResolution::PromptDesktopMode);

    // Non-existent path -> PromptDesktopMode
    let fake_path = PathBuf::from("C:\\fake_nonexistent_directory\\game_12345.exe");
    let res_missing = resolve_game_path_or_couch_prompt(Some(fake_path));
    assert_eq!(res_missing, PathResolution::PromptDesktopMode);

    // Existing path -> Found
    let existing_path = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("Cargo.toml");
    assert!(existing_path.exists());
    let res_found = resolve_game_path_or_couch_prompt(Some(existing_path.clone()));
    assert_eq!(res_found, PathResolution::Found(existing_path));

    // Also verify prompt_desktop_mode sets handler name on state
    let mut hub_state = CouchHubState::new();
    assert_eq!(hub_state.missing_exe_modal_handler, None);
    hub_state.prompt_desktop_mode("Borderlands 2");
    assert_eq!(
        hub_state.missing_exe_modal_handler,
        Some("Borderlands 2".to_string())
    );
}

#[test]
fn test_couch_hub_category_cycling() {
    let mut state = CouchHubState::new();
    assert_eq!(
        state.categories,
        vec!["Popular", "Trending", "Recent", "All"]
    );
    assert_eq!(state.selected_category, 0);
    assert_eq!(state.selected_index, 0);

    // Forward category cycling
    state.next_category();
    assert_eq!(state.selected_category, 1);
    assert_eq!(state.categories[state.selected_category], "Trending");

    state.next_category();
    assert_eq!(state.selected_category, 2);
    assert_eq!(state.categories[state.selected_category], "Recent");

    state.next_category();
    assert_eq!(state.selected_category, 3);
    assert_eq!(state.categories[state.selected_category], "All");

    // Wrap forward: 3 -> 0
    state.next_category();
    assert_eq!(state.selected_category, 0);
    assert_eq!(state.categories[state.selected_category], "Popular");

    // Wrap backward: 0 -> 3
    state.prev_category();
    assert_eq!(state.selected_category, 3);
    assert_eq!(state.categories[state.selected_category], "All");

    state.prev_category();
    assert_eq!(state.selected_category, 2);

    // Selection horizontal navigation
    let count = 4;
    state.selected_index = 0;

    // Left wrap: 0 -> 3
    state.move_left(count);
    assert_eq!(state.selected_index, 3);

    state.move_left(count);
    assert_eq!(state.selected_index, 2);

    // Right wrap: 2 -> 3 -> 0
    state.move_right(count);
    assert_eq!(state.selected_index, 3);
    state.move_right(count);
    assert_eq!(state.selected_index, 0);

    // Zero count safety
    state.move_left(0);
    assert_eq!(state.selected_index, 0);
    state.move_right(0);
    assert_eq!(state.selected_index, 0);
}

#[test]
fn test_couch_options_modal_state() {
    let opt1 = CouchOptionItem {
        key: "gamemode".to_string(),
        title: "Game Mode".to_string(),
        description: "Select Campaign or Versus".to_string(),
        choices: vec![
            "Campaign".to_string(),
            "Versus".to_string(),
            "Survival".to_string(),
        ],
        selected_choice: 0,
    };

    let opt2 = CouchOptionItem {
        key: "dx_version".to_string(),
        title: "DirectX Version".to_string(),
        description: "Choose DirectX backend".to_string(),
        choices: vec!["DirectX 9".to_string(), "DirectX 11".to_string()],
        selected_choice: 1,
    };

    let mut modal = CouchOptionsModalState {
        game_title: "Left 4 Dead 2".to_string(),
        options: vec![opt1, opt2],
        focused_option_idx: 0,
    };

    assert_eq!(modal.focused_option_idx, 0);

    // Move down: 0 -> 1
    modal.move_down();
    assert_eq!(modal.focused_option_idx, 1);

    // Move down wrap: 1 -> 0
    modal.move_down();
    assert_eq!(modal.focused_option_idx, 0);

    // Move up wrap: 0 -> 1
    modal.move_up();
    assert_eq!(modal.focused_option_idx, 1);

    // Move up: 1 -> 0
    modal.move_up();
    assert_eq!(modal.focused_option_idx, 0);

    // Cycle choice on option 0 forward: 0 (Campaign) -> 1 (Versus)
    modal.cycle_choice(true);
    assert_eq!(modal.options[0].selected_choice, 1);

    modal.cycle_choice(true);
    assert_eq!(modal.options[0].selected_choice, 2);

    // Wrap forward: 2 -> 0
    modal.cycle_choice(true);
    assert_eq!(modal.options[0].selected_choice, 0);

    // Wrap backward: 0 -> 2
    modal.cycle_choice(false);
    assert_eq!(modal.options[0].selected_choice, 2);

    // Export to HashMap
    let values_map = modal.to_hashmap();
    assert_eq!(
        values_map.get("gamemode").map(|s| s.as_str()),
        Some("Survival")
    );
    assert_eq!(
        values_map.get("dx_version").map(|s| s.as_str()),
        Some("DirectX 11")
    );
}

#[test]
fn test_couch_hub_missing_exe_prompt_dismiss() {
    let mut state = CouchHubState::new();
    assert!(state.missing_exe_modal_handler.is_none());

    state.prompt_desktop_mode("Halo: Combat Evolved");
    assert_eq!(
        state.missing_exe_modal_handler.as_deref(),
        Some("Halo: Combat Evolved")
    );

    state.dismiss_prompt();
    assert!(state.missing_exe_modal_handler.is_none());
}
