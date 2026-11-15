//! # NefosSegmentedControl Component
//!
//! Apple macOS-style segmented pill tab switcher engineered for the Nefos UI design system.
//! Provides smooth segmented states, optional icons, and flexible selection callbacks.

use std::rc::Rc;

use gpui::{
    div, px, AnyElement, App, Context, ElementId, FontWeight, InteractiveElement,
    IntoElement, ParentElement, StatefulInteractiveElement, Styled, Window,
};
use gpui_component::Icon;
use gpui_kit_assets::IconName;

use crate::theme;
use crate::window::MainWindow;

/// Callback for segmented item selection when running inside `MainWindow`.
pub type SegmentSelectHandler = Rc<dyn Fn(&mut MainWindow, usize, &mut Window, &mut Context<MainWindow>)>;

/// Callback for segmented item selection with general `App` context.
pub type SegmentClickHandler = Rc<dyn Fn(usize, &mut Window, &mut App)>;

/// Apple macOS-style segmented pill tab switcher for Nefos UI.
///
/// Features:
/// - Distinct active/inactive pill styling with subtle shadows and borders.
/// - Optional icons per segment.
/// - Type-erased `AnyElement` boundary flattening.
/// - Dual callback dispatch supporting both `Context<MainWindow>` and `App`.
pub struct NefosSegmentedControl {
    id: ElementId,
    items: Vec<(String, Option<IconName>, bool)>,
    on_select: Option<SegmentSelectHandler>,
    on_click: Option<SegmentClickHandler>,
}

impl NefosSegmentedControl {
    /// Creates a new `NefosSegmentedControl` with an empty item list.
    pub fn new(id: impl Into<ElementId>) -> Self {
        Self {
            id: id.into(),
            items: Vec::new(),
            on_select: None,
            on_click: None,
        }
    }

    /// Adds a segment item with a text label, optional icon, and active status flag.
    pub fn item(mut self, label: impl Into<String>, icon: Option<IconName>, is_active: bool) -> Self {
        self.items.push((label.into(), icon, is_active));
        self
    }

    /// Replaces the list of segment items.
    pub fn items(mut self, items: Vec<(String, Option<IconName>, bool)>) -> Self {
        self.items = items;
        self
    }

    /// Sets the selection callback for use in `MainWindow` views.
    pub fn on_select<F>(mut self, handler: F) -> Self
    where
        F: Fn(&mut MainWindow, usize, &mut Window, &mut Context<MainWindow>) + 'static,
    {
        self.on_select = Some(Rc::new(handler));
        self
    }

    /// Sets the selection callback via an existing `Rc` reference.
    pub fn on_select_rc(mut self, handler: SegmentSelectHandler) -> Self {
        self.on_select = Some(handler);
        self
    }

    /// Sets the selection callback for general `App` contexts.
    pub fn on_click<F>(mut self, handler: F) -> Self
    where
        F: Fn(usize, &mut Window, &mut App) + 'static,
    {
        self.on_click = Some(Rc::new(handler));
        self
    }

    /// Sets the selection callback for general `App` contexts via an existing `Rc`.
    pub fn on_click_rc(mut self, handler: SegmentClickHandler) -> Self {
        self.on_click = Some(handler);
        self
    }

    /// Converts the segmented control into a type-erased `AnyElement`.
    pub fn into_any_element(self) -> AnyElement {
        let mut outer = div()
            .id(self.id.clone())
            .h(px(32.0))
            .p_0p5()
            .rounded_lg()
            .bg(theme::bg_main())
            .border_1()
            .border_color(theme::border_subtle())
            .flex()
            .items_center()
            .gap_1();

        for (idx, (label, icon, is_active)) in self.items.into_iter().enumerate() {
            let item_id: ElementId = (self.id.clone(), format!("seg-{}", idx)).into();

            let mut seg = div()
                .id(item_id)
                .flex()
                .items_center()
                .justify_center()
                .gap_1p5()
                .cursor_pointer()
                .text_xs();

            if is_active {
                seg = seg
                    .h_full()
                    .px_3()
                    .rounded_md()
                    .bg(theme::bg_panel())
                    .border_1()
                    .border_color(theme::border_color())
                    .text_color(theme::text_primary())
                    .font_weight(FontWeight::SEMIBOLD)
                    .shadow_sm();
            } else {
                seg = seg
                    .h_full()
                    .px_3()
                    .rounded_md()
                    .text_color(theme::text_muted())
                    .hover(|s| s.text_color(theme::text_primary()).bg(theme::bg_hover()));
            }

            if let Some(icn) = icon {
                let icon_color = if is_active {
                    theme::text_primary()
                } else {
                    theme::text_muted()
                };
                seg = seg.child(Icon::new(icn).size(px(14.0)).text_color(icon_color));
            }

            seg = seg.child(label);

            let on_select = self.on_select.clone();
            let on_click = self.on_click.clone();

            seg = seg.on_click(move |_event, window, cx| {
                if let Some(ref cb) = on_click {
                    cb(idx, window, cx);
                }
                if let Some(ref cb) = on_select {
                    if let Some(handle) = window.window_handle().downcast::<MainWindow>() {
                        let cb = cb.clone();
                        let _ = handle.update(cx, |root, window, cx| {
                            cb(root, idx, window, cx);
                        });
                    }
                }
            });

            outer = outer.child(seg);
        }

        outer.into_any_element()
    }
}

impl IntoElement for NefosSegmentedControl {
    type Element = AnyElement;

    fn into_element(self) -> Self::Element {
        self.into_any_element()
    }
}
