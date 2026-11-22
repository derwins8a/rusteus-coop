//! SplitscreenMe Community Hub - Native Handler Details View.
//!
//! Provides a rich native presentation matching the Splitscreen.Me Hub design:
//! - Header: Game title, subtitle, verified badge, report handler button, close button.
//! - Genre & category tags via `NefosTag`.
//! - Metadata statistics strip: Max players, controller support, mouse+keyboard, hotness, downloads, author.
//! - Split content: Notes with rich Markdown (`TextView::markdown`) and action buttons ("Install Handler", "Reinstall", "Uninstall") on left; large 2:3 `NefosCover` poster on right.
//! - Tabs: Comments, JS Code, Versions history, Statistics.

use crate::models::HubPackageItem;
use crate::nefos::*;
use crate::theme;
use crate::window::MainWindow;
use crate::{Icon, IconName};
use gpui::prelude::*;
use gpui::*;
use nucleus_hub::{HubComment, HubPackage};

/// Active tab within the Hub Handler Details screen.
#[derive(Clone, Copy, PartialEq, Eq, Debug, Default)]
pub enum HubDetailTab {
    #[default]
    Comments,
    JsCode,
    VersionHistory,
    Statistics,
}

impl HubDetailTab {
    pub fn label(&self, comments_count: usize, versions_count: usize) -> String {
        match self {
            Self::Comments => {
                if comments_count > 0 {
                    format!("Comments ({})", comments_count)
                } else {
                    "Comments".to_string()
                }
            }
            Self::JsCode => "JS Code".to_string(),
            Self::VersionHistory => {
                if versions_count > 0 {
                    format!("Versions history ({})", versions_count)
                } else {
                    "Versions history".to_string()
                }
            }
            Self::Statistics => "Statistics".to_string(),
        }
    }

    pub fn icon(&self) -> IconName {
        match self {
            Self::Comments => IconName::MessageSquare,
            Self::JsCode => IconName::FileCode,
            Self::VersionHistory => IconName::Clock,
            Self::Statistics => IconName::ChartBar,
        }
    }
}

/// Helper to format relative time from an ISO timestamp.
pub fn format_relative_time(iso: &str) -> String {
    let trimmed = iso.trim();
    if trimmed.is_empty() {
        return "Unknown date".to_string();
    }
    if let Some(date_part) = trimmed.split('T').next() {
        if date_part.len() == 10 {
            if let Ok(yr) = date_part[0..4].parse::<i32>() {
                let diff = 2026 - yr;
                if diff > 1 {
                    return format!("{} years ago", diff);
                } else if diff == 1 {
                    return "1 year ago".to_string();
                } else {
                    return date_part.to_string();
                }
            }
            return date_part.to_string();
        }
    }
    trimmed.to_string()
}

/// Triggers background downloading and unpacking of a community handler.
fn spawn_install_flow(
    pkg_id: &str,
    pkg_title: &str,
    this: &mut MainWindow,
    cx: &mut Context<MainWindow>,
) {
    let p_id_cl = pkg_id.to_string();
    let p_id_cl2 = pkg_id.to_string();
    let p_title_cl = pkg_title.to_string();

    this.installing_handlers.insert(p_id_cl.clone(), 0.0);
    this.status_message = format!("Downloading handler '{}'...", p_title_cl);
    cx.notify();

    if let Some(ref cb) = this.callbacks.on_install_handler {
        let cb = cb.clone();
        let (tx, mut rx) =
            tokio::sync::mpsc::unbounded_channel::<crate::models::HubInstallProgressMsg>();
        let tx_prog = tx.clone();

        std::thread::spawn(move || {
            let item = cb(
                &p_id_cl,
                Box::new(move |p: f32| {
                    let _ = tx_prog.send(crate::models::HubInstallProgressMsg::Progress(p));
                }),
            );
            let _ = tx.send(crate::models::HubInstallProgressMsg::Done(item));
        });

        cx.spawn(async move |this, cx| {
            while let Some(msg) = rx.recv().await {
                let id_key = p_id_cl2.clone();
                let title_key = p_title_cl.clone();
                let _ = this.update(&mut *cx, |this, cx| match msg {
                    crate::models::HubInstallProgressMsg::Progress(p) => {
                        this.installing_handlers.insert(id_key, p);
                        cx.notify();
                    }
                    crate::models::HubInstallProgressMsg::Done(opt_h) => {
                        this.installing_handlers.remove(&id_key);
                        this.installed_package_ids.insert(id_key.clone());
                        if let Some(p) = this.hub_packages.iter_mut().find(|item| item.id == id_key)
                        {
                            p.is_installed = true;
                            p.has_update = false;
                            p.is_up_to_date = true;
                            p.installed_version = Some(p.version_number);
                        }
                        if let Some(ref mut inspecting) = this.inspecting_hub_package {
                            if inspecting.id == id_key {
                                inspecting.is_installed = true;
                                inspecting.has_update = false;
                                inspecting.is_up_to_date = true;
                                inspecting.installed_version = Some(inspecting.version_number);
                            }
                        }
                        if let Some(h) = opt_h {
                            let name = h.game_name.clone();
                            if let Some(existing) =
                                this.handlers.iter_mut().find(|e| e.guid == h.guid)
                            {
                                *existing = h.clone();
                            } else {
                                this.handlers.push(h.clone());
                            }
                            if !this.handler_names.contains(&name) {
                                this.handler_names.push(name.clone());
                            }
                            this.selected_game = Some(h.game_name.clone());
                        }
                        this.status_message =
                            format!("Successfully installed handler for '{}'!", title_key);
                        this.show_osd(&format!("Installed {}!", title_key), 3000, cx);
                        cx.notify();
                    }
                });
            }
        })
        .detach();
    }
}

/// Uninstalls an installed handler from memory and local tracking state.
fn perform_uninstall(
    pkg_id: &str,
    game_name: &str,
    this: &mut MainWindow,
    cx: &mut Context<MainWindow>,
) {
    let norm_game = crate::hub_browser::normalize_for_match(game_name);
    let norm_id = crate::hub_browser::normalize_for_match(pkg_id);

    // 1. Gather all matching handler GUIDs to remove
    let mut matching_guids = Vec::new();
    for h in &this.handlers {
        let norm_h_name = crate::hub_browser::normalize_for_match(&h.game_name);
        let norm_h_guid = crate::hub_browser::normalize_for_match(&h.guid);
        if norm_h_name == norm_game
            || norm_h_name == norm_id
            || norm_h_guid == norm_game
            || norm_h_guid == norm_id
            || h.guid.eq_ignore_ascii_case(pkg_id)
            || h.game_name.eq_ignore_ascii_case(game_name)
        {
            matching_guids.push(h.guid.clone());
        }
    }

    // 2. Remove all matching identifiers from installed tracking sets
    this.installed_package_ids.remove(pkg_id);
    this.installed_package_ids.retain(|id| {
        let n = crate::hub_browser::normalize_for_match(id);
        !id.eq_ignore_ascii_case(pkg_id)
            && !id.eq_ignore_ascii_case(game_name)
            && n != norm_game
            && n != norm_id
            && !matching_guids.iter().any(|g| g.eq_ignore_ascii_case(id))
    });

    // 3. Purge matching handlers from UI state
    this.handlers.retain(|h| {
        let norm_h_name = crate::hub_browser::normalize_for_match(&h.game_name);
        let norm_h_guid = crate::hub_browser::normalize_for_match(&h.guid);
        !(norm_h_name == norm_game
            || norm_h_name == norm_id
            || norm_h_guid == norm_game
            || norm_h_guid == norm_id
            || h.guid.eq_ignore_ascii_case(pkg_id)
            || h.game_name.eq_ignore_ascii_case(game_name)
            || matching_guids.contains(&h.guid))
    });
    this.handler_names.retain(|n| {
        let norm_n = crate::hub_browser::normalize_for_match(n);
        norm_n != norm_game
            && norm_n != norm_id
            && !n.eq_ignore_ascii_case(game_name)
            && !matching_guids.iter().any(|g| g.eq_ignore_ascii_case(n))
    });

    // 4. Update Hub package list items
    for p in &mut this.hub_packages {
        let norm_p_game = crate::hub_browser::normalize_for_match(&p.game_name);
        let norm_p_title = crate::hub_browser::normalize_for_match(&p.title);
        if p.id.eq_ignore_ascii_case(pkg_id)
            || norm_p_game == norm_game
            || norm_p_title == norm_game
            || matching_guids.iter().any(|g| g.eq_ignore_ascii_case(&p.id))
        {
            p.is_installed = false;
            p.has_update = false;
            p.is_up_to_date = false;
            p.installed_version = None;
        }
    }

    // 5. Update the inspecting modal package immediately
    if let Some(ref mut inspecting) = this.inspecting_hub_package {
        inspecting.is_installed = false;
        inspecting.has_update = false;
        inspecting.is_up_to_date = false;
        inspecting.installed_version = None;
    }

    // 6. Notify GameManager backend to delete script from disk
    if let Some(ref cb) = this.callbacks.on_remove_game {
        for g in &matching_guids {
            cb(g);
        }
        cb(pkg_id);
        cb(game_name);
    }

    this.status_message = format!("Uninstalled handler for '{}'.", game_name);
    this.show_osd(&format!("Uninstalled {}", game_name), 2000, cx);
    cx.notify();
}

/// Renders the complete native Hub Handler Details screen using NefosModal.
pub fn render_hub_details_screen(
    pkg: &HubPackageItem,
    is_installed: bool,
    install_progress: Option<f32>,
    active_tab: HubDetailTab,
    comments: &[HubComment],
    packages: &[HubPackage],
    is_loading: bool,
    has_extra_like: bool,
    cx: &mut Context<MainWindow>,
) -> AnyElement {
    let pkg_id = pkg.id.clone();
    let pkg_title = pkg.title.clone();
    let game_name = pkg.game_name.clone();
    let author_name = pkg.author.clone();
    let cover_url = pkg.cover_image_url.clone();
    let remote_version = pkg.version.clone();
    let max_players = pkg.max_players.clone();
    let total_downloads = pkg.downloads;
    let base_likes = pkg.likes;
    let display_likes = if has_extra_like {
        base_likes + 1
    } else {
        base_likes
    };
    let is_verified = pkg.is_verified;
    let desc_md = pkg.description.clone();

    // Extract tags
    let tags = if !pkg.tags.is_empty() {
        pkg.tags.clone()
    } else {
        vec!["Splitscreen".to_string(), "Multiplayer".to_string()]
    };

    // Sort packages by version descending
    let mut sorted_packages = packages.to_vec();
    sorted_packages.sort_by(|a, b| {
        let ver_a = a
            .meta
            .as_ref()
            .and_then(|m| nucleus_hub::value_to_i32(&m.handler_version))
            .unwrap_or(0);
        let ver_b = b
            .meta
            .as_ref()
            .and_then(|m| nucleus_hub::value_to_i32(&m.handler_version))
            .unwrap_or(0);
        ver_b.cmp(&ver_a)
    });

    let latest_js_code = sorted_packages
        .iter()
        .find_map(|p| {
            p.meta
                .as_ref()
                .and_then(|m| m.js_content.as_ref())
                .filter(|s| !s.trim().is_empty())
        })
        .cloned();

    let comments_count = if !comments.is_empty() {
        comments.len()
    } else {
        pkg.comments_count as usize
    };
    let versions_count = if !packages.is_empty() {
        packages.len()
    } else {
        1
    };

    let modal_id = ElementId::Name(format!("hub-details-modal-{}", pkg_id).into());
    let mut modal = NefosModal::new(modal_id, format!("{} - Details", game_name))
        .subtitle(format!("{} • By {}", pkg_title, author_name))
        .icon(IconName::Gamepad2)
        .width(NefosModalWidth::Lg)
        .max_h(px(760.0))
        .on_close(cx.listener(|this, _, _, cx| {
            this.inspecting_hub_package = None;
            cx.notify();
        }));

    // Configure Footer Action Buttons via NefosButton
    if let Some(progress) = install_progress {
        let pct = (progress * 100.0) as u32;
        modal = modal.action(
            div()
                .flex()
                .items_center()
                .gap_2()
                .h(px(36.0))
                .px_4()
                .rounded_md()
                .bg(theme::bg_active())
                .border_1()
                .border_color(theme::accent())
                .text_xs()
                .font_weight(FontWeight::BOLD)
                .text_color(theme::accent())
                .child(Icon::new(IconName::Loader).size_3p5())
                .child(format!("Downloading... {}%", pct))
                .into_any_element(),
        );
    } else if is_installed {
        let p_id_un = pkg_id.clone();
        let g_name_un = game_name.clone();
        modal = modal.action(
            NefosButton::danger("modal-btn-uninstall", "Uninstall")
                .icon(IconName::Trash)
                .size(NefosButtonSize::Md)
                .on_click(cx.listener(move |this, _, _, cx| {
                    perform_uninstall(&p_id_un, &g_name_un, this, cx);
                })),
        );

        let p_id_re = pkg_id.clone();
        let p_title_re = pkg_title.clone();
        modal = modal.action(if pkg.has_update {
            NefosButton::primary(
                "modal-btn-update",
                format!("Update Handler (v{})", remote_version),
            )
            .icon(IconName::RefreshCw)
            .size(NefosButtonSize::Md)
            .on_click(cx.listener(move |this, _, _, cx| {
                spawn_install_flow(&p_id_re, &p_title_re, this, cx);
            }))
        } else {
            NefosButton::secondary("modal-btn-reinstall", "Reinstall")
                .icon(IconName::RefreshCw)
                .size(NefosButtonSize::Md)
                .on_click(cx.listener(move |this, _, _, cx| {
                    spawn_install_flow(&p_id_re, &p_title_re, this, cx);
                }))
        });
    } else {
        let p_id_in = pkg_id.clone();
        let p_title_in = pkg_title.clone();
        modal = modal.action(
            NefosButton::primary("modal-btn-install", "Install Handler")
                .icon(IconName::Download)
                .size(NefosButtonSize::Md)
                .on_click(cx.listener(move |this, _, _, cx| {
                    spawn_install_flow(&p_id_in, &p_title_in, this, cx);
                })),
        );
    }

    let rep_id = pkg_id.clone();
    modal = modal.action(
        NefosButton::secondary("modal-btn-report-handler", "Report handler")
            .icon(IconName::TriangleAlert)
            .size(NefosButtonSize::Md)
            .tooltip("Report this handler for safety or breaking changes")
            .on_click(cx.listener(move |this, _, _, cx| {
                let url = format!("https://hub.splitscreen.me/handler/{}", rep_id);
                #[cfg(target_os = "windows")]
                let _ = std::process::Command::new("rundll32")
                    .args(["url.dll,FileProtocolHandler", &url])
                    .spawn();
                #[cfg(target_os = "linux")]
                let _ = std::process::Command::new("xdg-open").arg(&url).spawn();
                this.show_osd("Opening report page on Splitscreen.Me Hub...", 2500, cx);
            })),
    );

    modal = modal.action(
        NefosButton::secondary("modal-btn-close", "Close")
            .icon(IconName::Close)
            .size(NefosButtonSize::Md)
            .on_click(cx.listener(|this, _, _, cx| {
                this.inspecting_hub_package = None;
                cx.notify();
            })),
    );

    // Modal Body Content Viewport
    let body_element = div()
        .flex()
        .flex_col()
        .gap_5()
        .w_full()
        // 1. Header Title & Verified Badge Row
        .child(
            div()
                .flex()
                .items_baseline()
                .gap_3()
                .flex_wrap()
                .child(
                    div()
                        .text_2xl()
                        .font_weight(FontWeight::BOLD)
                        .text_color(theme::text_primary())
                        .child(game_name.clone()),
                )
                .child(
                    div()
                        .text_base()
                        .font_weight(FontWeight::MEDIUM)
                        .text_color(theme::text_muted())
                        .child(pkg_title.clone()),
                )
                .when(is_verified, |row| {
                    row.child(
                        NefosTag::new("Handler Verified")
                            .id("hub-details-verified-pill")
                            .tone(NefosTagTone::Success)
                            .icon(IconName::BadgeCheck)
                            .tooltip("The latest release of this handler has been validated by our team and is safe to use."),
                    )
                }),
        )
        // 2. Genre / Category Tags Row using NefosTag
        .child(
            div()
                .flex()
                .items_center()
                .gap_2()
                .flex_wrap()
                .children(tags.into_iter().enumerate().map(|(t_idx, tag)| {
                    NefosTag::new(tag)
                        .id(("hub-detail-tag", t_idx))
                        .tone(NefosTagTone::Neutral)
                })),
        )
        // 3. Metadata Metrics Bar using NefosTag (player counts, input types, versions, author)
        .child(
            div()
                .flex()
                .items_center()
                .gap_2p5()
                .p_3p5()
                .bg(theme::bg_panel())
                .rounded_xl()
                .border_1()
                .border_color(theme::border_color())
                .flex_wrap()
                .child(
                    NefosTag::new(format!("Max: {}", max_players))
                        .id("stat-max-players-tag")
                        .icon(IconName::Users)
                        .tone(NefosTagTone::Accent)
                        .tooltip("Maximum supported player count"),
                )
                .child(
                    NefosTag::new(if pkg.playable_controllers { "Gamepad: Supported" } else { "Gamepad: No" })
                        .id("stat-controller-tag")
                        .icon(IconName::Gamepad2)
                        .tone(if pkg.playable_controllers { NefosTagTone::Success } else { NefosTagTone::Neutral })
                        .tooltip(if pkg.playable_controllers { "Gamepads & controllers are supported" } else { "Gamepad not supported" }),
                )
                .child(
                    NefosTag::new(if pkg.playable_multi_mouse_keyboard {
                        "Multi K&M: ProtoInput"
                    } else if pkg.playable_mouse_keyboard {
                        "Single K&M"
                    } else {
                        "K&M: No"
                    })
                    .id("stat-km-tag")
                    .icon(IconName::Keyboard)
                    .tone(if pkg.playable_multi_mouse_keyboard { NefosTagTone::Accent } else { NefosTagTone::Neutral })
                    .tooltip(if pkg.playable_multi_mouse_keyboard { "Multiple keyboards & mice supported via ProtoInput" } else { "Keyboard & mouse support" }),
                )
                .child(
                    NefosTag::new(format!("Version: v{}", remote_version))
                        .id("stat-version-tag")
                        .icon(IconName::Package)
                        .tone(NefosTagTone::Neutral)
                        .tooltip(format!("Package Version: {}", remote_version)),
                )
                .child(
                    NefosTag::new(format!("Author: {}", author_name))
                        .id("stat-author-tag")
                        .icon(IconName::User)
                        .tone(NefosTagTone::Neutral)
                        .tooltip(format!("Handler Maintainer: {}", author_name)),
                )
                .child(
                    NefosTag::new(format!("{} hotness", display_likes))
                        .id("stat-hotness-tag")
                        .icon(IconName::Flame)
                        .tone(NefosTagTone::Warning)
                        .tooltip("Community upvotes"),
                )
                .child(
                    NefosTag::new(format!("{} downloads", format_number_k(total_downloads)))
                        .id("stat-downloads-tag")
                        .icon(IconName::Download)
                        .tone(NefosTagTone::Neutral)
                        .tooltip("Total downloads"),
                ),
        )
        // 4. Split Section: Left Description + Actions; Right Big Poster
        .child(
            div()
                .flex()
                .flex_row()
                .gap_8()
                .items_start()
                .w_full()
                // Left Column: Notes + Action Buttons
                .child(
                    div()
                        .flex_1()
                        .min_w_0()
                        .flex()
                        .flex_col()
                        .gap_4()
                        // Markdown Description using TextView::markdown
                        .child(
                            div()
                                .w_full()
                                .py_1()
                                .child(crate::markdown::render_markdown_with_id("hub-details-markdown", &desc_md)),
                        )
                        // Action Buttons Row using NefosButton
                        .child(
                            div()
                                .flex()
                                .items_center()
                                .gap_3()
                                .flex_wrap()
                                .child({
                                    let pkg_id_like = pkg_id.clone();
                                    NefosButton::new("btn-give-hotness")
                                        .label("Give hotness!")
                                        .icon(IconName::Flame)
                                        .size(NefosButtonSize::Md)
                                        .variant(if has_extra_like { NefosButtonVariant::Primary } else { NefosButtonVariant::Secondary })
                                        .selected(has_extra_like)
                                        .tooltip("Upvote this handler on the Splitscreen.Me Hub")
                                        .on_click(cx.listener(move |this, _, _, cx| {
                                            this.hub_detail_extra_likes.insert(pkg_id_like.clone());
                                            this.show_osd("Hotness vote sent to maintainer!", 2000, cx);
                                            cx.notify();
                                        }))
                                })
                                .child({
                                    let g_name = game_name.clone();
                                    NefosButton::secondary("btn-game-info", "Game info")
                                        .icon(IconName::Info)
                                        .size(NefosButtonSize::Md)
                                        .tooltip("Search game details and updates")
                                        .on_click(cx.listener(move |this, _, _, cx| {
                                            let query = format!("https://www.google.com/search?q={}+splitscreen+me", g_name.replace(' ', "+"));
                                            #[cfg(target_os = "windows")]
                                            let _ = std::process::Command::new("rundll32")
                                                .args(["url.dll,FileProtocolHandler", &query])
                                                .spawn();
                                            #[cfg(target_os = "linux")]
                                            let _ = std::process::Command::new("xdg-open")
                                                .arg(&query)
                                                .spawn();
                                            this.show_osd("Opening game informations...", 2000, cx);
                                        }))
                                }),
                        ),
                )
                // Right Column: Large 2:3 Cover Poster via NefosCover
                .child(
                    div()
                        .flex_shrink_0()
                        .w(px(200.0))
                        .h(px(300.0))
                        .child(
                            NefosCover::new(&game_name, px(200.0), px(300.0))
                                .cover_url(cover_url.clone())
                                .aspect_ratio(2.0 / 3.0)
                                .installed(is_installed)
                                .when(is_installed, |c| {
                                    c.badge(
                                        div()
                                            .absolute()
                                            .top(px(4.0))
                                            .left(px(4.0))
                                            .size(px(20.0))
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
                                            .child(Icon::new(IconName::Check).size_3p5()),
                                    )
                                }),
                        ),
                ),
        )
        // 5. Tabs Bar (Comments, JS Code, Versions history, Statistics)
        .child(
            div()
                .flex()
                .items_center()
                .gap_6()
                .border_b_1()
                .border_color(theme::border_color())
                .pt_2()
                .children([
                    HubDetailTab::Comments,
                    HubDetailTab::JsCode,
                    HubDetailTab::VersionHistory,
                    HubDetailTab::Statistics,
                ].into_iter().enumerate().map(|(t_idx, tab)| {
                    let is_active = active_tab == tab;
                    div()
                        .id(("hub-detail-tab-btn", t_idx))
                        .cursor_pointer()
                        .flex()
                        .items_center()
                        .gap_2()
                        .pb_2p5()
                        .border_b_2()
                        .border_color(if is_active { theme::accent() } else { gpui::transparent_black() })
                        .text_sm()
                        .font_weight(if is_active { FontWeight::BOLD } else { FontWeight::MEDIUM })
                        .text_color(if is_active { theme::accent() } else { theme::text_muted() })
                        .hover(|s| s.text_color(theme::text_primary()))
                        .on_click(cx.listener(move |this, _, _, cx| {
                            this.hub_detail_tab = tab;
                            cx.notify();
                        }))
                        .child(Icon::new(tab.icon()).size_3p5())
                        .child(tab.label(comments_count, versions_count))
                })),
        )
        // 6. Tab Content Area
        .child(
            div()
                .w_full()
                .p_2()
                .child(match active_tab {
                    HubDetailTab::Comments => render_comments_tab(comments, is_loading, cx),
                    HubDetailTab::JsCode => render_js_code_tab(latest_js_code.as_deref(), is_loading, cx),
                    HubDetailTab::VersionHistory => render_versions_history_tab(&sorted_packages, &pkg_id, &pkg_title, is_loading, cx),
                    HubDetailTab::Statistics => render_statistics_tab(pkg, display_likes, comments_count, versions_count, cx),
                }),
        );

    modal.body(body_element).into_any_element()
}

/// Renders Tab 1: Comments list matching the reference image, using TextView::markdown for comment contents.
fn render_comments_tab(
    comments: &[HubComment],
    is_loading: bool,
    _cx: &mut Context<MainWindow>,
) -> AnyElement {
    if is_loading {
        return div()
            .flex()
            .items_center()
            .justify_center()
            .p_8()
            .gap_2()
            .text_sm()
            .text_color(theme::text_muted())
            .child(Icon::new(IconName::Loader).size_4())
            .child("Loading community comments...")
            .into_any_element();
    }

    div()
        .flex()
        .flex_col()
        .gap_4()
        .w_full()
        // Notice banner: "You must log in to post comments."
        .child(
            div()
                .flex()
                .items_center()
                .gap_2()
                .p_3()
                .bg(theme::bg_panel())
                .rounded_xl()
                .border_1()
                .border_color(theme::border_color())
                .text_xs()
                .text_color(theme::text_muted())
                .child(Icon::new(IconName::Info).size_3p5())
                .child("You must log in to post comments."),
        )
        // Comments List
        .child(if comments.is_empty() {
            div()
                .flex()
                .flex_col()
                .items_center()
                .justify_center()
                .p_8()
                .text_sm()
                .text_color(theme::text_muted())
                .child("No comments posted yet for this handler.")
                .into_any_element()
        } else {
            div()
                .flex()
                .flex_col()
                .gap_4()
                .children(comments.iter().enumerate().map(|(c_idx, comment)| {
                    let is_reply = comment.reply_to.is_some();
                    let initial = comment
                        .author
                        .chars()
                        .next()
                        .unwrap_or('?')
                        .to_uppercase()
                        .to_string();
                    let rel_time = format_relative_time(&comment.created_at);

                    div()
                        .id(("hub-comment-item", c_idx))
                        .flex()
                        .flex_col()
                        .gap_2()
                        .p_4()
                        .when(is_reply, |s| {
                            s.ml_6().border_l_2().border_color(theme::accent())
                        })
                        .bg(theme::bg_panel())
                        .rounded_xl()
                        .border_1()
                        .border_color(theme::border_color())
                        .child(
                            div()
                                .flex()
                                .items_center()
                                .gap_3()
                                // User Avatar
                                .child(
                                    div()
                                        .size(px(32.0))
                                        .rounded_full()
                                        .bg(theme::accent_subtle())
                                        .border_1()
                                        .border_color(theme::accent())
                                        .flex()
                                        .items_center()
                                        .justify_center()
                                        .text_xs()
                                        .font_weight(FontWeight::BOLD)
                                        .text_color(theme::accent())
                                        .child(initial),
                                )
                                // Author Name & Timestamp
                                .child(
                                    div()
                                        .flex()
                                        .items_center()
                                        .gap_2()
                                        .child(
                                            div()
                                                .text_sm()
                                                .font_weight(FontWeight::BOLD)
                                                .text_color(theme::text_primary())
                                                .child(comment.author.clone()),
                                        )
                                        .child(
                                            div()
                                                .text_xs()
                                                .text_color(theme::text_muted())
                                                .child(rel_time),
                                        ),
                                ),
                        )
                        // Comment Body formatted via TextView::markdown
                        .child(div().text_sm().text_color(theme::text_primary()).child(
                            crate::markdown::render_markdown_with_id(
                                ElementId::Name(format!("hub-comment-body-{}", c_idx).into()),
                                &comment.content,
                            ),
                        ))
                        // Reply action link
                        .child(
                            div()
                                .id(("hub-comment-reply", c_idx))
                                .cursor_pointer()
                                .text_xs()
                                .font_weight(FontWeight::SEMIBOLD)
                                .text_color(theme::accent())
                                .hover(|s| s.underline())
                                .child("Reply"),
                        )
                }))
                .into_any_element()
        })
        .into_any_element()
}

/// Renders Tab 2: JavaScript Code Viewer with TextView::markdown syntax styling.
fn render_js_code_tab(
    js_code: Option<&str>,
    is_loading: bool,
    cx: &mut Context<MainWindow>,
) -> AnyElement {
    if is_loading {
        return div()
            .flex()
            .items_center()
            .justify_center()
            .p_8()
            .gap_2()
            .text_sm()
            .text_color(theme::text_muted())
            .child(Icon::new(IconName::Loader).size_4())
            .child("Loading handler JavaScript code...")
            .into_any_element();
    }

    let code = match js_code {
        Some(c) if !c.trim().is_empty() => c,
        _ => "/* No JavaScript content available for this handler release. */",
    };

    let line_count = code.lines().count();
    let size_bytes = code.len();
    let copy_code = code.to_string();

    div()
        .flex()
        .flex_col()
        .gap_3()
        .w_full()
        // Code header bar
        .child(
            div()
                .flex()
                .items_center()
                .justify_between()
                .p_3()
                .bg(theme::bg_panel())
                .rounded_xl()
                .border_1()
                .border_color(theme::border_color())
                .child(
                    div()
                        .flex()
                        .items_center()
                        .gap_2()
                        .child(Icon::new(IconName::FileCode).size_3p5())
                        .child(
                            div()
                                .text_xs()
                                .font_weight(FontWeight::BOLD)
                                .text_color(theme::text_primary())
                                .child("handler.js"),
                        )
                        .child(
                            div()
                                .text_xs()
                                .text_color(theme::text_muted())
                                .child(format!("• {} lines • {} bytes", line_count, size_bytes)),
                        ),
                )
                .child(
                    NefosButton::secondary("btn-copy-js-code", "Copy Script")
                        .icon(IconName::Copy)
                        .size(NefosButtonSize::Sm)
                        .on_click(cx.listener(move |this, _, _, cx| {
                            cx.write_to_clipboard(gpui::ClipboardItem::new_string(
                                copy_code.clone(),
                            ));
                            this.show_osd("Handler JavaScript copied to clipboard!", 2500, cx);
                        })),
                ),
        )
        // Monospace Code Container with TextView::markdown
        .child(
            div()
                .id("js-code-box-scroll")
                .w_full()
                .max_h(px(380.0))
                .overflow_y_scroll()
                .p_4()
                .bg(theme::bg_subtle())
                .rounded_xl()
                .border_1()
                .border_color(theme::border_color())
                .child(
                    div()
                        .font_family("Consolas, Monaco, Courier New, monospace")
                        .text_xs()
                        .line_height(relative(1.5))
                        .text_color(theme::text_primary())
                        .child(crate::markdown::render_markdown_with_id(
                            "hub-js-code-markdown",
                            &format!("```javascript\n{}\n```", code),
                        )),
                ),
        )
        .into_any_element()
}

/// Renders Tab 3: Versions history list with NefosTag and TextView::markdown.
fn render_versions_history_tab(
    packages: &[HubPackage],
    pkg_id: &str,
    pkg_title: &str,
    is_loading: bool,
    cx: &mut Context<MainWindow>,
) -> AnyElement {
    if is_loading {
        return div()
            .flex()
            .items_center()
            .justify_center()
            .p_8()
            .gap_2()
            .text_sm()
            .text_color(theme::text_muted())
            .child(Icon::new(IconName::Loader).size_4())
            .child("Loading release versions...")
            .into_any_element();
    }

    if packages.is_empty() {
        return div()
            .flex()
            .flex_col()
            .items_center()
            .justify_center()
            .p_8()
            .text_sm()
            .text_color(theme::text_muted())
            .child("No version history available for this handler.")
            .into_any_element();
    }

    div()
        .flex()
        .flex_col()
        .gap_3()
        .w_full()
        .children(packages.iter().enumerate().map(|(idx, package)| {
            let meta = package.meta.as_ref();
            let ver_num = meta
                .and_then(|m| nucleus_hub::value_to_i32(&m.handler_version))
                .unwrap_or(1);
            let rel_desc = meta
                .and_then(|m| m.release_description.as_deref())
                .unwrap_or("Regular release update.")
                .to_string();
            let rel_date = meta.and_then(|m| m.release_date.as_deref()).unwrap_or("");
            let downloads = meta
                .and_then(|m| nucleus_hub::value_to_u64(&m.downloads))
                .unwrap_or(0);
            let is_latest = idx == 0;
            let formatted_date = format_relative_time(rel_date);

            let dl_pkg_id = package.id.clone();
            let dl_title = pkg_title.to_string();
            let dl_handler_id = pkg_id.to_string();

            div()
                .id(("hub-ver-item", idx))
                .flex()
                .items_start()
                .justify_between()
                .gap_4()
                .p_4()
                .bg(theme::bg_panel())
                .rounded_xl()
                .border_1()
                .border_color(if is_latest {
                    theme::accent()
                } else {
                    theme::border_color()
                })
                .child(
                    div()
                        .flex_1()
                        .min_w_0()
                        .flex()
                        .flex_col()
                        .gap_1p5()
                        // Version badge + Release date + Downloads using NefosTag
                        .child(
                            div()
                                .flex()
                                .items_center()
                                .gap_2p5()
                                .child(
                                    NefosTag::new(format!("v{}", ver_num))
                                        .id(("hub-ver-pill", idx))
                                        .tone(if is_latest {
                                            NefosTagTone::Accent
                                        } else {
                                            NefosTagTone::Neutral
                                        }),
                                )
                                .when(is_latest, |r| {
                                    r.child(
                                        NefosTag::new("Latest")
                                            .id(("hub-ver-latest", idx))
                                            .tone(NefosTagTone::Success),
                                    )
                                })
                                .child(
                                    div()
                                        .text_xs()
                                        .text_color(theme::text_muted())
                                        .child(format!("Released {}", formatted_date)),
                                )
                                .child(
                                    div()
                                        .text_xs()
                                        .text_color(theme::text_muted())
                                        .child(format!(
                                            "• {} downloads",
                                            format_number_k(downloads)
                                        )),
                                ),
                        )
                        // Changelog notes via TextView::markdown
                        .child(div().text_sm().text_color(theme::text_primary()).child(
                            crate::markdown::render_markdown_with_id(
                                ElementId::Name(format!("ver-desc-md-{}", idx).into()),
                                &rel_desc,
                            ),
                        )),
                )
                // Download button for this specific version via NefosButton
                .child(
                    NefosButton::secondary(("btn-dl-ver", idx), format!("Download v{}", ver_num))
                        .icon(IconName::Download)
                        .size(NefosButtonSize::Sm)
                        .on_click(cx.listener(move |this, _, _, cx| {
                            let dl_id = dl_pkg_id.clone();
                            let dl_h_id = dl_handler_id.clone();
                            let title = dl_title.clone();
                            this.installing_handlers.insert(dl_h_id.clone(), 0.0);
                            this.status_message =
                                format!("Downloading version v{} of '{}'...", ver_num, title);
                            cx.notify();

                            if let Some(ref cb) = this.callbacks.on_install_handler {
                                let cb = cb.clone();
                                let (tx, mut rx) = tokio::sync::mpsc::unbounded_channel::<
                                    crate::models::HubInstallProgressMsg,
                                >();
                                let tx_prog = tx.clone();

                                std::thread::spawn(move || {
                                    let item = cb(
                                        &dl_id,
                                        Box::new(move |p: f32| {
                                            let _ = tx_prog.send(
                                                crate::models::HubInstallProgressMsg::Progress(p),
                                            );
                                        }),
                                    );
                                    let _ =
                                        tx.send(crate::models::HubInstallProgressMsg::Done(item));
                                });

                                cx.spawn(async move |this, cx| {
                                    while let Some(msg) = rx.recv().await {
                                        let h_key = dl_h_id.clone();
                                        let t_key = title.clone();
                                        let _ = this.update(&mut *cx, |this, cx| match msg {
                                            crate::models::HubInstallProgressMsg::Progress(p) => {
                                                this.installing_handlers.insert(h_key, p);
                                                cx.notify();
                                            }
                                            crate::models::HubInstallProgressMsg::Done(opt_h) => {
                                                this.installing_handlers.remove(&h_key);
                                                this.installed_package_ids.insert(h_key.clone());
                                                if let Some(h) = opt_h {
                                                    if !this
                                                        .handlers
                                                        .iter()
                                                        .any(|existing| existing.guid == h.guid)
                                                    {
                                                        this.handlers.push(h);
                                                    }
                                                }
                                                this.show_osd(
                                                    &format!("Installed {} v{}!", t_key, ver_num),
                                                    3000,
                                                    cx,
                                                );
                                                cx.notify();
                                            }
                                        });
                                    }
                                })
                                .detach();
                            }
                        })),
                )
        }))
        .into_any_element()
}

/// Renders Tab 4: Statistics analytics dashboard with NefosCard surface styling.
fn render_statistics_tab(
    pkg: &HubPackageItem,
    display_likes: u32,
    comments_count: usize,
    versions_count: usize,
    _cx: &mut Context<MainWindow>,
) -> AnyElement {
    div()
        .flex()
        .flex_col()
        .gap_4()
        .w_full()
        // Top row: 4 Metric Cards
        .child(
            div()
                .flex()
                .items_center()
                .gap_4()
                .w_full()
                .flex_wrap()
                .child(render_stat_box(
                    "Total Downloads",
                    &pkg.downloads.to_string(),
                    IconName::Download,
                    theme::accent(),
                ))
                .child(render_stat_box(
                    "Community Hotness",
                    &display_likes.to_string(),
                    IconName::Flame,
                    theme::warning(),
                ))
                .child(render_stat_box(
                    "Total Versions",
                    &format!("{} releases", versions_count),
                    IconName::Clock,
                    theme::success(),
                ))
                .child(render_stat_box(
                    "Comments Count",
                    &format!("{} comments", comments_count),
                    IconName::MessageSquare,
                    theme::purple(),
                )),
        )
        // Compatibility & Technical Breakdown Card
        .child(
            div()
                .flex()
                .flex_col()
                .gap_2p5()
                .p_4()
                .bg(theme::bg_panel())
                .rounded_xl()
                .border_1()
                .border_color(theme::border_color())
                .child(
                    div()
                        .text_sm()
                        .font_weight(FontWeight::BOLD)
                        .text_color(theme::text_primary())
                        .child("Compatibility Breakdown"),
                )
                .child(
                    div()
                        .flex()
                        .flex_col()
                        .gap_1p5()
                        .child(render_info_row("Supported Players", &pkg.max_players))
                        .child(render_info_row(
                            "Gamepads (XInput)",
                            if pkg.playable_controllers {
                                "Supported"
                            } else {
                                "Not supported"
                            },
                        ))
                        .child(render_info_row(
                            "Keyboards & Mice",
                            if pkg.playable_multi_mouse_keyboard {
                                "Multiple (ProtoInput)"
                            } else if pkg.playable_mouse_keyboard {
                                "Single"
                            } else {
                                "Not supported"
                            },
                        ))
                        .child(render_info_row(
                            "Verified by SplitscreenMe",
                            if pkg.is_verified {
                                "Yes (Official Release)"
                            } else {
                                "Community Contributed"
                            },
                        )),
                ),
        )
        // Release Lifecycle Card
        .child(
            div()
                .flex()
                .flex_col()
                .gap_2p5()
                .p_4()
                .bg(theme::bg_panel())
                .rounded_xl()
                .border_1()
                .border_color(theme::border_color())
                .child(
                    div()
                        .text_sm()
                        .font_weight(FontWeight::BOLD)
                        .text_color(theme::text_primary())
                        .child("Handler Lifecycle"),
                )
                .child(
                    div()
                        .flex()
                        .flex_col()
                        .gap_1p5()
                        .child(render_info_row("Author / Maintainer", &pkg.author))
                        .child(render_info_row("Package Version", &pkg.version))
                        .child(render_info_row(
                            "Created Date",
                            &format_relative_time(&pkg.created_at),
                        ))
                        .child(render_info_row(
                            "Last Updated",
                            &format_relative_time(&pkg.updated_at),
                        )),
                ),
        )
        .into_any_element()
}

/// Helper to render an analytics box card.
fn render_stat_box(
    title: &'static str,
    value: &str,
    icon: IconName,
    accent: gpui::Hsla,
) -> AnyElement {
    div()
        .flex_1()
        .min_w(px(140.0))
        .p_3p5()
        .bg(theme::bg_panel())
        .rounded_xl()
        .border_1()
        .border_color(theme::border_color())
        .flex()
        .flex_col()
        .gap_1()
        .child(
            div()
                .flex()
                .items_center()
                .justify_between()
                .child(
                    div()
                        .text_xs()
                        .font_weight(FontWeight::MEDIUM)
                        .text_color(theme::text_muted())
                        .child(title),
                )
                .child(div().text_color(accent).child(Icon::new(icon).size_3p5())),
        )
        .child(
            div()
                .text_lg()
                .font_weight(FontWeight::BOLD)
                .text_color(theme::text_primary())
                .child(value.to_string()),
        )
        .into_any_element()
}

/// Helper to render an info key-value row.
fn render_info_row(key: &'static str, val: &str) -> AnyElement {
    div()
        .flex()
        .items_center()
        .justify_between()
        .text_xs()
        .child(div().text_color(theme::text_muted()).child(key))
        .child(
            div()
                .font_weight(FontWeight::SEMIBOLD)
                .text_color(theme::text_primary())
                .child(val.to_string()),
        )
        .into_any_element()
}

/// Formats large numbers cleanly with 'k' or 'M' suffixes.
fn format_number_k(n: u64) -> String {
    if n >= 1_000_000 {
        format!("{:.1}M", n as f64 / 1_000_000.0)
    } else if n >= 1_000 {
        format!("{}k", n / 1_000)
    } else {
        n.to_string()
    }
}
