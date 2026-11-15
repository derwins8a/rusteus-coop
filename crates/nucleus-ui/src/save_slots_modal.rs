//! Savegame slot management modal powered by Ludusavi CLI integration.

use std::sync::Arc;
use gpui::prelude::*;
use gpui::*;
use gpui_kit_assets::IconName;
use nucleus_profile::save_slots::SaveSlot;
use crate::components::badge::{CouchTag, TagTone};
use crate::components::button::{CouchButton, CouchButtonSize};
use crate::components::modal::{CouchModal, ModalWidth};
use crate::theme;

/// State for the savegame slots management modal.
#[derive(Debug, Clone, Default)]
pub struct SaveSlotsModalState {
    pub is_open: bool,
    pub game_guid: String,
    pub game_name: String,
    pub target_player_index: usize,
    pub new_slot_name_input: String,
    pub creating_new_slot: bool,
    pub error_message: Option<String>,
}

impl SaveSlotsModalState {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn open(&mut self, guid: impl Into<String>, name: impl Into<String>, player_idx: usize) {
        self.is_open = true;
        self.game_guid = guid.into();
        self.game_name = name.into();
        self.target_player_index = player_idx;
        self.new_slot_name_input.clear();
        self.creating_new_slot = false;
        self.error_message = None;
    }

    pub fn close(&mut self) {
        self.is_open = false;
        self.new_slot_name_input.clear();
        self.creating_new_slot = false;
        self.error_message = None;
    }
}

/// Formats byte counts into human-readable strings (B, KB, MB, GB).
fn format_bytes(bytes: u64) -> String {
    if bytes >= 1024 * 1024 * 1024 {
        format!("{:.1} GB", bytes as f64 / (1024.0 * 1024.0 * 1024.0))
    } else if bytes >= 1024 * 1024 {
        format!("{:.1} MB", bytes as f64 / (1024.0 * 1024.0))
    } else if bytes >= 1024 {
        format!("{:.1} KB", bytes as f64 / 1024.0)
    } else {
        format!("{} B", bytes)
    }
}

/// Renders the multi-instance save slot manager modal dialog.
pub fn render_save_slots_modal(
    state: &SaveSlotsModalState,
    slots: &[SaveSlot],
    active_slot_id: Option<&str>,
    on_select_slot: Option<Arc<dyn Fn(&str, usize, Option<String>) + Send + Sync>>,
    on_create_slot: Option<Arc<dyn Fn(&str, &str, &str, &str) + Send + Sync>>,
    on_clone_slot: Option<Arc<dyn Fn(&str, &str, &str) + Send + Sync>>,
    on_delete_slot: Option<Arc<dyn Fn(&str, &str) + Send + Sync>>,
    on_close: Arc<dyn Fn(&ClickEvent, &mut Window, &mut App) + 'static>,
) -> AnyElement {
    if !state.is_open {
        return div().into_any_element();
    }

    let title = format!("Save Slots: {}", state.game_name);
    let modal_close = on_close.clone();
    let modal = CouchModal::new("modal-save-slots", title)
        .subtitle(format!("Managing isolated saves for Player {}", state.target_player_index + 1))
        .icon(IconName::Save)
        .width(ModalWidth::Lg)
        .on_close(move |ev, win, app| modal_close(ev, win, app));

    let guid = state.game_guid.clone();
    let player_idx = state.target_player_index;

    let body = div()
        .flex()
        .flex_col()
        .gap_4()
        .p_4()
        .child(
            div()
                .flex()
                .items_center()
                .justify_between()
                .child(
                    div()
                        .text_xs()
                        .text_color(theme::text_muted())
                        .child("Each instance receives dedicated save redirection via Ludusavi snapshot syncing."),
                )
                .child(
                    CouchButton::primary("btn-new-slot", "New Save Slot")
                        .size(CouchButtonSize::Sm)
                        .icon(IconName::Plus)
                        .on_click({
                            let create_cb = on_create_slot.clone();
                            let g = guid.clone();
                            let default_name = format!("Player {} Slot", player_idx + 1);
                            move |_ev, _win, _app| {
                                if let Some(ref cb) = create_cb {
                                    cb(&g, &default_name, &g, "manual");
                                }
                            }
                        }),
                ),
        )
        .child(
            // Slots list container
            div()
                .id("save-slots-list-scroll")
                .flex()
                .flex_col()
                .gap_2()
                .max_h(px(340.0))
                .overflow_y_scroll()
                .children(if slots.is_empty() {
                    vec![div()
                        .flex()
                        .flex_col()
                        .items_center()
                        .justify_center()
                        .p_8()
                        .rounded_md()
                        .bg(theme::bg_panel())
                        .border_1()
                        .border_color(theme::border_color())
                        .gap_2()
                        .child(
                            div()
                                .text_sm()
                                .font_weight(FontWeight::SEMIBOLD)
                                .text_color(theme::text_primary())
                                .child("No Save Slots Configured"),
                        )
                        .child(
                            div()
                                .text_xs()
                                .text_color(theme::text_muted())
                                .child("Click 'New Save Slot' to create an isolated save profile for this game."),
                        )
                        .into_any_element()]
                } else {
                    slots
                        .iter()
                        .map(|slot| {
                            let is_assigned = active_slot_id == Some(&slot.slot_id);
                            let slot_id = slot.slot_id.clone();
                            let select_cb = on_select_slot.clone();
                            let clone_cb = on_clone_slot.clone();
                            let delete_cb = on_delete_slot.clone();
                            let g = guid.clone();

                            div()
                                .id(ElementId::Name(format!("slot-card-{}", slot.slot_id).into()))
                                .flex()
                                .items_center()
                                .justify_between()
                                .p_3()
                                .rounded_md()
                                .border_1()
                                .border_color(if is_assigned {
                                    theme::accent()
                                } else {
                                    theme::border_color()
                                })
                                .bg(if is_assigned {
                                    theme::accent_subtle()
                                } else {
                                    theme::bg_panel()
                                })
                                .child(
                                    div()
                                        .flex()
                                        .flex_col()
                                        .gap_1()
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
                                                        .child(slot.display_name.clone()),
                                                )
                                                .when(is_assigned, |this| {
                                                    this.child(
                                                        CouchTag::new("Active for Player")
                                                            .tone(TagTone::Success)
                                                            .icon(IconName::CircleCheck),
                                                    )
                                                }),
                                        )
                                        .child(
                                            div()
                                                .flex()
                                                .items_center()
                                                .gap_3()
                                                .text_xs()
                                                .text_color(theme::text_muted())
                                                .child(format!("ID: {}", slot.slot_id))
                                                .child(format!("Files: {}", slot.file_count))
                                                .child(format!("Size: {}", format_bytes(slot.total_bytes)))
                                                .child(format!("Snapshots: {}", slot.snapshot_count)),
                                        ),
                                )
                                .child(
                                    div()
                                        .flex()
                                        .items_center()
                                        .gap_2()
                                        .child(
                                            (if is_assigned {
                                                CouchButton::secondary(
                                                    ElementId::Name(format!("btn-select-{}", slot.slot_id).into()),
                                                    "Deselect",
                                                )
                                            } else {
                                                CouchButton::primary(
                                                    ElementId::Name(format!("btn-select-{}", slot.slot_id).into()),
                                                    "Assign",
                                                )
                                            })
                                            .size(CouchButtonSize::Sm)
                                            .icon(if is_assigned {
                                                IconName::X
                                            } else {
                                                IconName::Check
                                            })
                                            .on_click({
                                                let s_id = slot_id.clone();
                                                move |_ev, _win, _app| {
                                                    if let Some(ref cb) = select_cb {
                                                        let new_val = if is_assigned {
                                                            None
                                                        } else {
                                                            Some(s_id.clone())
                                                        };
                                                        cb(&g, player_idx, new_val);
                                                    }
                                                }
                                            }),
                                        )
                                        .child(
                                            CouchButton::ghost(
                                                ElementId::Name(format!("btn-clone-{}", slot.slot_id).into()),
                                                "Clone",
                                            )
                                            .size(CouchButtonSize::Sm)
                                            .icon(IconName::Copy)
                                            .on_click({
                                                let s_id = slot_id.clone();
                                                let g2 = guid.clone();
                                                let new_name = format!("{} (Copy)", slot.display_name);
                                                move |_ev, _win, _app| {
                                                    if let Some(ref cb) = clone_cb {
                                                        cb(&g2, &s_id, &new_name);
                                                    }
                                                }
                                            }),
                                        )
                                        .child(
                                            CouchButton::danger(
                                                ElementId::Name(format!("btn-del-{}", slot.slot_id).into()),
                                                "Delete",
                                            )
                                            .size(CouchButtonSize::Sm)
                                            .icon(IconName::X)
                                            .on_click({
                                                let s_id = slot_id.clone();
                                                let g3 = guid.clone();
                                                move |_ev, _win, _app| {
                                                    if let Some(ref cb) = delete_cb {
                                                        cb(&g3, &s_id);
                                                    }
                                                }
                                            }),
                                        ),
                                )
                                .into_any_element()
                        })
                        .collect()
                }),
        )
        .child(
            div()
                .flex()
                .justify_end()
                .pt_2()
                .child(
                    CouchButton::secondary("btn-close-save-slots", "Done")
                        .size(CouchButtonSize::Sm)
                        .on_click(move |ev, win, app| {
                            on_close(ev, win, app);
                        }),
                ),
        );

    modal.body(body.into_any_element()).into_any_element()
}
