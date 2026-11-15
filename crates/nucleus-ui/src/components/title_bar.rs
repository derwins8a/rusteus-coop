//! CouchTitleBar meta-component.
//!
//! Provides the client-side decorations (CSD) top title bar for the application window,
//! including draggable window region, branding badge, active game title, Big Picture toggle,
//! and native window minimize, maximize/restore, and close controls.

use gpui::{
    div, px, rgb, AnyElement, App, ClickEvent, FontWeight, InteractiveElement,
    IntoElement, MouseButton, MouseDownEvent, ParentElement, StatefulInteractiveElement, Styled, Window,
};

use crate::components::button::{CouchButton, CouchButtonSize};
use crate::theme;
use crate::{Icon, IconName};

/// Callback handler type for title bar window actions.
pub type TitleBarEventHandler = Box<dyn Fn(&ClickEvent, &mut Window, &mut App) + 'static>;

/// Client-Side Decorations (CSD) custom title bar component.
pub struct CouchTitleBar {
    pub selected_game: Option<String>,
    pub is_maximized: Option<bool>,
    on_toggle_big_picture: Option<TitleBarEventHandler>,
    on_minimize: Option<TitleBarEventHandler>,
    on_maximize: Option<TitleBarEventHandler>,
    on_close: Option<TitleBarEventHandler>,
}

impl Default for CouchTitleBar {
    fn default() -> Self {
        Self::new()
    }
}

impl CouchTitleBar {
    /// Creates a new `CouchTitleBar` builder with default values.
    pub fn new() -> Self {
        Self {
            selected_game: None,
            is_maximized: None,
            on_toggle_big_picture: None,
            on_minimize: None,
            on_maximize: None,
            on_close: None,
        }
    }

    /// Sets the currently selected or configuring game title.
    pub fn selected_game(mut self, game: Option<&str>) -> Self {
        self.selected_game = game.map(|s| s.to_string());
        self
    }

    /// Explicitly sets whether the window is currently maximized.
    /// If not specified, it will dynamically detect the window state on Windows.
    pub fn is_maximized(mut self, is_maximized: bool) -> Self {
        self.is_maximized = Some(is_maximized);
        self
    }

    /// Sets the handler invoked when the Big Picture mode toggle button is clicked.
    pub fn on_toggle_big_picture<F>(mut self, handler: F) -> Self
    where
        F: Fn(&ClickEvent, &mut Window, &mut App) + 'static,
    {
        self.on_toggle_big_picture = Some(Box::new(handler));
        self
    }

    /// Sets the handler invoked when the Minimize window button is clicked.
    pub fn on_minimize<F>(mut self, handler: F) -> Self
    where
        F: Fn(&ClickEvent, &mut Window, &mut App) + 'static,
    {
        self.on_minimize = Some(Box::new(handler));
        self
    }

    /// Sets the handler invoked when the Maximize / Restore window button is clicked.
    pub fn on_maximize<F>(mut self, handler: F) -> Self
    where
        F: Fn(&ClickEvent, &mut Window, &mut App) + 'static,
    {
        self.on_maximize = Some(Box::new(handler));
        self
    }

    /// Sets the handler invoked when the Close window button is clicked.
    pub fn on_close<F>(mut self, handler: F) -> Self
    where
        F: Fn(&ClickEvent, &mut Window, &mut App) + 'static,
    {
        self.on_close = Some(Box::new(handler));
        self
    }

    /// Flattens and renders the component as an [`AnyElement`].
    pub fn into_any_element(self) -> AnyElement {
        let is_maximized = self.is_maximized.unwrap_or_else(|| {
            #[cfg(target_os = "windows")]
            unsafe {
                use windows::Win32::UI::Input::KeyboardAndMouse::GetActiveWindow;
                use windows::Win32::UI::WindowsAndMessaging::IsZoomed;
                let hwnd = GetActiveWindow();
                !hwnd.is_invalid() && IsZoomed(hwnd).as_bool()
            }
            #[cfg(not(target_os = "windows"))]
            false
        });

        // Left & Middle draggable region (including branding & subtitle text)
        let left_region = div()
            .id("csd-drag-region")
            .flex()
            .items_center()
            .flex_1()
            .h_full()
            .cursor_default()
            .on_mouse_down(
                MouseButton::Left,
                move |event: &MouseDownEvent, window: &mut Window, _cx: &mut App| {
                    if event.click_count == 2 {
                        #[cfg(target_os = "windows")]
                        unsafe {
                            use windows::Win32::UI::Input::KeyboardAndMouse::GetActiveWindow;
                            use windows::Win32::UI::WindowsAndMessaging::{
                                IsZoomed, PostMessageW, WM_SYSCOMMAND,
                            };
                            use windows::Win32::Foundation::{LPARAM, WPARAM};
                            let hwnd = GetActiveWindow();
                            if !hwnd.is_invalid() {
                                let is_zoomed = IsZoomed(hwnd).as_bool();
                                let sc_cmd = if is_zoomed {
                                    0xF120 /* SC_RESTORE */
                                } else {
                                    0xF030 /* SC_MAXIMIZE */
                                };
                                let _ = PostMessageW(hwnd, WM_SYSCOMMAND, WPARAM(sc_cmd), LPARAM(0));
                                return;
                            }
                        }
                        window.zoom_window();
                    } else {
                        #[cfg(target_os = "windows")]
                        unsafe {
                            use windows::Win32::UI::Input::KeyboardAndMouse::{
                                GetActiveWindow, ReleaseCapture,
                            };
                            use windows::Win32::UI::WindowsAndMessaging::{
                                PostMessageW, WM_NCLBUTTONDOWN,
                            };
                            use windows::Win32::Foundation::{LPARAM, WPARAM};
                            let hwnd = GetActiveWindow();
                            if !hwnd.is_invalid() {
                                let _ = ReleaseCapture();
                                let _ = PostMessageW(
                                    hwnd,
                                    WM_NCLBUTTONDOWN,
                                    WPARAM(2 /* HTCAPTION */),
                                    LPARAM(0),
                                );
                            }
                        }
                        #[cfg(not(target_os = "windows"))]
                        window.start_window_move();
                    }
                },
            )
            // Branding badge & title
            .child(
                div()
                    .flex()
                    .items_center()
                    .gap_2p5()
                    .child(
                        div()
                            .flex()
                            .items_center()
                            .justify_center()
                            .size(px(20.0))
                            .rounded_md()
                            .bg(theme::accent())
                            .text_color(theme::text_on_accent())
                            .font_weight(FontWeight::BOLD)
                            .text_xs()
                            .child("NC"),
                    )
                    .child(
                        div()
                            .flex()
                            .items_center()
                            .gap_2()
                            .child(
                                div()
                                    .text_xs()
                                    .font_weight(FontWeight::BOLD)
                                    .text_color(theme::text_primary())
                                    .child("Nucleus Co-op"),
                            )
                            .child(
                                div()
                                    .text_xs()
                                    .text_color(theme::text_muted())
                                    .child("•"),
                            )
                            .child(
                                div()
                                    .text_xs()
                                    .text_color(theme::text_muted())
                                    .child(if let Some(ref sel) = self.selected_game {
                                        format!("Configuring: {}", sel)
                                    } else {
                                        "Desktop Launcher".to_string()
                                    }),
                            ),
                    ),
            );

        // Big Picture toggle
        let mut big_picture_btn = CouchButton::ghost("btn-toggle-big-picture", "Big Picture")
            .icon(IconName::Tv)
            .size(CouchButtonSize::Sm)
            .tooltip("Toggle Big Picture 10-foot Couch Mode (F11)");

        if let Some(handler) = self.on_toggle_big_picture {
            big_picture_btn = big_picture_btn.on_click(move |ev, win, app| handler(ev, win, app));
        }

        // Minimize
        let mut min_btn = div()
            .id("btn-window-minimize")
            .cursor_pointer()
            .flex()
            .items_center()
            .justify_center()
            .size(px(26.0))
            .rounded_md()
            .text_xs()
            .text_color(theme::text_muted())
            .hover(|s| s.bg(theme::bg_hover()).text_color(theme::text_primary()))
            .tooltip(crate::make_tooltip("Minimize"));

        if let Some(handler) = self.on_minimize {
            min_btn = min_btn.on_click(move |ev, win, app| handler(ev, win, app));
        } else {
            min_btn = min_btn.on_click(|_ev, win, _app| {
                win.minimize_window();
            });
        }
        min_btn = min_btn.child(
            div()
                .w_2p5()
                .h(px(1.5))
                .rounded_full()
                .bg(theme::text_primary()),
        );

        // Maximize / Restore
        let mut max_btn = div()
            .id("btn-window-maximize")
            .cursor_pointer()
            .flex()
            .items_center()
            .justify_center()
            .size(px(26.0))
            .rounded_md()
            .text_xs()
            .text_color(theme::text_muted())
            .hover(|s| s.bg(theme::bg_hover()).text_color(theme::text_primary()))
            .tooltip(crate::make_tooltip(if is_maximized {
                "Restore"
            } else {
                "Maximize"
            }));

        if let Some(handler) = self.on_maximize {
            max_btn = max_btn.on_click(move |ev, win, app| handler(ev, win, app));
        } else {
            max_btn = max_btn.on_click(|_ev, win, _app| {
                #[cfg(target_os = "windows")]
                unsafe {
                    use windows::Win32::UI::Input::KeyboardAndMouse::GetActiveWindow;
                    use windows::Win32::UI::WindowsAndMessaging::{
                        IsZoomed, PostMessageW, WM_SYSCOMMAND,
                    };
                    use windows::Win32::Foundation::{LPARAM, WPARAM};
                    let hwnd = GetActiveWindow();
                    if !hwnd.is_invalid() {
                        let is_zoomed = IsZoomed(hwnd).as_bool();
                        let sc_cmd = if is_zoomed {
                            0xF120 /* SC_RESTORE */
                        } else {
                            0xF030 /* SC_MAXIMIZE */
                        };
                        let _ = PostMessageW(hwnd, WM_SYSCOMMAND, WPARAM(sc_cmd), LPARAM(0));
                        return;
                    }
                }
                win.zoom_window();
            });
        }
        max_btn = max_btn.child(
            Icon::new(if is_maximized {
                IconName::Copy
            } else {
                IconName::Maximize
            })
            .size_3(),
        );

        // Close
        let mut close_btn = div()
            .id("btn-window-close")
            .cursor_pointer()
            .flex()
            .items_center()
            .justify_center()
            .size(px(26.0))
            .rounded_md()
            .text_xs()
            .text_color(theme::text_muted())
            .hover(|s| s.bg(rgb(0xe81123)).text_color(rgb(0xffffff)))
            .tooltip(crate::make_tooltip("Close"));

        if let Some(handler) = self.on_close {
            close_btn = close_btn.on_click(move |ev, win, app| handler(ev, win, app));
        } else {
            close_btn = close_btn.on_click(|_ev, win, _app| {
                win.remove_window();
            });
        }
        close_btn = close_btn.child(Icon::new(IconName::X).size_3());

        let controls = div()
            .flex()
            .items_center()
            .gap_1()
            .child(big_picture_btn)
            .child(min_btn)
            .child(max_btn)
            .child(close_btn);

        div()
            .id("csd-title-bar")
            .flex()
            .items_center()
            .justify_between()
            .h(px(34.0))
            .w_full()
            .bg(theme::bg_panel())
            .border_b_1()
            .border_color(theme::border_color())
            .px_3()
            .child(left_region)
            .child(controls)
            .into_any_element()
    }
}

impl IntoElement for CouchTitleBar {
    type Element = AnyElement;

    fn into_element(self) -> Self::Element {
        self.into_any_element()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_couch_title_bar_builder() {
        let title_bar = CouchTitleBar::new()
            .selected_game(Some("Left 4 Dead 2"))
            .is_maximized(true)
            .on_toggle_big_picture(|_ev, _win, _app| {})
            .on_minimize(|_ev, _win, _app| {})
            .on_maximize(|_ev, _win, _app| {})
            .on_close(|_ev, _win, _app| {});

        assert_eq!(title_bar.selected_game.as_deref(), Some("Left 4 Dead 2"));
        assert_eq!(title_bar.is_maximized, Some(true));
        assert!(title_bar.on_toggle_big_picture.is_some());
        assert!(title_bar.on_minimize.is_some());
        assert!(title_bar.on_maximize.is_some());
        assert!(title_bar.on_close.is_some());

        // Verify conversion into AnyElement
        let _elem = title_bar.into_any_element();
    }

    #[test]
    fn test_couch_title_bar_defaults() {
        let title_bar = CouchTitleBar::default();
        assert_eq!(title_bar.selected_game, None);
        assert_eq!(title_bar.is_maximized, None);
        assert!(title_bar.on_toggle_big_picture.is_none());
        assert!(title_bar.on_minimize.is_none());
        assert!(title_bar.on_maximize.is_none());
        assert!(title_bar.on_close.is_none());

        // Verify conversion into AnyElement for default state
        let _elem = title_bar.into_any_element();
    }
}
