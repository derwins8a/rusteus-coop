//! Integration and unit tests for the Nucleus UI custom component meta-framework.
use gpui::{ParentElement, Styled, px};
use gpui_kit_assets::IconName;
use nucleus_ui::components::*;

#[test]
fn test_couch_button_presets_and_focus() {
    let btn_primary = CouchButton::primary("btn-p", "Launch")
        .icon(IconName::Play)
        .size(CouchButtonSize::Lg)
        .focused(true);
    let _elem = btn_primary.into_any_element();

    let btn_danger = CouchButton::danger("btn-d", "Delete").disabled(true);
    let _elem2 = btn_danger.into_any_element();

    let btn_ghost = CouchButton::ghost("btn-g", "Cancel").size(CouchButtonSize::Sm);
    let _elem3 = btn_ghost.into_any_element();

    assert_eq!(CouchButtonSize::Sm.height_px(), px(28.0));
    assert_eq!(CouchButtonSize::Md.height_px(), px(36.0));
    assert_eq!(CouchButtonSize::Lg.height_px(), px(44.0));
}

#[test]
fn test_couch_tag_tones_and_player_slots() {
    let tag_neutral = CouchTag::new("Ready").with_dot();
    let _e1 = tag_neutral.into_any_element();

    let tag_success = CouchTag::new("Connected")
        .tone(TagTone::Success)
        .icon(IconName::Check);
    let _e2 = tag_success.into_any_element();

    let tag_p1 = CouchTag::player(0, "PLAYER 1");
    let _e3 = tag_p1.into_any_element();

    let tag_p4 = CouchTag::player(3, "PLAYER 4");
    let _e4 = tag_p4.into_any_element();
}

#[test]
fn test_couch_modal_widths_and_structure() {
    assert_eq!(ModalWidth::Sm.to_pixels(), (px(480.0), px(520.0)));
    assert_eq!(ModalWidth::Md.to_pixels(), (px(640.0), px(700.0)));
    assert_eq!(ModalWidth::Lg.to_pixels(), (px(820.0), px(880.0)));

    let modal = CouchModal::new("modal-test", "Handler Options")
        .subtitle("Configure game options before launching")
        .icon(IconName::Settings)
        .width(ModalWidth::Lg)
        .body(gpui::div().child("Option body text"))
        .action(CouchButton::secondary("btn-cancel", "Cancel").into_any_element())
        .action(CouchButton::primary("btn-save", "Save Changes").into_any_element());

    let _elem = modal.into_any_element();
}

#[test]
fn test_couch_option_row_columns_and_events() {
    let row = CouchOptionRow::new(
        "opt-res",
        "Screen Resolution",
        "Sets the internal rendering resolution for all splitscreen instances",
        "1920x1080",
    )
    .focused(true)
    .on_prev(move |_, _, _| {})
    .on_next(move |_, _, _| {});

    let _elem = row.into_any_element();
}

#[test]
fn test_game_cover_dimensions_and_aspect_ratio() {
    let cover_selected = GameCover::new("Left 4 Dead 2", px(240.0), px(360.0))
        .cover_url(Some("https://example.com/cover.jpg".to_string()))
        .aspect_ratio(2.0 / 3.0)
        .selected(true)
        .installed(true);
    let _e1 = cover_selected.into_any_element();

    let cover_uninstalled = GameCover::new("Halo", px(200.0), px(300.0))
        .installed(false)
        .selected(false);
    let _e2 = cover_uninstalled.into_any_element();
}

#[test]
fn test_couch_picker_options_and_selection() {
    #[derive(Debug, Clone, Copy, PartialEq, Eq)]
    enum FilterMode {
        All,
        InstalledOnly,
        Coop,
    }

    let picker = CouchPicker::new("filter-picker", "Filter", FilterMode::InstalledOnly)
        .icon(IconName::SlidersHorizontal)
        .option(FilterMode::All, "All Handlers")
        .option(FilterMode::InstalledOnly, "Installed Only")
        .option(FilterMode::Coop, "Co-op Supported")
        .open(true)
        .focused(false);

    let _elem = picker.into_any_element();
}

#[test]
fn test_couch_slot_card_assignments_and_split() {
    let empty_slot = CouchSlotCard::new("slot-0", 0).drop_target(true);
    let _e1 = empty_slot.into_any_element();

    let assigned_slot = CouchSlotCard::new("slot-1", 1)
        .device_label(Some("Xbox Wireless Controller"))
        .focused(true);
    let _e2 = assigned_slot.into_any_element();

    let merged_kbm_slot = CouchSlotCard::new("slot-2", 2)
        .device_label(Some("Keyboard #1 + Mouse #1"))
        .merged_kbm(true)
        .on_split(|_, _, _| {});
    let _e3 = merged_kbm_slot.into_any_element();
}

#[test]
fn test_partition_preview_layouts() {
    let p_single = PartitionPreview::new(SplitLayout::Single);
    let _e1 = p_single.into_any_element();

    let p_side = PartitionPreview::new(SplitLayout::SideBySide);
    let _e2 = p_side.into_any_element();

    let p_stacked = PartitionPreview::new(SplitLayout::Stacked);
    let _e3 = p_stacked.into_any_element();

    let p_grid = PartitionPreview::new(SplitLayout::Grid4);
    let _e4 = p_grid.into_any_element();

    let p_top1 = PartitionPreview::new(SplitLayout::TopOneBottomTwo);
    let _e5 = p_top1.into_any_element();

    let p_top2 = PartitionPreview::new(SplitLayout::TopTwoBottomOne);
    let _e6 = p_top2.into_any_element();
}

#[test]
fn test_couch_toggle_variants() {
    let toggle_full = CouchToggle::new("t-full", "Topmost Windows", true)
        .description("Keeps splitscreen game viewports pinned to top of display")
        .disabled(false);
    let _e1 = toggle_full.into_any_element();

    let toggle_compact = CouchToggle::new("t-mini", "Offline Mode", false).compact(true);
    let _e2 = toggle_compact.into_any_element();
}

#[test]
fn test_couch_search_bar_states() {
    let sb_empty = CouchSearchBar::new("sb-1")
        .placeholder("Search games...")
        .query("");
    let _e1 = sb_empty.into_any_element();

    let sb_active = CouchSearchBar::new("sb-2")
        .placeholder("Search games...")
        .query("Left 4 Dead")
        .on_clear(|_, _, _| {});
    let _e2 = sb_active.into_any_element();
}

#[test]
fn test_main_window_callbacks_default_and_builders() {
    use nucleus_ui::MainWindowCallbacks;
    use std::sync::Arc;
    use std::sync::atomic::{AtomicBool, Ordering};

    let called = Arc::new(AtomicBool::new(false));
    let called_clone = called.clone();

    let cbs = MainWindowCallbacks::new().with_theme_changer(move |_| {
        called_clone.store(true, Ordering::SeqCst);
    });

    assert!(cbs.on_theme_changed.is_some());
    assert!(cbs.on_launch.is_none());
    assert!(cbs.on_toggle_favorite.is_none());

    if let Some(ref cb) = cbs.on_theme_changed {
        cb("dark");
    }
    assert!(called.load(Ordering::SeqCst));
}

#[test]
fn test_couch_sidebar_nav_component() {
    let sidebar = CouchSidebarNav::new()
        .active_tab(nucleus_ui::ActiveTab::Setup)
        .detected_devices(3)
        .handler_count(15)
        .is_collapsed(false)
        .on_tab_click(|_tab, _win, _cx| {})
        .on_toggle_collapse(|_ev, _win, _cx| {});

    let _elem = sidebar.into_any_element();

    let sidebar_collapsed = CouchSidebarNav::new()
        .is_collapsed(true)
        .detected_devices(0);

    let _elem_collapsed = sidebar_collapsed.into_any_element();
}

#[test]
fn test_couch_confirm_dialog_component() {
    let state = nucleus_ui::ConfirmDialogState {
        title: "Delete Game".to_string(),
        message: "Are you sure?".to_string(),
        confirm_label: "Delete".to_string(),
        is_destructive: true,
        action: nucleus_ui::ConfirmAction::RemoveGame("test-guid".to_string()),
    };

    let dialog = CouchConfirmDialog::new(state)
        .on_confirm(|_ev, _win, _app| {})
        .on_cancel(|_ev, _win, _app| {});

    let _elem = dialog.into_any_element();
}

#[test]
fn test_couch_context_menu_component() {
    let handler = nucleus_ui::HandlerItem::new(
        "test-guid-123",
        "Portal 2",
        "portal2.exe",
        2,
        "Test handler notes",
    );

    let menu = CouchContextMenu::new(handler)
        .handlers_dir("C:\\Games\\Handlers")
        .text_editor_path("notepad.exe")
        .language_picker_open(true)
        .on_close(|_ev, _win, _app| {})
        .on_open_handler(|_ev, _win, _app| {})
        .on_toggle_assign_by_index(|_ev, _win, _app| {})
        .on_select_language(|_lang, _win, _app| {});

    let _elem = menu.into_any_element();
}

#[test]
fn test_nefos_card_component() {
    use nucleus_ui::nefos::{NefosCard, NefosCardVariant};

    let card = NefosCard::new("test-card")
        .variant(NefosCardVariant::Surface)
        .title("Audio Routing Settings")
        .description("Configure per-instance endpoint device assignments")
        .header_action(gpui::div().child("Action"))
        .hover_elevate(true)
        .paddings(true)
        .child(gpui::div().child("Option row 1"))
        .child(gpui::div().child("Option row 2"))
        .footer(gpui::div().child("Card footer note"));

    let _elem = card.into_any_element();
}

#[test]
fn test_nefos_segmented_control_component() {
    use nucleus_ui::nefos::NefosSegmentedControl;

    let seg = NefosSegmentedControl::new("test-seg")
        .item("1x1 (Single)", Some(IconName::Monitor), true)
        .item("2x1 (Side-by-Side)", Some(IconName::Columns2), false)
        .item("2x2 (Grid)", Some(IconName::Grid2x2), false)
        .on_click(|idx, _win, _app| {
            assert_eq!(idx, 0);
        });

    let _elem = seg.into_any_element();
}

#[test]
fn test_couch_button_outline_variants() {
    let btn_success = CouchButton::success_outline("btn-so", "Verify")
        .icon(IconName::Check)
        .size(CouchButtonSize::Sm);
    let _e1 = btn_success.into_any_element();

    let btn_danger = CouchButton::danger_outline("btn-do", "Remove")
        .icon(IconName::X)
        .disabled(false);
    let _e2 = btn_danger.into_any_element();

    let btn_variant_success = CouchButton::new("btn-vs")
        .label("OK")
        .variant(CouchButtonVariant::SuccessOutline);
    let _e3 = btn_variant_success.into_any_element();

    let btn_variant_danger = CouchButton::new("btn-vd")
        .label("Delete")
        .variant(CouchButtonVariant::DangerOutline);
    let _e4 = btn_variant_danger.into_any_element();
}

#[test]
fn test_couch_modal_header_actions_and_close_button() {
    let modal_both = CouchModal::new("modal-hdr-both", "Handler Details")
        .subtitle("Manage local handler configuration")
        .header_action(
            CouchButton::ghost("btn-hdr-fav", "Favorite")
                .size(CouchButtonSize::Sm)
                .into_any_element(),
        )
        .header_action(
            CouchButton::secondary("btn-hdr-open", "Folder")
                .size(CouchButtonSize::Sm)
                .into_any_element(),
        )
        .on_close(|_, _, _| {})
        .body(gpui::div().child("Body"));
    let _e1 = modal_both.into_any_element();

    let modal_actions_only = CouchModal::new("modal-hdr-actions-only", "Actions Only")
        .header_action(CouchButton::primary("btn-hdr-act", "Apply").size(CouchButtonSize::Sm));
    let _e2 = modal_actions_only.into_any_element();

    let modal_close_only = CouchModal::new("modal-close-only", "Close Only").on_close(|_, _, _| {});
    let _e3 = modal_close_only.into_any_element();
}

#[test]
fn test_library_sort_option_ascii_labels_and_icons() {
    use nucleus_ui::models::LibrarySortOption;

    let cases = [
        (LibrarySortOption::Alphabetical, "Alphabetical"),
        (LibrarySortOption::LastPlayed, "Last Played"),
        (LibrarySortOption::PlayTime, "Play Time"),
        (LibrarySortOption::MaxPlayers, "Max Players"),
        (LibrarySortOption::DateAdded, "Date Added"),
    ];

    for (opt, expected_label) in cases {
        assert_eq!(opt.label(), expected_label);
        assert!(
            opt.label().is_ascii(),
            "Label must be ASCII: {}",
            opt.label()
        );
    }

    assert!(matches!(
        LibrarySortOption::Alphabetical.icon(),
        IconName::ArrowDownAZ
    ));
    assert!(matches!(
        LibrarySortOption::LastPlayed.icon(),
        IconName::Clock
    ));
    assert!(matches!(
        LibrarySortOption::PlayTime.icon(),
        IconName::Timer
    ));
    assert!(matches!(
        LibrarySortOption::MaxPlayers.icon(),
        IconName::Users
    ));
    assert!(matches!(
        LibrarySortOption::DateAdded.icon(),
        IconName::Calendar
    ));
}

#[test]
fn test_library_sort_direction_ascii_labels_and_icons() {
    use nucleus_ui::models::LibrarySortDirection;

    assert_eq!(LibrarySortDirection::Ascending.label(), "Asc");
    assert_eq!(LibrarySortDirection::Descending.label(), "Desc");
    assert!(LibrarySortDirection::Ascending.label().is_ascii());
    assert!(LibrarySortDirection::Descending.label().is_ascii());

    assert!(matches!(
        LibrarySortDirection::Ascending.icon(),
        IconName::ArrowUp
    ));
    assert!(matches!(
        LibrarySortDirection::Descending.icon(),
        IconName::ArrowDown
    ));
    assert_eq!(
        LibrarySortDirection::Ascending.toggle(),
        LibrarySortDirection::Descending
    );
}

#[test]
fn test_library_and_hub_visual_polish_elements() {
    use gpui::prelude::FluentBuilder;

    // 1. Details button styling matches between Library and Hub
    let btn_details_lib = CouchButton::secondary("btn-details-card", "Details")
        .size(CouchButtonSize::Sm)
        .icon(IconName::Info);
    let _e_dl = btn_details_lib.into_any_element();

    let btn_details_hub = CouchButton::secondary("btn-details-hub", "Details")
        .size(CouchButtonSize::Sm)
        .icon(IconName::Info);
    let _e_dh = btn_details_hub.into_any_element();

    // 2. Locate .exe button: styled as green outline with transparent fill
    let btn_locate = CouchButton::success_outline("btn-locate-card", "Locate .exe")
        .size(CouchButtonSize::Sm)
        .icon(IconName::FolderOpen);
    let _e_loc = btn_locate.into_any_element();

    // 3. Hub Details modal footer actions: Report handler and Close
    let btn_report = CouchButton::secondary("modal-btn-report-handler", "Report handler")
        .icon(IconName::TriangleAlert)
        .size(CouchButtonSize::Md);
    let _e_rep = btn_report.into_any_element();

    let btn_close = CouchButton::secondary("modal-btn-close", "Close")
        .icon(IconName::Close)
        .size(CouchButtonSize::Md);
    let _e_cls = btn_close.into_any_element();

    // 4. GameCover checkmark badge behavior:
    // Library covers never show checkmark badge
    let cover_library = GameCover::new("Left 4 Dead 2", px(56.0), px(84.0)).installed(true);
    let _e_cl = cover_library.into_any_element();

    // Hub uninstalled cover has no badge
    let cover_hub_uninstalled = GameCover::new("Portal 2", px(56.0), px(84.0)).installed(false);
    let _e_chu = cover_hub_uninstalled.into_any_element();

    // Hub installed cover shows checkmark badge
    let is_installed = true;
    let cover_hub_installed = GameCover::new("Portal 2", px(56.0), px(84.0))
        .installed(is_installed)
        .when(is_installed, |c| {
            c.badge(gpui::div().child(gpui_component::Icon::new(IconName::Check).size_3()))
        });
    let _e_chi = cover_hub_installed.into_any_element();
}
