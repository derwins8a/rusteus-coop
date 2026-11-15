use gpui::{
    AnyElement, App, ClickEvent, ElementId, FontWeight, InteractiveElement, IntoElement,
    ParentElement, Pixels, SharedString, StatefulInteractiveElement, Styled, Window, div, px, rgb,
};
use gpui_component::Icon;
use gpui_kit_assets::IconName;

use crate::theme;

/// Button sizing presets optimized for desktop dialogs and 10-foot Couch mode.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum CouchButtonSize {
    /// 28px height (compact toolbars and badges)
    Sm,
    /// 36px height (desktop modal controls)
    #[default]
    Md,
    /// 44px height (10-foot Couch mode for gamepad / touch ergonomics)
    Lg,
}

impl CouchButtonSize {
    pub fn height_px(&self) -> Pixels {
        match self {
            Self::Sm => px(28.0),
            Self::Md => px(36.0),
            Self::Lg => px(44.0),
        }
    }

    pub fn pad_px(&self) -> Pixels {
        match self {
            Self::Sm => px(10.0),
            Self::Md => px(16.0),
            Self::Lg => px(20.0),
        }
    }

    pub fn icon_size_px(&self) -> Pixels {
        match self {
            Self::Sm => px(14.0),
            Self::Md => px(16.0),
            Self::Lg => px(18.0),
        }
    }
}

/// Visual style variant for CouchButton.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum CouchButtonVariant {
    #[default]
    Secondary,
    Primary,
    Ghost,
    Danger,
    DangerOutline,
    Success,
    SuccessOutline,
}

impl From<gpui_component::button::ButtonVariant> for CouchButtonVariant {
    fn from(v: gpui_component::button::ButtonVariant) -> Self {
        match v {
            gpui_component::button::ButtonVariant::Primary => Self::Primary,
            gpui_component::button::ButtonVariant::Secondary => Self::Secondary,
            gpui_component::button::ButtonVariant::Ghost => Self::Ghost,
            gpui_component::button::ButtonVariant::Danger => Self::Danger,
            gpui_component::button::ButtonVariant::Success => Self::Success,
            _ => Self::Secondary,
        }
    }
}

/// A standardized button component engineered specifically for Nucleus Co-op.
/// Guarantees:
/// 1. Dark-gaming theme styling (theme::accent() for primary, theme::bg_subtle() for secondary).
/// 2. Active pointer cursor (cursor_pointer()) on mouse hover.
/// 3. Gamepad focus outline when focused.
/// 4. Auto-flattened to AnyElement via IntoElement to prevent compiler type explosions.
pub struct CouchButton {
    id: ElementId,
    label: Option<SharedString>,
    icon: Option<IconName>,
    variant: CouchButtonVariant,
    size: CouchButtonSize,
    is_disabled: bool,
    is_focused: bool,
    is_selected: bool,
    tooltip: Option<SharedString>,
    on_click: Option<Box<dyn Fn(&ClickEvent, &mut Window, &mut App) + 'static>>,
}

impl CouchButton {
    pub fn new(id: impl Into<ElementId>) -> Self {
        Self {
            id: id.into(),
            label: None,
            icon: None,
            variant: CouchButtonVariant::Secondary,
            size: CouchButtonSize::Md,
            is_disabled: false,
            is_focused: false,
            is_selected: false,
            tooltip: None,
            on_click: None,
        }
    }

    pub fn primary(id: impl Into<ElementId>, label: impl Into<SharedString>) -> Self {
        Self::new(id)
            .label(label)
            .variant(CouchButtonVariant::Primary)
    }

    pub fn secondary(id: impl Into<ElementId>, label: impl Into<SharedString>) -> Self {
        Self::new(id)
            .label(label)
            .variant(CouchButtonVariant::Secondary)
    }

    pub fn ghost(id: impl Into<ElementId>, label: impl Into<SharedString>) -> Self {
        Self::new(id)
            .label(label)
            .variant(CouchButtonVariant::Ghost)
    }

    pub fn danger(id: impl Into<ElementId>, label: impl Into<SharedString>) -> Self {
        Self::new(id)
            .label(label)
            .variant(CouchButtonVariant::Danger)
    }

    pub fn danger_outline(id: impl Into<ElementId>, label: impl Into<SharedString>) -> Self {
        Self::new(id)
            .label(label)
            .variant(CouchButtonVariant::DangerOutline)
    }

    pub fn success(id: impl Into<ElementId>, label: impl Into<SharedString>) -> Self {
        Self::new(id)
            .label(label)
            .variant(CouchButtonVariant::Success)
    }

    pub fn success_outline(id: impl Into<ElementId>, label: impl Into<SharedString>) -> Self {
        Self::new(id)
            .label(label)
            .variant(CouchButtonVariant::SuccessOutline)
    }

    pub fn label(mut self, label: impl Into<SharedString>) -> Self {
        self.label = Some(label.into());
        self
    }

    pub fn icon(mut self, icon: IconName) -> Self {
        self.icon = Some(icon);
        self
    }

    pub fn variant(mut self, variant: impl Into<CouchButtonVariant>) -> Self {
        self.variant = variant.into();
        self
    }

    pub fn size(mut self, size: CouchButtonSize) -> Self {
        self.size = size;
        self
    }

    pub fn disabled(mut self, disabled: bool) -> Self {
        self.is_disabled = disabled;
        self
    }

    pub fn focused(mut self, focused: bool) -> Self {
        self.is_focused = focused;
        self
    }

    pub fn selected(mut self, selected: bool) -> Self {
        self.is_selected = selected;
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

    /// Flattens the button and returns an `AnyElement` to guarantee bounded generic types and fast builds.
    pub fn into_any_element(self) -> AnyElement {
        let (bg, bg_hover, text_color, border_color) = match self.variant {
            CouchButtonVariant::Primary => (
                theme::accent(),
                theme::accent_hover(),
                theme::text_on_accent(),
                None,
            ),
            CouchButtonVariant::Secondary => (
                theme::bg_subtle(),
                theme::bg_hover(),
                theme::text_primary(),
                Some(theme::border_color()),
            ),
            CouchButtonVariant::Ghost => (
                rgb(0x000000).opacity(0.0).into(),
                theme::bg_hover(),
                theme::text_primary(),
                None,
            ),
            CouchButtonVariant::Danger => (
                theme::danger(),
                rgb(0xe11d48).into(),
                theme::text_on_accent(),
                None,
            ),
            CouchButtonVariant::DangerOutline => (
                theme::danger_subtle(),
                theme::danger().opacity(0.28),
                theme::danger(),
                Some(theme::danger().opacity(0.55)),
            ),
            CouchButtonVariant::Success => (
                theme::success(),
                theme::accent_hover(),
                theme::text_on_accent(),
                None,
            ),
            CouchButtonVariant::SuccessOutline => (
                rgb(0x000000).opacity(0.0).into(),
                theme::success_subtle(),
                theme::success(),
                Some(theme::success()),
            ),
        };

        let height = self.size.height_px();
        let pad_x = self.size.pad_px();
        let icon_size = self.size.icon_size_px();
        let is_icon_only = self.label.as_ref().map_or(true, |l| l.trim().is_empty());

        let mut btn = div()
            .id(self.id.clone())
            .flex()
            .flex_shrink_0()
            .items_center()
            .justify_center()
            .gap_2()
            .h(height)
            .rounded_lg()
            .bg(bg)
            .text_color(text_color)
            .font_weight(FontWeight::BOLD);

        if is_icon_only {
            btn = btn.w(height);
        } else {
            btn = btn.px(pad_x);
        }

        match self.size {
            CouchButtonSize::Sm => {
                btn = btn.text_xs();
            }
            CouchButtonSize::Md => {
                btn = btn.text_sm();
            }
            CouchButtonSize::Lg => {
                btn = btn.text_base();
            }
        }

        if let Some(border) = border_color {
            btn = btn.border_1().border_color(border);
        }

        if self.is_disabled {
            btn = btn.opacity(0.40).cursor_not_allowed();
        } else {
            btn = btn.cursor_pointer().hover(move |s| s.bg(bg_hover));

            if let Some(handler) = self.on_click {
                btn = btn.on_click(move |ev, win, cx| {
                    handler(ev, win, cx);
                });
            }
        }

        if let Some(icn) = self.icon {
            btn = btn.child(Icon::new(icn).size(icon_size).text_color(text_color));
        }

        if let Some(lbl) = self.label {
            if !lbl.trim().is_empty() {
                btn = btn.child(lbl);
            }
        }

        if let Some(tt) = self.tooltip {
            btn = btn.tooltip(crate::make_tooltip(tt));
        }

        if self.is_focused || self.is_selected {
            div()
                .relative()
                .rounded_lg()
                .border_2()
                .border_color(if self.is_selected {
                    theme::accent()
                } else {
                    theme::accent_hover()
                })
                .child(btn)
                .into_any_element()
        } else {
            btn.into_any_element()
        }
    }
}

impl IntoElement for CouchButton {
    type Element = AnyElement;

    fn into_element(self) -> Self::Element {
        self.into_any_element()
    }
}
