use gpui::{
    AnyElement, App, ClickEvent, ElementId, FontWeight, InteractiveElement, IntoElement,
    MouseButton, ParentElement, SharedString, StatefulInteractiveElement, Styled, Window, div, px,
};
use gpui_component::Icon;
use gpui_kit_assets::IconName;

use crate::theme;

/// A popover selector that replaces fragile cycling buttons with an explicit list of choices.
pub struct CouchPicker<T: Clone + PartialEq + 'static> {
    id: ElementId,
    title: SharedString,
    icon: Option<IconName>,
    options: Vec<(T, SharedString)>,
    selected: T,
    is_open: bool,
    is_focused: bool,
    on_toggle: Option<Box<dyn Fn(&ClickEvent, &mut Window, &mut App) + 'static>>,
    on_select: Option<Box<dyn Fn(T, &mut Window, &mut App) + 'static>>,
}

impl<T: Clone + PartialEq + 'static> CouchPicker<T> {
    pub fn new(id: impl Into<ElementId>, title: impl Into<SharedString>, selected: T) -> Self {
        Self {
            id: id.into(),
            title: title.into(),
            icon: None,
            options: Vec::new(),
            selected,
            is_open: false,
            is_focused: false,
            on_toggle: None,
            on_select: None,
        }
    }

    pub fn icon(mut self, icon: IconName) -> Self {
        self.icon = Some(icon);
        self
    }

    pub fn option(mut self, value: T, label: impl Into<SharedString>) -> Self {
        self.options.push((value, label.into()));
        self
    }

    pub fn open(mut self, open: bool) -> Self {
        self.is_open = open;
        self
    }

    pub fn focused(mut self, focused: bool) -> Self {
        self.is_focused = focused;
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
        F: Fn(T, &mut Window, &mut App) + 'static,
    {
        self.on_select = Some(Box::new(handler));
        self
    }

    pub fn into_any_element(self) -> AnyElement {
        let selected_label = self
            .options
            .iter()
            .find(|(val, _)| *val == self.selected)
            .map(|(_, lbl)| lbl.clone())
            .unwrap_or_else(|| self.title.clone());

        let is_open = self.is_open;
        let is_focused = self.is_focused;

        // 1. Trigger button
        let mut trigger = div()
            .id(self.id.clone())
            .cursor_pointer()
            .flex()
            .items_center()
            .gap_2()
            .px_3p5()
            .py_1p5()
            .rounded_lg()
            .bg(if is_open || is_focused {
                theme::accent_subtle()
            } else {
                theme::bg_subtle()
            })
            .border_1()
            .border_color(if is_open || is_focused {
                theme::accent()
            } else {
                theme::border_subtle()
            })
            .text_sm()
            .font_weight(FontWeight::MEDIUM)
            .text_color(if is_open || is_focused {
                theme::accent()
            } else {
                theme::text_primary()
            })
            .hover(|s| s.bg(theme::bg_hover()).border_color(theme::accent()));

        if let Some(icn) = self.icon {
            trigger = trigger.child(
                Icon::new(icn).size_4().text_color(if is_open || is_focused {
                    theme::accent()
                } else {
                    theme::text_muted()
                }),
            );
        }

        trigger = trigger.child(div().min_w_0().child(selected_label));
        trigger = trigger.child(
            Icon::new(if is_open {
                IconName::ChevronUp
            } else {
                IconName::ChevronDown
            })
            .size_4()
            .text_color(theme::text_muted()),
        );

        if let Some(h) = self.on_toggle {
            trigger = trigger.on_click(move |ev, win, cx| h(ev, win, cx));
        }

        // 2. Container
        let mut root = div().relative().child(trigger);

        // 3. Dropdown popover list (if open)
        if is_open {
            let mut menu = div()
                .absolute()
                .top_full()
                .left_0()
                .mt_1()
                .min_w(px(200.0))
                .flex()
                .flex_col()
                .p_1p5()
                .rounded_lg()
                .bg(theme::bg_panel())
                .border_1()
                .border_color(theme::border_color())
                .shadow_xl()
                .on_mouse_down(MouseButton::Left, |_, _, cx| {
                    cx.stop_propagation();
                });

            let on_select_rc = self.on_select.map(std::rc::Rc::new);

            for (opt_idx, (opt_val, opt_label)) in self.options.into_iter().enumerate() {
                let is_active = opt_val == self.selected;
                let opt_val_clone = opt_val.clone();
                let on_select = on_select_rc.clone();

                let mut item = div()
                    .id(("picker-item", opt_idx))
                    .cursor_pointer()
                    .flex()
                    .items_center()
                    .justify_between()
                    .px_3()
                    .py_2()
                    .rounded_md()
                    .text_sm()
                    .bg(if is_active {
                        theme::accent_subtle()
                    } else {
                        theme::bg_panel()
                    })
                    .text_color(if is_active {
                        theme::accent()
                    } else {
                        theme::text_primary()
                    })
                    .hover(|s| s.bg(theme::bg_hover()).text_color(theme::accent()))
                    .child(opt_label);

                if is_active {
                    item = item.child(Icon::new(IconName::Check).size_4().text_color(theme::accent()));
                }

                if let Some(handler) = on_select {
                    item = item.on_click(move |_, win, cx| {
                        handler(opt_val_clone.clone(), win, cx);
                    });
                }

                menu = menu.child(item);
            }

            root = root.child(menu);
        }

        root.into_any_element()
    }
}

impl<T: Clone + PartialEq + 'static> IntoElement for CouchPicker<T> {
    type Element = AnyElement;

    fn into_element(self) -> Self::Element {
        self.into_any_element()
    }
}
