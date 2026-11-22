//! Community Handler Hub browser view.
//!
//! Enables searching, viewing descriptions, comparing versions, filtering by community tags,
//! fuzzy searching, sorting by hotness/downloads/trending/date, and installing
//! community-contributed game handler scripts directly from the Splitscreen.Me Hub.

use crate::models::{HandlerItem, HubPackageItem};
use crate::nefos::*;
use crate::theme;
use crate::window::{HubSortOption, MainWindow};
use crate::{Icon, IconName, ScrollableElement};
use gpui::prelude::*;
use gpui::*;

pub fn normalize_for_match(s: &str) -> String {
    let lower = s.to_lowercase();
    let mut clean = String::with_capacity(lower.len());
    for c in lower.chars() {
        if c.is_alphanumeric() || c.is_whitespace() {
            clean.push(c);
        }
    }
    let tokens: Vec<&str> = clean.split_whitespace().collect();
    let converted: Vec<String> = tokens
        .into_iter()
        .map(|tok| match tok {
            "ii" => "2".to_string(),
            "iii" => "3".to_string(),
            "iv" => "4".to_string(),
            "v" => "5".to_string(),
            "vi" => "6".to_string(),
            _ => tok.to_string(),
        })
        .collect();
    converted.join(" ")
}

/// Standard filter tags requested for SplitscreenMe handlers catalog.
pub const COMMUNITY_TAGS: &[&str] = &[
    "Controller support",
    "Keyboard/mouse support",
    "Multiple keyboards/mice support",
    "3+ players",
    "4+ players",
    "8+ players",
    "16+ players",
    "Adventure",
    "Real Time Strategy",
    "Role-playing (RPG)",
    "Shooter",
    "Simulator",
    "Sport",
    "Strategy",
    "Tactical",
    "Turn-based strategy",
    "Arcade",
    "Fighting",
    "Hack and slash",
    "Indie",
    "MOBA",
    "Music",
    "Platform",
];

/// Computes a fuzzy matching score between a search query and target string.
/// Returns `Some(score)` if all pattern characters exist in order, or `None`.
pub fn fuzzy_score(pattern: &str, target: &str) -> Option<i64> {
    let pat = pattern.trim().to_lowercase();
    if pat.is_empty() {
        return Some(0);
    }
    let tgt = target.to_lowercase();
    let p_chars: Vec<char> = pat.chars().collect();
    let t_chars: Vec<char> = tgt.chars().collect();

    let mut p_idx = 0;
    let mut score: i64 = 0;
    let mut consecutive = 0;
    let mut prev_t_idx = 0;

    for (t_idx, &tc) in t_chars.iter().enumerate() {
        if p_idx < p_chars.len() && tc == p_chars[p_idx] {
            let mut char_score = 10;
            // Bonus for match at word boundary or start
            if t_idx == 0 || !t_chars[t_idx - 1].is_alphanumeric() {
                char_score += 15;
            }
            // Bonus for consecutive matching characters
            if consecutive > 0 {
                char_score += 20 * consecutive;
            }
            consecutive += 1;
            // Penalty for gaps
            if p_idx > 0 && t_idx > prev_t_idx + 1 {
                char_score -= (t_idx - prev_t_idx - 1) as i64;
            }
            prev_t_idx = t_idx;
            score += char_score;
            p_idx += 1;
        } else {
            consecutive = 0;
        }
    }

    if p_idx == p_chars.len() {
        Some(score)
    } else {
        None
    }
}

/// Hub browser view component.
#[derive(Default)]
pub struct HubBrowserView {
    /// Current search query string.
    pub search_query: String,
    /// Identifier of the selected hub package.
    pub selected_package_id: Option<String>,
}

impl HubBrowserView {
    /// Creates a new `HubBrowserView`.
    pub fn new() -> Self {
        Self::default()
    }
}

/// Renders the community hub browser view with search bar, tag filters, sorting, and packages list.
pub fn render_hub_view(
    packages: &[HubPackageItem],
    handlers: &[HandlerItem],
    search_query: &str,
    search_focus: &gpui::FocusHandle,
    current_page: usize,
    page_size: usize,
    current_sort: HubSortOption,
    sort_ascending: bool,
    active_tags: &std::collections::HashSet<String>,
    installing_handlers: &std::collections::HashMap<String, f32>,
    installed_package_ids: &std::collections::HashSet<String>,
    hub_loading: bool,
    cx: &mut Context<MainWindow>,
) -> AnyElement {
    let query_val = search_query.to_string();

    // 1. Generate Autocomplete Suggestions if query is non-empty
    let mut autocomplete_suggestions: Vec<String> = Vec::new();
    if query_val.len() >= 2 {
        for tag in COMMUNITY_TAGS {
            if tag.to_lowercase().contains(&query_val.to_lowercase())
                && !autocomplete_suggestions.contains(&tag.to_string())
            {
                autocomplete_suggestions.push(tag.to_string());
            }
        }
        for pkg in packages {
            if pkg
                .game_name
                .to_lowercase()
                .contains(&query_val.to_lowercase())
                && !autocomplete_suggestions.contains(&pkg.game_name)
            {
                autocomplete_suggestions.push(pkg.game_name.clone());
            } else if pkg.title.to_lowercase().contains(&query_val.to_lowercase())
                && !autocomplete_suggestions.contains(&pkg.title)
            {
                autocomplete_suggestions.push(pkg.title.clone());
            }
            if autocomplete_suggestions.len() >= 5 {
                break;
            }
        }
    }

    // 2. Filter packages by query (fuzzy match) and active tags
    let mut scored_packages: Vec<(i64, &HubPackageItem)> = packages
        .iter()
        .filter_map(|pkg| {
            // Multi-tag filter check: all active tags must match
            if !active_tags.is_empty() {
                let matches_all = active_tags.iter().all(|tag| {
                    pkg.tags.iter().any(|t| t.eq_ignore_ascii_case(tag))
                        || pkg.game_name.to_lowercase().contains(&tag.to_lowercase())
                        || pkg.description.to_lowercase().contains(&tag.to_lowercase())
                });
                if !matches_all {
                    return None;
                }
            }

            // Fuzzy query matching
            if query_val.is_empty() {
                Some((0, pkg))
            } else {
                let title_score = fuzzy_score(&query_val, &pkg.title);
                let game_score = fuzzy_score(&query_val, &pkg.game_name);
                let author_score = fuzzy_score(&query_val, &pkg.author);
                let tag_score = pkg
                    .tags
                    .iter()
                    .filter_map(|t| fuzzy_score(&query_val, t))
                    .max();

                let best_score = [title_score, game_score, author_score, tag_score]
                    .into_iter()
                    .flatten()
                    .max();

                best_score.map(|s| (s, pkg))
            }
        })
        .collect();

    // 3. Sort packages based on selected sort criteria and direction
    scored_packages.sort_by(|(s_a, a), (s_b, b)| {
        let cmp = match current_sort {
            HubSortOption::Trending => {
                let trend_a =
                    a.downloads * 2 + (a.likes as u64) * 5 + (a.comments_count as u64) * 3;
                let trend_b =
                    b.downloads * 2 + (b.likes as u64) * 5 + (b.comments_count as u64) * 3;
                trend_b.cmp(&trend_a).then_with(|| s_b.cmp(s_a))
            }
            HubSortOption::Hottest => b.likes.cmp(&a.likes).then_with(|| s_b.cmp(s_a)),
            HubSortOption::Downloads => b.downloads.cmp(&a.downloads).then_with(|| s_b.cmp(s_a)),
            HubSortOption::ReleaseDate => {
                let date_a = if a.updated_at.is_empty() {
                    &a.created_at
                } else {
                    &a.updated_at
                };
                let date_b = if b.updated_at.is_empty() {
                    &b.created_at
                } else {
                    &b.updated_at
                };
                date_b.cmp(date_a).then_with(|| s_b.cmp(s_a))
            }
            HubSortOption::Alphabetical => {
                a.game_name.to_lowercase().cmp(&b.game_name.to_lowercase())
            }
        };

        if sort_ascending { cmp.reverse() } else { cmp }
    });

    let filtered_packages: Vec<&HubPackageItem> =
        scored_packages.into_iter().map(|(_, p)| p).collect();
    let total_packages = filtered_packages.len();
    let total_pages = if total_packages == 0 {
        1
    } else {
        (total_packages + page_size - 1) / page_size
    };
    let page = current_page.min(total_pages.saturating_sub(1));
    let start_idx = page * page_size;
    let end_idx = (start_idx + page_size).min(total_packages);
    let page_items = if start_idx < total_packages {
        &filtered_packages[start_idx..end_idx]
    } else {
        &[]
    };

    div()
        .flex()
        .flex_col()
        .size_full()
        .min_w_0()
        .p_6()
        .gap_4()
        .overflow_hidden()
        // Sticky Controls Header (Header, Banner, Search Input, Sort & Filter Tags)
        .child(
            div()
                .flex()
                .flex_col()
                .flex_shrink_0()
                .gap_3()
                // Header
                .child(
                    div()
                        .flex()
                        .flex_col()
                        .gap_1()
                        .child(
                            div()
                                .text_xl()
                                .font_weight(FontWeight::BOLD)
                                .text_color(theme::text_primary())
                                .child("Splitscreen.Me Community Hub"),
                        )
                        .child(
                            div()
                                .text_sm()
                                .text_color(theme::text_muted())
                                .child("Discover, download, and install community handlers with automatic multi-instance hooks"),
                        ),
                )
        // Hub API & Online Notice Banner
        .child(
            div()
                .flex()
                .items_center()
                .justify_between()
                .p_3()
                .rounded_lg()
                .bg(theme::accent_subtle())
                .border_1()
                .border_color(theme::accent())
                .child(
                    div()
                        .flex()
                        .items_center()
                        .gap_3()
                        .child(
                            div()
                                .flex()
                                .items_center()
                                .gap_2()
                                .text_sm()
                                .text_color(theme::accent())
                                .font_weight(FontWeight::SEMIBOLD)
                                .child(Icon::new(IconName::Globe).size(px(16.0)).text_color(theme::accent()))
                                .child("SplitscreenMe API Live Sync"),
                        )
                        .child(
                            div()
                                .text_xs()
                                .text_color(theme::text_muted())
                                .child("Auto-fetches packages directly into your handlers directory with immediate library loading."),
                        ),
                )
                .child(
                    NefosButton::ghost("btn-hub-refresh", "Refresh")
                        .size(NefosButtonSize::Sm)
                        .icon(IconName::RotateCw)
                        .on_click(cx.listener(|this, _, _, cx| {
                            this.hub_page = 0;
                            this.trigger_hub_search_async(cx);
                            cx.notify();
                        })),
                ),
        )
        // Search & Controls Bar
        .child(
            div()
                .flex()
                .flex_col()
                .gap_2()
                .p_3()
                .bg(theme::bg_panel())
                .rounded_xl()
                .border_1()
                .border_color(theme::border_color())
                .child(
                    NefosSearchBar::new("hub-search-bar")
                        .placeholder("Search handlers with fuzzy matching & autocomplete... (click to type)")
                        .query(&query_val)
                        .focus_handle(search_focus)
                        .on_clear(cx.listener(|this: &mut MainWindow, _, _, cx| {
                            this.hub_search_query.clear();
                            this.hub_page = 0;
                            cx.notify();
                        }))
                        .on_key_down(cx.listener(|this: &mut MainWindow, event: &gpui::KeyDownEvent, _window, cx| {
                            if event.keystroke.modifiers.control || event.keystroke.modifiers.alt || event.keystroke.modifiers.platform {
                                return;
                            }
                            cx.stop_propagation();
                            let key = event.keystroke.key.as_str();
                            if key == "backspace" {
                                this.hub_search_query.pop();
                                this.hub_page = 0;
                                cx.notify();
                            } else if key == "escape" {
                                this.hub_search_query.clear();
                                this.hub_page = 0;
                                cx.notify();
                            } else if key == "space" {
                                this.hub_search_query.push(' ');
                                this.hub_page = 0;
                                cx.notify();
                            } else if key == "enter" {
                                this.hub_page = 0;
                                this.trigger_hub_search_async(cx);
                                cx.notify();
                            } else if let Some(ref s) = event.keystroke.key_char {
                                this.hub_search_query.push_str(s);
                                this.hub_page = 0;
                                cx.notify();
                            } else if key.chars().count() == 1 {
                                this.hub_search_query.push_str(key);
                                this.hub_page = 0;
                                cx.notify();
                            }
                        }))
                )
                // Autocomplete Suggestions Chips
                .when(!autocomplete_suggestions.is_empty(), |el| {
                    el.child(
                        div()
                            .flex()
                            .flex_wrap()
                            .items_center()
                            .gap_1p5()
                            .pt_2()
                            .border_t_1()
                            .border_color(theme::border_color())
                            .child(
                                div()
                                    .text_xs()
                                    .font_weight(FontWeight::SEMIBOLD)
                                    .text_color(theme::text_muted())
                                    .child("Suggestions:"),
                            )
                            .children(autocomplete_suggestions.into_iter().enumerate().map(|(s_idx, suggestion)| {
                                let sug_str = suggestion.clone();
                                NefosTag::new(suggestion)
                                    .id(("hub-sug", s_idx))
                                    .icon(IconName::Search)
                                    .tone(NefosTagTone::Accent)
                                    .on_click(cx.listener(move |this, _, _, cx| {
                                        this.hub_search_query = sug_str.clone();
                                        this.hub_page = 0;
                                        cx.notify();
                                    }))
                            })),
                    )
                }),
        )
        // Sort & Tag Filter Controls
        .child(
            div()
                .flex()
                .flex_col()
                .gap_2p5()
                .p_3()
                .bg(theme::bg_panel())
                .rounded_xl()
                .border_1()
                .border_color(theme::border_color())
                // Sort row
                .child(
                    div()
                        .flex()
                        .items_center()
                        .justify_between()
                        .gap_2()
                        .w_full()
                        .child(
                            div()
                                .id("hub-sort-scroll-row")
                                .flex()
                                .items_center()
                                .gap_2()
                                .min_w_0()
                                .overflow_x_scrollbar()
                                .child(
                                    div()
                                        .flex_shrink_0()
                                        .text_xs()
                                        .font_weight(FontWeight::SEMIBOLD)
                                        .text_color(theme::text_muted())
                                        .child("Sort By:"),
                                )
                                .children([
                                    ("Hot / Trending", HubSortOption::Trending),
                                    ("Hottest", HubSortOption::Hottest),
                                    ("Most Downloads", HubSortOption::Downloads),
                                    ("Recently Updated", HubSortOption::ReleaseDate),
                                    ("Alphabetical", HubSortOption::Alphabetical),
                                ].into_iter().enumerate().map(|(idx, (label, sort_opt))| {
                                    let is_active = current_sort == sort_opt;
                                    NefosButton::new(("hub-sort", idx))
                                        .label(label)
                                        .size(NefosButtonSize::Sm)
                                        .variant(if is_active {
                                            NefosButtonVariant::Primary
                                        } else {
                                            NefosButtonVariant::Secondary
                                        })
                                        .selected(is_active)
                                        .on_click(cx.listener(move |this, _, _, cx| {
                                            this.hub_sort = sort_opt;
                                            this.hub_page = 0;
                                            cx.notify();
                                        }))
                                })),
                        )
                        .child(
                            NefosButton::new("btn-sort-dir")
                                .label(if sort_ascending { "Ascending" } else { "Descending" })
                                .icon(if sort_ascending { IconName::ArrowUp } else { IconName::ArrowDown })
                                .size(NefosButtonSize::Sm)
                                .variant(NefosButtonVariant::Secondary)
                                .on_click(cx.listener(|this, _, _, cx| {
                                    this.hub_sort_ascending = !this.hub_sort_ascending;
                                    this.hub_page = 0;
                                    cx.notify();
                                })),
                        ),
                )
                // Tags horizontal filter bar
                .child(
                    div()
                        .id("hub-tags-scroll-row")
                        .flex()
                        .items_center()
                        .gap_1p5()
                        .w_full()
                        .overflow_x_scrollbar()
                        .child(
                            div()
                                .flex_shrink_0()
                                .text_xs()
                                .font_weight(FontWeight::SEMIBOLD)
                                .text_color(theme::text_muted())
                                .child("Tags:"),
                        )
                        .child({
                            let is_all = active_tags.is_empty();
                            NefosTag::new("All Tags")
                                .id("hub-tag-all")
                                .tone(if is_all {
                                    NefosTagTone::Accent
                                } else {
                                    NefosTagTone::Neutral
                                })
                                .on_click(cx.listener(|this, _, _, cx| {
                                    this.selected_tags.clear();
                                    this.hub_page = 0;
                                    cx.notify();
                                }))
                        })
                        .children(COMMUNITY_TAGS.iter().enumerate().map(|(t_idx, tag)| {
                            let tag_str = tag.to_string();
                            let is_selected = active_tags.contains(&tag_str);
                            let t_val = tag_str.clone();

                            NefosTag::new(*tag)
                                .id(("hub-tag", t_idx))
                                .tone(if is_selected {
                                    NefosTagTone::Accent
                                } else {
                                    NefosTagTone::Neutral
                                })
                                .on_click(cx.listener(move |this, _, _, cx| {
                                    if this.selected_tags.contains(&t_val) {
                                        this.selected_tags.remove(&t_val);
                                    } else {
                                        this.selected_tags.insert(t_val.clone());
                                    }
                                    this.hub_page = 0;
                                    cx.notify();
                                }))
                        })),
                ),
        )
        )
        // Scrollable Community Packages List (pinned below sticky search/filter header)
        .child(
            div()
                .id("hub-packages-scroll")
                .flex()
                .flex_col()
                .flex_1()
                .min_h_0()
                .gap_3()
                .overflow_y_scrollbar()
                .when(hub_loading, |el| {
                    el.child(
                        div()
                            .flex()
                            .flex_wrap()
                            .gap_3()
                            .p_4()
                            .children((0..6usize).map(|i| {
                                div()
                                    .id(("hub-skeleton", i))
                                    .w(px(260.0))
                                    .h(px(320.0))
                                    .rounded_lg()
                                    .bg(theme::bg_panel())
                                    .border_1()
                                    .border_color(theme::border_subtle())
                                    .p_3()
                                    .flex()
                                    .flex_col()
                                    .gap_3()
                                    .child(div().w_full().h(px(180.0)).rounded_md().bg(theme::bg_subtle()))
                                    .child(div().w(px(160.0)).h(px(16.0)).rounded_md().bg(theme::bg_subtle()))
                                    .child(div().w(px(100.0)).h(px(12.0)).rounded_md().bg(theme::bg_subtle()))
                                    .child(div().flex_1())
                                    .child(div().w_full().h(px(32.0)).rounded_md().bg(theme::bg_subtle()))
                            }))
                    )
                })
                .when(!hub_loading && total_packages == 0, |el| {
                    el.child(
                        div()
                            .flex()
                            .flex_col()
                            .items_center()
                            .justify_center()
                            .p_12()
                            .bg(theme::bg_panel())
                            .rounded_lg()
                            .border_1()
                            .border_dashed()
                            .border_color(theme::border_color())
                            .gap_3()
                            .child(
                                div()
                                    .text_base()
                                    .font_weight(FontWeight::SEMIBOLD)
                                    .text_color(theme::text_primary())
                                    .child("No Community Handlers Found"),
                            )
                            .child(
                                div()
                                    .text_sm()
                                    .text_color(theme::text_muted())
                                    .child("Try adjusting your search terms or clearing selected tags."),
                            ),
                    )
                })
                .when(!hub_loading && total_packages > 0, |el| {
                    el.child(
                        div()
                            .flex()
                            .items_center()
                            .justify_between()
                            .child(
                                div()
                                    .text_sm()
                                    .font_weight(FontWeight::BOLD)
                                    .text_color(theme::text_primary())
                                    .child(format!(
                                        "Available Handlers ({} Total - Page {} of {})",
                                        total_packages,
                                        page + 1,
                                        total_pages
                                    )),
                            )
                            .child(
                                div()
                                    .flex()
                                    .items_center()
                                    .gap_2()
                                    .when(page > 0, |el| {
                                        el.child(
                                            NefosButton::new("hub-prev-page")
                                                .label("Previous")
                                                .icon(IconName::ChevronLeft)
                                                .size(NefosButtonSize::Sm)
                                                .variant(NefosButtonVariant::Secondary)
                                                .on_click(cx.listener(|this, _, _, cx| {
                                                    if this.hub_page > 0 {
                                                        this.hub_page -= 1;
                                                        cx.notify();
                                                    }
                                                })),
                                        )
                                    })
                                    .when(page + 1 < total_pages, |el| {
                                        el.child(
                                            NefosButton::new("hub-next-page")
                                                .label("Next")
                                                .icon(IconName::ChevronRight)
                                                .size(NefosButtonSize::Sm)
                                                .variant(NefosButtonVariant::Secondary)
                                                .on_click(cx.listener(|this, _, _, cx| {
                                                    this.hub_page += 1;
                                                    cx.notify();
                                                })),
                                        )
                                    }),
                            ),
                    )
                })
                .when(!hub_loading && total_packages > 0, |el| {
                    el.children(page_items.iter().enumerate().map(|(rel_idx, pkg)| {
                    let idx = start_idx + rel_idx;
                    let pkg_item = (*pkg).clone();
                    let pkg_id = pkg.id.clone();
                    let pkg_title = pkg.title.clone();
                    let norm_pkg_game = normalize_for_match(&pkg.game_name);
                    let norm_pkg_title = normalize_for_match(&pkg.title);
                    let is_locally_present = handlers.iter().any(|h| {
                        let norm_h_name = normalize_for_match(&h.game_name);
                        let norm_h_guid = normalize_for_match(&h.guid);
                        norm_h_name == norm_pkg_game
                            || norm_h_name == norm_pkg_title
                            || norm_h_guid == norm_pkg_game
                            || norm_h_guid == norm_pkg_title
                            || h.guid.eq_ignore_ascii_case(&pkg.id)
                            || h.game_name.eq_ignore_ascii_case(&pkg.game_name)
                    });
                    let is_installed = pkg.is_installed || is_locally_present || installed_package_ids.contains(&pkg.id);
                    let installing_progress = installing_handlers.get(&pkg.id).copied();

                    // Truncate description preview cleanly to max 120 chars
                    let first_line = pkg.description.lines().find(|l| !l.trim().is_empty()).unwrap_or("");
                    let desc_preview = if first_line.chars().count() > 120 {
                        format!("{}...", first_line.chars().take(120).collect::<String>())
                    } else if first_line.is_empty() {
                        "Community split-screen handler profile.".to_string()
                    } else {
                        first_line.to_string()
                    };

                    let modal_item = pkg_item.clone();

                    NefosCard::surface(("hub-pkg", idx))
                        .hover_elevate(true)
                        .paddings(false)
                        .child(
                            div()
                                .flex()
                                .items_center()
                                .justify_between()
                                .w_full()
                                .p_4()
                                // LEFT SIDE: Thumbnail + Info (with min_w_0 to prevent flex displacement)
                                .child(
                                    div()
                                        .flex_1()
                                        .min_w_0()
                                        .overflow_hidden()
                                        .flex()
                                        .items_center()
                                        .gap_3p5()
                                        // Game Cover Image with strict 2:3 aspect ratio
                                        .child(
                                            NefosCover::new(&pkg.game_name, px(56.0), px(84.0))
                                                .cover_url(pkg.cover_image_url.clone())
                                                .aspect_ratio(2.0 / 3.0)
                                                .installed(is_installed)
                                                .when(is_installed, |c| {
                                                    c.badge(
                                                        div()
                                                            .absolute()
                                                            .top(px(2.0))
                                                            .left(px(2.0))
                                                            .size(px(16.0))
                                                            .rounded_full()
                                                            .bg(theme::success())
                                                            .border_1()
                                                            .border_color(theme::text_primary())
                                                            .flex()
                                                            .items_center()
                                                            .justify_center()
                                                            .text_xs()
                                                            .font_weight(FontWeight::BOLD)
                                                            .text_color(theme::text_on_accent())
                                                            .child(Icon::new(IconName::Check).size_3()),
                                                    )
                                                })
                                        )
                                        // Text Information Container
                                        .child(
                                            div()
                                                .flex_1()
                                                .min_w_0()
                                                .overflow_hidden()
                                                .flex()
                                                .flex_col()
                                                .gap_1()
                                                // Row 1: Game Title + Verified Badge + Badges
                                                .child({
                                                    let mut title_row = div()
                                                        .flex()
                                                        .items_center()
                                                        .gap_2()
                                                        .min_w_0();

                                                    if pkg.is_verified {
                                                        title_row = title_row.child(
                                                            NefosTag::new("Verified")
                                                                .id(("hub-verified-pill", idx))
                                                                .tone(NefosTagTone::Success)
                                                                .icon(IconName::BadgeCheck)
                                                                .tooltip("The latest release of this handler has been validated by our team and is safe to use."),
                                                        );
                                                    }

                                                    title_row = title_row.child(
                                                        div()
                                                            .text_base()
                                                            .font_weight(FontWeight::BOLD)
                                                            .text_color(theme::text_primary())
                                                            .truncate()
                                                            .child(pkg.game_name.clone()),
                                                    );

                                                    title_row = title_row.child(
                                                        div()
                                                            .flex_shrink_0()
                                                            .flex()
                                                            .items_center()
                                                            .gap_1()
                                                            .text_xs()
                                                            .text_color(theme::accent())
                                                            .child(Icon::new(IconName::User).size(px(12.0)).text_color(theme::accent()))
                                                            .child(pkg.author.clone()),
                                                    );

                                                    title_row = title_row.child(
                                                        NefosTag::new(pkg.version.clone())
                                                            .id(("hub-ver-tag", idx))
                                                            .tone(NefosTagTone::Neutral),
                                                    );

                                                    if pkg.has_update {
                                                        title_row = title_row.child(
                                                            NefosTag::new(format!("Update to v{} (Installed: v{})", pkg.version_number, pkg.installed_version.unwrap_or(1)))
                                                                .id(("hub-update-tag", idx))
                                                                .icon(IconName::Zap)
                                                                .tone(NefosTagTone::Warning),
                                                        );
                                                    } else if pkg.is_up_to_date {
                                                        title_row = title_row.child(
                                                            NefosTag::new("Up to date")
                                                                .id(("hub-uptodate-tag", idx))
                                                                .icon(IconName::Check)
                                                                .tone(NefosTagTone::Success),
                                                        );
                                                    }

                                                    title_row
                                                })
                                                // Row 2: Faint Subtitle (Handler Title & Maintainer)
                                                .child(
                                                    div()
                                                        .text_xs()
                                                        .text_color(theme::text_muted().opacity(0.85))
                                                        .truncate()
                                                        .child(format!("{} • Maintainer: {}", pkg.title, pkg.author)),
                                                )
                                                // Row 3: Stats (Player count, Input Support Icons, Likes/Hotness, Downloads, Comments)
                                                .child({
                                                    let mut stats_row = div()
                                                        .flex()
                                                        .items_center()
                                                        .gap_3()
                                                        .text_xs()
                                                        .text_color(theme::text_muted())
                                                        .child(
                                                            div()
                                                                .flex()
                                                                .items_center()
                                                                .gap_1()
                                                                .child(Icon::new(IconName::Users).size(px(13.0)).text_color(theme::text_muted()))
                                                                .child(div().child(pkg.max_players.clone())),
                                                        );

                                                    if pkg.playable_controllers || pkg.playable_mouse_keyboard || pkg.playable_multi_mouse_keyboard {
                                                        stats_row = stats_row.child(div().h(px(12.0)).w(px(1.0)).bg(theme::border_color()));
                                                        if pkg.playable_controllers {
                                                            stats_row = stats_row.child(
                                                                div()
                                                                    .id(("hub-input-ctrl", idx))
                                                                    .flex()
                                                                    .items_center()
                                                                    .gap_1()
                                                                    .tooltip(crate::make_tooltip("Supports gamepads & controllers"))
                                                                    .child(Icon::new(IconName::Gamepad2).size_3p5().text_color(theme::text_primary())),
                                                                );
                                                        }
                                                        if pkg.playable_multi_mouse_keyboard {
                                                            stats_row = stats_row.child(
                                                                div()
                                                                    .id(("hub-input-multikm", idx))
                                                                    .flex()
                                                                    .items_center()
                                                                    .gap_1()
                                                                    .tooltip(crate::make_tooltip("Supports multiple keyboards & mice via ProtoInput"))
                                                                    .child(Icon::new(IconName::Keyboard).size_3p5().text_color(theme::text_primary()))
                                                                    .child(Icon::new(IconName::Mouse).size_3p5().text_color(theme::text_primary())),
                                                            );
                                                        } else if pkg.playable_mouse_keyboard {
                                                            stats_row = stats_row.child(
                                                                div()
                                                                    .id(("hub-input-km", idx))
                                                                    .flex()
                                                                    .items_center()
                                                                    .gap_1()
                                                                    .tooltip(crate::make_tooltip("Supports keyboard & mouse"))
                                                                    .child(Icon::new(IconName::Keyboard).size_3p5().text_color(theme::text_primary())),
                                                            );
                                                        }
                                                        stats_row = stats_row.child(div().h(px(12.0)).w(px(1.0)).bg(theme::border_color()));
                                                    }

                                                    stats_row
                                                        .child(
                                                            div()
                                                                .flex()
                                                                .items_center()
                                                                .gap_1()
                                                                .child(Icon::new(IconName::Flame).size(px(13.0)).text_color(theme::warning()))
                                                                .child(div().child(format!("{} hotness", pkg.likes))),
                                                        )
                                                        .child(
                                                            div()
                                                                .flex()
                                                                .items_center()
                                                                .gap_1()
                                                                .child(Icon::new(IconName::Download).size(px(13.0)).text_color(theme::text_muted()))
                                                                .child(div().child(format!("{} downloads", pkg.downloads))),
                                                        )
                                                        .child(
                                                            div()
                                                                .flex()
                                                                .items_center()
                                                                .gap_1()
                                                                .child(Icon::new(IconName::MessageSquare).size(px(13.0)).text_color(theme::text_muted()))
                                                                .child(div().child(format!("{} comments", pkg.comments_count))),
                                                        )
                                                })
                                                // Row 4: Truncated Description Preview with "Read More / Details"
                                                .child(
                                                    div()
                                                        .flex()
                                                        .items_center()
                                                        .gap_2()
                                                        .min_w_0()
                                                        .child(
                                                            div()
                                                                .flex_1()
                                                                .min_w_0()
                                                                .text_xs()
                                                                .text_color(theme::text_muted())
                                                                .truncate()
                                                                .child(desc_preview),
                                                        )
                                                        .child(
                                                            NefosButton::secondary(("btn-details", idx), "Details")
                                                                .size(NefosButtonSize::Sm)
                                                                .icon(IconName::Info)
                                                                .tooltip("View Handler Documentation & Notes")
                                                                .on_click(cx.listener(move |this, _, _, cx| {
                                                                    this.inspecting_hub_package = Some(modal_item.clone());
                                                                    this.load_hub_package_details(&modal_item.id, cx);
                                                                    cx.notify();
                                                                })),
                                                        ),
                                                ),
                                        ),
                                )
                                // RIGHT SIDE: Action Buttons (strictly flex_shrink_0 to ensure they are NEVER displaced)
                                .child(
                                    div()
                                        .flex_shrink_0()
                                        .flex()
                                        .items_center()
                                        .gap_2()
                                        .child(
                                            if let Some(progress) = installing_progress {
                                                div()
                                                    .flex()
                                                    .flex_col()
                                                    .gap_1()
                                                    .w(px(140.0))
                                                    .child(
                                                        div()
                                                            .text_xs()
                                                            .font_weight(FontWeight::BOLD)
                                                            .text_color(theme::text_primary())
                                                            .child(format!("Downloading... {:.0}%", progress * 100.0)),
                                                    )
                                                    .child(
                                                        div()
                                                            .w_full()
                                                            .h(px(6.0))
                                                            .rounded_full()
                                                            .bg(theme::bg_subtle())
                                                            .border_1()
                                                            .border_color(theme::border_subtle())
                                                            .child(
                                                                div()
                                                                    .h_full()
                                                                    .rounded_full()
                                                                    .bg(theme::accent())
                                                                    .w(px((progress * 138.0).max(4.0))),
                                                            ),
                                                    )
                                                    .into_any_element()
                                            } else if pkg.has_update {
                                                NefosButton::secondary(("btn-update-pkg", idx), "Update")
                                                    .size(NefosButtonSize::Sm)
                                                    .on_click(cx.listener(move |this, _, _, cx| {
                                                        println!("Updating handler from Hub: ID '{}' ({})", pkg_id, pkg_title);
                                                        let pkg_id_clone = pkg_id.clone();
                                                        let pkg_id_done = pkg_id.clone();
                                                        let pkg_title_done = pkg_title.clone();

                                                        this.installing_handlers.insert(pkg_id_clone.clone(), 0.0);
                                                        this.status_message = format!("Updating package '{}'...", pkg_title_done);
                                                        cx.notify();

                                                        if let Some(ref cb) = this.callbacks.on_install_handler {
                                                            let cb = cb.clone();
                                                            let (tx, mut rx) = tokio::sync::mpsc::unbounded_channel::<crate::models::HubInstallProgressMsg>();
                                                            let tx_prog = tx.clone();

                                                            std::thread::spawn(move || {
                                                                let item = cb(
                                                                    &pkg_id_clone,
                                                                    Box::new(move |progress: f32| {
                                                                        let _ = tx_prog.send(crate::models::HubInstallProgressMsg::Progress(progress));
                                                                    }),
                                                                );
                                                                let _ = tx.send(crate::models::HubInstallProgressMsg::Done(item));
                                                            });

                                                            cx.spawn(async move |this, cx| {
                                                                while let Some(msg) = rx.recv().await {
                                                                    let pkg_id = pkg_id_done.clone();
                                                                    let pkg_title = pkg_title_done.clone();
                                                                    let _ = this.update(&mut *cx, |this, cx| {
                                                                        match msg {
                                                                            crate::models::HubInstallProgressMsg::Progress(p) => {
                                                                                this.installing_handlers.insert(pkg_id, p);
                                                                                cx.notify();
                                                                            }
                                                                            crate::models::HubInstallProgressMsg::Done(installed_item) => {
                                                                                this.installing_handlers.remove(&pkg_id);
                                                                                this.installed_package_ids.insert(pkg_id.clone());
                                                                                if let Some(p) = this.hub_packages.iter_mut().find(|item| item.id == pkg_id) {
                                                                                    p.is_installed = true;
                                                                                    p.has_update = false;
                                                                                    p.is_up_to_date = true;
                                                                                    p.installed_version = Some(p.version_number);
                                                                                }
                                                                                if let Some(h) = installed_item {
                                                                                    let name = h.game_name.clone();
                                                                                    if let Some(existing) = this.handlers.iter_mut().find(|e| e.guid == h.guid) {
                                                                                        *existing = h;
                                                                                    } else {
                                                                                        this.handlers.push(h);
                                                                                    }
                                                                                    if !this.handler_names.contains(&name) {
                                                                                        this.handler_names.push(name.clone());
                                                                                    }
                                                                                    this.selected_game = Some(name.clone());
                                                                                    this.status_message = format!("Updated handler for '{}'!", name);
                                                                                } else {
                                                                                    this.status_message = format!("Downloaded package '{}'. Check Library.", pkg_title);
                                                                                }
                                                                                cx.notify();
                                                                            }
                                                                        }
                                                                    });
                                                                }
                                                            }).detach();
                                                        }
                                                    }))
                                                    .into_any_element()
                                            } else if is_installed {
                                                NefosTag::new("Up to date")
                                                    .id(("hub-installed-badge", idx))
                                                    .icon(IconName::Check)
                                                    .tone(NefosTagTone::Success)
                                                    .into_any_element()
                                            } else {
                                                NefosButton::primary(("btn-install-pkg", idx), "Install")
                                                    .size(NefosButtonSize::Sm)
                                                    .icon(IconName::Download)
                                                    .on_click(cx.listener(move |this, _, _, cx| {
                                                        println!("Installing handler from Hub: ID '{}' ({})", pkg_id, pkg_title);
                                                        let pkg_id_clone = pkg_id.clone();
                                                        let pkg_id_done = pkg_id.clone();
                                                        let pkg_title_done = pkg_title.clone();

                                                        this.installing_handlers.insert(pkg_id_clone.clone(), 0.0);
                                                        this.status_message = format!("Downloading package '{}'...", pkg_title_done);
                                                        cx.notify();

                                                        if let Some(ref cb) = this.callbacks.on_install_handler {
                                                            let cb = cb.clone();
                                                            let (tx, mut rx) = tokio::sync::mpsc::unbounded_channel::<crate::models::HubInstallProgressMsg>();
                                                            let tx_prog = tx.clone();

                                                            std::thread::spawn(move || {
                                                                let item = cb(
                                                                    &pkg_id_clone,
                                                                    Box::new(move |progress: f32| {
                                                                        let _ = tx_prog.send(crate::models::HubInstallProgressMsg::Progress(progress));
                                                                    }),
                                                                );
                                                                let _ = tx.send(crate::models::HubInstallProgressMsg::Done(item));
                                                            });

                                                            cx.spawn(async move |this, cx| {
                                                                while let Some(msg) = rx.recv().await {
                                                                    let pkg_id = pkg_id_done.clone();
                                                                    let pkg_title = pkg_title_done.clone();
                                                                    let _ = this.update(&mut *cx, |this, cx| {
                                                                        match msg {
                                                                            crate::models::HubInstallProgressMsg::Progress(p) => {
                                                                                this.installing_handlers.insert(pkg_id, p);
                                                                                cx.notify();
                                                                            }
                                                                            crate::models::HubInstallProgressMsg::Done(installed_item) => {
                                                                                this.installing_handlers.remove(&pkg_id);
                                                                                this.installed_package_ids.insert(pkg_id.clone());
                                                                                if let Some(p) = this.hub_packages.iter_mut().find(|item| item.id == pkg_id) {
                                                                                    p.is_installed = true;
                                                                                    p.has_update = false;
                                                                                    p.is_up_to_date = true;
                                                                                    p.installed_version = Some(p.version_number);
                                                                                }
                                                                                if let Some(h) = installed_item {
                                                                                    let name = h.game_name.clone();
                                                                                    if let Some(existing) = this.handlers.iter_mut().find(|e| e.guid == h.guid) {
                                                                                        *existing = h;
                                                                                    } else {
                                                                                        this.handlers.push(h);
                                                                                    }
                                                                                    if !this.handler_names.contains(&name) {
                                                                                        this.handler_names.push(name.clone());
                                                                                    }
                                                                                    this.selected_game = Some(name.clone());
                                                                                    this.status_message = format!("Installed handler for '{}'!", name);
                                                                                } else {
                                                                                    this.status_message = format!("Downloaded package '{}'. Check Library.", pkg_title);
                                                                                }
                                                                                cx.notify();
                                                                            }
                                                                        }
                                                                    });
                                                                }
                                                            }).detach();
                                                        }
                                                    }))
                                                    .into_any_element()
                                            }
                                        ),
                                )
                        )
                    }))
                })
            )
        .into_any_element()
}
