use gpui::{
    AnyElement, App, ClickEvent, ElementId, FontWeight, InteractiveElement, IntoElement,
    ParentElement, SharedString, StatefulInteractiveElement, Styled, Window, div, px,
};
use gpui_component::Icon;
use gpui_kit_assets::IconName;

use crate::components::badge::CouchTag;
use crate::theme;

/// Player controller & K&M slot card for the Couch Setup screen.
pub struct CouchSlotCard {
    id: ElementId,
    player_index: usize,
    device_label: Option<SharedString>,
    device_guid: Option<SharedString>,
    is_kbm: bool,
    is_merged_kbm: bool,
    is_raw_device: bool,
    is_drop_target: bool,
    is_focused: bool,
    is_grabbed: bool,
    status_badge: Option<AnyElement>,
    on_click: Option<Box<dyn Fn(&ClickEvent, &mut Window, &mut App) + 'static>>,
    on_split: Option<Box<dyn Fn(&ClickEvent, &mut Window, &mut App) + 'static>>,
    on_remove: Option<Box<dyn Fn(&ClickEvent, &mut Window, &mut App) + 'static>>,
    on_ignore: Option<Box<dyn Fn(&ClickEvent, &mut Window, &mut App) + 'static>>,
}

impl CouchSlotCard {
    pub fn new(id: impl Into<ElementId>, player_index: usize) -> Self {
        Self {
            id: id.into(),
            player_index,
            device_label: None,
            device_guid: None,
            is_kbm: false,
            is_merged_kbm: false,
            is_raw_device: false,
            is_drop_target: false,
            is_focused: false,
            is_grabbed: false,
            status_badge: None,
            on_click: None,
            on_split: None,
            on_remove: None,
            on_ignore: None,
        }
    }

    pub fn device_label(mut self, label: Option<impl Into<SharedString>>) -> Self {
        self.device_label = label.map(|l| l.into());
        self
    }

    pub fn device_guid(mut self, guid: Option<impl Into<SharedString>>) -> Self {
        self.device_guid = guid.map(|g| g.into());
        self
    }

    pub fn kbm(mut self, is_kbm: bool) -> Self {
        self.is_kbm = is_kbm;
        self
    }

    pub fn merged_kbm(mut self, merged: bool) -> Self {
        self.is_merged_kbm = merged;
        self
    }

    pub fn raw_device(mut self, raw: bool) -> Self {
        self.is_raw_device = raw;
        self
    }

    pub fn drop_target(mut self, is_target: bool) -> Self {
        self.is_drop_target = is_target;
        self
    }

    pub fn focused(mut self, focused: bool) -> Self {
        self.is_focused = focused;
        self
    }

    pub fn grabbed(mut self, grabbed: bool) -> Self {
        self.is_grabbed = grabbed;
        self
    }

    pub fn status_badge(mut self, badge: impl IntoElement) -> Self {
        self.status_badge = Some(badge.into_any_element());
        self
    }

    pub fn on_click<F>(mut self, handler: F) -> Self
    where
        F: Fn(&ClickEvent, &mut Window, &mut App) + 'static,
    {
        self.on_click = Some(Box::new(handler));
        self
    }

    pub fn on_split<F>(mut self, handler: F) -> Self
    where
        F: Fn(&ClickEvent, &mut Window, &mut App) + 'static,
    {
        self.on_split = Some(Box::new(handler));
        self
    }

    pub fn on_remove<F>(mut self, handler: F) -> Self
    where
        F: Fn(&ClickEvent, &mut Window, &mut App) + 'static,
    {
        self.on_remove = Some(Box::new(handler));
        self
    }

    pub fn on_ignore<F>(mut self, handler: F) -> Self
    where
        F: Fn(&ClickEvent, &mut Window, &mut App) + 'static,
    {
        self.on_ignore = Some(Box::new(handler));
        self
    }

    pub fn into_any_element(self) -> AnyElement {
        let player_idx = self.player_index;
        let is_assigned = self.device_label.is_some();

        let border_color = if self.is_grabbed || self.is_drop_target {
            theme::accent()
        } else if self.is_focused {
            theme::accent_hover()
        } else if is_assigned {
            theme::border_color()
        } else {
            theme::border_subtle()
        };

        let mut card = div()
            .id(self.id.clone())
            .flex()
            .flex_col()
            .justify_between()
            .p_4()
            .rounded_xl()
            .bg(if self.is_drop_target {
                theme::accent_subtle()
            } else {
                theme::bg_panel()
            })
            .border_2()
            .border_color(border_color)
            .min_h(px(160.0));

        if self.is_focused && !self.is_grabbed {
            card = card.shadow_lg();
        }

        if let Some(h) = self.on_click {
            card = card
                .cursor_pointer()
                .on_click(move |ev, win, cx| h(ev, win, cx));
        }

        if self.is_grabbed {
            card = card.child(
                div()
                    .py_1()
                    .px_2()
                    .rounded_md()
                    .bg(theme::accent())
                    .text_color(theme::text_on_accent())
                    .text_xs()
                    .font_weight(FontWeight::BOLD)
                    .text_center()
                    .child("GRABBED (Tap target slot to swap)"),
            );
        }

        // 1. Header: Player badge + Status Badge
        let header = div()
            .flex()
            .items_center()
            .justify_between()
            .child(
                CouchTag::player(player_idx, format!("PLAYER {}", player_idx + 1))
                    .into_any_element(),
            )
            .children(self.status_badge);

        // 2. Center: Device representation
        let device_view = if let Some(ref lbl) = self.device_label {
            let icon_name = if self.is_kbm || self.is_merged_kbm {
                IconName::Keyboard
            } else if self.is_raw_device {
                IconName::Mouse
            } else {
                IconName::Gamepad2
            };

            let mut center = div()
                .flex()
                .flex_col()
                .items_center()
                .gap_2()
                .py_2()
                .child(Icon::new(icon_name).size_8().text_color(theme::accent()))
                .child(
                    div()
                        .text_sm()
                        .font_weight(FontWeight::BOLD)
                        .text_color(theme::text_primary())
                        .text_center()
                        .child(lbl.clone()),
                );

            if let Some(ref guid) = self.device_guid {
                center = center.child(
                    div()
                        .text_xs()
                        .font_weight(FontWeight::MEDIUM)
                        .text_color(theme::text_subtle())
                        .child(format!("GUID: {}", guid)),
                );
            }
            center
        } else {
            div()
                .flex()
                .flex_col()
                .items_center()
                .gap_2()
                .py_4()
                .child(
                    Icon::new(IconName::Plus)
                        .size_6()
                        .text_color(theme::text_muted()),
                )
                .child(
                    div()
                        .text_xs()
                        .text_color(theme::text_muted())
                        .text_center()
                        .child("Drop or Press (A) to assign device"),
                )
        };

        // 3. Footer: Action buttons (Remove, Split K&M, Ignore) or raw indicator
        let has_bottom_actions = self.on_remove.is_some()
            || self.is_merged_kbm
            || self.on_ignore.is_some()
            || self.is_raw_device;
        let footer: Option<AnyElement> = if has_bottom_actions {
            let mut bar = div()
                .flex()
                .items_center()
                .justify_between()
                .gap_2()
                .w_full()
                .pt_3()
                .border_t_1()
                .border_color(theme::border_subtle());

            if let Some(h) = self.on_remove {
                bar = bar.child(
                    div()
                        .id((self.id.clone(), "btn-remove"))
                        .cursor_pointer()
                        .flex()
                        .items_center()
                        .justify_center()
                        .gap_1p5()
                        .px_3()
                        .py_1p5()
                        .rounded_md()
                        .bg(theme::bg_subtle())
                        .hover(|s| s.bg(theme::bg_hover()))
                        .border_1()
                        .border_color(theme::border_subtle())
                        .text_xs()
                        .font_weight(FontWeight::MEDIUM)
                        .text_color(theme::text_muted())
                        .on_click(move |ev, win, cx| h(ev, win, cx))
                        .child(Icon::new(IconName::X).size_3())
                        .child("Remove (X)"),
                );
            }

            if self.is_merged_kbm {
                let mut split_btn = div()
                    .id((self.id.clone(), "btn-split-kbm"))
                    .cursor_pointer()
                    .flex()
                    .items_center()
                    .justify_center()
                    .gap_1p5()
                    .px_2p5()
                    .py_1p5()
                    .rounded_md()
                    .bg(theme::accent_subtle())
                    .border_1()
                    .border_color(theme::accent())
                    .text_xs()
                    .font_weight(FontWeight::BOLD)
                    .text_color(theme::accent())
                    .hover(|s| s.bg(theme::accent()).text_color(theme::text_on_accent()))
                    .child(Icon::new(IconName::Split).size_3p5())
                    .child("Split K&M");

                if let Some(h) = self.on_split {
                    split_btn = split_btn.on_click(move |ev, win, cx| h(ev, win, cx));
                }
                bar = bar.child(split_btn);
            } else if self.is_raw_device {
                bar = bar.child(
                    div()
                        .text_xs()
                        .text_color(theme::text_muted())
                        .text_center()
                        .child("ProtoInput Raw Device"),
                );
            }

            if let Some(h) = self.on_ignore {
                bar = bar.child(
                    div()
                        .id((self.id.clone(), "btn-ignore"))
                        .cursor_pointer()
                        .flex()
                        .items_center()
                        .justify_center()
                        .gap_1p5()
                        .px_3()
                        .py_1p5()
                        .rounded_md()
                        .bg(theme::bg_subtle())
                        .hover(|s| s.bg(theme::danger_subtle()))
                        .border_1()
                        .border_color(theme::border_subtle())
                        .text_xs()
                        .font_weight(FontWeight::MEDIUM)
                        .text_color(theme::danger())
                        .on_click(move |ev, win, cx| h(ev, win, cx))
                        .child(Icon::new(IconName::TriangleAlert).size_3())
                        .child("Ignore (Y)"),
                );
            }

            Some(bar.into_any_element())
        } else {
            None
        };

        card = card.child(header).child(device_view);
        if let Some(f) = footer {
            card = card.child(f);
        }

        card.into_any_element()
    }
}

impl IntoElement for CouchSlotCard {
    type Element = AnyElement;

    fn into_element(self) -> Self::Element {
        self.into_any_element()
    }
}
