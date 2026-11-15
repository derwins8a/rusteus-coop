use gpui::{
    AnyElement, App, ClickEvent, ElementId, FontWeight, Hsla, InteractiveElement, IntoElement,
    ParentElement, SharedString, StatefulInteractiveElement, Styled, Window, div, rgb,
};
use gpui_component::Icon;
use gpui_kit_assets::IconName;

use crate::theme;

/// Semantic tone for status tags and player badges.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum TagTone {
    #[default]
    Neutral,
    Accent,
    Success,
    Warning,
    Danger,
    Player(usize),
}

/// A standardized chip/badge component for displaying connection status, tags,
/// and colored player indicators (P1, P2, P3, P4).
pub struct CouchTag {
    id: Option<ElementId>,
    label: SharedString,
    tone: TagTone,
    icon: Option<IconName>,
    has_dot: bool,
    tooltip: Option<SharedString>,
    on_click: Option<Box<dyn Fn(&ClickEvent, &mut Window, &mut App) + 'static>>,
}

impl CouchTag {
    pub fn new(label: impl Into<SharedString>) -> Self {
        Self {
            id: None,
            label: label.into(),
            tone: TagTone::Neutral,
            icon: None,
            has_dot: false,
            tooltip: None,
            on_click: None,
        }
    }

    pub fn player(player_idx: usize, label: impl Into<SharedString>) -> Self {
        Self {
            id: None,
            label: label.into(),
            tone: TagTone::Player(player_idx),
            icon: Some(IconName::Gamepad2),
            has_dot: false,
            tooltip: None,
            on_click: None,
        }
    }

    pub fn id(mut self, id: impl Into<ElementId>) -> Self {
        self.id = Some(id.into());
        self
    }

    pub fn tone(mut self, tone: TagTone) -> Self {
        self.tone = tone;
        self
    }

    pub fn icon(mut self, icon: IconName) -> Self {
        self.icon = Some(icon);
        self
    }

    pub fn with_dot(mut self) -> Self {
        self.has_dot = true;
        self
    }

    pub fn tooltip(mut self, tooltip: impl Into<SharedString>) -> Self {
        self.tooltip = Some(tooltip.into());
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
        let (bg, text_color, border_color, dot_color): (Hsla, Hsla, Hsla, Hsla) = match self.tone {
            TagTone::Neutral => (
                theme::bg_subtle(),
                theme::text_muted(),
                theme::border_subtle(),
                theme::text_muted(),
            ),
            TagTone::Accent => (
                theme::accent_subtle(),
                theme::accent(),
                theme::accent(),
                theme::accent(),
            ),
            TagTone::Success => (
                rgb(0x064e3b).into(),
                rgb(0x34d399).into(),
                rgb(0x059669).into(),
                rgb(0x10b981).into(),
            ),
            TagTone::Warning => (
                rgb(0x451a03).into(),
                rgb(0xfbbf24).into(),
                rgb(0xd97706).into(),
                rgb(0xf59e0b).into(),
            ),
            TagTone::Danger => (
                rgb(0x4c0519).into(),
                rgb(0xf43f5e).into(),
                rgb(0xe11d48).into(),
                rgb(0xef4444).into(),
            ),
            TagTone::Player(idx) => match idx % 4 {
                0 => (
                    rgb(0x064e3b).into(),
                    rgb(0x22c55e).into(), // P1 Emerald
                    rgb(0x22c55e).into(),
                    rgb(0x22c55e).into(),
                ),
                1 => (
                    rgb(0x1e3a8a).into(),
                    rgb(0x3b82f6).into(), // P2 Blue
                    rgb(0x3b82f6).into(),
                    rgb(0x3b82f6).into(),
                ),
                2 => (
                    rgb(0x451a03).into(),
                    rgb(0xf59e0b).into(), // P3 Amber
                    rgb(0xf59e0b).into(),
                    rgb(0xf59e0b).into(),
                ),
                _ => (
                    rgb(0x4c0519).into(),
                    rgb(0xf43f5e).into(), // P4 Rose
                    rgb(0xf43f5e).into(),
                    rgb(0xf43f5e).into(),
                ),
            },
        };

        let mut pill = div()
            .flex()
            .items_center()
            .gap_1p5()
            .px_2p5()
            .py_0p5()
            .rounded_md()
            .bg(bg)
            .border_1()
            .border_color(border_color)
            .text_xs()
            .font_weight(FontWeight::BOLD)
            .text_color(text_color);

        if self.has_dot {
            pill = pill.child(div().w_1p5().h_1p5().rounded_full().bg(dot_color));
        }

        if let Some(icn) = self.icon {
            pill = pill.child(Icon::new(icn).size_3().text_color(text_color));
        }

        if self.on_click.is_some() {
            pill = pill.cursor_pointer().hover(|s| s.opacity(0.85));
        }

        let pill_base = pill.child(self.label.clone());

        if let Some(id) = self.id {
            let mut stateful = pill_base.id(id);
            if let Some(tooltip) = self.tooltip {
                stateful = stateful.tooltip(crate::make_tooltip(tooltip));
            }
            if let Some(h) = self.on_click {
                stateful = stateful.on_click(move |ev, win, cx| h(ev, win, cx));
            }
            stateful.into_any_element()
        } else if let Some(h) = self.on_click {
            pill_base
                .id(ElementId::Name(format!("tag-{}", self.label).into()))
                .on_click(move |ev, win, cx| h(ev, win, cx))
                .into_any_element()
        } else {
            pill_base.into_any_element()
        }
    }
}

impl IntoElement for CouchTag {
    type Element = AnyElement;

    fn into_element(self) -> Self::Element {
        self.into_any_element()
    }
}
