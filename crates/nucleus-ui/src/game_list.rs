//! Game list and grid catalog view.
//!
//! Renders the library of discovered or manually added games with cover art,
//! search filtering, and sorting options.

use crate::models::{HandlerItem, LibrarySortDirection, LibrarySortOption};
use crate::nefos::*;
use crate::theme;
use crate::window::MainWindow;
use crate::{Icon, IconName};
use gpui::prelude::*;
use gpui::*;

/// State and layout representation for the game catalog view.
#[derive(Clone, Default)]
pub struct GameListView {
    /// Search filter query string.
    pub search_query: String,
    /// Selected game profile or handler identifier.
    pub selected_game_id: Option<String>,
    /// Tracked scroll state handle for virtualized list rendering.
    pub scroll_handle: gpui::UniformListScrollHandle,
    /// Selected filter tags (e.g., "Favorites", "Ready to Launch", "4+ Players").
    pub selected_filters: std::collections::HashSet<String>,
}

/// Helper to determine if a handler satisfies active search query and multi-filter tags.
pub fn handler_matches_filters(
    h: &HandlerItem,
    search_query: &str,
    selected_filters: &std::collections::HashSet<String>,
) -> bool {
    let q = search_query.trim().to_lowercase();
    if !q.is_empty() && q != "all" {
        let matches_text = h.game_name.to_lowercase().contains(&q)
            || h.executable_name.to_lowercase().contains(&q)
            || h.description.to_lowercase().contains(&q);
        if !matches_text {
            return false;
        }
    }

    for filter in selected_filters {
        let f = filter.as_str();
        let matches = match f {
            "Favorites" => h.is_favorite,
            "Ready to Launch" => h.is_installed,
            "Gamepads" => h.supports_gamepad || h.supports_xinput || h.supports_dinput,
            "Keyboard/Mouse" => {
                h.supports_keyboard
                    || h.supports_raw_input
                    || h.supports_multiple_keyboards_and_mice
            }
            "4+ Players" => h.max_players >= 4,
            "2 Players" => h.max_players == 2,
            "Single Player" => h.max_players == 1,
            _ => true,
        };
        if !matches {
            return false;
        }
    }

    true
}

impl GameListView {
    /// Creates a new `GameListView`.
    pub fn new() -> Self {
        Self::default()
    }

    /// Sets the search filter text.
    pub fn set_filter(&mut self, query: impl Into<String>) {
        self.search_query = query.into();
    }

    /// Selects a specific game in the catalog.
    pub fn select_game(&mut self, game_id: Option<String>) {
        self.selected_game_id = game_id;
    }
}

/// Renders the complete game library view with handler cards, badges, and launch buttons.
pub fn render_library_view(
    handlers: &[HandlerItem],
    _selected_game: Option<&str>,
    search_query: &str,
    search_focus: &gpui::FocusHandle,
    scroll_handle: &gpui::UniformListScrollHandle,
    library_sort: LibrarySortOption,
    library_sort_dir: LibrarySortDirection,
    selected_filters: &std::collections::HashSet<String>,
    cx: &mut Context<MainWindow>,
) -> AnyElement {
    div()
        .flex()
        .flex_col()
        .size_full()
        .p_6()
        .gap_4()
        // Library Header
        .child(
            div()
                .flex_shrink_0()
                .flex()
                .items_center()
                .justify_between()
                .child(
                    div()
                        .flex()
                        .flex_col()
                        .child(
                            div()
                                .text_xl()
                                .font_weight(FontWeight::BOLD)
                                .text_color(theme::text_primary())
                                .child("Game Library"),
                        )
                        .child(
                            div()
                                .text_sm()
                                .text_color(theme::text_muted())
                                .child("Discovered game titles ready for splitscreen play"),
                        ),
                ),
        )
        // Library Search & Add Game Row
        .child(
            div()
                .flex_shrink_0()
                .flex()
                .items_center()
                .gap_3()
                .w_full()
                .child(
                    NefosSearchBar::new("lib-search-input-box")
                        .placeholder("Search installed library games by title or executable...")
                        .query(search_query)
                        .focus_handle(search_focus)
                        .on_key_down(cx.listener(|this: &mut MainWindow, event: &gpui::KeyDownEvent, _window, cx| {
                            if event.keystroke.modifiers.control || event.keystroke.modifiers.alt || event.keystroke.modifiers.platform {
                                return;
                            }
                            cx.stop_propagation();
                            let key = event.keystroke.key.as_str();
                            if key == "backspace" {
                                this.game_list.search_query.pop();
                                cx.notify();
                            } else if key == "escape" {
                                this.game_list.search_query.clear();
                                cx.notify();
                            } else if key == "space" {
                                this.game_list.search_query.push(' ');
                                cx.notify();
                            } else if let Some(ref s) = event.keystroke.key_char {
                                this.game_list.search_query.push_str(s);
                                cx.notify();
                            } else if key.chars().count() == 1 {
                                this.game_list.search_query.push_str(key);
                                cx.notify();
                            }
                        }))
                        .on_clear(cx.listener(|this, _, _, cx| {
                            this.game_list.search_query.clear();
                            cx.notify();
                        }))
                )
                .child(
                    NefosButton::primary("btn-add-game-file", "Add Game (.exe)")
                        .icon(IconName::FolderOpen)
                        .tooltip("Add a custom game executable to your library")
                        .on_click(cx.listener(|this, _, _, cx| {
                            this.status_message = "Browsing for game executable...".to_string();
                            cx.notify();
                            cx.spawn(async move |this, cx| {
                                let exe_opt = crate::dialogs::pick_file_async(
                                    "Select Game Executable",
                                    "Executables (*.exe)|*.exe|All Files (*.*)|*.*",
                                )
                                .await;
                                if let Some(exe) = exe_opt {
                                    let exe_path = exe.to_string_lossy().to_string();
                                    let _ = this.update(&mut *cx, |this, cx| {
                                        let mut added_handler = None;
                                        if let Some(ref cb) = this.callbacks.on_add_game {
                                            added_handler = cb(&exe_path);
                                        }
                                        if let Some(h) = added_handler {
                                            let name = h.game_name.clone();
                                            if !this.handlers.iter().any(|existing| existing.guid == h.guid) {
                                                this.handlers.push(h);
                                            }
                                            if !this.handler_names.contains(&name) {
                                                this.handler_names.push(name.clone());
                                            }
                                            this.selected_game = Some(name.clone());
                                            this.status_message = format!("Added game: {}", name);
                                        } else {
                                            this.status_message = format!("Selected game executable: {}", exe_path);
                                        }
                                        cx.notify();
                                    });
                                } else {
                                    let _ = this.update(&mut *cx, |this, cx| {
                                        this.status_message = "Executable selection cancelled.".to_string();
                                        cx.notify();
                                    });
                                }
                            })
                            .detach();
                        }))
                ),
        )
        // Library Filter Chips Bar
        .child(
            div()
                .id("lib-filter-scroll-row")
                .flex_shrink_0()
                .flex()
                .items_center()
                .gap_2()
                .w_full()
                .overflow_x_scroll()
                .child(
                    div()
                        .flex_shrink_0()
                        .text_xs()
                        .font_weight(FontWeight::SEMIBOLD)
                        .text_color(theme::text_muted())
                        .child("Filter:"),
                )
                .children([
                    ("All", None),
                    ("Favorites", Some(IconName::Star)),
                    ("Ready to Launch", Some(IconName::Play)),
                    ("Gamepads", Some(IconName::Gamepad2)),
                    ("Keyboard/Mouse", Some(IconName::Keyboard)),
                    ("4+ Players", Some(IconName::Users)),
                    ("2 Players", Some(IconName::UserCheck)),
                    ("Single Player", Some(IconName::User)),
                ].into_iter().enumerate().map(|(idx, (label, icon_opt))| {
                    let is_all = label == "All";
                    let is_active = if is_all {
                        selected_filters.is_empty()
                    } else {
                        selected_filters.contains(label)
                    };
                    let f_str = label.to_string();

                    let mut tag = NefosTag::new(label)
                        .tone(if is_active {
                            NefosTagTone::Accent
                        } else {
                            NefosTagTone::Neutral
                        });
                    if let Some(ic) = icon_opt {
                        tag = tag.icon(ic);
                    }

                    div()
                        .id(("lib-filter-chip", idx))
                        .cursor_pointer()
                        .flex_shrink_0()
                        .on_click(cx.listener(move |this, _, _, cx| {
                            if f_str == "All" {
                                this.game_list.selected_filters.clear();
                            } else if this.game_list.selected_filters.contains(&f_str) {
                                this.game_list.selected_filters.remove(&f_str);
                            } else {
                                this.game_list.selected_filters.insert(f_str.clone());
                            }
                            cx.notify();
                        }))
                        .child(tag)
                })),
        )
        // Library Sort & Order Controls Bar
        .child(
            div()
                .flex_shrink_0()
                .flex()
                .items_center()
                .justify_between()
                .gap_2()
                .w_full()
                .p_2p5()
                .bg(theme::bg_panel())
                .rounded_xl()
                .border_1()
                .border_color(theme::border_color())
                .child(
                    div()
                        .id("lib-sort-scroll-row")
                        .flex()
                        .items_center()
                        .gap_2()
                        .min_w_0()
                        .overflow_x_scroll()
                        .child(
                            div()
                                .flex_shrink_0()
                                .text_xs()
                                .font_weight(FontWeight::SEMIBOLD)
                                .text_color(theme::text_muted())
                                .child("Sort:"),
                        )
                        .children([
                            (LibrarySortOption::Alphabetical, "Alphabetical (A-Z)"),
                            (LibrarySortOption::MaxPlayers, "Player Count"),
                            (LibrarySortOption::LastPlayed, "Recently Played"),
                            (LibrarySortOption::PlayTime, "Play Time"),
                            (LibrarySortOption::DateAdded, "Date Added"),
                        ].into_iter().enumerate().map(|(idx, (sort_opt, label))| {
                            let is_active = library_sort == sort_opt;
                            NefosButton::ghost(("lib-sort-opt", idx), label)
                                .size(NefosButtonSize::Sm)
                                .icon(sort_opt.icon())
                                .selected(is_active)
                                .on_click(cx.listener(move |this, _, _, cx| {
                                    this.library_sort = sort_opt;
                                    cx.notify();
                                }))
                        })),
                )
                .child(
                    NefosButton::secondary("btn-lib-sort-direction", library_sort_dir.label())
                        .size(NefosButtonSize::Sm)
                        .icon(library_sort_dir.icon())
                        .tooltip("Toggle Sort Direction (Ascending / Descending)")
                        .on_click(cx.listener(|this, _, _, cx| {
                            this.library_sort_dir = this.library_sort_dir.toggle();
                            cx.notify();
                        }))
                ),
        )
        // Virtualized list of game cards or empty states
        .child({
            let scroll_handle = scroll_handle.clone();
            let filtered_count = handlers
                .iter()
                .filter(|h| handler_matches_filters(h, search_query, selected_filters))
                .count();

            div()
                .id("library-scroll-container")
                .flex()
                .flex_col()
                .flex_1()
                .min_h_0()
                .overflow_hidden()
                // Empty state: No games in library at all
                .when(filtered_count == 0 && search_query.is_empty() && selected_filters.is_empty(), |this_cont| {
                    this_cont.child(
                        div()
                            .flex()
                            .flex_col()
                            .items_center()
                            .justify_center()
                            .p_8()
                            .w_full()
                            .child(
                                NefosCard::surface("lib-no-games-card")
                                    .hover_elevate(false)
                                    .paddings(true)
                                    .child(
                                        div()
                                            .flex()
                                            .flex_col()
                                            .items_center()
                                            .justify_center()
                                            .py_8()
                                            .px_12()
                                            .gap_3()
                                            .child(
                                                Icon::new(IconName::Gamepad2)
                                                    .size_12()
                                                    .text_color(theme::accent()),
                                            )
                                            .child(
                                                div()
                                                    .text_lg()
                                                    .font_weight(FontWeight::SEMIBOLD)
                                                    .text_color(theme::text_primary())
                                                    .child("Your Library is Empty"),
                                            )
                                            .child(
                                                div()
                                                    .text_sm()
                                                    .text_color(theme::text_muted())
                                                    .child("Add game executables from disk or browse and download handlers from the Community Hub."),
                                            )
                                            .child(
                                                div()
                                                    .pt_2()
                                                    .child(
                                                        NefosButton::primary("btn-empty-add-game", "Add Game (.exe)")
                                                            .icon(IconName::FolderOpen)
                                                            .size(NefosButtonSize::Sm)
                                                            .on_click(cx.listener(|this, _, _, cx| {
                                                                this.status_message = "Browsing for game executable...".to_string();
                                                                cx.notify();
                                                                cx.spawn(async move |this, cx| {
                                                                    let exe_opt = crate::dialogs::pick_file_async(
                                                                        "Select Game Executable",
                                                                        "Executables (*.exe)|*.exe|All Files (*.*)|*.*",
                                                                    )
                                                                    .await;
                                                                    if let Some(exe) = exe_opt {
                                                                        let exe_path = exe.to_string_lossy().to_string();
                                                                        let _ = this.update(&mut *cx, |this, cx| {
                                                                            let mut added_handler = None;
                                                                            if let Some(ref cb) = this.callbacks.on_add_game {
                                                                                added_handler = cb(&exe_path);
                                                                            }
                                                                            if let Some(h) = added_handler {
                                                                                let name = h.game_name.clone();
                                                                                if !this.handlers.iter().any(|existing| existing.guid == h.guid) {
                                                                                    this.handlers.push(h);
                                                                                }
                                                                                if !this.handler_names.contains(&name) {
                                                                                    this.handler_names.push(name.clone());
                                                                                }
                                                                                this.selected_game = Some(name.clone());
                                                                                this.status_message = format!("Added game: {}", name);
                                                                            } else {
                                                                                this.status_message = format!("Selected game executable: {}", exe_path);
                                                                            }
                                                                            cx.notify();
                                                                        });
                                                                    } else {
                                                                        let _ = this.update(&mut *cx, |this, cx| {
                                                                            this.status_message = "Executable selection cancelled.".to_string();
                                                                            cx.notify();
                                                                        });
                                                                    }
                                                                })
                                                                .detach();
                                                            })),
                                                    ),
                                            ),
                                    ),
                            ),
                    )
                })
                // Empty state: Active query or filters returned 0 matches
                .when(filtered_count == 0 && (!search_query.is_empty() || !selected_filters.is_empty()), |this_cont| {
                    this_cont.child(
                        div()
                            .flex()
                            .flex_col()
                            .items_center()
                            .justify_center()
                            .p_8()
                            .w_full()
                            .child(
                                NefosCard::surface("lib-empty-state-card")
                                    .hover_elevate(false)
                                    .paddings(true)
                                    .child(
                                        div()
                                            .flex()
                                            .flex_col()
                                            .items_center()
                                            .justify_center()
                                            .py_8()
                                            .px_12()
                                            .gap_3()
                                            .child(
                                                Icon::new(IconName::SearchX)
                                                    .size_12()
                                                    .text_color(theme::text_muted()),
                                            )
                                            .child(
                                                div()
                                                    .text_lg()
                                                    .font_weight(FontWeight::SEMIBOLD)
                                                    .text_color(theme::text_primary())
                                                    .child("No matching games found"),
                                            )
                                            .child(
                                                div()
                                                    .text_sm()
                                                    .text_color(theme::text_muted())
                                                    .child("Try adjusting your search query or active filter tags."),
                                            )
                                            .child(
                                                div()
                                                    .pt_2()
                                                    .child(
                                                        NefosButton::secondary("btn-clear-filters-empty", "Clear filters")
                                                            .icon(IconName::X)
                                                            .size(NefosButtonSize::Sm)
                                                            .on_click(cx.listener(|this, _, _, cx| {
                                                                this.game_list.search_query.clear();
                                                                this.game_list.selected_filters.clear();
                                                                cx.notify();
                                                            })),
                                                    ),
                                            ),
                                    ),
                            ),
                    )
                })
                .when(filtered_count > 0, |this_cont| {
                    this_cont.child(
                        div()
                            .flex()
                            .flex_row()
                            .flex_1()
                            .min_h_0()
                            .size_full()
                            .child(
                                uniform_list(
                                    "library-cards-virtual",
                                    filtered_count,
                                    cx.processor(move |this, range: std::ops::Range<usize>, _window, cx| {
                                        let mut matched: Vec<HandlerItem> = this
                                            .handlers
                                            .iter()
                                            .filter(|h| handler_matches_filters(h, &this.game_list.search_query, &this.game_list.selected_filters))
                                            .cloned()
                                            .collect();

                                        // Sorting criteria implementation
                                        match this.library_sort {
                                            LibrarySortOption::Alphabetical => {
                                                matched.sort_by(|a, b| a.game_name.to_lowercase().cmp(&b.game_name.to_lowercase()));
                                            }
                                            LibrarySortOption::LastPlayed => {
                                                matched.sort_by(|a, b| {
                                                    let a_has = a.last_played_timestamp.is_some() || a.last_played_str.is_some();
                                                    let b_has = b.last_played_timestamp.is_some() || b.last_played_str.is_some();
                                                    match (a_has, b_has) {
                                                        (true, true) => {
                                                            let a_ts = a.last_played_timestamp.unwrap_or(0);
                                                            let b_ts = b.last_played_timestamp.unwrap_or(0);
                                                            b_ts.cmp(&a_ts)
                                                        }
                                                        (true, false) => std::cmp::Ordering::Less,
                                                        (false, true) => std::cmp::Ordering::Greater,
                                                        (false, false) => a.game_name.to_lowercase().cmp(&b.game_name.to_lowercase()),
                                                    }
                                                });
                                            }
                                            LibrarySortOption::PlayTime => {
                                                matched.sort_by(|a, b| b.play_time_seconds.cmp(&a.play_time_seconds));
                                            }
                                            LibrarySortOption::MaxPlayers => {
                                                matched.sort_by(|a, b| b.max_players.cmp(&a.max_players));
                                            }
                                            LibrarySortOption::DateAdded => {
                                                // Preserves directory / discovery load order
                                            }
                                        }

                                        if this.library_sort_dir == LibrarySortDirection::Descending {
                                            if this.library_sort == LibrarySortOption::Alphabetical {
                                                matched.reverse();
                                            }
                                        } else if this.library_sort != LibrarySortOption::Alphabetical {
                                            matched.reverse();
                                        }

                                        let mut items = Vec::with_capacity(range.len());
                                        for ix in range {
                                            if let Some(h) = matched.get(ix) {
                                                items.push(render_game_card_item(ix, h, this.selected_game.as_deref(), cx));
                                            }
                                        }
                                        items
                                    }),
                                )
                                .track_scroll(&scroll_handle)
                                .flex_1()
                                .size_full()
                            )
                            .child({
                                let (offset_y, max_offset_y) = {
                                    let state = scroll_handle.0.borrow();
                                    (
                                        state.base_handle.offset().y,
                                        state.base_handle.max_offset().y,
                                    )
                                };
                                let ratio = if max_offset_y > px(1.0) {
                                    (offset_y / max_offset_y).abs().clamp(0.0, 1.0)
                                } else {
                                    0.0
                                };
                                let thumb_offset = ratio * 320.0;
                                let scroll_handle_click = scroll_handle.clone();
                                let total_items = filtered_count;

                                // Visual moving dynamic scrollbar indicator on the right edge
                                div()
                                    .id("library-custom-scrollbar")
                                    .cursor_pointer()
                                    .w(px(8.0))
                                    .my_1()
                                    .mr_0p5()
                                    .rounded_full()
                                    .bg(theme::bg_panel())
                                    .border_1()
                                    .border_color(theme::border_color())
                                    .relative()
                                    .on_click(cx.listener(move |_this, _event, _window, cx| {
                                        if total_items > 0 {
                                            let next_idx = if ratio < 0.5 { total_items.saturating_sub(1) } else { 0 };
                                            scroll_handle_click.scroll_to_item(next_idx, gpui::ScrollStrategy::Top);
                                            cx.notify();
                                        }
                                    }))
                                    .child(
                                        div()
                                            .relative()
                                            .w_full()
                                            .h(px(60.0))
                                            .top(px(thumb_offset))
                                            .rounded_full()
                                            .bg(theme::accent())
                                            .hover(|s| s.bg(theme::accent_hover()))
                                    )
                            })
                    )
                })
        })
        .into_any_element()
}

/// Renders an individual game handler card inside the virtualized library list.
fn render_game_card_item(
    idx: usize,
    h: &HandlerItem,
    selected_game: Option<&str>,
    cx: &mut Context<MainWindow>,
) -> AnyElement {
    crate::game_card::render_game_card(idx, h, selected_game, cx)
}

#[cfg(test)]
mod tests {
    use super::*;
    use core::prelude::v1::test;

    #[test]
    fn test_game_list_filtering_and_selection() {
        let mut list = GameListView::new();
        assert_eq!(list.search_query, "");
        assert_eq!(list.selected_game_id, None);

        list.set_filter("Borderlands");
        assert_eq!(list.search_query, "Borderlands");

        list.select_game(Some("borderlands-2".to_string()));
        assert_eq!(list.selected_game_id, Some("borderlands-2".to_string()));
    }

    #[test]
    fn test_game_list_ready_to_launch_filter() {
        let mut h1 = HandlerItem::new("l4d2", "Left 4 Dead 2", "left4dead2.exe", 4, "desc");
        h1.is_installed = true;

        let mut h2 = HandlerItem::new("halo", "Halo MCC", "mcclauncher.exe", 8, "desc");
        h2.is_installed = false;

        let handlers = vec![h1, h2];

        // Filter: "ready to launch"
        let q_filter = "ready to launch".to_lowercase();
        let ready: Vec<&HandlerItem> = handlers
            .iter()
            .filter(|h| {
                if q_filter.contains("ready") || q_filter.contains("installed") {
                    h.is_installed
                } else {
                    true
                }
            })
            .collect();

        assert_eq!(ready.len(), 1);
        assert_eq!(ready[0].guid, "l4d2");
    }

    #[test]
    fn test_game_list_multi_filter_matching() {
        let mut h1 = HandlerItem::new("l4d2", "Left 4 Dead 2", "left4dead2.exe", 4, "desc");
        h1.is_installed = true;
        h1.is_favorite = true;
        h1.supports_gamepad = true;

        let mut h2 = HandlerItem::new("halo", "Halo MCC", "mcclauncher.exe", 8, "desc");
        h2.is_installed = false;
        h2.is_favorite = true;

        let mut filters = std::collections::HashSet::new();
        filters.insert("Favorites".to_string());
        filters.insert("Ready to Launch".to_string());

        assert!(handler_matches_filters(&h1, "", &filters));
        assert!(!handler_matches_filters(&h2, "", &filters)); // h2 is favorite but not installed

        // With search query
        assert!(handler_matches_filters(&h1, "Left 4", &filters));
        assert!(!handler_matches_filters(&h1, "Portal", &filters));
    }
}
