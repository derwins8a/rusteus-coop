use gpui::prelude::FluentBuilder;
use gpui::{
    AnyElement, App, ClickEvent, ElementId, FontWeight, InteractiveElement, IntoElement,
    ParentElement, SharedString, StatefulInteractiveElement, Styled, Window, div, px, rgb,
};

use crate::components::option_row::{OPTION_CARD_MIN_HEIGHT_PX, OPTION_CONTROL_HEIGHT_PX};
use crate::theme;

/// A standardized toggle / switch component for boolean application settings and options.
pub struct CouchToggle {
    id: ElementId,
    title: SharedString,
    description: Option<SharedString>,
    enabled: bool,
    disabled: bool,
    compact: bool,
    is_focused: bool,
    on_click: Option<Box<dyn Fn(&ClickEvent, &mut Window, &mut App) + 'static>>,
}

impl CouchToggle {
    pub fn new(id: impl Into<ElementId>, title: impl Into<SharedString>, enabled: bool) -> Self {
        Self {
            id: id.into(),
            title: title.into(),
            description: None,
            enabled,
            disabled: false,
            compact: false,
            is_focused: false,
            on_click: None,
        }
    }

    pub fn description(mut self, desc: impl Into<SharedString>) -> Self {
        self.description = Some(desc.into());
        self
    }

    pub fn disabled(mut self, disabled: bool) -> Self {
        self.disabled = disabled;
        self
    }

    pub fn compact(mut self, compact: bool) -> Self {
        self.compact = compact;
        self
    }

    pub fn focused(mut self, focused: bool) -> Self {
        self.is_focused = focused;
        self
    }

    pub fn on_click<F>(mut self, handler: F) -> Self
    where
        F: Fn(&ClickEvent, &mut Window, &mut App) + 'static,
    {
        self.on_click = Some(Box::new(handler));
        self
    }

    pub fn into_any_element(self) -> AnyElement {
        let toggle_pill = div()
            .flex_shrink_0()
            .flex()
            .items_center()
            .w(px(38.0))
            .h(px(22.0))
            .px_0p5()
            .rounded_full()
            .border_1()
            .when(self.enabled, |this| {
                this.bg(theme::accent())
                    .border_color(theme::accent())
                    .justify_end()
            })
            .when(!self.enabled, |this| {
                this.bg(theme::bg_active())
                    .border_color(theme::border_color())
                    .justify_start()
            })
            .child(
                div()
                    .size(px(16.0))
                    .rounded_full()
                    .bg(rgb(0xffffff))
                    .shadow_sm(),
            );

        if self.compact {
            let mut mini = div().id(self.id.clone()).flex().items_center().gap_2();

            if !self.disabled {
                mini = mini.cursor_pointer();
                if let Some(h) = self.on_click {
                    mini = mini.on_click(move |ev, win, cx| h(ev, win, cx));
                }
            } else {
                mini = mini.opacity(0.4).cursor_not_allowed();
            }

            return mini
                .child(toggle_pill)
                .child(
                    div()
                        .text_xs()
                        .font_weight(FontWeight::MEDIUM)
                        .text_color(theme::text_primary())
                        .child(self.title),
                )
                .into_any_element();
        }

        let mut row = div()
            .id(self.id.clone())
            .w_full()
            .min_h(px(OPTION_CARD_MIN_HEIGHT_PX))
            .flex()
            .items_center()
            .justify_between()
            .gap_3()
            .px_3p5()
            .py_2p5()
            .bg(if self.is_focused {
                theme::bg_hover()
            } else {
                theme::bg_subtle()
            })
            .rounded_lg()
            .border_1()
            .border_color(if self.is_focused {
                theme::accent()
            } else {
                theme::border_subtle()
            });

        if !self.disabled {
            row = row.cursor_pointer().hover(|s| s.bg(theme::bg_panel()));

            if let Some(h) = self.on_click {
                row = row.on_click(move |ev, win, cx| h(ev, win, cx));
            }
        } else {
            row = row.opacity(0.4).cursor_not_allowed();
        }

        let mut label_col = div().flex().flex_col().flex_1().min_w_0().gap_0p5().child(
            div()
                .text_xs()
                .font_weight(FontWeight::SEMIBOLD)
                .text_color(theme::text_primary())
                .child(self.title),
        );

        if let Some(desc) = self.description {
            if !desc.trim().is_empty() {
                label_col =
                    label_col.child(div().text_xs().text_color(theme::text_muted()).child(desc));
            }
        }

        let control_slot = div()
            .h(px(OPTION_CONTROL_HEIGHT_PX))
            .flex()
            .items_center()
            .justify_end()
            .flex_shrink_0()
            .child(toggle_pill);

        row.child(label_col).child(control_slot).into_any_element()
    }
}

impl IntoElement for CouchToggle {
    type Element = AnyElement;

    fn into_element(self) -> Self::Element {
        self.into_any_element()
    }
}
