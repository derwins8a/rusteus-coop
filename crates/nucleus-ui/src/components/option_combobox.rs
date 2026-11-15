use gpui::prelude::FluentBuilder;
use gpui::{
    AnyElement, App, ClickEvent, ElementId, FocusHandle, FontWeight, InteractiveElement,
    IntoElement, KeyDownEvent, MouseButton, ParentElement, SharedString,
    StatefulInteractiveElement, Styled, Window, deferred, div, px,
};
use gpui_component::Icon;
use gpui_kit_assets::IconName;

use crate::components::option_row::{OPTION_CARD_MIN_HEIGHT_PX, OPTION_CONTROL_HEIGHT_PX};
use crate::theme;

/// Returns true when the provided option values represent a strict boolean pair
/// (`true`/`false`, `yes`/`no`, `1`/`0`, `enabled`/`disabled`, `on`/`off`).
pub fn is_boolean_values(values: &[String]) -> bool {
    if values.len() != 2 {
        return false;
    }
    let a = values[0].trim().to_ascii_lowercase();
    let b = values[1].trim().to_ascii_lowercase();
    matches!(
        (a.as_str(), b.as_str()),
        ("true", "false")
            | ("false", "true")
            | ("yes", "no")
            | ("no", "yes")
            | ("1", "0")
            | ("0", "1")
            | ("enabled", "disabled")
            | ("disabled", "enabled")
            | ("on", "off")
            | ("off", "on")
    )
}

/// Returns true when a handler option should render as a searchable, strictly validated
/// combobox (non-boolean options with more than 4 values).
pub fn should_use_combobox(values: &[String]) -> bool {
    !is_boolean_values(values) && values.len() > 4
}

/// Filters and ranks option values against the typed draft:
/// exact match first, prefix matches second, remaining substring matches third,
/// preserving relative order within each bucket. Empty/whitespace draft returns all values.
pub fn filter_combobox_options<'a>(values: &'a [String], draft: &str) -> Vec<&'a String> {
    let query = draft.trim().to_lowercase();
    if query.is_empty() {
        return values.iter().collect();
    }

    let mut exact = Vec::new();
    let mut prefix = Vec::new();
    let mut substring = Vec::new();

    for value in values {
        let lower = value.to_lowercase();
        if lower == query {
            exact.push(value);
        } else if lower.starts_with(&query) {
            prefix.push(value);
        } else if lower.contains(&query) {
            substring.push(value);
        }
    }

    exact.into_iter().chain(prefix).chain(substring).collect()
}

/// Validates that the trimmed draft matches one of `values` case-insensitively.
/// Returns `Ok(canonical_value)` on match or a descriptive `Err(...)` otherwise.
pub fn validate_combobox_input(draft: &str, values: &[String]) -> Result<String, String> {
    let trimmed = draft.trim();
    if let Some(canonical) = values
        .iter()
        .find(|v| v.trim().eq_ignore_ascii_case(trimmed))
    {
        return Ok(canonical.clone());
    }

    Err(format!(
        "Invalid option '{}': must match one of {} available choices",
        trimmed,
        values.len()
    ))
}

/// Searchable dropdown & strictly validated combobox row matching `CouchOptionRow` and `CouchToggle` geometry.
pub struct CouchOptionCombobox {
    id: ElementId,
    title: SharedString,
    description: SharedString,
    current_value: SharedString,
    values: Vec<String>,
    is_open: bool,
    search_draft: String,
    highlighted_idx: usize,
    validation_error: Option<String>,
    is_focused: bool,
    focus_handle: Option<FocusHandle>,
    on_toggle: Option<Box<dyn Fn(&ClickEvent, &mut Window, &mut App) + 'static>>,
    on_select: Option<Box<dyn Fn(String, &mut Window, &mut App) + 'static>>,
    on_key_down: Option<Box<dyn Fn(&KeyDownEvent, &mut Window, &mut App) + 'static>>,
    on_clear_draft: Option<Box<dyn Fn(&ClickEvent, &mut Window, &mut App) + 'static>>,
}

impl CouchOptionCombobox {
    pub fn new(
        id: impl Into<ElementId>,
        title: impl Into<SharedString>,
        description: impl Into<SharedString>,
        current_value: impl Into<SharedString>,
        values: Vec<String>,
    ) -> Self {
        Self {
            id: id.into(),
            title: title.into(),
            description: description.into(),
            current_value: current_value.into(),
            values,
            is_open: false,
            search_draft: String::new(),
            highlighted_idx: 0,
            validation_error: None,
            is_focused: false,
            focus_handle: None,
            on_toggle: None,
            on_select: None,
            on_key_down: None,
            on_clear_draft: None,
        }
    }

    pub fn is_open(mut self, is_open: bool) -> Self {
        self.is_open = is_open;
        self
    }

    pub fn search_draft(mut self, draft: impl Into<String>) -> Self {
        self.search_draft = draft.into();
        self
    }

    pub fn highlighted_idx(mut self, idx: usize) -> Self {
        self.highlighted_idx = idx;
        self
    }

    pub fn validation_error(mut self, error: Option<String>) -> Self {
        self.validation_error = error;
        self
    }

    pub fn focused(mut self, focused: bool) -> Self {
        self.is_focused = focused;
        self
    }

    pub fn focus_handle(mut self, handle: Option<&FocusHandle>) -> Self {
        self.focus_handle = handle.cloned();
        self
    }

    pub fn on_toggle<F>(mut self, handler: F) -> Self
    where
        F: Fn(&ClickEvent, &mut Window, &mut App) + 'static,
    {
        self.on_toggle = Some(Box::new(handler));
        self
    }

    pub fn on_select<F>(mut self, handler: F) -> Self
    where
        F: Fn(String, &mut Window, &mut App) + 'static,
    {
        self.on_select = Some(Box::new(handler));
        self
    }

    pub fn on_key_down<F>(mut self, handler: F) -> Self
    where
        F: Fn(&KeyDownEvent, &mut Window, &mut App) + 'static,
    {
        self.on_key_down = Some(Box::new(handler));
        self
    }

    pub fn on_clear_draft<F>(mut self, handler: F) -> Self
    where
        F: Fn(&ClickEvent, &mut Window, &mut App) + 'static,
    {
        self.on_clear_draft = Some(Box::new(handler));
        self
    }

    pub fn into_any_element(self) -> AnyElement {
        let has_error = self.validation_error.is_some();
        let has_desc = !self.description.trim().is_empty();
        let border_color = if has_error {
            theme::danger()
        } else if self.is_open || self.is_focused {
            theme::accent()
        } else {
            theme::border_subtle()
        };

        let row_bg = if self.is_open || self.is_focused {
            theme::bg_hover()
        } else {
            theme::bg_subtle()
        };

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

        let trigger_display: SharedString = if self.is_open && !self.search_draft.is_empty() {
            self.search_draft.clone().into()
        } else {
            self.current_value.clone()
        };

        let mut trigger_btn = div()
            .id((self.id.clone(), "combobox-trigger"))
            .cursor_pointer()
            .w_full()
            .h(px(OPTION_CONTROL_HEIGHT_PX))
            .px_3()
            .flex()
            .items_center()
            .justify_between()
            .gap_2()
            .rounded_md()
            .bg(theme::bg_panel())
            .border_1()
            .border_color(if has_error {
                theme::danger()
            } else if self.is_open {
                theme::accent()
            } else {
                theme::border_color()
            })
            .hover(|s| s.bg(theme::bg_hover()).border_color(theme::accent()));

        if let Some(ref handle) = self.focus_handle {
            trigger_btn = trigger_btn.track_focus(handle);
        }
        if let Some(on_toggle) = self.on_toggle {
            trigger_btn = trigger_btn.on_click(move |ev, win, cx| on_toggle(ev, win, cx));
        }
        if let Some(on_key) = self.on_key_down {
            trigger_btn = trigger_btn.on_key_down(move |ev, win, cx| on_key(ev, win, cx));
        }

        trigger_btn = trigger_btn
            .child(
                div()
                    .flex_1()
                    .min_w_0()
                    .text_xs()
                    .font_weight(FontWeight::SEMIBOLD)
                    .text_color(theme::text_primary())
                    .truncate()
                    .child(trigger_display),
            )
            .child(
                Icon::new(if self.is_open {
                    IconName::ChevronUp
                } else {
                    IconName::ChevronDown
                })
                .size_3p5()
                .text_color(theme::text_muted()),
            );

        let mut right_anchor = div()
            .relative()
            .flex_shrink_0()
            .h(px(OPTION_CONTROL_HEIGHT_PX))
            .min_w(px(220.0))
            .max_w(px(280.0))
            .flex()
            .items_center()
            .justify_end()
            .child(trigger_btn);

        if self.is_open {
            let filtered = filter_combobox_options(&self.values, &self.search_draft);
            let on_select_shared: Option<
                std::rc::Rc<dyn Fn(String, &mut Window, &mut App) + 'static>,
            > = self.on_select.map(std::rc::Rc::from);
            let clamped_hi = if filtered.is_empty() {
                0
            } else {
                self.highlighted_idx.min(filtered.len() - 1)
            };

            let mut search_bar = div()
                .flex()
                .items_center()
                .gap_2()
                .px_2p5()
                .py_1p5()
                .rounded_md()
                .bg(theme::bg_subtle())
                .border_1()
                .border_color(if has_error {
                    theme::danger()
                } else {
                    theme::accent()
                })
                .child(
                    Icon::new(IconName::Search)
                        .size_3p5()
                        .text_color(theme::text_muted()),
                )
                .child(
                    div()
                        .flex_1()
                        .min_w_0()
                        .text_xs()
                        .text_color(if self.search_draft.is_empty() {
                            theme::text_muted()
                        } else {
                            theme::text_primary()
                        })
                        .truncate()
                        .child(if self.search_draft.is_empty() {
                            "Type to filter choices...".to_string()
                        } else {
                            self.search_draft.clone()
                        }),
                );

            if !self.search_draft.is_empty() {
                let mut clear_btn = div()
                    .id((self.id.clone(), "combobox-clear"))
                    .cursor_pointer()
                    .p_0p5()
                    .rounded_sm()
                    .hover(|s| s.bg(theme::bg_hover()))
                    .child(
                        Icon::new(IconName::X)
                            .size_3()
                            .text_color(theme::text_muted()),
                    );
                if let Some(on_clear) = self.on_clear_draft {
                    clear_btn = clear_btn.on_click(move |ev, win, cx| on_clear(ev, win, cx));
                }
                search_bar = search_bar.child(clear_btn);
            }

            let mut list_box = div()
                .id(SharedString::from(format!("{:?}-list", self.id)))
                .flex()
                .flex_col()
                .gap_1()
                .max_h(px(220.0))
                .overflow_y_scroll();

            if filtered.is_empty() {
                list_box = list_box.child(
                    div()
                        .px_2p5()
                        .py_2()
                        .flex()
                        .items_center()
                        .gap_2()
                        .text_xs()
                        .text_color(theme::danger())
                        .child(
                            Icon::new(IconName::CircleAlert)
                                .size_3p5()
                                .text_color(theme::danger()),
                        )
                        .child("No matching options"),
                );
            } else {
                for (row_idx, candidate) in filtered.into_iter().enumerate() {
                    let is_current = candidate.as_str() == self.current_value.as_ref();
                    let is_highlighted = row_idx == clamped_hi;
                    let candidate_owned = candidate.clone();
                    let cb_clone = on_select_shared.clone();

                    let mut item = div()
                        .id(SharedString::from(format!("{:?}-opt-{}", self.id, row_idx)))
                        .cursor_pointer()
                        .px_2p5()
                        .py_1p5()
                        .rounded_md()
                        .flex()
                        .items_center()
                        .justify_between()
                        .gap_2()
                        .bg(if is_highlighted {
                            theme::accent_subtle()
                        } else {
                            theme::bg_panel()
                        })
                        .border_1()
                        .border_color(if is_highlighted {
                            theme::accent()
                        } else {
                            theme::border_subtle()
                        })
                        .hover(|s| s.bg(theme::bg_hover()))
                        .child(
                            div()
                                .flex_1()
                                .min_w_0()
                                .text_xs()
                                .font_weight(if is_current || is_highlighted {
                                    FontWeight::SEMIBOLD
                                } else {
                                    FontWeight::NORMAL
                                })
                                .text_color(theme::text_primary())
                                .truncate()
                                .child(candidate_owned.clone()),
                        );

                    if is_current {
                        item = item.child(
                            Icon::new(IconName::Check)
                                .size_3p5()
                                .text_color(theme::success()),
                        );
                    }

                    if let Some(cb) = cb_clone {
                        item = item.on_click(move |_, win, cx| {
                            cb(candidate_owned.clone(), win, cx);
                        });
                    }

                    list_box = list_box.child(item);
                }
            }

            let mut popover = div()
                .id((self.id.clone(), "combobox-popover"))
                .occlude()
                .on_mouse_down(MouseButton::Left, |_, _, cx| {
                    cx.stop_propagation();
                })
                .absolute()
                .top(px(OPTION_CONTROL_HEIGHT_PX + 6.0))
                .right_0()
                .w(px(280.0))
                .p_2()
                .flex()
                .flex_col()
                .gap_2()
                .rounded_lg()
                .bg(theme::bg_panel())
                .border_1()
                .border_color(theme::accent())
                .shadow_xl()
                .child(search_bar)
                .child(list_box);

            if let Some(ref err) = self.validation_error {
                popover = popover.child(
                    div()
                        .flex()
                        .items_center()
                        .gap_1p5()
                        .px_2()
                        .py_1()
                        .rounded_md()
                        .bg(theme::danger_subtle())
                        .text_xs()
                        .text_color(theme::danger())
                        .child(
                            Icon::new(IconName::CircleAlert)
                                .size_3()
                                .text_color(theme::danger()),
                        )
                        .child(err.clone()),
                );
            }

            right_anchor = right_anchor.child(deferred(popover).with_priority(10));
        }

        let main_row = div()
            .w_full()
            .flex()
            .items_center()
            .justify_between()
            .gap_3()
            .child(left_col)
            .child(right_anchor);

        let mut card = div()
            .id(self.id)
            .w_full()
            .min_h(px(OPTION_CARD_MIN_HEIGHT_PX))
            .flex()
            .flex_col()
            .justify_center()
            .gap_1p5()
            .px_3p5()
            .py_2p5()
            .bg(row_bg)
            .rounded_lg()
            .border_1()
            .border_color(border_color)
            .hover(|s| s.bg(theme::bg_panel()))
            .child(main_row);

        if let Some(err) = self.validation_error {
            card = card.child(
                div()
                    .flex()
                    .items_center()
                    .gap_1p5()
                    .text_xs()
                    .text_color(theme::danger())
                    .child(
                        Icon::new(IconName::CircleAlert)
                            .size_3()
                            .text_color(theme::danger()),
                    )
                    .child(err),
            );
        }

        card.into_any_element()
    }
}

impl IntoElement for CouchOptionCombobox {
    type Element = AnyElement;

    fn into_element(self) -> Self::Element {
        self.into_any_element()
    }
}
