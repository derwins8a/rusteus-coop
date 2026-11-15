//! Handler details modal dialog for inspecting handler metadata and instructions.

use gpui::prelude::*;
use gpui::*;
use gpui_component::Icon;
use gpui_kit_assets::IconName;

use crate::models::HandlerItem;
use crate::nefos::*;
use crate::theme;
use crate::window::{ActiveTab, MainWindow};

/// Renders the handler inspection modal overlay using the `NefosModal` component.
///
/// Displays cover art, game title, handler author, GUID, badges for supported input methods,
/// scrollable rich Markdown documentation/instructions, and action buttons (Copy, Close, Configure).
pub fn render_handler_details_modal(
    h: &HandlerItem,
    cx: &mut gpui::Context<MainWindow>,
) -> gpui::AnyElement {
    let h_name = h.game_name.clone();
    let desc_md = h.description.clone();

    // Top info block: Cover art + title, guid, and metadata tags
    let header_meta = div()
        .flex()
        .flex_row()
        .items_start()
        .gap_4()
        .child(if let Some(ref cover) = h.cover_image_url {
            NefosCover::new(&h.game_name, px(68.0), px(96.0))
                .cover_url(Some(cover.clone()))
                .into_any_element()
        } else {
            div()
                .w(px(68.0))
                .h(px(96.0))
                .rounded_lg()
                .bg(theme::accent_subtle())
                .border_1()
                .border_color(theme::border_color())
                .flex()
                .items_center()
                .justify_center()
                .child(
                    Icon::new(IconName::Gamepad2)
                        .size(px(28.0))
                        .text_color(theme::accent()),
                )
                .into_any_element()
        })
        .child(
            div()
                .flex()
                .flex_col()
                .gap_1p5()
                .flex_1()
                .min_w_0()
                .child(
                    div()
                        .text_lg()
                        .font_weight(FontWeight::BOLD)
                        .text_color(theme::text_primary())
                        .child(h.game_name.clone()),
                )
                .child(
                    div()
                        .text_xs()
                        .text_color(theme::text_muted())
                        .child(format!("{} • GUID: {}", h.handler_title, h.guid)),
                )
                .child(
                    div()
                        .flex()
                        .flex_wrap()
                        .items_center()
                        .gap_2()
                        .child(
                            NefosTag::new(format!("Up to {} Players", h.max_players))
                                .tone(NefosTagTone::Accent),
                        )
                        .when(!h.executable_name.is_empty(), |el| {
                            el.child(
                                NefosTag::new(h.executable_name.clone())
                                    .tone(NefosTagTone::Neutral),
                            )
                        })
                        .when(h.supports_xinput, |el| {
                            el.child(
                                NefosTag::new("XInput")
                                    .tone(NefosTagTone::Success)
                                    .icon(IconName::Gamepad),
                            )
                        })
                        .when(h.supports_dinput, |el| {
                            el.child(
                                NefosTag::new("DirectInput")
                                    .tone(NefosTagTone::Neutral)
                                    .icon(IconName::Gamepad2),
                            )
                        })
                        .when(
                            h.supports_raw_input || h.supports_multiple_keyboards_and_mice,
                            |el| {
                                el.child(
                                    NefosTag::new("RawInput (ProtoInput)")
                                        .tone(NefosTagTone::Warning)
                                        .icon(IconName::Zap),
                                )
                            },
                        )
                        .when(h.supports_keyboard, |el| {
                            el.child(
                                NefosTag::new("Keyboard / Mouse")
                                    .tone(NefosTagTone::Neutral)
                                    .icon(IconName::Keyboard),
                            )
                        }),
                ),
        );

    // Markdown description scroll container
    let body_content = div().flex().flex_col().gap_3().child(header_meta).child(
        div()
            .id("handler-modal-desc-scroll")
            .h(px(320.0))
            .w_full()
            .min_w_0()
            .overflow_y_scroll()
            .p_4()
            .bg(theme::bg_subtle())
            .rounded_xl()
            .border_1()
            .border_color(theme::border_subtle())
            .child(crate::markdown::render_markdown_with_id(
                "library-details-notes",
                &desc_md,
            )),
    );

    let copy_text = desc_md.clone();
    let config_h_name = h_name.clone();

    NefosModal::new(
        "handler-details-dialog",
        format!("Game Handler - {}", h.game_name),
    )
    .subtitle("Splitscreen package inspection and setup guide")
    .icon(IconName::Info)
    .tone(NefosTagTone::Accent)
    .width(NefosModalWidth::Lg)
    .body(body_content)
    .on_close(cx.listener(|this, _, _, cx| {
        this.inspecting_handler = None;
        cx.notify();
    }))
    .action(
        NefosButton::secondary("btn-copy-handler-desc-modal", "Copy Notes")
            .icon(IconName::Copy)
            .on_click(cx.listener(move |this, _, _, cx| {
                cx.write_to_clipboard(gpui::ClipboardItem::new_string(copy_text.clone()));
                this.show_osd("Description copied to clipboard!", 2000, cx);
            })),
    )
    .action(
        NefosButton::secondary("btn-close-handler-modal-footer", "Close")
            .icon(IconName::X)
            .on_click(cx.listener(|this, _, _, cx| {
                this.inspecting_handler = None;
                cx.notify();
            })),
    )
    .action(
        NefosButton::primary("btn-configure-from-modal", "Configure Splitscreen")
            .icon(IconName::SlidersHorizontal)
            .on_click(cx.listener(move |this, _, _, cx| {
                this.selected_game = Some(config_h_name.clone());
                this.game_list.select_game(Some(config_h_name.clone()));
                this.active_tab = ActiveTab::Setup;
                this.inspecting_handler = None;
                this.status_message = format!("Configuring splitscreen for '{}'...", config_h_name);
                cx.notify();
            })),
    )
    .into_any_element()
}
