use gpui::{
    AnyElement, FontWeight, IntoElement, ParentElement, Pixels, SharedString, Styled,
    StyledImage, div, img, px,
};
use gpui_component::Icon;
use gpui_kit_assets::IconName;

use crate::theme;

/// Standardized edge-to-edge game cover component.
/// Eliminates letterbox/pillarbox gaps permanently by setting explicit aspect ratio
/// and using `ObjectFit::Cover`.
pub struct GameCover {
    cover_url: Option<String>,
    fallback_title: SharedString,
    width: Pixels,
    height: Pixels,
    aspect_ratio: f32,
    is_installed: bool,
    is_selected: bool,
    badge: Option<AnyElement>,
}

impl GameCover {
    /// Constructs a standard 2:3 aspect ratio cover (e.g. 240x360 or 200x300).
    pub fn new(fallback_title: impl Into<SharedString>, width: Pixels, height: Pixels) -> Self {
        Self {
            cover_url: None,
            fallback_title: fallback_title.into(),
            width,
            height,
            aspect_ratio: 2.0 / 3.0,
            is_installed: true,
            is_selected: false,
            badge: None,
        }
    }

    pub fn cover_url(mut self, url: Option<String>) -> Self {
        self.cover_url = url;
        self
    }

    pub fn aspect_ratio(mut self, ratio: f32) -> Self {
        self.aspect_ratio = ratio;
        self
    }

    pub fn installed(mut self, installed: bool) -> Self {
        self.is_installed = installed;
        self
    }

    pub fn selected(mut self, selected: bool) -> Self {
        self.is_selected = selected;
        self
    }

    pub fn badge(mut self, badge: impl IntoElement) -> Self {
        self.badge = Some(badge.into_any_element());
        self
    }

    pub fn into_any_element(self) -> AnyElement {
        let is_compact = self.width <= px(100.0);
        let border_color = if self.is_selected {
            theme::accent()
        } else {
            theme::border_color()
        };

        let cover_content: AnyElement = if let Some(ref url) = self.cover_url {
            let mut im = img(crate::make_image_source(url))
                .absolute()
                .inset_0()
                .size_full()
                .aspect_ratio(self.aspect_ratio)
                .object_fit(gpui::ObjectFit::Cover);
            if is_compact {
                im = im.rounded_md();
            } else {
                im = im.rounded_xl();
            }
            im.into_any_element()
        } else {
            let mut ph = div()
                .flex()
                .flex_col()
                .items_center()
                .justify_center()
                .size_full()
                .bg(theme::bg_subtle());

            if is_compact {
                ph = ph
                    .gap_1()
                    .p_1()
                    .child(Icon::new(IconName::Gamepad2).size_4().text_color(theme::accent()))
                    .child(
                        div()
                            .text_xs()
                            .font_weight(FontWeight::BOLD)
                            .text_color(theme::accent())
                            .child(self.fallback_title.chars().take(4).collect::<String>()),
                    );
            } else {
                ph = ph
                    .gap_3()
                    .p_4()
                    .child(Icon::new(IconName::Gamepad2).size_10().text_color(theme::text_muted()))
                    .child(
                        div()
                            .text_sm()
                            .font_weight(FontWeight::BOLD)
                            .text_center()
                            .text_color(theme::text_primary())
                            .child(self.fallback_title),
                    );
            }
            ph.into_any_element()
        };

        let mut card = div()
            .relative()
            .w(self.width)
            .h(self.height)
            .overflow_hidden()
            .bg(theme::bg_panel())
            .child(cover_content);

        if is_compact {
            card = card.rounded_md().border_1().border_color(border_color);
        } else {
            card = card.rounded_xl().border_2().border_color(border_color);
        }

        if !self.is_installed {
            card = card.opacity(if self.is_selected { 0.85 } else { 0.50 });
        }

        if let Some(b) = self.badge {
            card = card.child(b);
        }

        card.into_any_element()
    }
}

impl IntoElement for GameCover {
    type Element = AnyElement;

    fn into_element(self) -> Self::Element {
        self.into_any_element()
    }
}
