//! Big Picture library carousel screen for 10-foot television interface.
//!
//! Provides horizontal carousel browsing of installed game handlers, oversized
//! cover cards with focus highlight/glow, metadata badges, couch button prompts,
//! and seamless desktop/couch mode switching.

use crate::components::{CouchButton, CouchButtonSize, CouchTag, TagTone, *};
use crate::models::HandlerItem;
use crate::theme;
use crate::window::MainWindow;
use crate::{Icon, IconName};
use gpui::prelude::*;
use gpui::*;

/// Active screen within Couch Big Picture mode.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum BigPictureScreen {
    /// 10-foot game library carousel browsing screen.
    #[default]
    Carousel,
    /// Couch controller slot auto-assignment and monitor partition setup.
    CouchSetup,
    /// Couch community handler browser.
    CouchHub,
}

impl BigPictureScreen {
    /// Returns the next Big Picture screen tab (Carousel -> CouchSetup -> CouchHub -> Carousel).
    pub fn next(&self) -> Self {
        match self {
            Self::Carousel => Self::CouchSetup,
            Self::CouchSetup => Self::CouchHub,
            Self::CouchHub => Self::Carousel,
        }
    }

    /// Returns the previous Big Picture screen tab (Carousel -> CouchHub -> CouchSetup -> Carousel).
    pub fn prev(&self) -> Self {
        match self {
            Self::Carousel => Self::CouchHub,
            Self::CouchHub => Self::CouchSetup,
            Self::CouchSetup => Self::Carousel,
        }
    }
}

/// Filter mode for the 10-foot Big Picture library carousel.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum CarouselFilterMode {
    /// All handlers without filtering.
    #[default]
    All,
    /// Only games with an executable verified on disk (`is_installed == true`).
    InstalledOnly,
    /// Only marked favorite games.
    Favorites,
    /// Games supporting controller/gamepad input.
    ControllerSupport,
    /// Games supporting single keyboard and mouse input.
    KeyboardOnly,
    /// Games supporting multiple keyboards and mice (ProtoInput/RawInput).
    MultiKeyboardMouse,
    /// Games supporting 4 or more players (`max_players >= 4`).
    FourPlusPlayers,
    /// Games supporting 2 or more players (`max_players >= 2`).
    TwoPlusPlayers,
}

impl CarouselFilterMode {
    /// Slice of all selectable filter modes in preferred presentation order.
    pub const ALL: &'static [CarouselFilterMode] = &[
        Self::All,
        Self::InstalledOnly,
        Self::Favorites,
        Self::ControllerSupport,
        Self::KeyboardOnly,
        Self::MultiKeyboardMouse,
        Self::FourPlusPlayers,
        Self::TwoPlusPlayers,
    ];

    /// User-facing label for the filter mode.
    pub fn label(&self) -> &'static str {
        match self {
            Self::All => "All Games",
            Self::InstalledOnly => "Ready to Launch",
            Self::Favorites => "Favorites",
            Self::ControllerSupport => "Controllers",
            Self::KeyboardOnly => "Keyboard/Mouse",
            Self::MultiKeyboardMouse => "Multi-K&M (ProtoInput)",
            Self::FourPlusPlayers => "4+ Players",
            Self::TwoPlusPlayers => "2+ Players",
        }
    }

    /// Icon corresponding to this filter mode.
    pub fn icon(&self) -> gpui_kit_assets::IconName {
        match self {
            Self::All => gpui_kit_assets::IconName::LayoutGrid,
            Self::InstalledOnly => gpui_kit_assets::IconName::Check,
            Self::Favorites => gpui_kit_assets::IconName::Star,
            Self::ControllerSupport => gpui_kit_assets::IconName::Gamepad2,
            Self::KeyboardOnly => gpui_kit_assets::IconName::Keyboard,
            Self::MultiKeyboardMouse => gpui_kit_assets::IconName::Mouse,
            Self::FourPlusPlayers => gpui_kit_assets::IconName::Users,
            Self::TwoPlusPlayers => gpui_kit_assets::IconName::Users,
        }
    }
}

/// Sort ordering for the 10-foot Big Picture library carousel.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum CarouselSortMode {
    /// Sorted alphabetically A-Z by game name.
    #[default]
    Alphabetical,
    /// Sorted alphabetically Z-A by game name.
    AlphabeticalDesc,
    /// Sorted descending by most recently played date.
    RecentlyPlayed,
    /// Sorted descending by accumulated total play time.
    PlayTime,
    /// Sorted descending by maximum supported player count.
    MaxPlayers,
    /// Installed games first, followed by uninstalled games.
    InstalledFirst,
}

impl CarouselSortMode {
    /// Slice of all selectable sort modes in presentation order.
    pub const ALL: &'static [CarouselSortMode] = &[
        Self::Alphabetical,
        Self::AlphabeticalDesc,
        Self::RecentlyPlayed,
        Self::PlayTime,
        Self::MaxPlayers,
        Self::InstalledFirst,
    ];

    /// User-facing label for the sort mode.
    pub fn label(&self) -> &'static str {
        match self {
            Self::Alphabetical => "Alphabetical (A-Z)",
            Self::AlphabeticalDesc => "Alphabetical (Z-A)",
            Self::RecentlyPlayed => "Recently Played",
            Self::PlayTime => "Total Play Time",
            Self::MaxPlayers => "Maximum Players",
            Self::InstalledFirst => "Installed First",
        }
    }

    /// Icon corresponding to this sort mode.
    pub fn icon(&self) -> gpui_kit_assets::IconName {
        match self {
            Self::Alphabetical => gpui_kit_assets::IconName::ArrowDownAZ,
            Self::AlphabeticalDesc => gpui_kit_assets::IconName::ArrowDownAZ,
            Self::RecentlyPlayed => gpui_kit_assets::IconName::Clock,
            Self::PlayTime => gpui_kit_assets::IconName::Timer,
            Self::MaxPlayers => gpui_kit_assets::IconName::Users,
            Self::InstalledFirst => gpui_kit_assets::IconName::Check,
        }
    }
}

/// State tracking selected item index and total items in the horizontal carousel.
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct CarouselState {
    /// Total count of game handler items in the carousel.
    pub items_count: usize,
    /// Currently focused/selected game index (0-based).
    pub selected_index: usize,
    /// Active filter mode for the carousel library.
    pub filter_mode: CarouselFilterMode,
    /// Active sort mode for the carousel library.
    pub sort_mode: CarouselSortMode,
    /// Active search query string.
    pub search_query: String,
    /// Whether the interactive search input bar is currently open.
    pub search_input_open: bool,
    /// Whether the filter options list picker popover is currently open.
    pub filter_picker_open: bool,
    /// Whether the sort options list picker popover is currently open.
    pub sort_picker_open: bool,
}

impl CarouselState {
    /// Creates a new `CarouselState` with the specified item count.
    pub fn new(count: usize) -> Self {
        Self {
            items_count: count,
            selected_index: 0,
            filter_mode: CarouselFilterMode::All,
            sort_mode: CarouselSortMode::Alphabetical,
            search_query: String::new(),
            search_input_open: false,
            filter_picker_open: false,
            sort_picker_open: false,
        }
    }

    /// Moves selection to the left, wrapping from 0 to the last item.
    pub fn move_left(&mut self) {
        if self.items_count == 0 {
            self.selected_index = 0;
            return;
        }
        if self.selected_index == 0 {
            self.selected_index = self.items_count.saturating_sub(1);
        } else {
            self.selected_index -= 1;
        }
    }

    /// Moves selection to the right, wrapping from the last item to 0.
    pub fn move_right(&mut self) {
        if self.items_count == 0 {
            self.selected_index = 0;
            return;
        }
        if self.selected_index + 1 >= self.items_count {
            self.selected_index = 0;
        } else {
            self.selected_index += 1;
        }
    }

    /// Sets the selected index directly, clamped to `items_count - 1`.
    pub fn select(&mut self, idx: usize) {
        if self.items_count == 0 {
            self.selected_index = 0;
            return;
        }
        let max_idx = self.items_count.saturating_sub(1);
        self.selected_index = idx.min(max_idx);
    }

    /// Returns the currently selected item index.
    pub fn selected(&self) -> usize {
        self.selected_index
    }

    /// Updates the total item count and ensures the selected index remains in bounds.
    pub fn set_count(&mut self, count: usize) {
        self.items_count = count;
        if self.items_count == 0 {
            self.selected_index = 0;
        } else if self.selected_index >= self.items_count {
            self.selected_index = self.items_count.saturating_sub(1);
        }
    }

    /// Filters and sorts the given slice of `HandlerItem`s according to `filter_mode`, `sort_mode`, and `search_query`.
    pub fn filter_and_sort_handlers<'a>(
        &self,
        handlers: &'a [HandlerItem],
        search_query: &str,
    ) -> Vec<&'a HandlerItem> {
        let q = search_query.trim().to_lowercase();
        let mut list: Vec<&'a HandlerItem> = handlers
            .iter()
            .filter(|h| {
                // Search filter (if query is non-empty)
                if !q.is_empty() {
                    let name_match = h.game_name.to_lowercase().contains(&q);
                    let exe_match = h.executable_name.to_lowercase().contains(&q);
                    let title_match = h.handler_title.to_lowercase().contains(&q);
                    if !name_match && !exe_match && !title_match {
                        return false;
                    }
                }

                // Mode filter
                match self.filter_mode {
                    CarouselFilterMode::All => true,
                    CarouselFilterMode::InstalledOnly => h.is_installed,
                    CarouselFilterMode::Favorites => h.is_favorite,
                    CarouselFilterMode::ControllerSupport => {
                        h.supports_gamepad || h.supports_xinput || h.supports_dinput
                    }
                    CarouselFilterMode::KeyboardOnly => h.supports_keyboard,
                    CarouselFilterMode::MultiKeyboardMouse => {
                        h.supports_multiple_keyboards_and_mice || h.supports_raw_input
                    }
                    CarouselFilterMode::FourPlusPlayers => h.max_players >= 4,
                    CarouselFilterMode::TwoPlusPlayers => h.max_players >= 2,
                }
            })
            .collect();

        // Sort
        match self.sort_mode {
            CarouselSortMode::Alphabetical => {
                list.sort_by(|a, b| a.game_name.to_lowercase().cmp(&b.game_name.to_lowercase()));
            }
            CarouselSortMode::AlphabeticalDesc => {
                list.sort_by(|a, b| b.game_name.to_lowercase().cmp(&a.game_name.to_lowercase()));
            }
            CarouselSortMode::RecentlyPlayed => {
                list.sort_by(|a, b| b.last_played_timestamp.cmp(&a.last_played_timestamp));
            }
            CarouselSortMode::PlayTime => {
                list.sort_by(|a, b| b.play_time_seconds.cmp(&a.play_time_seconds));
            }
            CarouselSortMode::MaxPlayers => {
                // Descending by max_players; secondary tie-breaker alphabetical
                list.sort_by(|a, b| {
                    b.max_players
                        .cmp(&a.max_players)
                        .then_with(|| a.game_name.to_lowercase().cmp(&b.game_name.to_lowercase()))
                });
            }
            CarouselSortMode::InstalledFirst => {
                // Installed first, then uninstalled; secondary tie-breaker alphabetical
                list.sort_by(|a, b| {
                    b.is_installed
                        .cmp(&a.is_installed)
                        .then_with(|| a.game_name.to_lowercase().cmp(&b.game_name.to_lowercase()))
                });
            }
        }

        list
    }
}

/// Renders the complete Couch Big Picture view shell.
pub fn render_big_picture_view(
    win: &MainWindow,
    _window: &mut Window,
    cx: &mut Context<MainWindow>,
) -> AnyElement {
    let toast = win
        .couch_setup_state
        .active_disconnect_banner()
        .map(|(idx, slot, rem, tot)| {
            crate::big_picture::setup::render_disconnect_toast_banner(idx, slot, rem, tot)
        });

    div()
        .relative()
        .flex()
        .flex_col()
        .size_full()
        .bg(theme::bg_main())
        .text_color(theme::text_primary())
        .child(render_big_picture_header(win, cx))
        .child(
            div()
                .flex()
                .flex_col()
                .flex_1()
                .min_h_0()
                .w_full()
                .overflow_hidden()
                .child(match win.big_picture_screen {
                    BigPictureScreen::Carousel => render_carousel_view(win, cx),
                    BigPictureScreen::CouchSetup => {
                        crate::big_picture::render_couch_setup_view(win, cx)
                    }
                    BigPictureScreen::CouchHub => {
                        crate::big_picture::render_couch_hub_view(win, cx)
                    }
                }),
        )
        .child(render_couch_prompt_bar(win.big_picture_screen))
        .children(toast)
        .children(if win.big_picture_screen != BigPictureScreen::CouchHub {
            win.couch_hub_state
                .custom_options_modal
                .as_ref()
                .map(|m| crate::big_picture::render_custom_options_modal(m, cx))
        } else {
            None
        })
        .children(if win.big_picture_screen != BigPictureScreen::CouchHub {
            win.couch_hub_state
                .missing_exe_modal_handler
                .as_deref()
                .map(|h| crate::big_picture::render_missing_exe_modal(h, cx))
        } else {
            None
        })
        .into_any_element()
}

fn render_big_picture_header(win: &MainWindow, cx: &mut Context<MainWindow>) -> AnyElement {
    let controller_label = if win.detected_devices == 1 {
        "1 Controller Connected".to_string()
    } else {
        format!("{} Controllers Connected", win.detected_devices)
    };

    div()
        .id("big-picture-header")
        .flex()
        .items_center()
        .justify_between()
        .h(px(60.0))
        .w_full()
        .px_8()
        .bg(theme::bg_panel())
        .border_b_1()
        .border_color(theme::border_color())
        // Left Branding & Controller Count
        .child(
            div()
                .flex()
                .items_center()
                .gap_4()
                .min_w_0()
                .child(
                    div()
                        .flex()
                        .items_center()
                        .justify_center()
                        .size(px(32.0))
                        .rounded_lg()
                        .bg(theme::accent())
                        .text_color(theme::text_on_accent())
                        .font_weight(FontWeight::BOLD)
                        .text_sm()
                        .child("NC"),
                )
                .child(
                    div()
                        .flex()
                        .items_center()
                        .gap_2p5()
                        .child(
                            div()
                                .text_base()
                                .font_weight(FontWeight::BOLD)
                                .text_color(theme::text_primary())
                                .child("Nucleus Co-op"),
                        )
                        .child(div().text_sm().text_color(theme::text_muted()).child("•"))
                        .child(
                            div()
                                .text_sm()
                                .font_weight(FontWeight::MEDIUM)
                                .text_color(theme::accent())
                                .child("Big Picture"),
                        ),
                )
                .child(
                    div()
                        .flex()
                        .items_center()
                        .gap_2()
                        .px_3()
                        .py_1()
                        .rounded_full()
                        .bg(theme::bg_subtle())
                        .border_1()
                        .border_color(theme::border_subtle())
                        .child(
                            Icon::new(IconName::Gamepad2)
                                .size_4()
                                .text_color(theme::accent()),
                        )
                        .child(
                            div()
                                .text_xs()
                                .font_weight(FontWeight::MEDIUM)
                                .text_color(theme::text_primary())
                                .child(controller_label),
                        ),
                ),
        )
        // Center 3-Tab Navigation Bar with Bumper Hints
        .child(
            div()
                .flex()
                .items_center()
                .gap_3()
                .min_w_0()
                .child(render_bumper_hint("LB / Q"))
                .child(
                    div()
                        .flex()
                        .items_center()
                        .gap_2()
                        .min_w_0()
                        .child(render_header_tab(
                            "Library",
                            IconName::Gamepad2,
                            win.big_picture_screen == BigPictureScreen::Carousel,
                            BigPictureScreen::Carousel,
                            cx,
                        ))
                        .child(render_header_tab(
                            "Setup",
                            IconName::SlidersHorizontal,
                            win.big_picture_screen == BigPictureScreen::CouchSetup,
                            BigPictureScreen::CouchSetup,
                            cx,
                        ))
                        .child(render_header_tab(
                            "Community Hub",
                            IconName::Globe,
                            win.big_picture_screen == BigPictureScreen::CouchHub,
                            BigPictureScreen::CouchHub,
                            cx,
                        )),
                )
                .child(render_bumper_hint("RB / E")),
        )
        // Right Exit to Desktop Button
        .child(
            CouchButton::secondary("btn-couch-exit", "Exit Couch Mode")
                .size(CouchButtonSize::Md)
                .on_click(cx.listener(|this, _, _, cx| {
                    this.toggle_big_picture(cx);
                })),
        )
        .into_any_element()
}

fn render_bumper_hint(badge: &'static str) -> AnyElement {
    div()
        .flex()
        .items_center()
        .justify_center()
        .px_2()
        .py_1()
        .rounded_md()
        .bg(theme::bg_subtle())
        .border_1()
        .border_color(theme::border_subtle())
        .text_xs()
        .font_weight(FontWeight::BOLD)
        .text_color(theme::text_muted())
        .child(badge)
        .into_any_element()
}

fn render_header_tab(
    label: &'static str,
    icon: IconName,
    is_active: bool,
    target_screen: BigPictureScreen,
    cx: &mut Context<MainWindow>,
) -> AnyElement {
    let tab_id = match target_screen {
        BigPictureScreen::Carousel => "couch-tab-library",
        BigPictureScreen::CouchSetup => "couch-tab-setup",
        BigPictureScreen::CouchHub => "couch-tab-hub",
    };

    let mut pill = div()
        .id(tab_id)
        .cursor_pointer()
        .flex()
        .items_center()
        .gap_2()
        .px_3p5()
        .py_1p5()
        .rounded_lg()
        .min_w_0()
        .text_sm();

    if is_active {
        pill = pill
            .bg(theme::accent_subtle())
            .border_1()
            .border_color(theme::accent())
            .text_color(theme::accent())
            .font_weight(FontWeight::BOLD)
            .child(Icon::new(icon).size_4().text_color(theme::accent()))
            .child(div().min_w_0().child(label));
    } else {
        pill = pill
            .bg(theme::bg_subtle())
            .border_1()
            .border_color(theme::border_subtle())
            .text_color(theme::text_muted())
            .hover(|s| s.bg(theme::bg_hover()).text_color(theme::text_primary()))
            .child(Icon::new(icon).size_4().text_color(theme::text_muted()))
            .child(div().min_w_0().child(label));
    }

    pill.on_click(cx.listener(move |this, _, _, cx| {
        this.big_picture_screen = target_screen;
        if target_screen == BigPictureScreen::CouchHub && this.hub_packages.is_empty() {
            this.trigger_hub_search_async(cx);
        }
        cx.notify();
    }))
    .into_any_element()
}

/// Computes the sub-list of visible handler indices centered around `sel_idx`.
fn get_carousel_window_indices(sel_idx: usize, total: usize) -> Vec<usize> {
    if total <= 1 {
        return vec![0];
    }
    if total == 2 {
        return vec![0, 1];
    }
    if total == 3 {
        return vec![
            (sel_idx + total - 1) % total,
            sel_idx,
            (sel_idx + 1) % total,
        ];
    }
    if total == 4 {
        return vec![
            (sel_idx + total - 1) % total,
            sel_idx,
            (sel_idx + 1) % total,
            (sel_idx + 2) % total,
        ];
    }
    // 5 or more: 2 left, 1 center, 2 right
    vec![
        (sel_idx + total - 2) % total,
        (sel_idx + total - 1) % total,
        sel_idx,
        (sel_idx + 1) % total,
        (sel_idx + 2) % total,
    ]
}

/// Renders the 10-foot game library carousel browsing view.
pub fn render_carousel_view(win: &MainWindow, cx: &mut Context<MainWindow>) -> AnyElement {
    if win.handlers.is_empty() {
        return div()
            .flex()
            .flex_col()
            .items_center()
            .justify_center()
            .size_full()
            .child(
                div()
                    .flex()
                    .flex_col()
                    .items_center()
                    .justify_center()
                    .gap_4()
                    .p_8()
                    .rounded_xl()
                    .bg(theme::bg_panel())
                    .border_1()
                    .border_color(theme::border_color())
                    .max_w(px(520.0))
                    .child(
                        div()
                            .flex()
                            .items_center()
                            .justify_center()
                            .size(px(64.0))
                            .rounded_full()
                            .bg(theme::bg_subtle())
                            .child(Icon::new(IconName::Gamepad2).size_8().text_color(theme::text_muted())),
                    )
                    .child(
                        div()
                            .text_xl()
                            .font_weight(FontWeight::BOLD)
                            .text_color(theme::text_primary())
                            .child("No Games Installed"),
                    )
                    .child(
                        div()
                            .text_sm()
                            .text_center()
                            .text_color(theme::text_muted())
                            .child("No game handlers installed. Switch to Hub or Desktop Mode to download handlers."),
                    ),
            )
            .into_any_element();
    }

    let filtered_handlers = win
        .carousel_state
        .filter_and_sort_handlers(&win.handlers, &win.carousel_state.search_query);

    let total_matching = filtered_handlers.len();
    let action_bar = render_carousel_action_bar(win, total_matching, cx);

    if total_matching == 0 {
        return div()
            .flex()
            .flex_col()
            .items_center()
            .justify_between()
            .size_full()
            .py_6()
            .px_8()
            .child(action_bar)
            .child(
                div()
                    .flex()
                    .flex_col()
                    .items_center()
                    .justify_center()
                    .gap_4()
                    .p_8()
                    .rounded_xl()
                    .bg(theme::bg_panel())
                    .border_1()
                    .border_color(theme::border_color())
                    .max_w(px(520.0))
                    .child(
                        div()
                            .flex()
                            .items_center()
                            .justify_center()
                            .size(px(64.0))
                            .rounded_full()
                            .bg(theme::bg_subtle())
                            .child(Icon::new(IconName::ListFilter).size_8().text_color(theme::text_muted())),
                    )
                    .child(
                        div()
                            .text_xl()
                            .font_weight(FontWeight::BOLD)
                            .text_color(theme::text_primary())
                            .child("No Matching Games"),
                    )
                    .child(
                        div()
                            .text_sm()
                            .text_center()
                            .text_color(theme::text_muted())
                            .child("No games match the current filter or search criteria. Try cycling the filter or resetting."),
                    )
                    .child(
                        CouchButton::secondary("btn-reset-carousel-filters", "Reset Filters")
                            .size(CouchButtonSize::Md)
                            .on_click(cx.listener(|this, _, _, cx| {
                                this.carousel_state.filter_mode = CarouselFilterMode::All;
                                this.carousel_state.sort_mode = CarouselSortMode::Alphabetical;
                                this.carousel_state.search_query.clear();
                                let total = this.carousel_state.filter_and_sort_handlers(&this.handlers, &this.carousel_state.search_query).len();
                                this.carousel_state.set_count(total);
                                cx.notify();
                            })),
                    ),
            )
            .child(div().h(px(52.0)))
            .into_any_element();
    }

    let sel_idx = win
        .carousel_state
        .selected_index
        .min(total_matching.saturating_sub(1));
    let selected_handler = filtered_handlers[sel_idx];

    let visible_indices = get_carousel_window_indices(sel_idx, total_matching);

    div()
        .flex()
        .flex_col()
        .items_center()
        .justify_between()
        .size_full()
        .py_6()
        .px_8()
        // Top Action Bar with Search, Filter, Sort, and Index Indicator
        .child(action_bar)
        // Middle horizontal card row with left/right buttons
        .child(
            div()
                .flex()
                .flex_row()
                .items_center()
                .justify_center()
                .gap_6()
                .w_full()
                .py_2()
                // Left Arrow Button
                .child(
                    CouchButton::ghost("btn-carousel-left", "◀")
                        .size(CouchButtonSize::Lg)
                        .on_click(cx.listener(|this, _, _, cx| {
                            let total = this
                                .carousel_state
                                .filter_and_sort_handlers(
                                    &this.handlers,
                                    &this.carousel_state.search_query,
                                )
                                .len();
                            this.carousel_state.set_count(total);
                            this.carousel_state.move_left();
                            cx.notify();
                        })),
                )
                // Cards Container
                .child(
                    div()
                        .flex()
                        .flex_row()
                        .items_center()
                        .justify_center()
                        .gap_5()
                        .children(visible_indices.iter().map(|&idx| {
                            let handler = filtered_handlers[idx];
                            let is_selected = idx == sel_idx;
                            render_carousel_card(handler, idx, is_selected, cx)
                        })),
                )
                // Right Arrow Button
                .child(
                    CouchButton::ghost("btn-carousel-right", "▶")
                        .size(CouchButtonSize::Lg)
                        .on_click(cx.listener(|this, _, _, cx| {
                            let total = this
                                .carousel_state
                                .filter_and_sort_handlers(
                                    &this.handlers,
                                    &this.carousel_state.search_query,
                                )
                                .len();
                            this.carousel_state.set_count(total);
                            this.carousel_state.move_right();
                            cx.notify();
                        })),
                ),
        )
        // Bottom Metadata Strip for Selected Game
        .child(render_selected_game_metadata(selected_handler, sel_idx, cx))
        .into_any_element()
}

/// Renders the action strip with Search, Filter, Sort, and Index indicator at the top of the carousel library.
fn render_carousel_action_bar(
    win: &MainWindow,
    total_matching: usize,
    cx: &mut Context<MainWindow>,
) -> AnyElement {
    let sel_idx = win
        .carousel_state
        .selected_index
        .min(total_matching.saturating_sub(1));
    let has_search = !win.carousel_state.search_query.is_empty();
    let is_filtered = win.carousel_state.filter_mode != CarouselFilterMode::All;
    let is_sorted = win.carousel_state.sort_mode != CarouselSortMode::Alphabetical;

    let search_label = if has_search {
        format!("Search: \"{}\"", win.carousel_state.search_query)
    } else {
        "Search".to_string()
    };

    div()
        .id("carousel-action-bar")
        .relative()
        .flex()
        .items_center()
        .justify_center()
        .gap_3()
        .min_w_0()
        // 1. Search Pill + Popover
        .child(
            div()
                .relative()
                .child(
                    div()
                        .id("btn-carousel-search")
                        .cursor_pointer()
                        .flex()
                        .items_center()
                        .gap_2()
                        .px_3p5()
                        .py_1p5()
                        .rounded_full()
                        .min_w_0()
                        .bg(if has_search || win.carousel_state.search_input_open {
                            theme::accent_subtle()
                        } else {
                            theme::bg_panel()
                        })
                        .border_1()
                        .border_color(if has_search || win.carousel_state.search_input_open {
                            theme::accent()
                        } else {
                            theme::border_subtle()
                        })
                        .text_color(if has_search || win.carousel_state.search_input_open {
                            theme::accent()
                        } else {
                            theme::text_primary()
                        })
                        .hover(|s| s.bg(theme::bg_hover()).border_color(theme::border_color()))
                        .on_mouse_down(MouseButton::Left, |_, _, cx| {
                            cx.stop_propagation();
                        })
                        .on_click(cx.listener(|this, _, _, cx| {
                            cx.stop_propagation();
                            this.carousel_state.search_input_open =
                                !this.carousel_state.search_input_open;
                            this.carousel_state.filter_picker_open = false;
                            this.carousel_state.sort_picker_open = false;
                            cx.notify();
                        }))
                        .child(Icon::new(IconName::Search).size_3p5().text_color(
                            if has_search || win.carousel_state.search_input_open {
                                theme::accent()
                            } else {
                                theme::text_muted()
                            },
                        ))
                        .child(
                            div()
                                .text_xs()
                                .font_weight(FontWeight::MEDIUM)
                                .min_w_0()
                                .child(search_label),
                        )
                        .children(if has_search {
                            Some(
                                div()
                                    .id("btn-clear-search-pill")
                                    .cursor_pointer()
                                    .text_xs()
                                    .text_color(theme::text_muted())
                                    .hover(|s| s.text_color(theme::danger()))
                                    .on_mouse_down(MouseButton::Left, |_, _, cx| {
                                        cx.stop_propagation();
                                    })
                                    .on_click(cx.listener(|this, _, _, cx| {
                                        cx.stop_propagation();
                                        this.carousel_state.search_query.clear();
                                        let total = this
                                            .carousel_state
                                            .filter_and_sort_handlers(
                                                &this.handlers,
                                                &this.carousel_state.search_query,
                                            )
                                            .len();
                                        this.carousel_state.set_count(total);
                                        cx.notify();
                                    }))
                                    .child(Icon::new(IconName::X).size_3())
                                    .into_any_element(),
                            )
                        } else {
                            None
                        }),
                )
                .when(win.carousel_state.search_input_open, |popover_anchor| {
                    popover_anchor.child(
                        gpui::deferred(
                            div()
                                .id("carousel-search-popover")
                                .occlude()
                                .on_mouse_down(MouseButton::Left, |_, _, cx| {
                                    cx.stop_propagation();
                                })
                                .on_mouse_down_out(cx.listener(|this, _, _, cx| {
                                    this.carousel_state.search_input_open = false;
                                    cx.notify();
                                }))
                                .absolute()
                                .top(px(40.0))
                                .left_0()
                                .flex()
                                .flex_col()
                                .min_w(px(280.0))
                                .bg(theme::bg_panel())
                                .border_1()
                                .border_color(theme::accent())
                                .rounded_lg()
                                .shadow_lg()
                                .p_3()
                                .gap_2()
                                .child(
                                    div()
                                        .flex()
                                        .items_center()
                                        .justify_between()
                                        .child(
                                            div()
                                                .text_xs()
                                                .font_weight(FontWeight::BOLD)
                                                .text_color(theme::text_primary())
                                                .child("Search Library"),
                                        )
                                        .child(
                                            div()
                                                .id("btn-close-search-popover")
                                                .cursor_pointer()
                                                .flex()
                                                .items_center()
                                                .gap_1()
                                                .text_xs()
                                                .text_color(theme::text_muted())
                                                .hover(|s| s.text_color(theme::danger()))
                                                .on_mouse_down(MouseButton::Left, |_, _, cx| {
                                                    cx.stop_propagation();
                                                })
                                                .on_click(cx.listener(|this, _, _, cx| {
                                                    cx.stop_propagation();
                                                    this.carousel_state.search_input_open = false;
                                                    cx.notify();
                                                }))
                                                .child(Icon::new(IconName::X).size_3())
                                                .child("Close"),
                                        ),
                                )
                                .child(
                                    div()
                                        .flex()
                                        .items_center()
                                        .gap_2()
                                        .px_2p5()
                                        .py_1p5()
                                        .bg(theme::bg_main())
                                        .border_1()
                                        .border_color(theme::border_color())
                                        .rounded_md()
                                        .child(
                                            Icon::new(IconName::Search)
                                                .size_3p5()
                                                .text_color(theme::accent()),
                                        )
                                        .child(
                                            div()
                                                .text_xs()
                                                .text_color(if has_search {
                                                    theme::text_primary()
                                                } else {
                                                    theme::text_muted()
                                                })
                                                .child(if has_search {
                                                    format!("{} █", win.carousel_state.search_query)
                                                } else {
                                                    "Type query or click preset below..."
                                                        .to_string()
                                                }),
                                        ),
                                )
                                .child(
                                    div().flex().flex_wrap().gap_1p5().pt_1().children(
                                        [
                                            "Left 4 Dead",
                                            "Portal",
                                            "Borderlands",
                                            "Halo",
                                            "Resident Evil",
                                        ]
                                        .iter()
                                        .enumerate()
                                        .map(
                                            |(idx, &preset)| {
                                                div()
                                                    .id(("search-preset", idx))
                                                    .cursor_pointer()
                                                    .px_2()
                                                    .py_1()
                                                    .rounded_full()
                                                    .bg(theme::bg_subtle())
                                                    .border_1()
                                                    .border_color(theme::border_color())
                                                    .text_xs()
                                                    .text_color(theme::text_muted())
                                                    .hover(|s| {
                                                        s.bg(theme::accent().opacity(0.15))
                                                            .text_color(theme::accent())
                                                            .border_color(theme::accent())
                                                    })
                                                    .on_mouse_down(MouseButton::Left, |_, _, cx| {
                                                        cx.stop_propagation();
                                                    })
                                                    .on_click(cx.listener(move |this, _, _, cx| {
                                                        cx.stop_propagation();
                                                        this.carousel_state.search_query =
                                                            preset.to_string();
                                                        this.carousel_state.search_input_open =
                                                            false;
                                                        let total = this
                                                            .carousel_state
                                                            .filter_and_sort_handlers(
                                                                &this.handlers,
                                                                &this.carousel_state.search_query,
                                                            )
                                                            .len();
                                                        this.carousel_state.set_count(total);
                                                        cx.notify();
                                                    }))
                                                    .child(preset)
                                            },
                                        ),
                                    ),
                                ),
                        )
                        .with_priority(10),
                    )
                }),
        )
        // 2. Filter Pill + Popover Options
        .child(
            div()
                .relative()
                .child(
                    div()
                        .id("btn-carousel-filter")
                        .cursor_pointer()
                        .flex()
                        .items_center()
                        .gap_2()
                        .px_3p5()
                        .py_1p5()
                        .rounded_full()
                        .min_w_0()
                        .bg(if is_filtered || win.carousel_state.filter_picker_open {
                            theme::accent_subtle()
                        } else {
                            theme::bg_panel()
                        })
                        .border_1()
                        .border_color(if is_filtered || win.carousel_state.filter_picker_open {
                            theme::accent()
                        } else {
                            theme::border_subtle()
                        })
                        .text_color(if is_filtered || win.carousel_state.filter_picker_open {
                            theme::accent()
                        } else {
                            theme::text_primary()
                        })
                        .hover(|s| s.bg(theme::bg_hover()).border_color(theme::border_color()))
                        .on_mouse_down(MouseButton::Left, |_, _, cx| {
                            cx.stop_propagation();
                        })
                        .on_click(cx.listener(|this, _, _, cx| {
                            cx.stop_propagation();
                            this.carousel_state.filter_picker_open =
                                !this.carousel_state.filter_picker_open;
                            this.carousel_state.sort_picker_open = false;
                            this.carousel_state.search_input_open = false;
                            cx.notify();
                        }))
                        .child(Icon::new(IconName::ListFilter).size_3p5().text_color(
                            if is_filtered || win.carousel_state.filter_picker_open {
                                theme::accent()
                            } else {
                                theme::text_muted()
                            },
                        ))
                        .child(
                            div()
                                .text_xs()
                                .font_weight(FontWeight::MEDIUM)
                                .min_w_0()
                                .child(format!(
                                    "Filter: {}",
                                    win.carousel_state.filter_mode.label()
                                )),
                        )
                        .child(div().text_xs().text_color(theme::accent()).child(
                            if win.carousel_state.filter_picker_open {
                                "▲"
                            } else {
                                "▼"
                            },
                        )),
                )
                .when(win.carousel_state.filter_picker_open, |popover_anchor| {
                    popover_anchor.child(
                        gpui::deferred(
                            div()
                                .id("carousel-filter-popover")
                                .occlude()
                                .on_mouse_down(MouseButton::Left, |_, _, cx| {
                                    cx.stop_propagation();
                                })
                                .on_mouse_down_out(cx.listener(|this, _, _, cx| {
                                    this.carousel_state.filter_picker_open = false;
                                    cx.notify();
                                }))
                                .absolute()
                                .top(px(40.0))
                                .left_0()
                                .flex()
                                .flex_col()
                                .min_w(px(240.0))
                                .bg(theme::bg_panel())
                                .border_1()
                                .border_color(theme::border_color())
                                .rounded_lg()
                                .shadow_lg()
                                .p_2()
                                .gap_1()
                                .children(CarouselFilterMode::ALL.iter().enumerate().map(
                                    |(idx, &mode)| {
                                        let is_active = win.carousel_state.filter_mode == mode;
                                        div()
                                            .id(("filter-opt", idx))
                                            .cursor_pointer()
                                            .flex()
                                            .items_center()
                                            .justify_between()
                                            .px_3()
                                            .py_2()
                                            .rounded_md()
                                            .text_xs()
                                            .when(is_active, |s| {
                                                s.bg(theme::accent().opacity(0.15))
                                                    .text_color(theme::accent())
                                                    .font_weight(FontWeight::BOLD)
                                            })
                                            .when(!is_active, |s| {
                                                s.text_color(theme::text_primary())
                                                    .hover(|h| h.bg(theme::bg_hover()))
                                            })
                                            .on_mouse_down(MouseButton::Left, |_, _, cx| {
                                                cx.stop_propagation();
                                            })
                                            .on_click(cx.listener(move |this, _, _, cx| {
                                                cx.stop_propagation();
                                                this.carousel_state.filter_mode = mode;
                                                this.carousel_state.filter_picker_open = false;
                                                let total = this
                                                    .carousel_state
                                                    .filter_and_sort_handlers(
                                                        &this.handlers,
                                                        &this.carousel_state.search_query,
                                                    )
                                                    .len();
                                                this.carousel_state.set_count(total);
                                                cx.notify();
                                            }))
                                            .child(mode.label())
                                            .when(is_active, |el| {
                                                el.child(
                                                    Icon::new(IconName::Check)
                                                        .size_3p5()
                                                        .text_color(theme::accent()),
                                                )
                                            })
                                    },
                                )),
                        )
                        .with_priority(10),
                    )
                }),
        )
        // 3. Sort Pill + Popover Options
        .child(
            div()
                .relative()
                .child(
                    div()
                        .id("btn-carousel-sort")
                        .cursor_pointer()
                        .flex()
                        .items_center()
                        .gap_2()
                        .px_3p5()
                        .py_1p5()
                        .rounded_full()
                        .min_w_0()
                        .bg(if is_sorted || win.carousel_state.sort_picker_open {
                            theme::accent_subtle()
                        } else {
                            theme::bg_panel()
                        })
                        .border_1()
                        .border_color(if is_sorted || win.carousel_state.sort_picker_open {
                            theme::accent()
                        } else {
                            theme::border_subtle()
                        })
                        .text_color(if is_sorted || win.carousel_state.sort_picker_open {
                            theme::accent()
                        } else {
                            theme::text_primary()
                        })
                        .hover(|s| s.bg(theme::bg_hover()).border_color(theme::border_color()))
                        .on_mouse_down(MouseButton::Left, |_, _, cx| {
                            cx.stop_propagation();
                        })
                        .on_click(cx.listener(|this, _, _, cx| {
                            cx.stop_propagation();
                            this.carousel_state.sort_picker_open =
                                !this.carousel_state.sort_picker_open;
                            this.carousel_state.filter_picker_open = false;
                            this.carousel_state.search_input_open = false;
                            cx.notify();
                        }))
                        .child(Icon::new(IconName::ArrowUpDown).size_3p5().text_color(
                            if is_sorted || win.carousel_state.sort_picker_open {
                                theme::accent()
                            } else {
                                theme::text_muted()
                            },
                        ))
                        .child(
                            div()
                                .text_xs()
                                .font_weight(FontWeight::MEDIUM)
                                .min_w_0()
                                .child(format!("Sort: {}", win.carousel_state.sort_mode.label())),
                        )
                        .child(div().text_xs().text_color(theme::accent()).child(
                            if win.carousel_state.sort_picker_open {
                                "▲"
                            } else {
                                "▼"
                            },
                        )),
                )
                .when(win.carousel_state.sort_picker_open, |popover_anchor| {
                    popover_anchor.child(
                        gpui::deferred(
                            div()
                                .id("carousel-sort-popover")
                                .occlude()
                                .on_mouse_down(MouseButton::Left, |_, _, cx| {
                                    cx.stop_propagation();
                                })
                                .on_mouse_down_out(cx.listener(|this, _, _, cx| {
                                    this.carousel_state.sort_picker_open = false;
                                    cx.notify();
                                }))
                                .absolute()
                                .top(px(40.0))
                                .left_0()
                                .flex()
                                .flex_col()
                                .min_w(px(220.0))
                                .bg(theme::bg_panel())
                                .border_1()
                                .border_color(theme::border_color())
                                .rounded_lg()
                                .shadow_lg()
                                .p_2()
                                .gap_1()
                                .children(CarouselSortMode::ALL.iter().enumerate().map(
                                    |(idx, &mode)| {
                                        let is_active = win.carousel_state.sort_mode == mode;
                                        div()
                                            .id(("sort-opt", idx))
                                            .cursor_pointer()
                                            .flex()
                                            .items_center()
                                            .justify_between()
                                            .px_3()
                                            .py_2()
                                            .rounded_md()
                                            .text_xs()
                                            .when(is_active, |s| {
                                                s.bg(theme::accent().opacity(0.15))
                                                    .text_color(theme::accent())
                                                    .font_weight(FontWeight::BOLD)
                                            })
                                            .when(!is_active, |s| {
                                                s.text_color(theme::text_primary())
                                                    .hover(|h| h.bg(theme::bg_hover()))
                                            })
                                            .on_mouse_down(MouseButton::Left, |_, _, cx| {
                                                cx.stop_propagation();
                                            })
                                            .on_click(cx.listener(move |this, _, _, cx| {
                                                cx.stop_propagation();
                                                this.carousel_state.sort_mode = mode;
                                                this.carousel_state.sort_picker_open = false;
                                                let total = this
                                                    .carousel_state
                                                    .filter_and_sort_handlers(
                                                        &this.handlers,
                                                        &this.carousel_state.search_query,
                                                    )
                                                    .len();
                                                this.carousel_state.set_count(total);
                                                cx.notify();
                                            }))
                                            .child(mode.label())
                                            .when(is_active, |el| {
                                                el.child(
                                                    Icon::new(IconName::Check)
                                                        .size_3p5()
                                                        .text_color(theme::accent()),
                                                )
                                            })
                                    },
                                )),
                        )
                        .with_priority(10),
                    )
                }),
        )
        // 4. Index Counter Indicator
        .child(
            div()
                .id("carousel-index-counter")
                .flex()
                .items_center()
                .gap_1p5()
                .px_3p5()
                .py_1p5()
                .rounded_full()
                .min_w_0()
                .bg(theme::bg_panel())
                .border_1()
                .border_color(theme::border_subtle())
                .child(
                    div()
                        .text_xs()
                        .font_weight(FontWeight::BOLD)
                        .text_color(theme::accent())
                        .child(if total_matching > 0 {
                            format!("{}", sel_idx + 1)
                        } else {
                            "0".to_string()
                        }),
                )
                .child(
                    div()
                        .text_xs()
                        .text_color(theme::text_muted())
                        .child(format!("of {}", total_matching)),
                ),
        )
        .into_any_element()
}

/// Returns the visual opacity for a carousel game card based on installation status and selection state.
pub fn carousel_card_opacity(is_installed: bool, is_selected: bool) -> f32 {
    if !is_installed {
        if is_selected { 0.60 } else { 0.40 }
    } else if !is_selected {
        0.70
    } else {
        1.0
    }
}

/// Prominent badge text displayed on uninstalled carousel cards.
pub const UNINSTALLED_BADGE_TEXT: &str = "Uninstalled (.exe missing)";

/// Callout text displayed in the metadata strip when a selected game's executable is missing.
pub const MISSING_EXE_METADATA_CALLOUT: &str = "Executable Missing - Press (A) to Locate .exe";

fn render_carousel_card(
    handler: &HandlerItem,
    idx: usize,
    is_selected: bool,
    cx: &mut Context<MainWindow>,
) -> AnyElement {
    // Sizing: IGDB 600x900 aspect ratio (2:3)
    let (w, h) = if is_selected {
        (px(240.0), px(360.0))
    } else {
        (px(200.0), px(300.0))
    };

    // Prominent amber/warning status badge on top right of uninstalled cards
    let status_badge: Option<AnyElement> = if !handler.is_installed {
        Some(
            div()
                .absolute()
                .top_3()
                .right_3()
                .child(CouchTag::new("Uninstalled").tone(TagTone::Warning))
                .into_any_element(),
        )
    } else {
        None
    };

    let cover = GameCover::new(handler.game_name.clone(), w, h)
        .cover_url(handler.cover_image_url.clone())
        .installed(handler.is_installed)
        .selected(is_selected);

    let cover = if let Some(badge) = status_badge {
        cover.badge(badge)
    } else {
        cover
    };

    let card_element = div()
        .id(("carousel-card", idx))
        .cursor_pointer()
        .on_click(cx.listener(move |this, _, _, cx| {
            this.select_carousel_game(idx, cx);
        }))
        .child(cover)
        .into_any_element();

    if is_selected {
        div()
            .p(px(3.0))
            .rounded_2xl()
            .border_2()
            .border_color(theme::accent())
            .shadow_lg()
            .bg(theme::accent().opacity(0.12))
            .child(card_element)
            .into_any_element()
    } else {
        card_element
    }
}

fn render_selected_game_metadata(
    handler: &HandlerItem,
    sel_idx: usize,
    cx: &mut Context<MainWindow>,
) -> AnyElement {
    let player_badge = if handler.max_players > 1 {
        format!("1–{} Players", handler.max_players)
    } else {
        "1 Player".to_string()
    };

    let input_badge =
        if handler.supports_gamepad || handler.supports_xinput || handler.supports_dinput {
            "Gamepad Supported"
        } else if handler.supports_keyboard {
            "Keyboard & Mouse"
        } else {
            "Controller"
        };

    let play_time_text = if handler.play_time_seconds > 0 {
        let hours = handler.play_time_seconds / 3600;
        let mins = (handler.play_time_seconds % 3600) / 60;
        if hours > 0 {
            format!("{}h {}m played", hours, mins)
        } else {
            format!("{}m played", mins)
        }
    } else {
        "Never played".to_string()
    };

    div()
        .flex()
        .flex_col()
        .items_center()
        .gap_3()
        .max_w(px(700.0))
        // Game Title
        .child(
            div()
                .text_2xl()
                .font_weight(FontWeight::BOLD)
                .text_color(theme::text_primary())
                .text_center()
                .child(handler.game_name.clone()),
        )
        // Badges Row
        .child(
            div()
                .flex()
                .flex_row()
                .items_center()
                .justify_center()
                .flex_wrap()
                .gap_2p5()
                // Player Count Badge
                .child(render_metadata_badge(
                    IconName::Users,
                    &player_badge,
                    theme::accent(),
                ))
                // Input Mode Badge
                .child(render_metadata_badge(
                    IconName::Gamepad2,
                    input_badge,
                    theme::accent(),
                ))
                // Play Time Badge
                .child(render_metadata_badge(
                    IconName::Clock,
                    &play_time_text,
                    theme::text_muted(),
                ))
                // Status Badge / Missing Executable Callout
                .child(
                    div()
                        .flex()
                        .items_center()
                        .gap_1p5()
                        .px_3()
                        .py_1()
                        .rounded_full()
                        .bg(if handler.is_installed {
                            theme::success_subtle()
                        } else {
                            theme::warning_subtle()
                        })
                        .border_1()
                        .border_color(if handler.is_installed {
                            theme::success().opacity(0.4)
                        } else {
                            theme::warning().opacity(0.4)
                        })
                        .text_xs()
                        .font_weight(FontWeight::BOLD)
                        .text_color(if handler.is_installed {
                            theme::success()
                        } else {
                            theme::warning()
                        })
                        .child(if handler.is_installed {
                            "Ready to Play"
                        } else {
                            MISSING_EXE_METADATA_CALLOUT
                        }),
                ),
        )
        // Action Button: Select Game / Locate .exe
        .child(
            CouchButton::primary("btn-carousel-select-game", "Configure / Launch Splitscreen")
                .size(CouchButtonSize::Lg)
                .on_click(cx.listener(move |this, _, _, cx| {
                    this.select_carousel_game(sel_idx, cx);
                })),
        )
        .into_any_element()
}

fn render_metadata_badge(icon: IconName, label: &str, icon_color: Hsla) -> AnyElement {
    div()
        .flex()
        .items_center()
        .gap_1p5()
        .px_3()
        .py_1()
        .rounded_full()
        .bg(theme::bg_panel())
        .border_1()
        .border_color(theme::border_color())
        .shadow_sm()
        .child(Icon::new(icon).size_3p5().text_color(icon_color))
        .child(
            div()
                .text_xs()
                .font_weight(FontWeight::MEDIUM)
                .text_color(theme::text_primary())
                .child(label.to_string()),
        )
        .into_any_element()
}

fn render_couch_prompt_bar(screen: BigPictureScreen) -> AnyElement {
    let mut bar = div()
        .id("couch-prompt-bar")
        .flex()
        .items_center()
        .justify_center()
        .gap_8()
        .h(px(52.0))
        .w_full()
        .bg(theme::bg_panel())
        .border_t_1()
        .border_color(theme::border_color());

    match screen {
        BigPictureScreen::CouchSetup => {
            bar = bar
                .child(render_couch_prompt("A", "Grab / Swap Slot"))
                .child(render_couch_prompt("X", "Remove Controller"))
                .child(render_couch_prompt("Y", "Ignore Controller"))
                .child(render_couch_prompt("Start", "Launch Session"))
                .child(render_couch_prompt("LB / RB", "Switch Tab"))
                .child(render_couch_prompt("B", "Back to Library"));
        }
        BigPictureScreen::CouchHub => {
            bar = bar
                .child(render_couch_prompt("A", "Download & Play"))
                .child(render_couch_prompt("LB / RB", "Switch Tab"))
                .child(render_couch_prompt("B", "Back to Library Carousel"))
                .child(render_couch_prompt("F11 / Mode", "Desktop Mode"));
        }
        BigPictureScreen::Carousel => {
            bar = bar
                .child(render_couch_prompt("A", "Select Game"))
                .child(render_couch_prompt("LB / RB", "Switch Tab"))
                .child(render_couch_prompt("Start", "Quick Launch"))
                .child(render_couch_prompt("F11 / Mode", "Desktop Mode"));
        }
    }

    bar.into_any_element()
}

fn render_couch_prompt(btn: &str, action: &str) -> AnyElement {
    div()
        .flex()
        .items_center()
        .gap_2()
        .child(
            div()
                .flex()
                .items_center()
                .justify_center()
                .px_2()
                .py_0p5()
                .rounded_md()
                .bg(theme::bg_panel())
                .border_1()
                .border_color(theme::border_color())
                .shadow_sm()
                .text_xs()
                .font_weight(FontWeight::BOLD)
                .text_color(theme::accent())
                .child(btn.to_string()),
        )
        .child(
            div()
                .text_xs()
                .font_weight(FontWeight::MEDIUM)
                .text_color(theme::text_muted())
                .child(action.to_string()),
        )
        .into_any_element()
}
