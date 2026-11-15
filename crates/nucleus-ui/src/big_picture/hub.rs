//! 10-foot Couch Community Hub browsing, 1-click install, and custom options modal.
//!
//! Provides a gamepad-navigable 10-foot television interface for discovering,
//! downloading, and immediately launching Splitscreen.Me community game handlers,
//! with missing executable fallback prompts and interactive custom options dialogs.

use gpui::prelude::*;
use gpui::*;
use std::collections::HashMap;
use std::path::PathBuf;

use crate::models::HubPackageItem;
use crate::theme;
use crate::window::MainWindow;
use crate::{Icon, IconName};
use crate::components::{CouchButton, CouchButtonSize, CouchTag, TagTone, *};

/// Result of checking whether a downloaded game handler's executable is resolved on disk.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum PathResolution {
    /// Executable exists at the specified disk path.
    Found(PathBuf),
    /// Executable was not detected in standard library paths; prompt user to switch to Desktop mode.
    PromptDesktopMode,
}

/// Checks whether the candidate executable path exists; if missing or None, prompts desktop mode.
pub fn resolve_game_path_or_couch_prompt(candidate: Option<PathBuf>) -> PathResolution {
    match candidate {
        Some(p) if p.exists() => PathResolution::Found(p),
        _ => PathResolution::PromptDesktopMode,
    }
}

/// Dynamic setting option row for the gamepad-navigable handler custom options modal.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CouchOptionItem {
    /// Configuration key identifier (e.g. "gamemode", "dx_version").
    pub key: String,
    /// Human-readable option title.
    pub title: String,
    /// Explanatory description.
    pub description: String,
    /// Ordered list of candidate values.
    pub choices: Vec<String>,
    /// Index of currently selected choice.
    pub selected_choice: usize,
}

impl CouchOptionItem {
    /// Creates a new `CouchOptionItem`.
    pub fn new(
        key: impl Into<String>,
        title: impl Into<String>,
        description: impl Into<String>,
        choices: Vec<String>,
    ) -> Self {
        Self {
            key: key.into(),
            title: title.into(),
            description: description.into(),
            choices,
            selected_choice: 0,
        }
    }
}

impl From<&nucleus_script::HandlerOption> for CouchOptionItem {
    fn from(opt: &nucleus_script::HandlerOption) -> Self {
        let choices = if opt.values.is_empty() {
            vec!["Enabled".to_string()]
        } else {
            opt.values.clone()
        };
        Self {
            key: opt.key.clone(),
            title: if opt.name.is_empty() {
                opt.key.clone()
            } else {
                opt.name.clone()
            },
            description: opt.description.clone(),
            choices,
            selected_choice: 0,
        }
    }
}

/// State for the handler custom settings modal dialog.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CouchOptionsModalState {
    /// Official game title.
    pub game_title: String,
    /// List of dynamic configurable options.
    pub options: Vec<CouchOptionItem>,
    /// Index of option row currently focused by gamepad/keyboard.
    pub focused_option_idx: usize,
}

impl CouchOptionsModalState {
    /// Creates a new `CouchOptionsModalState`.
    pub fn new(game_title: impl Into<String>, options: Vec<CouchOptionItem>) -> Self {
        Self {
            game_title: game_title.into(),
            options,
            focused_option_idx: 0,
        }
    }

    /// Moves focus to the previous option row, wrapping around.
    pub fn move_up(&mut self) {
        if self.options.is_empty() {
            return;
        }
        if self.focused_option_idx == 0 {
            self.focused_option_idx = self.options.len() - 1;
        } else {
            self.focused_option_idx -= 1;
        }
    }

    /// Moves focus to the next option row, wrapping around.
    pub fn move_down(&mut self) {
        if self.options.is_empty() {
            return;
        }
        if self.focused_option_idx + 1 >= self.options.len() {
            self.focused_option_idx = 0;
        } else {
            self.focused_option_idx += 1;
        }
    }

    /// Cycles the choice of the currently focused option forward or backward.
    pub fn cycle_choice(&mut self, forward: bool) {
        if self.focused_option_idx >= self.options.len() {
            return;
        }
        let opt = &mut self.options[self.focused_option_idx];
        if opt.choices.is_empty() {
            return;
        }
        if forward {
            opt.selected_choice = (opt.selected_choice + 1) % opt.choices.len();
        } else if opt.selected_choice == 0 {
            opt.selected_choice = opt.choices.len() - 1;
        } else {
            opt.selected_choice -= 1;
        }
    }

    /// Exports selected options to a `HashMap<key, value>` for script context consumption.
    pub fn to_hashmap(&self) -> HashMap<String, String> {
        let mut map = HashMap::new();
        for opt in &self.options {
            if let Some(val) = opt.choices.get(opt.selected_choice) {
                map.insert(opt.key.clone(), val.clone());
            } else if let Some(first) = opt.choices.first() {
                map.insert(opt.key.clone(), first.clone());
            }
        }
        map
    }
}

/// State for the 10-foot Couch Community Hub browsing screen.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CouchHubState {
    /// Currently focused package card index (0-based).
    pub selected_index: usize,
    /// Currently active category tab index.
    pub selected_category: usize,
    /// Available category filter tab names.
    pub categories: Vec<&'static str>,
    /// Handler name for the missing executable fallback prompt, if active.
    pub missing_exe_modal_handler: Option<String>,
    /// Active handler custom options modal dialog, if open.
    pub custom_options_modal: Option<CouchOptionsModalState>,
    /// Search query string for filtering community hub packages.
    pub search_query: String,
    /// Whether search popover input is currently open.
    pub search_input_open: bool,
    /// Whether to filter for verified handlers only.
    pub filter_verified: bool,
    /// Whether to filter for gamepad-supported handlers only.
    pub filter_gamepads: bool,
    /// Whether to filter for keyboard/mouse supported handlers only.
    pub filter_km: bool,
}

impl CouchHubState {
    /// Creates a new `CouchHubState` with default categories.
    pub fn new() -> Self {
        Self {
            selected_index: 0,
            selected_category: 0,
            categories: vec!["Popular", "Trending", "Recent", "All"],
            missing_exe_modal_handler: None,
            custom_options_modal: None,
            search_query: String::new(),
            search_input_open: false,
            filter_verified: false,
            filter_gamepads: false,
            filter_km: false,
        }
    }

    /// Moves selection to the left, wrapping around if bounds exceeded.
    pub fn move_left(&mut self, items_count: usize) {
        if items_count == 0 {
            self.selected_index = 0;
            return;
        }
        if self.selected_index == 0 {
            self.selected_index = items_count.saturating_sub(1);
        } else {
            self.selected_index -= 1;
        }
    }

    /// Moves selection to the right, wrapping around if bounds exceeded.
    pub fn move_right(&mut self, items_count: usize) {
        if items_count == 0 {
            self.selected_index = 0;
            return;
        }
        if self.selected_index + 1 >= items_count {
            self.selected_index = 0;
        } else {
            self.selected_index += 1;
        }
    }

    /// Cycles to the next category tab.
    pub fn next_category(&mut self) {
        if self.categories.is_empty() {
            return;
        }
        self.selected_category = (self.selected_category + 1) % self.categories.len();
        self.selected_index = 0;
    }

    /// Cycles to the previous category tab.
    pub fn prev_category(&mut self) {
        if self.categories.is_empty() {
            return;
        }
        if self.selected_category == 0 {
            self.selected_category = self.categories.len() - 1;
        } else {
            self.selected_category -= 1;
        }
        self.selected_index = 0;
    }

    /// Triggers the missing executable prompt for the specified handler.
    pub fn prompt_missing_exe(&mut self, handler_name: &str) {
        self.missing_exe_modal_handler = Some(handler_name.to_string());
    }

    /// Triggers the missing executable fallback prompt for the specified handler.
    pub fn prompt_desktop_mode(&mut self, handler_name: &str) {
        self.prompt_missing_exe(handler_name);
    }

    /// Dismisses the missing executable fallback prompt.
    pub fn dismiss_prompt(&mut self) {
        self.missing_exe_modal_handler = None;
    }
}

impl Default for CouchHubState {
    fn default() -> Self {
        Self::new()
    }
}

/// Returns a sorted slice of package references filtered by active category, search query, and tags.
pub fn get_filtered_packages<'a>(
    packages: &'a [HubPackageItem],
    category_idx: usize,
    search_query: &str,
    verified_only: bool,
    gamepads_only: bool,
    km_only: bool,
) -> Vec<&'a HubPackageItem> {
    let q = search_query.trim().to_lowercase();
    let mut list: Vec<&'a HubPackageItem> = packages
        .iter()
        .filter(|pkg| {
            if !q.is_empty() {
                let matches_title = pkg.title.to_lowercase().contains(&q);
                let matches_game = pkg.game_name.to_lowercase().contains(&q);
                let matches_author = pkg.author.to_lowercase().contains(&q);
                if !matches_title && !matches_game && !matches_author {
                    return false;
                }
            }
            if verified_only && !pkg.is_verified {
                return false;
            }
            if gamepads_only && !pkg.playable_controllers {
                return false;
            }
            if km_only && !pkg.playable_mouse_keyboard && !pkg.playable_multi_mouse_keyboard {
                return false;
            }
            true
        })
        .collect();

    match category_idx {
        0 => {
            // Popular: highest downloads then likes
            list.sort_by(|a, b| b.downloads.cmp(&a.downloads).then_with(|| b.likes.cmp(&a.likes)));
        }
        1 => {
            // Trending: weighted likes + downloads, verified first
            list.sort_by(|a, b| {
                let score_a = (a.likes * 2) as u64 + (a.downloads / 10);
                let score_b = (b.likes * 2) as u64 + (b.downloads / 10);
                b.is_verified.cmp(&a.is_verified).then_with(|| score_b.cmp(&score_a))
            });
        }
        2 => {
            // Recent: updated_at then created_at
            list.sort_by(|a, b| b.updated_at.cmp(&a.updated_at).then_with(|| b.created_at.cmp(&a.created_at)));
        }
        _ => {
            // All: preserve default order
        }
    }
    list
}

/// Renders the 10-foot Couch Community Hub browsing view.
pub fn render_couch_hub_view(win: &MainWindow, cx: &mut Context<MainWindow>) -> AnyElement {
    let filtered_packages = get_filtered_packages(
        &win.hub_packages,
        win.couch_hub_state.selected_category,
        &win.couch_hub_state.search_query,
        win.couch_hub_state.filter_verified,
        win.couch_hub_state.filter_gamepads,
        win.couch_hub_state.filter_km,
    );
    let total_packages = filtered_packages.len();
    let sel_idx = if total_packages == 0 {
        0
    } else {
        win.couch_hub_state.selected_index.min(total_packages - 1)
    };

    let missing_exe_modal = win
        .couch_hub_state
        .missing_exe_modal_handler
        .as_deref()
        .map(|h| render_missing_exe_modal(h, cx));

    let custom_options_modal = win
        .couch_hub_state
        .custom_options_modal
        .as_ref()
        .map(|modal| render_custom_options_modal(modal, cx));

    div()
        .relative()
        .flex()
        .flex_col()
        .items_center()
        .size_full()
        .px_8()
        .py_4()
        .bg(theme::bg_main())
        .text_color(theme::text_primary())
        // Top Header
        .child(render_couch_hub_header())
        // Category Filter Strip
        .child(render_couch_hub_categories(win, cx))
        // Package Cards Grid / Carousel Area
        .child(
            div()
                .flex()
                .flex_col()
                .items_center()
                .justify_center()
                .flex_1()
                .w_full()
                .min_h_0()
                .child(if total_packages == 0 {
                    render_empty_hub_state(win.hub_loading)
                } else {
                    render_package_cards_row(win, &filtered_packages, sel_idx, cx)
                }),
        )
        .children(missing_exe_modal)
        .children(custom_options_modal)
        .into_any_element()
}

fn render_couch_hub_header() -> AnyElement {
    div()
        .id("couch-hub-header")
        .flex()
        .flex_col()
        .items_center()
        .gap_1()
        .pt_2()
        .pb_1()
        .child(
            div()
                .flex()
                .items_center()
                .gap_2p5()
                .child(Icon::new(IconName::Globe).size_6().text_color(theme::accent()))
                .child(
                    div()
                        .text_2xl()
                        .font_weight(FontWeight::BOLD)
                        .text_color(theme::text_primary())
                        .child("Community Game Handlers"),
                ),
        )
        .child(
            div()
                .text_xs()
                .font_weight(FontWeight::MEDIUM)
                .text_color(theme::text_muted())
                .child("[Browse & 1-Click Install Splitscreen Me Handlers]"),
        )
        .into_any_element()
}

fn render_couch_hub_categories(win: &MainWindow, cx: &mut Context<MainWindow>) -> AnyElement {
    let has_search = !win.couch_hub_state.search_query.is_empty();
    let search_label = if has_search {
        format!("Search: \"{}\"", win.couch_hub_state.search_query)
    } else {
        "Search Hub".to_string()
    };

    div()
        .id("couch-hub-categories-bar")
        .flex()
        .flex_wrap()
        .items_center()
        .justify_center()
        .gap_3()
        .py_2()
        // 1. Search Pill + Popover
        .child(
            div()
                .relative()
                .child(
                    div()
                        .id("btn-couch-hub-search")
                        .cursor_pointer()
                        .flex()
                        .items_center()
                        .gap_2()
                        .px_3p5()
                        .py_1p5()
                        .rounded_full()
                        .bg(if has_search || win.couch_hub_state.search_input_open { theme::accent_subtle() } else { theme::bg_panel() })
                        .border_1()
                        .border_color(if has_search || win.couch_hub_state.search_input_open { theme::accent() } else { theme::border_subtle() })
                        .text_color(if has_search || win.couch_hub_state.search_input_open { theme::accent() } else { theme::text_primary() })
                        .hover(|s| s.bg(theme::bg_hover()).border_color(theme::border_color()))
                        .on_mouse_down(MouseButton::Left, |_, _, cx| {
                            cx.stop_propagation();
                        })
                        .on_click(cx.listener(|this, _, _, cx| {
                            cx.stop_propagation();
                            this.couch_hub_state.search_input_open = !this.couch_hub_state.search_input_open;
                            cx.notify();
                        }))
                        .child(Icon::new(IconName::Search).size_3p5().text_color(if has_search || win.couch_hub_state.search_input_open { theme::accent() } else { theme::text_muted() }))
                        .child(div().text_xs().font_weight(FontWeight::MEDIUM).child(search_label))
                        .children(if has_search {
                            Some(
                                div()
                                    .id("btn-clear-couch-hub-search")
                                    .cursor_pointer()
                                    .text_xs()
                                    .text_color(theme::text_muted())
                                    .hover(|s| s.text_color(theme::danger()))
                                    .on_mouse_down(MouseButton::Left, |_, _, cx| {
                                        cx.stop_propagation();
                                    })
                                    .on_click(cx.listener(|this, _, _, cx| {
                                        cx.stop_propagation();
                                        this.couch_hub_state.search_query.clear();
                                        this.couch_hub_state.selected_index = 0;
                                        cx.notify();
                                    }))
                                    .child(Icon::new(IconName::X).size_3())
                                    .into_any_element(),
                            )
                        } else {
                            None
                        }),
                )
                .when(win.couch_hub_state.search_input_open, |popover_anchor| {
                    popover_anchor.child(
                        gpui::deferred(
                            div()
                                .id("couch-hub-search-popover")
                                .occlude()
                                .on_mouse_down(MouseButton::Left, |_, _, cx| {
                                    cx.stop_propagation();
                                })
                                .on_mouse_down_out(cx.listener(|this, _, _, cx| {
                                    this.couch_hub_state.search_input_open = false;
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
                                        .child(div().text_xs().font_weight(FontWeight::BOLD).text_color(theme::text_primary()).child("Search Community Hub"))
                                        .child(
                                            div()
                                                .id("btn-close-couch-hub-search")
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
                                                    this.couch_hub_state.search_input_open = false;
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
                                        .child(Icon::new(IconName::Search).size_3p5().text_color(theme::accent()))
                                        .child(
                                            div()
                                                .text_xs()
                                                .text_color(if has_search { theme::text_primary() } else { theme::text_muted() })
                                                .child(if has_search { format!("{} █", win.couch_hub_state.search_query) } else { "Type query or click preset below...".to_string() }),
                                        ),
                                )
                                .child(
                                    div()
                                        .flex()
                                        .flex_wrap()
                                        .gap_1p5()
                                        .pt_1()
                                        .children(["Left 4 Dead", "Halo", "Borderlands", "Call of Duty", "Portal", "Resident Evil", "Risk of Rain"].iter().enumerate().map(|(idx, &preset)| {
                                            div()
                                                .id(("hub-preset", idx))
                                                .cursor_pointer()
                                                .px_2()
                                                .py_1()
                                                .rounded_full()
                                                .bg(theme::bg_subtle())
                                                .border_1()
                                                .border_color(theme::border_color())
                                                .text_xs()
                                                .text_color(theme::text_muted())
                                                .hover(|s| s.bg(theme::accent().opacity(0.15)).text_color(theme::accent()).border_color(theme::accent()))
                                                .on_mouse_down(MouseButton::Left, |_, _, cx| {
                                                    cx.stop_propagation();
                                                })
                                                .on_click(cx.listener(move |this, _, _, cx| {
                                                    cx.stop_propagation();
                                                    this.couch_hub_state.search_query = preset.to_string();
                                                    this.couch_hub_state.search_input_open = false;
                                                    this.couch_hub_state.selected_index = 0;
                                                    cx.notify();
                                                }))
                                                .child(preset)
                                        }))
                                )
                        )
                        .with_priority(10)
                    )
                }),
        )
        // 2. Categories
        .children(win.couch_hub_state.categories.iter().enumerate().map(|(idx, &name)| {
            let is_active = idx == win.couch_hub_state.selected_category;
            let tab = div()
                .id(("couch-hub-cat-tab", idx))
                .cursor_pointer()
                .px_4()
                .py_1p5()
                .rounded_full()
                .text_xs()
                .font_weight(FontWeight::BOLD)
                .on_click(cx.listener(move |this, _, _, cx| {
                    this.couch_hub_state.selected_category = idx;
                    this.couch_hub_state.selected_index = 0;
                    cx.notify();
                }));

            let styled_tab = if is_active {
                tab.bg(theme::accent())
                    .text_color(theme::text_on_accent())
                    .border_2()
                    .border_color(theme::accent())
                    .shadow_md()
            } else {
                tab.bg(theme::bg_panel())
                    .text_color(theme::text_muted())
                    .border_1()
                    .border_color(theme::border_subtle())
                    .hover(|s| s.bg(theme::bg_hover()).text_color(theme::text_primary()))
            };

            styled_tab.child(name)
        }))
        // 3. Tag Filter Pills (Verified, Controllers, Multi-KB/M)
        .child(
            div()
                .id("couch-hub-tag-verified")
                .cursor_pointer()
                .px_3()
                .py_1p5()
                .rounded_full()
                .text_xs()
                .font_weight(FontWeight::BOLD)
                .when(win.couch_hub_state.filter_verified, |s| s.bg(theme::accent()).text_color(theme::text_on_accent()).border_2().border_color(theme::accent()))
                .when(!win.couch_hub_state.filter_verified, |s| s.bg(theme::bg_panel()).text_color(theme::text_muted()).border_1().border_color(theme::border_subtle()).hover(|h| h.bg(theme::bg_hover()).text_color(theme::text_primary())))
                .on_click(cx.listener(|this, _, _, cx| {
                    this.couch_hub_state.filter_verified = !this.couch_hub_state.filter_verified;
                    this.couch_hub_state.selected_index = 0;
                    cx.notify();
                }))
                .flex()
                .items_center()
                .gap_1()
                .child(Icon::new(IconName::BadgeCheck).size_3p5())
                .child("Verified Only")
        )
        .child(
            div()
                .id("couch-hub-tag-gamepads")
                .cursor_pointer()
                .px_3()
                .py_1p5()
                .rounded_full()
                .text_xs()
                .font_weight(FontWeight::BOLD)
                .when(win.couch_hub_state.filter_gamepads, |s| s.bg(theme::accent()).text_color(theme::text_on_accent()).border_2().border_color(theme::accent()))
                .when(!win.couch_hub_state.filter_gamepads, |s| s.bg(theme::bg_panel()).text_color(theme::text_muted()).border_1().border_color(theme::border_subtle()).hover(|h| h.bg(theme::bg_hover()).text_color(theme::text_primary())))
                .on_click(cx.listener(|this, _, _, cx| {
                    this.couch_hub_state.filter_gamepads = !this.couch_hub_state.filter_gamepads;
                    this.couch_hub_state.selected_index = 0;
                    cx.notify();
                }))
                .flex()
                .items_center()
                .gap_1()
                .child(Icon::new(IconName::Gamepad2).size_3p5())
                .child("Controllers")
        )
        .child(
            div()
                .id("couch-hub-tag-km")
                .cursor_pointer()
                .px_3()
                .py_1p5()
                .rounded_full()
                .text_xs()
                .font_weight(FontWeight::BOLD)
                .when(win.couch_hub_state.filter_km, |s| s.bg(theme::accent()).text_color(theme::text_on_accent()).border_2().border_color(theme::accent()))
                .when(!win.couch_hub_state.filter_km, |s| s.bg(theme::bg_panel()).text_color(theme::text_muted()).border_1().border_color(theme::border_subtle()).hover(|h| h.bg(theme::bg_hover()).text_color(theme::text_primary())))
                .on_click(cx.listener(|this, _, _, cx| {
                    this.couch_hub_state.filter_km = !this.couch_hub_state.filter_km;
                    this.couch_hub_state.selected_index = 0;
                    cx.notify();
                }))
                .flex()
                .items_center()
                .gap_1()
                .child(Icon::new(IconName::Keyboard).size_3p5())
                .child("Multi-KB/M")
        )
        .into_any_element()
}

fn render_empty_hub_state(is_loading: bool) -> AnyElement {
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
                .child(if is_loading {
                    "Loading Community Packages..."
                } else {
                    "No Packages Found"
                }),
        )
        .child(
            div()
                .text_sm()
                .text_center()
                .text_color(theme::text_muted())
                .child(if is_loading {
                    "Fetching verified handlers from Splitscreen.Me Hub..."
                } else {
                    "No packages found matching active filters. Try clearing search or tag filters."
                }),
        )
        .into_any_element()
}

fn render_package_cards_row(
    win: &MainWindow,
    packages: &[&HubPackageItem],
    sel_idx: usize,
    cx: &mut Context<MainWindow>,
) -> AnyElement {
    let total = packages.len();
    if total == 0 {
        return div().into_any_element();
    }

    let window_size = 5.min(total);
    let half_window = window_size / 2;
    let window_start = if sel_idx <= half_window {
        0
    } else if sel_idx + half_window >= total {
        total.saturating_sub(window_size)
    } else {
        sel_idx - half_window
    };
    let window_end = (window_start + window_size).min(total);

    div()
        .id("couch-hub-packages-carousel")
        .flex()
        .flex_row()
        .items_center()
        .justify_center()
        .gap_4()
        .py_4()
        .px_4()
        .w_full()
        // Left Nav Button
        .child(
            CouchButton::ghost("btn-hub-nav-left", "◀")
                .size(CouchButtonSize::Lg)
                .on_click(cx.listener(move |this, _, _, cx| {
                    this.couch_hub_state.move_left(total);
                    cx.notify();
                })),
        )
        // Windowed package cards centered around selected card
        .children((window_start..window_end).map(|idx| {
            let pkg = packages[idx];
            render_package_card(win, pkg, idx, idx == sel_idx, cx)
        }))
        // Right Nav Button
        .child(
            CouchButton::ghost("btn-hub-nav-right", "▶")
                .size(CouchButtonSize::Lg)
                .on_click(cx.listener(move |this, _, _, cx| {
                    this.couch_hub_state.move_right(total);
                    cx.notify();
                })),
        )
        .into_any_element()
}

fn render_package_card(
    win: &MainWindow,
    pkg: &HubPackageItem,
    idx: usize,
    is_selected: bool,
    cx: &mut Context<MainWindow>,
) -> AnyElement {
    let is_installed = pkg.is_installed || win.installed_package_ids.contains(&pkg.id);
    let install_progress = win.installing_handlers.get(&pkg.id).copied();
    let pkg_id = pkg.id.clone();

    let cover_content: AnyElement = if let Some(ref cover_url) = pkg.cover_image_url {
        img(crate::make_image_source(cover_url))
            .absolute()
            .inset_0()
            .size_full()
            .aspect_ratio(2.0 / 3.0)
            .object_fit(gpui::ObjectFit::Cover)
            .rounded_lg()
            .into_any_element()
    } else {
        div()
            .flex()
            .flex_col()
            .items_center()
            .justify_center()
            .size_full()
            .bg(theme::bg_subtle())
            .p_4()
            .child(Icon::new(IconName::Gamepad2).size_10().text_color(theme::text_muted()))
            .child(
                div()
                    .text_xs()
                    .font_weight(FontWeight::BOLD)
                    .text_center()
                    .text_color(theme::text_muted())
                    .child(pkg.game_name.clone()),
            )
            .into_any_element()
    };

    let action_btn = if let Some(p) = install_progress {
        CouchButton::secondary(
            format!("btn-hub-download-{}", pkg.id),
            format!("Downloading {:.0}%", p * 100.0),
        )
        .icon(IconName::Loader)
        .size(CouchButtonSize::Lg)
        .disabled(true)
    } else if is_installed {
        let pid = pkg.id.clone();
        CouchButton::primary(format!("btn-hub-play-{}", pkg.id), "Play (A)")
            .icon(IconName::Play)
            .size(CouchButtonSize::Lg)
            .on_click(cx.listener(move |this, _, _, cx| {
                this.couch_hub_state.selected_index = idx;
                this.download_and_play_couch_package(&pid, cx);
            }))
    } else {
        let pid = pkg.id.clone();
        CouchButton::secondary(
            format!("btn-hub-install-{}", pkg.id),
            "Download & Play (A)",
        )
        .icon(IconName::Download)
        .size(CouchButtonSize::Lg)
        .on_click(cx.listener(move |this, _, _, cx| {
            this.couch_hub_state.selected_index = idx;
            this.download_and_play_couch_package(&pid, cx);
        }))
    };

    let card_border = if is_selected {
        theme::accent()
    } else {
        theme::border_color()
    };

    let card_inner = div()
        .id(("couch-hub-package-card", idx))
        .cursor_pointer()
        .w(px(220.0))
        .h(px(330.0))
        .flex()
        .flex_col()
        .justify_between()
        .rounded_xl()
        .bg(theme::bg_panel())
        .border_2()
        .border_color(card_border)
        .p_3()
        .shadow_lg()
        .on_click(cx.listener({
            let pid = pkg_id.clone();
            move |this, _, _, cx| {
                this.couch_hub_state.selected_index = idx;
                this.download_and_play_couch_package(&pid, cx);
            }
        }))
        // Top Image & Badges
        .child(
            div()
                .relative()
                .w_full()
                .h(px(180.0))
                .rounded_lg()
                .overflow_hidden()
                .child(cover_content)
                // Verified Badge
                .children(if pkg.is_verified {
                    Some(
                        div()
                            .absolute()
                            .top_2()
                            .left_2()
                            .child(CouchTag::new("Verified").icon(IconName::BadgeCheck).tone(TagTone::Success)),
                    )
                } else {
                    None
                })
                // Downloads Badge
                .child(
                    div()
                        .absolute()
                        .bottom_2()
                        .right_2()
                        .px_2()
                        .py_0p5()
                        .rounded_full()
                        .bg(theme::bg_dark().opacity(0.8))
                        .text_color(theme::text_primary())
                        .text_xs()
                        .font_weight(FontWeight::MEDIUM)
                        .flex()
                        .items_center()
                        .gap_1()
                        .child(Icon::new(IconName::Download).size_3())
                        .child(format!("{}", pkg.downloads)),
                ),
        )
        // Middle Details
        .child(
            div()
                .flex()
                .flex_col()
                .gap_0p5()
                .px_1()
                .child(
                    div()
                        .text_sm()
                        .font_weight(FontWeight::BOLD)
                        .text_color(theme::text_primary())
                        .line_clamp(1)
                        .child(pkg.title.clone()),
                )
                .child(
                    div()
                        .text_xs()
                        .text_color(theme::text_muted())
                        .line_clamp(1)
                        .child(format!("by {}", pkg.author)),
                ),
        )
        // Bottom Action Button
        .child(action_btn)
        .into_any_element();

    if is_selected {
        div()
            .p(px(4.0))
            .rounded_2xl()
            .bg(theme::focus_glow())
            .child(card_inner)
            .into_any_element()
    } else {
        card_inner
    }
}

/// Renders the missing executable fallback modal dialog card.
pub fn render_missing_exe_modal(handler_name: &str, cx: &mut Context<MainWindow>) -> AnyElement {
    let h_name = handler_name.to_string();

    CouchModal::new("couch-missing-exe-modal", format!("Game Executable Not Found - {}", handler_name))
        .width(ModalWidth::Md)
        .icon(IconName::TriangleAlert)
        .tone(TagTone::Warning)
        .body(
            div()
                .text_sm()
                .text_color(theme::text_muted())
                .child(format!(
                    "No executable is currently associated with '{}'. Would you like to locate the game executable (.exe) now?",
                    handler_name
                )),
        )
        .action(
            CouchButton::secondary("btn-cancel-missing-exe", "Cancel (B)")
                .icon(IconName::X)
                .on_click(cx.listener(|this, _, _, cx| {
                    this.couch_hub_state.dismiss_prompt();
                    cx.notify();
                }))
        )
        .action(
            CouchButton::secondary("btn-switch-to-desktop", "Desktop Mode (F11)")
                .icon(IconName::Monitor)
                .on_click(cx.listener(|this, _, _, cx| {
                    this.couch_hub_state.dismiss_prompt();
                    this.toggle_big_picture(cx);
                }))
        )
        .action(
            CouchButton::primary("btn-locate-exe", "Locate .exe (A)")
                .icon(IconName::FolderOpen)
                .on_click(cx.listener(move |this, _, _, cx| {
                    this.locate_exe_for_handler(&h_name, cx);
                }))
        )
        .into_any_element()
}

/// Renders the gamepad-navigable handler custom options modal dialog.
pub fn render_custom_options_modal(
    modal: &CouchOptionsModalState,
    cx: &mut Context<MainWindow>,
) -> AnyElement {
    let options_list = div()
        .id("couch-options-list")
        .flex()
        .flex_col()
        .gap_3()
        .py_1()
        .children(modal.options.iter().enumerate().map(|(idx, opt)| {
            let is_focused = idx == modal.focused_option_idx;
            let current_val = opt
                .choices
                .get(opt.selected_choice)
                .map(|s| s.as_str())
                .unwrap_or("Default");

            CouchOptionRow::new(
                ("couch-opt-row", idx),
                opt.title.clone(),
                opt.description.clone(),
                current_val,
            )
            .focused(is_focused)
            .on_prev(cx.listener(move |this, _, _, cx| {
                if let Some(ref mut m) = this.couch_hub_state.custom_options_modal {
                    m.focused_option_idx = idx;
                    m.cycle_choice(false);
                }
                cx.notify();
            }))
            .on_next(cx.listener(move |this, _, _, cx| {
                if let Some(ref mut m) = this.couch_hub_state.custom_options_modal {
                    m.focused_option_idx = idx;
                    m.cycle_choice(true);
                }
                cx.notify();
            }))
            .into_any_element()
        }));

    CouchModal::new("couch-handler-options-modal", format!("Splitscreen Options - {}", modal.game_title))
        .subtitle("Select your gameplay preferences (defaults pre-selected):")
        .width(ModalWidth::Lg)
        .icon(IconName::SlidersHorizontal)
        .tone(TagTone::Accent)
        .body(options_list)
        .action(
            CouchButton::secondary("btn-cancel-options", "Cancel (B)")
                .icon(IconName::X)
                .on_click(cx.listener(|this, _, _, cx| {
                    this.couch_hub_state.custom_options_modal = None;
                    cx.notify();
                }))
        )
        .action(
            CouchButton::primary("btn-confirm-options-launch", "Start Session with These Options (Start / A)")
                .icon(IconName::Play)
                .on_click(cx.listener(|this, _, _, cx| {
                    this.confirm_couch_options_and_launch(cx);
                }))
        )
        .into_any_element()
}