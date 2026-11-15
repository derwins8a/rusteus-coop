use gpui::prelude::FluentBuilder;
use gpui::{
    div, px, rgb, AnyElement, ClickEvent, ElementId, FontWeight, InteractiveElement,
    IntoElement, MouseButton, ParentElement, Pixels, SharedString,
    StatefulInteractiveElement, Styled,
};
use gpui_component::scroll::ScrollableElement;
use gpui_component::Icon;
use gpui_kit_assets::IconName;

use crate::components::badge::TagTone;
use crate::theme;

/// Standard modal dialog widths.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum ModalWidth {
    /// 480px (Confirmations, alerts)
    Sm,
    /// 640px (Executable locator, standard forms)
    Md,
    /// 820px (Splitscreen handler options — guarantees zero label truncation)
    #[default]
    Lg,
}

impl ModalWidth {
    pub fn to_pixels(&self) -> (Pixels, Pixels) {
        match self {
            Self::Sm => (px(480.0), px(520.0)),
            Self::Md => (px(640.0), px(700.0)),
            Self::Lg => (px(820.0), px(880.0)),
        }
    }
}

/// A universal modal dialog component that encapsulates backdrop scrim,
/// click pass-through prevention (`stop_propagation`), standard headers, and footers.
pub struct CouchModal {
    id: ElementId,
    title: SharedString,
    subtitle: Option<SharedString>,
    icon: Option<IconName>,
    tone: Option<TagTone>,
    width: ModalWidth,
    max_height: Option<Pixels>,
    body: Option<AnyElement>,
    header_actions: Vec<AnyElement>,
    footer_actions: Vec<AnyElement>,
    on_close: Option<std::sync::Arc<dyn Fn(&ClickEvent, &mut gpui::Window, &mut gpui::App) + 'static>>,
}

impl CouchModal {
    pub fn new(id: impl Into<ElementId>, title: impl Into<SharedString>) -> Self {
        Self {
            id: id.into(),
            title: title.into(),
            subtitle: None,
            icon: None,
            tone: None,
            width: ModalWidth::Lg,
            max_height: None,
            body: None,
            header_actions: Vec::new(),
            footer_actions: Vec::new(),
            on_close: None,
        }
    }

    pub fn subtitle(mut self, subtitle: impl Into<SharedString>) -> Self {
        self.subtitle = Some(subtitle.into());
        self
    }

    pub fn icon(mut self, icon: IconName) -> Self {
        self.icon = Some(icon);
        self
    }

    pub fn tone(mut self, tone: TagTone) -> Self {
        self.tone = Some(tone);
        self
    }

    pub fn width(mut self, width: ModalWidth) -> Self {
        self.width = width;
        self
    }

    pub fn max_h(mut self, max_height: impl Into<Pixels>) -> Self {
        self.max_height = Some(max_height.into());
        self
    }

    pub fn body(mut self, body: impl IntoElement) -> Self {
        self.body = Some(body.into_any_element());
        self
    }

    pub fn header_action(mut self, action: impl IntoElement) -> Self {
        self.header_actions.push(action.into_any_element());
        self
    }

    pub fn action(mut self, action: impl IntoElement) -> Self {
        self.footer_actions.push(action.into_any_element());
        self
    }

    pub fn on_close<F>(mut self, handler: F) -> Self
    where
        F: Fn(&ClickEvent, &mut gpui::Window, &mut gpui::App) + 'static,
    {
        self.on_close = Some(std::sync::Arc::new(handler));
        self
    }

    pub fn into_any_element(self) -> AnyElement {
        let (min_w, max_w) = self.width.to_pixels();
        let max_h = self.max_height.unwrap_or(px(720.0));

        let (border_color, icon_color) = match self.tone {
            Some(TagTone::Warning) => (theme::warning(), theme::warning()),
            Some(TagTone::Danger) => (theme::danger(), theme::danger()),
            Some(TagTone::Accent) => (theme::accent(), theme::accent()),
            Some(TagTone::Success) => (theme::success(), theme::success()),
            _ => (theme::border_color(), theme::accent()),
        };

        let has_footer = !self.footer_actions.is_empty();

        // 1. Backdrop scrim — fixed over entire viewport, stopping click pass-through
        div()
            .absolute()
            .inset_0()
            .id(self.id.clone())
            .occlude()
            .flex()
            .items_center()
            .justify_center()
            .bg(rgb(0x000000).opacity(0.80))
            .on_mouse_down(MouseButton::Left, |_, _, cx| {
                cx.stop_propagation();
            })
            // 2. Dialog card window container
            .child(
                div()
                    .relative()
                    .w(min_w)
                    .max_w(max_w)
                    .max_h(max_h)
                    .flex()
                    .flex_col()
                    .rounded_xl()
                    .bg(theme::bg_panel())
                    .border_2()
                    .border_color(border_color)
                    .shadow_xl()
                    .overflow_hidden()
                    .on_mouse_down(MouseButton::Left, |_, _, cx| {
                        // Crucial safety invariant: click inside dialog must NOT pass through to elements behind
                        cx.stop_propagation();
                    })
                    // 3. Header bar
                    .child({
                        let mut header_bar = div()
                            .flex()
                            .items_center()
                            .justify_between()
                            .px_6()
                            .py_4()
                            .border_b_1()
                            .border_color(theme::border_color())
                            .bg(theme::bg_subtle())
                            .child(
                                div()
                                    .flex()
                                    .items_center()
                                    .gap_3()
                                    .min_w_0()
                                    .children(self.icon.map(|icn| {
                                        Icon::new(icn).size_5().text_color(icon_color)
                                    }))
                                    .child(
                                        div()
                                            .flex()
                                            .flex_col()
                                            .min_w_0()
                                            .child(
                                                div()
                                                    .text_base()
                                                    .font_weight(FontWeight::BOLD)
                                                    .text_color(theme::text_primary())
                                                    .child(self.title),
                                            )
                                            .children(self.subtitle.map(|sub| {
                                                div()
                                                    .text_xs()
                                                    .text_color(theme::text_muted())
                                                    .child(sub)
                                            })),
                                    ),
                            );

                        if !self.header_actions.is_empty() || self.on_close.is_some() {
                            let mut right_actions = div()
                                .flex()
                                .items_center()
                                .gap_2()
                                .flex_shrink_0()
                                .children(self.header_actions);

                            if let Some(cb) = self.on_close.clone() {
                                right_actions = right_actions.child(
                                    div()
                                        .id((self.id.clone(), "modal-close-btn"))
                                        .cursor_pointer()
                                        .flex()
                                        .items_center()
                                        .justify_center()
                                        .size(px(28.0))
                                        .rounded_md()
                                        .bg(theme::bg_subtle())
                                        .border_1()
                                        .border_color(theme::border_color())
                                        .text_color(theme::text_primary())
                                        .hover(|s| {
                                            s.bg(theme::danger_subtle())
                                                .border_color(theme::danger().opacity(0.55))
                                                .text_color(theme::danger())
                                        })
                                        .on_click(move |ev, win, cx| cb(ev, win, cx))
                                        .child(Icon::new(IconName::X).size_3p5()),
                                );
                            }

                            header_bar = header_bar.child(right_actions);
                        }

                        header_bar
                    })
                    // 4. Body content
                    .child(
                        div()
                            .flex_1()
                            .min_h_0()
                            .overflow_y_scrollbar()
                            .p_6()
                            .children(self.body),
                    )
                    // 5. Footer actions (if any)
                    .when(has_footer, |container| {
                        container.child(
                            div()
                                .flex()
                                .items_center()
                                .justify_end()
                                .gap_3()
                                .px_6()
                                .py_4()
                                .border_t_1()
                                .border_color(theme::border_color())
                                .bg(theme::bg_subtle())
                                .children(self.footer_actions),
                        )
                    }),
            )
            .into_any_element()
    }
}

impl IntoElement for CouchModal {
    type Element = AnyElement;

    fn into_element(self) -> Self::Element {
        self.into_any_element()
    }
}
