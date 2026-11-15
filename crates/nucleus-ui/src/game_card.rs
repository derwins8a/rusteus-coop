//! Single game card UI component.
//!
//! Visual tile displaying game title, cover artwork or thumbnail, developer info,
//! supported player counts, and quick-action play buttons.

use crate::models::HandlerItem;
use crate::nefos::*;
use crate::theme;
use crate::window::{ActiveTab, MainWindow};
use crate::{Icon, IconName};
use gpui::prelude::*;
use gpui::*;

/// Component representing an individual game tile in the library view.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct GameCardComponent {
    /// Associated game profile or handler identifier.
    pub id: String,
    /// Display title of the game.
    pub title: String,
    /// Maximum supported player count.
    pub max_players: i32,
    /// Path or resource URI for the cover art image.
    pub cover_image_path: Option<String>,
}

impl GameCardComponent {
    /// Creates a new `GameCardComponent`.
    pub fn new(id: impl Into<String>, title: impl Into<String>, max_players: i32) -> Self {
        Self {
            id: id.into(),
            title: title.into(),
            max_players,
            cover_image_path: None,
        }
    }
}

/// Standard row height inside the virtualized library list.
pub const LIBRARY_CARD_ROW_HEIGHT_PX: f32 = 124.0;

/// Formats Setup-screen Play Time and Last Played badge strings consistently.
pub fn format_setup_stats_labels(h: &HandlerItem) -> (String, String) {
    (
        format!("Play time: {}", h.formatted_play_time()),
        format!("Last played: {}", h.formatted_last_played()),
    )
}

/// Renders an individual game handler card inside the virtualized library list.
pub fn render_game_card(
    idx: usize,
    h: &HandlerItem,
    selected_game: Option<&str>,
    cx: &mut gpui::Context<MainWindow>,
) -> gpui::AnyElement {
    let _ = idx;
    let name_str = h.game_name.clone();
    let name_launch = h.game_name.clone();
    let is_selected =
        selected_game == Some(h.game_name.as_str()) || selected_game == Some(h.guid.as_str());

    let first_line = h
        .description
        .lines()
        .find(|l| !l.trim().is_empty())
        .unwrap_or("");
    let desc_preview = if first_line.chars().count() > 120 {
        format!("{}...", first_line.chars().take(120).collect::<String>())
    } else if first_line.is_empty() {
        "Split-screen multi-instance profile ready. Supports gamepad hooks & window placement."
            .to_string()
    } else {
        first_line.to_string()
    };

    // Card Row Container (for virtualized uniform_list height tracking)
    div()
        .id(ElementId::Name(format!("game-card-row-{}", h.guid).into()))
        .w_full()
        .h(px(LIBRARY_CARD_ROW_HEIGHT_PX))
        .pb_3()
        .on_mouse_down(MouseButton::Right, cx.listener({
            let card_h = h.clone();
            move |this, _, _, cx| {
                this.context_menu_handler = Some(card_h.clone());
                cx.notify();
            }
        }))
        .child(
            NefosCard::surface(ElementId::Name(format!("game-card-{}", h.guid).into()))
                .selected(is_selected)
                .hover_elevate(true)
                .paddings(false)
                .h(px(112.0))
                .child(
                    div()
                        .flex()
                        .items_center()
                        .justify_between()
                        .w_full()
                        .h_full()
                        .px_4()
                        .py_2()
                        // Left side: Cover Art + Title + Tags + Subtitle + Description
                        .child(
                            div()
                                .flex_1()
                                .min_w_0()
                                .overflow_hidden()
                                .flex()
                                .items_center()
                                .gap_3p5()
                                // Game Cover Image with strict 2:3 aspect ratio (56x84) — never shows checkmark badge in Library
                                .child(
                                    div()
                                        .when(!h.is_installed, |c| c.opacity(0.45))
                                        .child(
                                            NefosCover::new(&h.game_name, px(56.0), px(84.0))
                                                .cover_url(h.cover_image_url.clone())
                                                .aspect_ratio(2.0 / 3.0)
                                                .installed(h.is_installed)
                                                .selected(is_selected),
                                        ),
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
                                        // Row 1: Favorite Star + Game Title ONLY
                                        .child(
                                            div()
                                                .flex()
                                                .items_center()
                                                .gap_2()
                                                .min_w_0()
                                                .child(
                                                    div()
                                                        .id(ElementId::Name(format!("btn-fav-star-{}", h.guid).into()))
                                                        .cursor_pointer()
                                                        .text_sm()
                                                        .text_color(if h.is_favorite { rgb(0xf59e0b).into() } else { theme::text_muted() })
                                                        .hover(|s| s.opacity(0.8))
                                                        .tooltip(crate::make_tooltip(if h.is_favorite {
                                                            "Remove from Favorites"
                                                        } else {
                                                            "Add to Favorites"
                                                        }))
                                                        .on_click(cx.listener({
                                                            let fav_guid = h.guid.clone();
                                                            let new_fav = !h.is_favorite;
                                                            move |this, _, _, cx| {
                                                                if let Some(item) = this.handlers.iter_mut().find(|it| it.guid == fav_guid) {
                                                                    item.is_favorite = new_fav;
                                                                }
                                                                if let Some(ref cb) = this.callbacks.on_toggle_favorite {
                                                                    cb(&fav_guid, new_fav);
                                                                }
                                                                cx.notify();
                                                            }
                                                        }))
                                                        .child(
                                                            Icon::new(IconName::Star)
                                                                .size(px(14.0))
                                                                .text_color(if h.is_favorite { theme::warning() } else { theme::text_muted() })
                                                        ),
                                                )
                                                .child(
                                                    div()
                                                        .text_base()
                                                        .font_weight(FontWeight::BOLD)
                                                        .text_color(if h.is_installed { theme::text_primary() } else { theme::text_muted() })
                                                        .truncate()
                                                        .child(h.game_name.clone()),
                                                ),
                                        )
                                        // Row 2: Tags Row below Title
                                        .child({
                                            let mut tags_row = div()
                                                .flex()
                                                .items_center()
                                                .flex_wrap()
                                                .gap_1p5()
                                                .min_w_0();

                                            if h.is_verified {
                                                tags_row = tags_row.child(
                                                    NefosTag::new("Verified")
                                                        .id(format!("verified-pill-{}", h.guid))
                                                        .tone(NefosTagTone::Success)
                                                        .icon(IconName::BadgeCheck)
                                                        .tooltip("The latest release of this handler has been validated by our team and is safe to use."),
                                                );
                                            }

                                            if h.is_installed {
                                                tags_row = tags_row.child(
                                                    NefosTag::new(h.executable_name.clone())
                                                        .id(format!("exe-badge-{}", h.guid))
                                                        .tone(NefosTagTone::Neutral),
                                                );
                                            } else {
                                                tags_row = tags_row.child(
                                                    NefosTag::new("Uninstalled (.exe missing)")
                                                        .id(format!("uninstalled-badge-{}", h.guid))
                                                        .tone(NefosTagTone::Warning),
                                                );
                                            }

                                            let player_count_label = if h.max_players <= 1 {
                                                "1 Player".to_string()
                                            } else {
                                                format!("1-{} Players", h.max_players)
                                            };
                                            tags_row = tags_row.child(
                                                NefosTag::new(player_count_label)
                                                    .id(format!("players-badge-{}", h.guid))
                                                    .icon(IconName::Users)
                                                    .tone(if h.is_installed { NefosTagTone::Accent } else { NefosTagTone::Neutral }),
                                            );

                                            let has_gamepad = h.supports_gamepad || h.supports_xinput || h.supports_dinput;
                                            if has_gamepad {
                                                tags_row = tags_row.child(
                                                    NefosTag::new("Gamepad")
                                                        .id(format!("input-gamepad-{}", h.guid))
                                                        .icon(IconName::Gamepad2)
                                                        .tone(NefosTagTone::Neutral)
                                                        .tooltip(if h.supports_xinput && h.supports_dinput {
                                                            "Supports XInput and DirectInput gamepads"
                                                        } else if h.supports_xinput {
                                                            "Supports XInput gamepads"
                                                        } else if h.supports_dinput {
                                                            "Supports DirectInput gamepads"
                                                        } else {
                                                            "Supports gamepads"
                                                        }),
                                                );
                                            }

                                            let has_km = h.supports_keyboard || h.supports_raw_input || h.supports_multiple_keyboards_and_mice;
                                            if has_km {
                                                let is_proto = h.supports_raw_input || h.supports_multiple_keyboards_and_mice;
                                                tags_row = tags_row.child(
                                                    NefosTag::new("K&M")
                                                        .id(format!("input-km-{}", h.guid))
                                                        .icon(IconName::Keyboard)
                                                        .tone(if is_proto { NefosTagTone::Accent } else { NefosTagTone::Neutral })
                                                        .tooltip(if is_proto {
                                                            "Supports multiple keyboards & mice via ProtoInput"
                                                        } else {
                                                            "Supports keyboard & mouse"
                                                        }),
                                                );
                                            }

                                            tags_row
                                        })
                                        // Row 3: Handler Subtitle (Play Time / Last Played live on Setup screen)
                                        .child(
                                            div()
                                                .flex()
                                                .items_center()
                                                .gap_2()
                                                .text_xs()
                                                .text_color(theme::text_muted().opacity(0.85))
                                                .min_w_0()
                                                .child(
                                                    div()
                                                        .truncate()
                                                        .child(format!("{} • Handler: {}", h.handler_title, h.guid)),
                                                ),
                                        )
                                        // Row 4: Description Preview
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
                                                ),
                                        ),
                                ),
                        )
                        // Right side: Action buttons (Details on ALL cards; Locate .exe outline for uninstalled; Wrench/Setup/Play for installed)
                        .child({
                            let modal_h_act = h.clone();
                            div()
                                .flex_shrink_0()
                                .flex()
                                .items_center()
                                .gap_2()
                                .child(
                                    NefosButton::secondary(format!("btn-details-card-{}", h.guid), "Details")
                                        .size(NefosButtonSize::Sm)
                                        .icon(IconName::Info)
                                        .tooltip("View Handler Documentation & Notes")
                                        .on_click(cx.listener(move |this, _, _, cx| {
                                            this.inspecting_handler = Some(modal_h_act.clone());
                                            cx.notify();
                                        })),
                                )
                                .when(!h.is_installed, |this_act| {
                                    let h_guid = h.guid.clone();
                                    let h_name = h.game_name.clone();
                                    let expected_exe = h.executable_name.clone();
                                    this_act.child(
                                        NefosButton::success_outline(format!("btn-locate-card-{}", h.guid), "Locate .exe")
                                            .size(NefosButtonSize::Sm)
                                            .icon(IconName::FolderOpen)
                                            .tooltip("Locate Game Executable on Disk")
                                            .on_click(cx.listener(move |this, _, _, cx| {
                                                let filter = if !expected_exe.is_empty() {
                                                    format!("Game Executable ({})|{}|All Files (*.*)|*.*", expected_exe, expected_exe)
                                                } else {
                                                    "Executables (*.exe)|*.exe|All Files (*.*)|*.*".to_string()
                                                };
                                                let title = format!("Locate executable for '{}'", h_name);
                                                let h_guid = h_guid.clone();
                                                let h_name = h_name.clone();
                                                this.status_message = format!("Locating executable for '{}'...", h_name);
                                                cx.notify();
                                                cx.spawn(async move |this, cx| {
                                                    if let Some(exe) = crate::dialogs::pick_file_async(title, filter).await {
                                                        let exe_path = exe.to_string_lossy().to_string();
                                                        this.update(&mut *cx, |this, cx| {
                                                            if let Some(ref cb) = this.callbacks.on_add_game {
                                                                let _ = cb(&exe_path);
                                                            }
                                                            if let Some(item) = this.handlers.iter_mut().find(|it| it.guid == h_guid) {
                                                                item.is_installed = true;
                                                                item.executable_path = Some(exe_path.clone());
                                                            }
                                                            if !this.handler_names.contains(&h_name) {
                                                                this.handler_names.push(h_name.clone());
                                                            }
                                                            this.selected_game = Some(h_name.clone());
                                                            this.status_message = format!("Located executable for '{}': {}", h_name, exe_path);
                                                            cx.notify();
                                                        }).ok();
                                                    }
                                                }).detach();
                                            })),
                                    )
                                })
                                .when(h.is_installed, |this_act| {
                                    let name_str_cfg = name_str.clone();
                                    let name_launch_btn = name_launch.clone();
                                    let wrench_h = h.clone();
                                    this_act
                                        .child(
                                            NefosButton::secondary(format!("btn-wrench-card-{}", h.guid), "")
                                                .size(NefosButtonSize::Sm)
                                                .icon(IconName::Wrench)
                                                .tooltip("Game Options & Configuration")
                                                .on_click(cx.listener(move |this, _, _, cx| {
                                                    this.context_menu_handler = Some(wrench_h.clone());
                                                    cx.notify();
                                                })),
                                        )
                                        .child(
                                            NefosButton::secondary(format!("btn-setup-card-{}", h.guid), "Setup")
                                                .size(NefosButtonSize::Sm)
                                                .icon(IconName::SlidersHorizontal)
                                                .tooltip("Configure Splitscreen Settings")
                                                .on_click(cx.listener(move |this, _, _, cx| {
                                                    this.selected_game = Some(name_str_cfg.clone());
                                                    this.game_list.select_game(Some(name_str_cfg.clone()));
                                                    this.active_tab = ActiveTab::Setup;
                                                    this.refresh_saved_profiles();
                                                    this.status_message = format!("Configuring splitscreen for '{}'...", name_str_cfg);
                                                    cx.notify();
                                                })),
                                        )
                                        .child(
                                            NefosButton::primary(format!("btn-play-card-{}", h.guid), "Play")
                                                .size(NefosButtonSize::Sm)
                                                .icon(IconName::Play)
                                                .tooltip("Launch Splitscreen Session")
                                                .on_click(cx.listener(move |this, _, _, cx| {
                                                    println!("Launch clicked for game: {}", name_launch_btn);
                                                    this.selected_game = Some(name_launch_btn.clone());
                                                    if let Some(ref launcher) = this.callbacks.on_launch {
                                                        launcher(
                                                            &name_launch_btn,
                                                            this.player_count,
                                                            &this.selected_layout,
                                                            &this.selected_handler_options,
                                                            &this.player_device_assignments,
                                                            &this.player_monitor_assignments,
                                                            &this.player_audio_device_assignments,
                                                        );
                                                        this.status_message = format!("Launched session for '{}'! Press Ctrl+Q to toggle input lock.", name_launch_btn);
                                                    } else {
                                                        this.status_message = format!("Session launched for '{}'. Press Ctrl+Q to toggle input lock.", name_launch_btn);
                                                    }
                                                    cx.notify();
                                                })),
                                        )
                                })
                        }),
                ),
        )
        .into_any_element()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_game_card_component() {
        let card = GameCardComponent::new("g1", "Left 4 Dead 2", 4);
        assert_eq!(card.id, "g1");
        assert_eq!(card.title, "Left 4 Dead 2");
        assert_eq!(card.max_players, 4);
        assert_eq!(card.cover_image_path, None);
    }
}
