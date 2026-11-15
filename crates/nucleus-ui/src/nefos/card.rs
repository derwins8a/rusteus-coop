//! # NefosCard Component
//!
//! Apple-style surface card component engineered for the Nefos UI design system.
//! Provides unified surface elevation variants, structured headers/footers,
//! seamless child embedding, and interactive hover elevation.

use gpui::{
    AnyElement, ElementId, FontWeight, InteractiveElement, IntoElement, ParentElement, Pixels,
    SharedString, Styled, div,
};
use smallvec::SmallVec;

use crate::theme;

/// Surface elevation and visual variants for Nefos Apple-style cards.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum NefosCardVariant {
    /// Default surface panel card (Layer 1).
    #[default]
    Surface,
    /// Elevated panel card with extra depth and shadow (Layer 2).
    Elevated,
    /// Transparent card with subtle border outline.
    Outline,
    /// Translucent frosted glass panel with subtle border.
    Glass,
}

/// An Apple-style surface card component engineered for the Nefos UI design system.
///
/// Features:
/// - Semantic surface variants (`Surface`, `Elevated`, `Outline`, `Glass`).
/// - Optional header section with `title`, `description`, and `header_action`.
/// - Optional `footer` section.
/// - Inline children via `ParentElement` (`.child(...)`, `.children(...)`).
/// - Optional interactive elevation and glow on hover (`hover_elevate`).
/// - Standard Apple rounded corners (`.rounded_xl()`).
/// - Zero-boilerplate flattening to `AnyElement` via `IntoElement`.
pub struct NefosCard {
    id: ElementId,
    variant: NefosCardVariant,
    title: Option<AnyElement>,
    description: Option<AnyElement>,
    header_action: Option<AnyElement>,
    footer: Option<AnyElement>,
    children: SmallVec<[AnyElement; 2]>,
    hover_elevate: bool,
    paddings: bool,
    selected: bool,
    height: Option<Pixels>,
}

impl NefosCard {
    /// Creates a new `NefosCard` with default surface styling and padding enabled.
    pub fn new(id: impl Into<ElementId>) -> Self {
        Self {
            id: id.into(),
            variant: NefosCardVariant::Surface,
            title: None,
            description: None,
            header_action: None,
            footer: None,
            children: SmallVec::new(),
            hover_elevate: false,
            paddings: true,
            selected: false,
            height: None,
        }
    }

    /// Convenience constructor for a default surface panel card.
    pub fn surface(id: impl Into<ElementId>) -> Self {
        Self::new(id).variant(NefosCardVariant::Surface)
    }

    /// Convenience constructor for an elevated panel card.
    pub fn elevated(id: impl Into<ElementId>) -> Self {
        Self::new(id).variant(NefosCardVariant::Elevated)
    }

    /// Convenience constructor for an outline card.
    pub fn outline(id: impl Into<ElementId>) -> Self {
        Self::new(id).variant(NefosCardVariant::Outline)
    }

    /// Convenience constructor for a translucent glass card.
    pub fn glass(id: impl Into<ElementId>) -> Self {
        Self::new(id).variant(NefosCardVariant::Glass)
    }

    /// Sets the surface style variant.
    pub fn variant(mut self, variant: NefosCardVariant) -> Self {
        self.variant = variant;
        self
    }

    /// Sets a custom title element in the card header.
    pub fn title(mut self, title: impl IntoElement) -> Self {
        self.title = Some(title.into_any_element());
        self
    }

    /// Sets a text title with standardized typography in the card header.
    pub fn title_text(mut self, text: impl Into<SharedString>) -> Self {
        let text = text.into();
        self.title = Some(
            div()
                .text_sm()
                .font_weight(FontWeight::SEMIBOLD)
                .text_color(theme::text_primary())
                .child(text)
                .into_any_element(),
        );
        self
    }

    /// Sets a custom description element in the card header.
    pub fn description(mut self, description: impl IntoElement) -> Self {
        self.description = Some(description.into_any_element());
        self
    }

    /// Sets a text description with standardized muted typography in the card header.
    pub fn description_text(mut self, text: impl Into<SharedString>) -> Self {
        let text = text.into();
        self.description = Some(
            div()
                .text_xs()
                .text_color(theme::text_muted())
                .child(text)
                .into_any_element(),
        );
        self
    }

    /// Sets an action element aligned to the top-right of the card header.
    pub fn header_action(mut self, action: impl IntoElement) -> Self {
        self.header_action = Some(action.into_any_element());
        self
    }

    /// Sets a footer element at the bottom of the card.
    pub fn footer(mut self, footer: impl IntoElement) -> Self {
        self.footer = Some(footer.into_any_element());
        self
    }

    /// Enables or disables interactive border highlighting and surface glow on hover.
    pub fn hover_elevate(mut self, hover_elevate: bool) -> Self {
        self.hover_elevate = hover_elevate;
        self
    }

    /// Sets whether standard internal padding (`p_4`) is applied. Defaults to true.
    pub fn paddings(mut self, paddings: bool) -> Self {
        self.paddings = paddings;
        self
    }

    /// Sets whether this card is currently selected or highlighted.
    pub fn selected(mut self, selected: bool) -> Self {
        self.selected = selected;
        self
    }

    /// Sets an explicit height for the card.
    pub fn h(mut self, height: impl Into<Pixels>) -> Self {
        self.height = Some(height.into());
        self
    }

    /// Converts the card builder into a type-erased `AnyElement`.
    pub fn into_any_element(self) -> AnyElement {
        let has_header =
            self.title.is_some() || self.description.is_some() || self.header_action.is_some();
        let has_footer = self.footer.is_some();

        let mut card = div()
            .id(self.id)
            .w_full()
            .flex()
            .flex_col()
            .flex_shrink_0()
            .rounded_xl()
            .overflow_hidden();

        if let Some(h) = self.height {
            card = card.h(h);
        }

        // Apply variant background and border
        match self.variant {
            NefosCardVariant::Surface => {
                let (bg, border) = if self.selected {
                    (theme::bg_hover(), theme::accent())
                } else {
                    (theme::bg_panel(), theme::border_color())
                };
                card = card.bg(bg).border_1().border_color(border);
            }
            NefosCardVariant::Elevated => {
                let (bg, border) = if self.selected {
                    (theme::bg_hover(), theme::accent())
                } else {
                    (theme::bg_elevated(), theme::border_color())
                };
                card = card.bg(bg).border_1().border_color(border).shadow_md();
            }
            NefosCardVariant::Outline => {
                let border = if self.selected {
                    theme::accent()
                } else {
                    theme::border_color()
                };
                card = card.border_1().border_color(border);
            }
            NefosCardVariant::Glass => {
                let border = if self.selected {
                    theme::accent()
                } else {
                    theme::border_subtle()
                };
                card = card
                    .bg(theme::bg_panel().opacity(0.85))
                    .border_1()
                    .border_color(border);
            }
        }

        if self.hover_elevate {
            card = card.hover(|s| {
                s.border_color(theme::border_focus())
                    .bg(theme::bg_panel_hover())
            });
        }

        let has_children = !self.children.is_empty();

        if self.paddings {
            card = card.p_4();
        }

        // Header section
        if has_header {
            let mut header_div = div()
                .flex()
                .flex_shrink_0()
                .items_start()
                .justify_between()
                .gap_3();

            if has_children || has_footer {
                header_div = header_div.pb_3();
            }

            let mut title_col = div().flex().flex_col().gap_0p5().flex_1();
            if let Some(t) = self.title {
                title_col = title_col.child(t);
            }
            if let Some(d) = self.description {
                title_col = title_col.child(d);
            }
            header_div = header_div.child(title_col);

            if let Some(action) = self.header_action {
                header_div =
                    header_div.child(div().flex().items_center().flex_shrink_0().child(action));
            }

            card = card.child(header_div);
        }

        // Main body children
        if has_children {
            let mut body_div = div().flex().flex_col().w_full();
            if self.height.is_some() {
                body_div = body_div.flex_1();
            }
            if self.paddings {
                body_div = body_div.gap_2();
            }
            for child in self.children {
                body_div = body_div.child(child);
            }
            card = card.child(body_div);
        }

        // Footer section
        if let Some(f) = self.footer {
            let mut footer_div = div()
                .flex()
                .flex_shrink_0()
                .items_center()
                .w_full()
                .mt_auto();

            if self.paddings && (has_header || has_children) {
                footer_div = footer_div
                    .pt_3()
                    .border_t_1()
                    .border_color(theme::border_subtle());
            }

            footer_div = footer_div.child(f);
            card = card.child(footer_div);
        }

        card.into_any_element()
    }
}

impl ParentElement for NefosCard {
    fn extend(&mut self, elements: impl IntoIterator<Item = AnyElement>) {
        self.children.extend(elements);
    }
}

impl IntoElement for NefosCard {
    type Element = AnyElement;

    fn into_element(self) -> Self::Element {
        self.into_any_element()
    }
}
