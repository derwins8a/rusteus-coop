use gpui::prelude::FluentBuilder;
use gpui::{
    AnyElement, App, ClickEvent, ElementId, FontWeight, InteractiveElement, IntoElement,
    ParentElement, SharedString, StatefulInteractiveElement, Styled, Window, div, px,
};
use gpui_component::Icon;
use gpui_kit_assets::IconName;

use crate::theme;

/// Shared outer card minimum height across all handler option rows (toggles, steppers, comboboxes).
pub const OPTION_CARD_MIN_HEIGHT_PX: f32 = 56.0;
/// Shared interactive right-hand control height across all handler option rows.
pub const OPTION_CONTROL_HEIGHT_PX: f32 = 32.0;

/// A dedicated two-column option row for splitscreen handler options.
/// Eliminates text truncation and button clipping permanently.
pub struct CouchOptionRow {
    id: ElementId,
    title: SharedString,
    description: SharedString,
    current_value: SharedString,
    is_focused: bool,
    on_prev: Option<Box<dyn Fn(&ClickEvent, &mut Window, &mut App) + 'static>>,
    on_next: Option<Box<dyn Fn(&ClickEvent, &mut Window, &mut App) + 'static>>,
}

impl CouchOptionRow {
    pub fn new(
        id: impl Into<ElementId>,
        title: impl Into<SharedString>,
        description: impl Into<SharedString>,
        current_value: impl Into<SharedString>,
    ) -> Self {
        Self {
            id: id.into(),
            title: title.into(),
            description: description.into(),
            current_value: current_value.into(),
            is_focused: false,
            on_prev: None,
            on_next: None,
        }
    }

    pub fn focused(mut self, focused: bool) -> Self {
        self.is_focused = focused;
        self
    }

    pub fn on_prev<F>(mut self, handler: F) -> Self
    where
        F: Fn(&ClickEvent, &mut Window, &mut App) + 'static,
    {
        self.on_prev = Some(Box::new(handler));
        self
    }

    pub fn on_next<F>(mut self, handler: F) -> Self
    where
        F: Fn(&ClickEvent, &mut Window, &mut App) + 'static,
    {
        self.on_next = Some(Box::new(handler));
        self
    }

    pub fn into_any_element(self) -> AnyElement {
        let is_focused = self.is_focused;

        let row = div()
            .id(self.id.clone())
            .w_full()
            .min_h(px(OPTION_CARD_MIN_HEIGHT_PX))
            .flex()
            .items_center()
            .justify_between()
            .gap_3()
            .px_3p5()
            .py_2p5()
            .rounded_lg()
            .border_1()
            .border_color(if is_focused {
                theme::accent()
            } else {
                theme::border_subtle()
            })
            .bg(if is_focused {
                theme::bg_hover()
            } else {
                theme::bg_subtle()
            })
            .hover(|s| s.bg(theme::bg_panel()));

        // 1. Left column: Title and multi-line wrapping description
        let has_desc = !self.description.trim().is_empty();
        let left_col = div()
            .flex()
            .flex_col()
            .gap_0p5()
            .flex_1()
            .min_w_0()
            .pr_4()
            .child(
                div()
                    .text_xs()
                    .font_weight(FontWeight::SEMIBOLD)
                    .text_color(theme::text_primary())
                    .child(self.title),
            )
            .when(has_desc, |col| {
                col.child(
                    div()
                        .text_xs()
                        .text_color(theme::text_muted())
                        .child(self.description.clone()),
                )
            });

        // 2. Right column: Fixed controls (<, value, >) that never shrink or wrap
        let mut prev_btn = div()
            .id((self.id.clone(), "opt-prev"))
            .cursor_pointer()
            .flex()
            .items_center()
            .justify_center()
            .h(px(OPTION_CONTROL_HEIGHT_PX))
            .px_2p5()
            .rounded_md()
            .bg(theme::bg_subtle())
            .border_1()
            .border_color(theme::border_subtle())
            .hover(|s| s.bg(theme::bg_hover()).border_color(theme::accent()))
            .child(Icon::new(IconName::ChevronLeft).size_4().text_color(theme::text_primary()));

        if let Some(h) = self.on_prev {
            prev_btn = prev_btn.on_click(move |ev, win, cx| h(ev, win, cx));
        }

        let mut next_btn = div()
            .id((self.id.clone(), "opt-next"))
            .cursor_pointer()
            .flex()
            .items_center()
            .justify_center()
            .h(px(OPTION_CONTROL_HEIGHT_PX))
            .px_2p5()
            .rounded_md()
            .bg(theme::bg_subtle())
            .border_1()
            .border_color(theme::border_subtle())
            .hover(|s| s.bg(theme::bg_hover()).border_color(theme::accent()))
            .child(Icon::new(IconName::ChevronRight).size_4().text_color(theme::text_primary()));

        if let Some(h) = self.on_next {
            next_btn = next_btn.on_click(move |ev, win, cx| h(ev, win, cx));
        }

        let value_box = div()
            .h(px(OPTION_CONTROL_HEIGHT_PX))
            .min_w(px(140.0))
            .max_w(px(220.0))
            .px_3()
            .flex()
            .items_center()
            .justify_center()
            .rounded_md()
            .bg(theme::bg_panel())
            .border_1()
            .border_color(theme::border_color())
            .text_center()
            .text_xs()
            .font_weight(FontWeight::SEMIBOLD)
            .text_color(theme::text_primary())
            .child(self.current_value);

        let right_col = div()
            .flex_shrink_0()
            .h(px(OPTION_CONTROL_HEIGHT_PX))
            .min_w(px(220.0))
            .flex()
            .items_center()
            .justify_end()
            .gap_2()
            .child(prev_btn)
            .child(value_box)
            .child(next_btn);

        row.child(left_col).child(right_col).into_any_element()
    }
}

impl IntoElement for CouchOptionRow {
    type Element = AnyElement;

    fn into_element(self) -> Self::Element {
        self.into_any_element()
    }
}
