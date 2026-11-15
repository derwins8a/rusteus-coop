//! # NefosLaunchPromptModal Component
//!
//! Apple-style interactive modal prompt for multi-instance game staging.
//! Renders player badges, handler instructions, optional countdown timers,
//! and Proceed/Cancel action controls.

use gpui::{
    AnyElement, App, ClickEvent, ElementId, FontWeight, InteractiveElement, IntoElement,
    MouseButton, ParentElement, SharedString, Styled, Window, div, px, rgb,
};
use gpui_component::Icon;
use gpui_kit_assets::IconName;

use crate::components::badge::CouchTag;
use crate::components::button::{CouchButton, CouchButtonSize, CouchButtonVariant};
use crate::theme;

/// Interactive launch prompt modal for user action during instance launch.
pub struct NefosLaunchPromptModal {
    id: ElementId,
    player_index: usize,
    player_count: usize,
    title: SharedString,
    message: SharedString,
    remaining_seconds: Option<u32>,
    on_proceed: Option<Box<dyn Fn(&ClickEvent, &mut Window, &mut App) + 'static>>,
    on_cancel: Option<Box<dyn Fn(&ClickEvent, &mut Window, &mut App) + 'static>>,
}

impl NefosLaunchPromptModal {
    pub fn new(player_index: usize, player_count: usize, message: impl Into<SharedString>) -> Self {
        Self {
            id: ElementId::Name(format!("launch-prompt-p{}", player_index).into()),
            player_index,
            player_count,
            title: "Instance Setup Action Required".into(),
            message: message.into(),
            remaining_seconds: None,
            on_proceed: None,
            on_cancel: None,
        }
    }

    pub fn title(mut self, title: impl Into<SharedString>) -> Self {
        self.title = title.into();
        self
    }

    pub fn countdown(mut self, seconds: Option<u32>) -> Self {
        self.remaining_seconds = seconds;
        self
    }

    pub fn on_proceed<F>(mut self, handler: F) -> Self
    where
        F: Fn(&ClickEvent, &mut Window, &mut App) + 'static,
    {
        self.on_proceed = Some(Box::new(handler));
        self
    }

    pub fn on_cancel<F>(mut self, handler: F) -> Self
    where
        F: Fn(&ClickEvent, &mut Window, &mut App) + 'static,
    {
        self.on_cancel = Some(Box::new(handler));
        self
    }

    pub fn into_any_element(self) -> AnyElement {
        let player_label = format!("Player {} / {}", self.player_index + 1, self.player_count);

        let mut panel = div()
            .id(self.id.clone())
            .w(px(520.0))
            .p_6()
            .rounded_2xl()
            .bg(theme::bg_panel())
            .border_1()
            .border_color(theme::border_color())
            .shadow_xl()
            .flex()
            .flex_col()
            .gap_4()
            .on_mouse_down(MouseButton::Left, |_, _, _| {});

        // Header: Player Badge + Title
        let header = div()
            .flex()
            .flex_row()
            .items_center()
            .justify_between()
            .child(
                div()
                    .flex()
                    .flex_row()
                    .items_center()
                    .gap_3()
                    .child(CouchTag::player(self.player_index, player_label).into_any_element())
                    .child(
                        div()
                            .text_base()
                            .font_weight(FontWeight::SEMIBOLD)
                            .text_color(theme::text_primary())
                            .child(self.title),
                    ),
            );

        panel = panel.child(header);

        // Body: Handler instructions message
        let body = div()
            .p_4()
            .rounded_xl()
            .bg(theme::bg_subtle())
            .border_1()
            .border_color(theme::border_color().opacity(0.5))
            .child(
                div()
                    .text_sm()
                    .text_color(theme::text_primary())
                    .line_height(px(22.0))
                    .child(self.message),
            );

        panel = panel.child(body);

        // Countdown Timer indicator (if enabled)
        if let Some(sec) = self.remaining_seconds {
            let timer_row = div()
                .flex()
                .flex_row()
                .items_center()
                .justify_between()
                .px_2()
                .child(
                    div()
                        .flex()
                        .flex_row()
                        .items_center()
                        .gap_2()
                        .text_color(theme::accent())
                        .child(Icon::new(IconName::Timer).size(px(16.0)))
                        .child(
                            div()
                                .text_xs()
                                .font_weight(FontWeight::MEDIUM)
                                .child(format!("Auto-proceeding in {}s...", sec)),
                        ),
                );
            panel = panel.child(timer_row);
        }

        // Actions: Proceed Now & Cancel Launch
        let mut footer = div()
            .flex()
            .flex_row()
            .items_center()
            .justify_end()
            .gap_3()
            .pt_2();

        if let Some(cancel_handler) = self.on_cancel {
            let cancel_btn = CouchButton::new("Cancel Launch")
                .variant(CouchButtonVariant::Ghost)
                .size(CouchButtonSize::Md)
                .on_click(move |ev, window, app| {
                    (cancel_handler)(ev, window, app);
                });
            footer = footer.child(cancel_btn);
        }

        if let Some(proceed_handler) = self.on_proceed {
            let proceed_btn = CouchButton::new("Proceed Now")
                .variant(CouchButtonVariant::Primary)
                .size(CouchButtonSize::Md)
                .icon(IconName::Check)
                .on_click(move |ev, window, app| {
                    (proceed_handler)(ev, window, app);
                });
            footer = footer.child(proceed_btn);
        }

        panel = panel.child(footer);

        // Full screen scrim overlay
        div()
            .id(ElementId::Name(format!("{}-scrim", self.id).into()))
            .absolute()
            .inset_0()
            .bg(rgb(0x000000).opacity(0.65))
            .flex()
            .items_center()
            .justify_center()
            .occlude()
            .child(panel)
            .into_any_element()
    }
}

impl IntoElement for NefosLaunchPromptModal {
    type Element = AnyElement;

    fn into_element(self) -> Self::Element {
        self.into_any_element()
    }
}
