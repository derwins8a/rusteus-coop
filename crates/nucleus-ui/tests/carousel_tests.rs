use gpui_kit_assets::IconName;
use nucleus_ui::big_picture::{
    BigPictureScreen, CarouselFilterMode, CarouselSortMode, CarouselState, CouchHubState,
    MISSING_EXE_METADATA_CALLOUT, UNINSTALLED_BADGE_TEXT, carousel_card_opacity,
};
use nucleus_ui::models::HandlerItem;

#[test]
fn test_carousel_selection_wraps_cleanly() {
    let mut state = CarouselState::new(3);
    assert_eq!(state.selected(), 0);

    // Left wrap: 0 -> 2
    state.move_left();
    assert_eq!(state.selected(), 2);

    state.move_left();
    assert_eq!(state.selected(), 1);

    state.move_left();
    assert_eq!(state.selected(), 0);

    // Right wrap: 0 -> 1 -> 2 -> 0
    state.move_right();
    assert_eq!(state.selected(), 1);

    state.move_right();
    assert_eq!(state.selected(), 2);

    state.move_right();
    assert_eq!(state.selected(), 0);
}

#[test]
fn test_carousel_empty_count_no_panic() {
    let mut state = CarouselState::new(0);
    assert_eq!(state.selected(), 0);
    assert_eq!(state.items_count, 0);

    // Should not panic on operations with 0 items
    state.move_left();
    assert_eq!(state.selected(), 0);

    state.move_right();
    assert_eq!(state.selected(), 0);

    state.select(10);
    assert_eq!(state.selected(), 0);

    state.set_count(0);
    assert_eq!(state.selected(), 0);
}

#[test]
fn test_carousel_select_bounds_clamping() {
    let mut state = CarouselState::new(5);
    state.select(2);
    assert_eq!(state.selected(), 2);

    // Clamps to max valid index (4)
    state.select(100);
    assert_eq!(state.selected(), 4);

    state.select(0);
    assert_eq!(state.selected(), 0);
}

#[test]
fn test_carousel_set_count_adjusts_selection() {
    let mut state = CarouselState::new(5);
    state.select(4);
    assert_eq!(state.selected(), 4);

    // Shrink count: selection must clamp to 2
    state.set_count(3);
    assert_eq!(state.items_count, 3);
    assert_eq!(state.selected(), 2);

    // Shrink to 0: selection must clamp to 0
    state.set_count(0);
    assert_eq!(state.items_count, 0);
    assert_eq!(state.selected(), 0);

    // Grow count: selection remains 0
    state.set_count(4);
    assert_eq!(state.items_count, 4);
    assert_eq!(state.selected(), 0);
}

#[test]
fn test_big_picture_screen_default_and_variants() {
    let screen = BigPictureScreen::default();
    assert_eq!(screen, BigPictureScreen::Carousel);

    let setup = BigPictureScreen::CouchSetup;
    assert_eq!(setup, BigPictureScreen::CouchSetup);

    let hub = BigPictureScreen::CouchHub;
    assert_eq!(hub, BigPictureScreen::CouchHub);
}

#[test]
fn test_big_picture_screen_cycle_next() {
    let screen = BigPictureScreen::Carousel;
    let next = screen.next();
    assert_eq!(next, BigPictureScreen::CouchSetup);

    let next2 = next.next();
    assert_eq!(next2, BigPictureScreen::CouchHub);

    let next3 = next2.next();
    assert_eq!(next3, BigPictureScreen::Carousel);
}

#[test]
fn test_big_picture_screen_cycle_prev() {
    let screen = BigPictureScreen::Carousel;
    let prev = screen.prev();
    assert_eq!(prev, BigPictureScreen::CouchHub);

    let prev2 = prev.prev();
    assert_eq!(prev2, BigPictureScreen::CouchSetup);

    let prev3 = prev2.prev();
    assert_eq!(prev3, BigPictureScreen::Carousel);
}

#[test]
fn test_big_picture_screen_bidirectional_inverses() {
    for screen in [
        BigPictureScreen::Carousel,
        BigPictureScreen::CouchSetup,
        BigPictureScreen::CouchHub,
    ] {
        assert_eq!(screen.next().prev(), screen);
        assert_eq!(screen.prev().next(), screen);
    }
}

#[test]
fn test_carousel_filter_mode_options_and_labels() {
    assert_eq!(CarouselFilterMode::ALL.len(), 8);
    assert_eq!(CarouselFilterMode::All.label(), "All Games");
    assert_eq!(CarouselFilterMode::InstalledOnly.label(), "Ready to Launch");
    assert_eq!(CarouselFilterMode::Favorites.label(), "Favorites");
    assert_eq!(CarouselFilterMode::ControllerSupport.label(), "Controllers");
    assert_eq!(CarouselFilterMode::KeyboardOnly.label(), "Keyboard/Mouse");
    assert_eq!(CarouselFilterMode::MultiKeyboardMouse.label(), "Multi-K&M (ProtoInput)");
    assert_eq!(CarouselFilterMode::FourPlusPlayers.label(), "4+ Players");
    assert_eq!(CarouselFilterMode::TwoPlusPlayers.label(), "2+ Players");

    for mode in CarouselFilterMode::ALL {
        assert!(mode.label().is_ascii(), "Filter label must be ASCII: {}", mode.label());
    }

    assert!(matches!(CarouselFilterMode::All.icon(), IconName::LayoutGrid));
    assert!(matches!(CarouselFilterMode::InstalledOnly.icon(), IconName::Check));
    assert!(matches!(CarouselFilterMode::Favorites.icon(), IconName::Star));
    assert!(matches!(CarouselFilterMode::ControllerSupport.icon(), IconName::Gamepad2));
    assert!(matches!(CarouselFilterMode::KeyboardOnly.icon(), IconName::Keyboard));
    assert!(matches!(CarouselFilterMode::MultiKeyboardMouse.icon(), IconName::Mouse));
    assert!(matches!(CarouselFilterMode::FourPlusPlayers.icon(), IconName::Users));
    assert!(matches!(CarouselFilterMode::TwoPlusPlayers.icon(), IconName::Users));
}

#[test]
fn test_carousel_sort_mode_options_and_labels() {
    assert_eq!(CarouselSortMode::ALL.len(), 6);
    assert_eq!(CarouselSortMode::Alphabetical.label(), "Alphabetical (A-Z)");
    assert_eq!(CarouselSortMode::AlphabeticalDesc.label(), "Alphabetical (Z-A)");
    assert_eq!(CarouselSortMode::RecentlyPlayed.label(), "Recently Played");
    assert_eq!(CarouselSortMode::PlayTime.label(), "Total Play Time");
    assert_eq!(CarouselSortMode::MaxPlayers.label(), "Maximum Players");
    assert_eq!(CarouselSortMode::InstalledFirst.label(), "Installed First");

    for mode in CarouselSortMode::ALL {
        assert!(mode.label().is_ascii(), "Sort label must be ASCII: {}", mode.label());
    }

    assert!(matches!(CarouselSortMode::Alphabetical.icon(), IconName::ArrowDownAZ));
    assert!(matches!(CarouselSortMode::AlphabeticalDesc.icon(), IconName::ArrowDownAZ));
    assert!(matches!(CarouselSortMode::RecentlyPlayed.icon(), IconName::Clock));
    assert!(matches!(CarouselSortMode::PlayTime.icon(), IconName::Timer));
    assert!(matches!(CarouselSortMode::MaxPlayers.icon(), IconName::Users));
    assert!(matches!(CarouselSortMode::InstalledFirst.icon(), IconName::Check));
}

fn create_sample_handlers() -> Vec<HandlerItem> {
    let mut h1 = HandlerItem::new("l4d2", "Left 4 Dead 2", "left4dead2.exe", 4, "desc");
    h1.is_installed = true;
    h1.supports_keyboard = true;
    h1.supports_gamepad = true;
    h1.supports_multiple_keyboards_and_mice = true;
    h1.is_favorite = true;

    let mut h2 = HandlerItem::new("halo", "Halo MCC", "mcclauncher.exe", 8, "desc");
    h2.is_installed = false;
    h2.supports_keyboard = false;
    h2.supports_gamepad = true;
    h2.supports_multiple_keyboards_and_mice = false;

    let mut h3 = HandlerItem::new("portal2", "Portal 2", "portal2.exe", 2, "desc");
    h3.is_installed = true;
    h3.supports_keyboard = false;
    h3.supports_gamepad = true;
    h3.supports_multiple_keyboards_and_mice = false;

    let mut h4 = HandlerItem::new("solitaire", "Solitaire", "solitaire.exe", 1, "desc");
    h4.is_installed = false;
    h4.supports_keyboard = true;
    h4.supports_gamepad = false;
    h4.supports_xinput = false;
    h4.supports_multiple_keyboards_and_mice = true;

    vec![h1, h2, h3, h4]
}

#[test]
fn test_filter_and_sort_handlers_filtering() {
    let handlers = create_sample_handlers();
    let mut state = CarouselState::new(handlers.len());

    // 1. All
    state.filter_mode = CarouselFilterMode::All;
    let res = state.filter_and_sort_handlers(&handlers, "");
    assert_eq!(res.len(), 4);

    // 2. InstalledOnly (Left 4 Dead 2, Portal 2)
    state.filter_mode = CarouselFilterMode::InstalledOnly;
    let res = state.filter_and_sort_handlers(&handlers, "");
    assert_eq!(res.len(), 2);
    assert!(res.iter().all(|h| h.is_installed));

    // 3. Favorites (Left 4 Dead 2)
    state.filter_mode = CarouselFilterMode::Favorites;
    let res = state.filter_and_sort_handlers(&handlers, "");
    assert_eq!(res.len(), 1);
    assert_eq!(res[0].guid, "l4d2");

    // 4. ControllerSupport (Left 4 Dead 2, Halo MCC, Portal 2)
    state.filter_mode = CarouselFilterMode::ControllerSupport;
    let res = state.filter_and_sort_handlers(&handlers, "");
    assert_eq!(res.len(), 3);
    assert!(res.iter().all(|h| h.supports_gamepad));

    // 5. MultiKeyboardMouse (Left 4 Dead 2, Solitaire)
    state.filter_mode = CarouselFilterMode::MultiKeyboardMouse;
    let res = state.filter_and_sort_handlers(&handlers, "");
    assert_eq!(res.len(), 2);
    assert!(res.iter().all(|h| h.supports_multiple_keyboards_and_mice));

    // 6. TwoPlusPlayers (Halo, L4D2, Portal 2)
    state.filter_mode = CarouselFilterMode::TwoPlusPlayers;
    let res = state.filter_and_sort_handlers(&handlers, "");
    assert_eq!(res.len(), 3);
}

#[test]
fn test_filter_and_sort_handlers_sorting() {
    let handlers = create_sample_handlers();
    let mut state = CarouselState::new(handlers.len());
    state.filter_mode = CarouselFilterMode::All;

    // 1. Alphabetical (Halo MCC, Left 4 Dead 2, Portal 2, Solitaire)
    state.sort_mode = CarouselSortMode::Alphabetical;
    let res = state.filter_and_sort_handlers(&handlers, "");
    let names: Vec<&str> = res.iter().map(|h| h.game_name.as_str()).collect();
    assert_eq!(names, vec!["Halo MCC", "Left 4 Dead 2", "Portal 2", "Solitaire"]);

    // 2. Alphabetical descending (Solitaire, Portal 2, Left 4 Dead 2, Halo MCC)
    state.sort_mode = CarouselSortMode::AlphabeticalDesc;
    let res = state.filter_and_sort_handlers(&handlers, "");
    let names: Vec<&str> = res.iter().map(|h| h.game_name.as_str()).collect();
    assert_eq!(names, vec!["Solitaire", "Portal 2", "Left 4 Dead 2", "Halo MCC"]);

    // 3. MaxPlayers descending (Halo [8], Left 4 Dead 2 [4], Portal 2 [2], Solitaire [1])
    state.sort_mode = CarouselSortMode::MaxPlayers;
    let res = state.filter_and_sort_handlers(&handlers, "");
    let players: Vec<i32> = res.iter().map(|h| h.max_players).collect();
    assert_eq!(players, vec![8, 4, 2, 1]);
}

#[test]
fn test_filter_and_sort_handlers_with_search_query() {
    let handlers = create_sample_handlers();
    let mut state = CarouselState::new(handlers.len());

    // Search query matches title
    let res = state.filter_and_sort_handlers(&handlers, "portal");
    assert_eq!(res.len(), 1);
    assert_eq!(res[0].guid, "portal2");

    // Search query matches executable
    let res = state.filter_and_sort_handlers(&handlers, "mcclauncher");
    assert_eq!(res.len(), 1);
    assert_eq!(res[0].guid, "halo");

    // Search query with no match
    let res = state.filter_and_sort_handlers(&handlers, "nonexistent");
    assert_eq!(res.len(), 0);

    // Search query combined with filter
    state.filter_mode = CarouselFilterMode::InstalledOnly;
    // Halo matches query "mcc" but is not installed
    let res = state.filter_and_sort_handlers(&handlers, "mcc");
    assert_eq!(res.len(), 0);

    // Left 4 Dead 2 matches query "dead" and is installed
    let res = state.filter_and_sort_handlers(&handlers, "dead");
    assert_eq!(res.len(), 1);
    assert_eq!(res[0].guid, "l4d2");
}

#[test]
fn test_uninstalled_carousel_card_dimming_opacity() {
    // Uninstalled games are dimmed: 0.60 when focused/selected, 0.40 when unselected
    assert_eq!(carousel_card_opacity(false, true), 0.60);
    assert_eq!(carousel_card_opacity(false, false), 0.40);

    // Installed games: 1.0 (full brightness) when selected, 0.70 when unselected
    assert_eq!(carousel_card_opacity(true, true), 1.0);
    assert_eq!(carousel_card_opacity(true, false), 0.70);
}

#[test]
fn test_uninstalled_carousel_badge_and_callout_constants() {
    assert_eq!(UNINSTALLED_BADGE_TEXT, "Uninstalled (.exe missing)");
    assert_eq!(
        MISSING_EXE_METADATA_CALLOUT,
        "Executable Missing - Press (A) to Locate .exe"
    );
    assert!(UNINSTALLED_BADGE_TEXT.is_ascii());
    assert!(MISSING_EXE_METADATA_CALLOUT.is_ascii());
}

#[test]
fn test_uninstalled_handler_missing_exe_prompt_detection() {
    let handlers = create_sample_handlers();
    let uninstalled = handlers.iter().find(|h| !h.is_installed).expect("uninstalled handler");
    assert!(!uninstalled.is_installed);
    assert_eq!(uninstalled.game_name, "Halo MCC");

    let mut hub_state = CouchHubState::new();
    assert_eq!(hub_state.missing_exe_modal_handler, None);

    // Trigger prompt on uninstalled game
    hub_state.prompt_missing_exe(&uninstalled.game_name);
    assert_eq!(
        hub_state.missing_exe_modal_handler,
        Some("Halo MCC".to_string())
    );

    // Dismiss prompt
    hub_state.dismiss_prompt();
    assert_eq!(hub_state.missing_exe_modal_handler, None);
}

#[test]
fn test_uninstalled_handler_selection_routing_simulation() {
    let handlers = create_sample_handlers();
    let mut hub_state = CouchHubState::new();

    for handler in &handlers {
        if !handler.is_installed {
            // Emulates select_carousel_game routing: if !is_installed, prompts missing exe modal
            hub_state.prompt_missing_exe(&handler.game_name);
            assert_eq!(
                hub_state.missing_exe_modal_handler.as_deref(),
                Some(handler.game_name.as_str())
            );
            hub_state.dismiss_prompt();
        } else {
            // Installed handler proceeds to couch setup without triggering prompt
            assert_eq!(hub_state.missing_exe_modal_handler, None);
        }
    }
}
