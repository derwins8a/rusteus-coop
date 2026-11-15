//! # CouchSidebarNav Meta-Component
//!
//! Left-hand navigation sidebar for Nucleus Co-op desktop and couch layouts.
//! Supports collapsible full mode (200px) and compact icon-only mode (60px).

use std::rc::Rc;

use gpui::prelude::FluentBuilder;
use gpui::{
    AnyElement, App, ClickEvent, FontWeight, InteractiveElement, IntoElement, ParentElement,
    Pixels, StatefulInteractiveElement, Styled, Window, div, px,
};

use crate::models::ActiveTab;
use crate::theme;
use crate::{Icon, IconName};

/// Extension trait providing human-readable labels for navigation tabs.
pub trait ActiveTabExt {
    fn label(&self) -> &'static str;
}

impl ActiveTabExt for ActiveTab {
    fn label(&self) -> &'static str {
        match self {
            ActiveTab::Library => "Library",
            ActiveTab::Setup => "Setup",
            ActiveTab::Hub => "Hub",
            ActiveTab::Extensions => "Extensions",
            ActiveTab::Debugger => "Debugger",
            ActiveTab::Settings => "Settings",
        }
    }
}

/// Callback for navigation tab selection.
pub type TabClickHandler = Rc<dyn Fn(ActiveTab, &mut Window, &mut App) + 'static>;

/// Callback for toggling sidebar collapsed state.
pub type CollapseToggleHandler = Box<dyn Fn(&ClickEvent, &mut Window, &mut App) + 'static>;

/// Left sidebar navigation component displaying top-level application views,
/// branding, and live system status (gamepads & installed handlers).
pub struct CouchSidebarNav {
    active_tab: ActiveTab,
    detected_devices: usize,
    handler_count: usize,
    is_collapsed: bool,
    on_tab_click: Option<TabClickHandler>,
    on_toggle_collapse: Option<CollapseToggleHandler>,
}

impl Default for CouchSidebarNav {
    fn default() -> Self {
        Self::new()
    }
}

impl CouchSidebarNav {
    /// Creates a new `CouchSidebarNav` with default values.
    pub fn new() -> Self {
        Self {
            active_tab: ActiveTab::Library,
            detected_devices: 0,
            handler_count: 0,
            is_collapsed: false,
            on_tab_click: None,
            on_toggle_collapse: None,
        }
    }

    /// Sets the currently active navigation view tab.
    pub fn active_tab(mut self, tab: ActiveTab) -> Self {
        self.active_tab = tab;
        self
    }

    /// Sets the number of detected physical or virtual controllers.
    pub fn detected_devices(mut self, count: usize) -> Self {
        self.detected_devices = count;
        self
    }

    /// Sets the total count of discovered or installed game handlers.
    pub fn handler_count(mut self, count: usize) -> Self {
        self.handler_count = count;
        self
    }

    /// Sets whether the sidebar is in compact icon-only mode (60px) or expanded mode (200px).
    pub fn is_collapsed(mut self, collapsed: bool) -> Self {
        self.is_collapsed = collapsed;
        self
    }

    /// Registers a callback triggered when any navigation tab is clicked.
    pub fn on_tab_click<F>(mut self, handler: F) -> Self
    where
        F: Fn(ActiveTab, &mut Window, &mut App) + 'static,
    {
        self.on_tab_click = Some(Rc::new(handler));
        self
    }

    /// Registers a callback triggered when the collapse/expand toggle button is clicked.
    pub fn on_toggle_collapse<F>(mut self, handler: F) -> Self
    where
        F: Fn(&ClickEvent, &mut Window, &mut App) + 'static,
    {
        self.on_toggle_collapse = Some(Box::new(handler));
        self
    }

    /// Flattens the sidebar navigation into an `AnyElement`.
    pub fn into_any_element(self) -> AnyElement {
        let is_collapsed = self.is_collapsed;
        let width: Pixels = if is_collapsed { px(60.0) } else { px(200.0) };

        let tabs = [
            (ActiveTab::Library, "Library", IconName::BookOpen),
            (ActiveTab::Setup, "Setup", IconName::Gamepad2),
            (ActiveTab::Hub, "Hub", IconName::Globe),
            (ActiveTab::Extensions, "Extensions", IconName::LayoutGrid),
            (ActiveTab::Debugger, "Debugger", IconName::SlidersHorizontal),
            (ActiveTab::Settings, "Settings", IconName::Settings),
        ];

        // Top section: Branding badge ("NC") + app name "Nucleus Co-op" (if not collapsed) + collapse toggle button
        let mut toggle_btn = div()
            .id("btn-toggle-sidebar")
            .cursor_pointer()
            .flex()
            .items_center()
            .justify_center()
            .size(px(24.0))
            .rounded_md()
            .text_xs()
            .text_color(theme::text_muted())
            .hover(|s| s.bg(theme::bg_hover()).text_color(theme::text_primary()))
            .tooltip(crate::make_tooltip(if is_collapsed {
                "Expand sidebar"
            } else {
                "Collapse sidebar"
            }))
            .child(
                Icon::new(if is_collapsed {
                    IconName::ChevronRight
                } else {
                    IconName::ChevronLeft
                })
                .size_3p5(),
            );

        if let Some(on_toggle) = self.on_toggle_collapse {
            toggle_btn = toggle_btn.on_click(move |ev, win, cx| {
                on_toggle(ev, win, cx);
            });
        }

        let branding_badge = div()
            .flex()
            .items_center()
            .justify_center()
            .size(px(32.0))
            .rounded_md()
            .bg(theme::accent())
            .text_color(theme::text_on_accent())
            .font_weight(FontWeight::BOLD)
            .text_sm()
            .child("NC");

        let top_section = if is_collapsed {
            div()
                .flex()
                .items_center()
                .border_b_1()
                .border_color(theme::border_color())
                .flex_col()
                .gap_2()
                .p_2()
                .justify_center()
                .child(branding_badge)
                .child(toggle_btn)
        } else {
            div()
                .flex()
                .items_center()
                .border_b_1()
                .border_color(theme::border_color())
                .flex_row()
                .justify_between()
                .p_3()
                .child(
                    div()
                        .flex()
                        .items_center()
                        .gap_2p5()
                        .child(branding_badge)
                        .child(
                            div()
                                .flex()
                                .flex_col()
                                .child(
                                    div()
                                        .text_sm()
                                        .font_weight(FontWeight::BOLD)
                                        .text_color(theme::text_primary())
                                        .child("Nucleus Co-op"),
                                )
                                .child(
                                    div()
                                        .text_xs()
                                        .text_color(theme::text_muted())
                                        .child("Rust Native v0.1"),
                                ),
                        ),
                )
                .child(toggle_btn)
        };

        // Navigation items for the 4 tabs
        let on_tab = self.on_tab_click;
        let nav_items =
            div()
                .flex()
                .flex_col()
                .p_2()
                .gap_1()
                .children(
                    tabs.into_iter()
                        .enumerate()
                        .map(|(idx, (tab, label, icon))| {
                            let is_active = self.active_tab == tab;
                            let mut tab_btn = div()
                                .id(("sidebar-tab", idx))
                                .cursor_pointer()
                                .flex()
                                .items_center()
                                .when(is_collapsed, |this| this.justify_center().py_2p5().px_0())
                                .when(!is_collapsed, |this| this.gap_3().px_3().py_2())
                                .rounded_md()
                                .when(is_active, |this| {
                                    this.bg(theme::accent())
                                        .text_color(theme::text_on_accent())
                                        .font_weight(FontWeight::SEMIBOLD)
                                })
                                .when(!is_active, |this| {
                                    this.text_color(theme::text_muted()).hover(|s| {
                                        s.bg(theme::bg_hover()).text_color(theme::text_primary())
                                    })
                                });

                            if let Some(ref cb) = on_tab {
                                let cb = cb.clone();
                                tab_btn = tab_btn.on_click(move |_ev, win, cx| {
                                    cb(tab, win, cx);
                                });
                            }

                            tab_btn =
                                tab_btn.child(Icon::new(icon).size_4().text_color(if is_active {
                                    theme::text_on_accent()
                                } else {
                                    theme::text_muted()
                                }));
                            if !is_collapsed {
                                tab_btn = tab_btn.child(div().text_sm().child(label));
                            } else {
                                tab_btn = tab_btn.tooltip(crate::make_tooltip(tab.label()));
                            }

                            tab_btn
                        }),
                );

        // Bottom section: Gamepad status dot (green if count > 0, warning otherwise) + Gamepads count text + Handler count text.
        let status_dot = div()
            .size(px(8.0))
            .rounded_full()
            .bg(if self.detected_devices > 0 {
                theme::success()
            } else {
                theme::warning()
            });

        let bottom_section = div()
            .flex()
            .flex_col()
            .gap_1()
            .p_3()
            .border_t_1()
            .border_color(theme::border_color())
            .child(
                div()
                    .flex()
                    .items_center()
                    .when(is_collapsed, |this| this.justify_center())
                    .when(!is_collapsed, |this| this.gap_2())
                    .child(status_dot)
                    .when(!is_collapsed, |el| {
                        el.child(
                            div()
                                .text_xs()
                                .font_weight(FontWeight::MEDIUM)
                                .text_color(theme::text_primary())
                                .child(format!("{} Gamepads detected", self.detected_devices)),
                        )
                    }),
            )
            .when(!is_collapsed, |el| {
                el.child(
                    div()
                        .text_xs()
                        .text_color(theme::text_muted())
                        .child(format!("{} Game Handlers", self.handler_count)),
                )
            });

        div()
            .flex()
            .flex_col()
            .w(width)
            .h_full()
            .bg(theme::bg_panel())
            .border_r_1()
            .border_color(theme::border_color())
            .child(top_section)
            .child(nav_items)
            .child(div().flex_1())
            .child(bottom_section)
            .into_any_element()
    }
}

impl IntoElement for CouchSidebarNav {
    type Element = AnyElement;

    fn into_element(self) -> Self::Element {
        self.into_any_element()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_couch_sidebar_nav_builder_defaults() {
        let nav = CouchSidebarNav::new();
        assert_eq!(nav.active_tab, ActiveTab::Library);
        assert_eq!(nav.detected_devices, 0);
        assert_eq!(nav.handler_count, 0);
        assert!(!nav.is_collapsed);
        assert!(nav.on_tab_click.is_none());
        assert!(nav.on_toggle_collapse.is_none());

        let _element = nav.into_any_element();
    }

    #[test]
    fn test_couch_sidebar_nav_builder_custom() {
        let nav = CouchSidebarNav::new()
            .active_tab(ActiveTab::Setup)
            .detected_devices(2)
            .handler_count(42)
            .is_collapsed(true)
            .on_tab_click(|_tab, _win, _cx| {})
            .on_toggle_collapse(|_ev, _win, _cx| {});

        assert_eq!(nav.active_tab, ActiveTab::Setup);
        assert_eq!(nav.detected_devices, 2);
        assert_eq!(nav.handler_count, 42);
        assert!(nav.is_collapsed);
        assert!(nav.on_tab_click.is_some());
        assert!(nav.on_toggle_collapse.is_some());

        let _element = nav.into_any_element();
    }

    #[test]
    fn test_couch_sidebar_nav_tabs_coverage() {
        for tab in [
            ActiveTab::Library,
            ActiveTab::Setup,
            ActiveTab::Hub,
            ActiveTab::Settings,
        ] {
            let nav = CouchSidebarNav::new().active_tab(tab);
            let _element = nav.into_any_element();
        }
    }
}
