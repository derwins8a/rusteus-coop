//! # NefosBanner Component
//!
//! Apple-style dismissible banner and alert surface engineered for the Nefos UI design system.
//! Used for startup pre-flight diagnostics, filesystem warnings, and system status alerts.

use gpui::{
    AnyElement, App, ClickEvent, ElementId, FontWeight, Hsla, InteractiveElement, IntoElement,
    ParentElement, SharedString, StatefulInteractiveElement, Styled, Window, div, px, rgb,
};
use gpui_component::Icon;
use gpui_kit_assets::IconName;

use crate::components::button::{CouchButton, CouchButtonSize, CouchButtonVariant};
use crate::theme;

/// Visual tone for the NefosBanner component.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum NefosBannerTone {
    #[default]
    Info,
    Warning,
    Danger,
    Success,
}

/// A dismissible banner component for notifications and system alerts in Nefos UI.
pub struct NefosBanner {
    id: ElementId,
    tone: NefosBannerTone,
    title: Option<SharedString>,
    message: SharedString,
    icon: Option<IconName>,
    action_label: Option<SharedString>,
    on_action: Option<Box<dyn Fn(&ClickEvent, &mut Window, &mut App) + 'static>>,
    on_dismiss: Option<Box<dyn Fn(&ClickEvent, &mut Window, &mut App) + 'static>>,
}

impl NefosBanner {
    /// Creates a new `NefosBanner` with default info styling.
    pub fn new(id: impl Into<ElementId>, message: impl Into<SharedString>) -> Self {
        Self {
            id: id.into(),
            tone: NefosBannerTone::Info,
            title: None,
            message: message.into(),
            icon: None,
            action_label: None,
            on_action: None,
            on_dismiss: None,
        }
    }

    /// Sets the banner tone.
    pub fn tone(mut self, tone: NefosBannerTone) -> Self {
        self.tone = tone;
        self
    }

    /// Sets an optional title header above the message.
    pub fn title(mut self, title: impl Into<SharedString>) -> Self {
        self.title = Some(title.into());
        self
    }

    /// Sets a custom icon for the banner.
    pub fn icon(mut self, icon: IconName) -> Self {
        self.icon = Some(icon);
        self
    }

    /// Attaches an action button with a click handler.
    pub fn action(
        mut self,
        label: impl Into<SharedString>,
        handler: impl Fn(&ClickEvent, &mut Window, &mut App) + 'static,
    ) -> Self {
        self.action_label = Some(label.into());
        self.on_action = Some(Box::new(handler));
        self
    }

    /// Attaches a dismiss handler (renders a close button).
    pub fn on_dismiss(
        mut self,
        handler: impl Fn(&ClickEvent, &mut Window, &mut App) + 'static,
    ) -> Self {
        self.on_dismiss = Some(Box::new(handler));
        self
    }

    /// Converts this banner into an `AnyElement`.
    pub fn into_any_element(self) -> AnyElement {
        let (bg_color, border_color, text_color, default_icon): (Hsla, Hsla, Hsla, IconName) =
            match self.tone {
                NefosBannerTone::Info => (
                    theme::accent().opacity(0.12),
                    theme::accent().opacity(0.35),
                    theme::accent(),
                    IconName::Info,
                ),
                NefosBannerTone::Warning => {
                    let warn: Hsla = rgb(0xf59e0b).into();
                    let warn_text: Hsla = rgb(0xfbbf24).into();
                    (
                        warn.opacity(0.12),
                        warn.opacity(0.35),
                        warn_text,
                        IconName::TriangleAlert,
                    )
                }
                NefosBannerTone::Danger => {
                    let danger: Hsla = rgb(0xef4444).into();
                    let danger_text: Hsla = rgb(0xf87171).into();
                    (
                        danger.opacity(0.12),
                        danger.opacity(0.35),
                        danger_text,
                        IconName::CircleAlert,
                    )
                }
                NefosBannerTone::Success => {
                    let success: Hsla = rgb(0x10b981).into();
                    let success_text: Hsla = rgb(0x34d399).into();
                    (
                        success.opacity(0.12),
                        success.opacity(0.35),
                        success_text,
                        IconName::CircleCheck,
                    )
                }
            };

        let icon_name = self.icon.unwrap_or(default_icon);

        let mut row = div()
            .id(self.id.clone())
            .w_full()
            .flex()
            .flex_row()
            .items_center()
            .justify_between()
            .px_4()
            .py_2p5()
            .rounded_xl()
            .bg(bg_color)
            .border_1()
            .border_color(border_color)
            .shadow_sm();

        // Left content: Icon + Title & Message
        let mut left_content = div().flex().flex_row().items_center().gap_3().flex_1();

        left_content = left_content.child(
            div()
                .flex_shrink_0()
                .text_color(text_color)
                .child(Icon::new(icon_name).size(px(18.0))),
        );

        let mut text_stack = div().flex().flex_col().gap_0p5();

        if let Some(title) = self.title {
            text_stack = text_stack.child(
                div()
                    .text_sm()
                    .font_weight(FontWeight::SEMIBOLD)
                    .text_color(theme::text_primary())
                    .child(title),
            );
        }

        text_stack = text_stack.child(
            div()
                .text_xs()
                .text_color(theme::text_muted())
                .child(self.message),
        );

        left_content = left_content.child(text_stack);
        row = row.child(left_content);

        // Right actions
        let mut right_actions = div().flex().flex_row().items_center().gap_2();

        if let (Some(label), Some(handler)) = (self.action_label, self.on_action) {
            let action_btn = CouchButton::new(label)
                .variant(CouchButtonVariant::Secondary)
                .size(CouchButtonSize::Sm)
                .on_click(move |ev, window, app| {
                    (handler)(ev, window, app);
                });
            right_actions = right_actions.child(action_btn);
        }

        if let Some(dismiss_handler) = self.on_dismiss {
            let dismiss_btn = div()
                .id(ElementId::Name(format!("{}-dismiss", self.id).into()))
                .p_1()
                .rounded_md()
                .cursor_pointer()
                .text_color(theme::text_muted())
                .hover(|s| s.bg(theme::bg_panel()).text_color(theme::text_primary()))
                .on_click(move |ev, window, app| {
                    (dismiss_handler)(ev, window, app);
                })
                .child(Icon::new(IconName::X).size(px(14.0)));
            right_actions = right_actions.child(dismiss_btn);
        }

        row.child(right_actions).into_any_element()
    }
}

impl IntoElement for NefosBanner {
    type Element = AnyElement;

    fn into_element(self) -> Self::Element {
        self.into_any_element()
    }
}
