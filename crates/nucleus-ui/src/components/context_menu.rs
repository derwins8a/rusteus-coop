//! # CouchContextMenu Component
//!
//! Right-click and contextual options menu dialog for game library items in Nucleus Co-op.
//!
//! ## Invariants & Guarantees:
//! 1. **Backdrop Scrim**: Full-screen overlay (`theme::overlay_scrim()`) with `id("context-menu-backdrop")`.
//!    Clicking outside the menu via Left OR Right mouse button triggers `on_close`.
//! 2. **Event Shielding**: Context menu panel intercepts and stops click and mouse-down propagation
//!    for both Left and Right mouse buttons to prevent pass-through to underlying cards.
//! 3. **Ergonomic Sizing**: Bounded panel width (`max_w(px(360.0))`), rounded borders (`rounded_xl()`),
//!    panel background (`theme::bg_panel()`), and drop shadow.
//! 4. **Type Boundary Flattening**: Implements `into_any_element()` returning `AnyElement` to guarantee
//!    bounded generic types and fast build times.

use std::rc::Rc;

use gpui::{
    AnyElement, App, ClickEvent, FontWeight, InteractiveElement, IntoElement, MouseButton,
    ParentElement, StatefulInteractiveElement, Styled, Window, div, px,
};

use crate::models::HandlerItem;
use crate::theme;
use crate::{Icon, IconName};

/// Callback handler type for generic context menu click actions.
pub type ContextMenuClickHandler = Box<dyn Fn(&ClickEvent, &mut Window, &mut App) + 'static>;

/// Callback handler type for language selection actions.
pub type LanguageSelectHandler = Box<dyn Fn(&str, &mut Window, &mut App) + 'static>;

/// Contextual actions menu dialog component displayed upon right-clicking a game card in Library view.
pub struct CouchContextMenu {
    /// Associated game handler item.
    pub handler: HandlerItem,
    /// Directory where handler scripts are stored.
    pub handlers_dir: String,
    /// Configured text editor path for viewing handler scripts.
    pub text_editor_path: String,
    /// Whether the expandable Steam language picker grid is open.
    pub is_language_picker_open: bool,

    /// Callback invoked when backdrop or close button is clicked.
    pub on_close: Option<ContextMenuClickHandler>,
    /// Callback invoked to open the handler script or folder in text editor/explorer.
    pub on_open_handler: Option<ContextMenuClickHandler>,
    /// Callback invoked to open the directory of the original game executable.
    pub on_open_exe: Option<ContextMenuClickHandler>,
    /// Callback invoked to open the UserProfile save data directory for this game.
    pub on_open_userprofile: Option<ContextMenuClickHandler>,
    /// Callback invoked to prompt confirmation for deleting the UserProfile save data directory.
    pub on_delete_userprofile: Option<ContextMenuClickHandler>,
    /// Callback invoked to toggle "Assign Gamepads By Index".
    pub on_toggle_assign_by_index: Option<ContextMenuClickHandler>,
    /// Callback invoked to toggle "Assign Gamepads On Button Press".
    pub on_toggle_assign_on_press: Option<ContextMenuClickHandler>,
    /// Callback invoked to toggle "Keep Instances Content Folder".
    pub on_toggle_keep_content: Option<ContextMenuClickHandler>,
    /// Callback invoked to toggle "Disable Profile".
    pub on_toggle_disable_profile: Option<ContextMenuClickHandler>,
    /// Callback invoked to toggle "Disable Handler Update".
    pub on_toggle_disable_update: Option<ContextMenuClickHandler>,
    /// Callback invoked to open custom game icon selection dialog.
    pub on_change_icon: Option<ContextMenuClickHandler>,
    /// Callback invoked to open local game assets folder (covers, screenshots, icons).
    pub on_open_assets: Option<ContextMenuClickHandler>,
    /// Callback invoked to toggle the Steam language dropdown picker.
    pub on_toggle_language_picker: Option<ContextMenuClickHandler>,
    /// Callback invoked when a language is selected from the language picker.
    pub on_select_language: Option<LanguageSelectHandler>,
    /// Callback invoked to create a desktop shortcut (.lnk / .desktop).
    pub on_create_desktop_shortcut: Option<ContextMenuClickHandler>,
    /// Callback invoked to copy the CLI launch command to the clipboard.
    pub on_copy_launch_cmd: Option<ContextMenuClickHandler>,
    /// Callback invoked to prompt confirmation for removing the game from the library.
    pub on_remove_game: Option<ContextMenuClickHandler>,
}

impl CouchContextMenu {
    /// Creates a new `CouchContextMenu` for the given handler item.
    pub fn new(handler: HandlerItem) -> Self {
        Self {
            handler,
            handlers_dir: String::new(),
            text_editor_path: String::new(),
            is_language_picker_open: false,
            on_close: None,
            on_open_handler: None,
            on_open_exe: None,
            on_open_userprofile: None,
            on_delete_userprofile: None,
            on_toggle_assign_by_index: None,
            on_toggle_assign_on_press: None,
            on_toggle_keep_content: None,
            on_toggle_disable_profile: None,
            on_toggle_disable_update: None,
            on_change_icon: None,
            on_open_assets: None,
            on_toggle_language_picker: None,
            on_select_language: None,
            on_create_desktop_shortcut: None,
            on_copy_launch_cmd: None,
            on_remove_game: None,
        }
    }

    /// Sets the handlers directory path.
    pub fn handlers_dir(mut self, dir: impl Into<String>) -> Self {
        self.handlers_dir = dir.into();
        self
    }

    /// Sets the text editor executable path.
    pub fn text_editor_path(mut self, path: impl Into<String>) -> Self {
        self.text_editor_path = path.into();
        self
    }

    /// Sets whether the Steam language selection grid is expanded.
    pub fn language_picker_open(mut self, open: bool) -> Self {
        self.is_language_picker_open = open;
        self
    }

    /// Sets the handler invoked when closing the context menu.
    pub fn on_close<F>(mut self, handler: F) -> Self
    where
        F: Fn(&ClickEvent, &mut Window, &mut App) + 'static,
    {
        self.on_close = Some(Box::new(handler));
        self
    }

    /// Sets the handler invoked to open the game handler script.
    pub fn on_open_handler<F>(mut self, handler: F) -> Self
    where
        F: Fn(&ClickEvent, &mut Window, &mut App) + 'static,
    {
        self.on_open_handler = Some(Box::new(handler));
        self
    }

    /// Sets the handler invoked to open the original executable directory.
    pub fn on_open_exe<F>(mut self, handler: F) -> Self
    where
        F: Fn(&ClickEvent, &mut Window, &mut App) + 'static,
    {
        self.on_open_exe = Some(Box::new(handler));
        self
    }

    /// Sets the handler invoked to open the UserProfile save directory.
    pub fn on_open_userprofile<F>(mut self, handler: F) -> Self
    where
        F: Fn(&ClickEvent, &mut Window, &mut App) + 'static,
    {
        self.on_open_userprofile = Some(Box::new(handler));
        self
    }

    /// Sets the handler invoked to delete UserProfile save data.
    pub fn on_delete_userprofile<F>(mut self, handler: F) -> Self
    where
        F: Fn(&ClickEvent, &mut Window, &mut App) + 'static,
    {
        self.on_delete_userprofile = Some(Box::new(handler));
        self
    }

    /// Sets the handler invoked to toggle assign gamepads by index.
    pub fn on_toggle_assign_by_index<F>(mut self, handler: F) -> Self
    where
        F: Fn(&ClickEvent, &mut Window, &mut App) + 'static,
    {
        self.on_toggle_assign_by_index = Some(Box::new(handler));
        self
    }

    /// Sets the handler invoked to toggle assign gamepads on button press.
    pub fn on_toggle_assign_on_press<F>(mut self, handler: F) -> Self
    where
        F: Fn(&ClickEvent, &mut Window, &mut App) + 'static,
    {
        self.on_toggle_assign_on_press = Some(Box::new(handler));
        self
    }

    /// Sets the handler invoked to toggle keeping instances content folder.
    pub fn on_toggle_keep_content<F>(mut self, handler: F) -> Self
    where
        F: Fn(&ClickEvent, &mut Window, &mut App) + 'static,
    {
        self.on_toggle_keep_content = Some(Box::new(handler));
        self
    }

    /// Sets the handler invoked to toggle disable profile.
    pub fn on_toggle_disable_profile<F>(mut self, handler: F) -> Self
    where
        F: Fn(&ClickEvent, &mut Window, &mut App) + 'static,
    {
        self.on_toggle_disable_profile = Some(Box::new(handler));
        self
    }

    /// Sets the handler invoked to toggle disable handler update.
    pub fn on_toggle_disable_update<F>(mut self, handler: F) -> Self
    where
        F: Fn(&ClickEvent, &mut Window, &mut App) + 'static,
    {
        self.on_toggle_disable_update = Some(Box::new(handler));
        self
    }

    /// Sets the handler invoked to change the game icon.
    pub fn on_change_icon<F>(mut self, handler: F) -> Self
    where
        F: Fn(&ClickEvent, &mut Window, &mut App) + 'static,
    {
        self.on_change_icon = Some(Box::new(handler));
        self
    }

    /// Sets the handler invoked to open the local game assets directory.
    pub fn on_open_assets<F>(mut self, handler: F) -> Self
    where
        F: Fn(&ClickEvent, &mut Window, &mut App) + 'static,
    {
        self.on_open_assets = Some(Box::new(handler));
        self
    }

    /// Sets the handler invoked to toggle the Steam language dropdown picker.
    pub fn on_toggle_language_picker<F>(mut self, handler: F) -> Self
    where
        F: Fn(&ClickEvent, &mut Window, &mut App) + 'static,
    {
        self.on_toggle_language_picker = Some(Box::new(handler));
        self
    }

    /// Sets the handler invoked when selecting a Steam language.
    pub fn on_select_language<F>(mut self, handler: F) -> Self
    where
        F: Fn(&str, &mut Window, &mut App) + 'static,
    {
        self.on_select_language = Some(Box::new(handler));
        self
    }

    /// Sets the handler invoked to create a desktop shortcut.
    pub fn on_create_desktop_shortcut<F>(mut self, handler: F) -> Self
    where
        F: Fn(&ClickEvent, &mut Window, &mut App) + 'static,
    {
        self.on_create_desktop_shortcut = Some(Box::new(handler));
        self
    }

    /// Sets the handler invoked to copy the CLI launch command to clipboard.
    pub fn on_copy_launch_cmd<F>(mut self, handler: F) -> Self
    where
        F: Fn(&ClickEvent, &mut Window, &mut App) + 'static,
    {
        self.on_copy_launch_cmd = Some(Box::new(handler));
        self
    }

    /// Sets the handler invoked to remove the game from the library.
    pub fn on_remove_game<F>(mut self, handler: F) -> Self
    where
        F: Fn(&ClickEvent, &mut Window, &mut App) + 'static,
    {
        self.on_remove_game = Some(Box::new(handler));
        self
    }

    /// Flattens and renders the contextual menu dialog as an [`AnyElement`].
    pub fn into_any_element(self) -> AnyElement {
        let game_name = self.handler.game_name.clone();
        let exe_available = self.handler.executable_path.is_some();
        let assign_by_index = self.handler.assign_gamepads_by_index;
        let assign_on_press = self.handler.assign_gamepads_on_button_press;
        let keep_folder = self.handler.keep_instances_content_folder;
        let disable_prof = self.handler.disable_profile;
        let disable_upd = self.handler.disable_handler_update;
        let cur_lang = self
            .handler
            .steam_language
            .clone()
            .unwrap_or_else(|| "english".to_string());
        let is_picker_open = self.is_language_picker_open;

        // Wrap on_close into Rc so it can be called by backdrop left/right clicks and close button.
        let on_close_shared: Option<Rc<dyn Fn(&ClickEvent, &mut Window, &mut App) + 'static>> =
            self.on_close.map(Rc::from);

        // 1. Full-screen backdrop overlay
        let mut backdrop = div()
            .id("context-menu-backdrop")
            .absolute()
            .inset_0()
            .occlude()
            .bg(theme::overlay_scrim());

        if let Some(ref cb) = on_close_shared {
            let cb_left = cb.clone();
            let cb_right = cb.clone();
            let cb_click = cb.clone();
            backdrop = backdrop
                .on_mouse_down(MouseButton::Left, move |_ev, win, app| {
                    cb_left(&ClickEvent::default(), win, app);
                })
                .on_mouse_down(MouseButton::Right, move |_ev, win, app| {
                    cb_right(&ClickEvent::default(), win, app);
                })
                .on_click(move |ev, win, app| {
                    cb_click(ev, win, app);
                });
        }

        // 2. Context menu panel
        let mut panel = div()
            .id("context-menu-panel")
            .relative()
            .flex()
            .flex_col()
            .w_full()
            .max_w(px(360.0))
            .p_3()
            .gap_1()
            .bg(theme::bg_panel())
            .rounded_xl()
            .border_1()
            .border_color(theme::border_color())
            .shadow_lg()
            .on_mouse_down(MouseButton::Left, |_, _, cx| {
                cx.stop_propagation();
            })
            .on_mouse_down(MouseButton::Right, |_, _, cx| {
                cx.stop_propagation();
            });

        // Header
        let mut close_btn = div()
            .id("ctx-close-btn")
            .cursor_pointer()
            .px_1p5()
            .py_0p5()
            .rounded_md()
            .bg(theme::bg_active())
            .text_xs()
            .text_color(theme::text_muted())
            .hover(|s| s.text_color(theme::text_primary()))
            .child(Icon::new(IconName::X).size(px(12.0)));

        if let Some(ref cb) = on_close_shared {
            let cb = cb.clone();
            close_btn = close_btn.on_click(move |ev, win, app| {
                cb(ev, win, app);
            });
        }

        let header = div()
            .flex()
            .items_center()
            .justify_between()
            .px_2()
            .py_1p5()
            .border_b_1()
            .border_color(theme::border_color().opacity(0.5))
            .child(
                div()
                    .flex()
                    .items_center()
                    .gap_2()
                    .child(
                        Icon::new(IconName::Wrench)
                            .size(px(16.0))
                            .text_color(theme::accent()),
                    )
                    .child(
                        div()
                            .text_sm()
                            .font_weight(FontWeight::BOLD)
                            .text_color(theme::text_primary())
                            .truncate()
                            .child(game_name),
                    ),
            )
            .child(close_btn);

        panel = panel.child(header);

        // Action 1: Open Game Handler
        let mut open_handler_row = div()
            .id("ctx-open-handler")
            .cursor_pointer()
            .flex()
            .items_center()
            .gap_2p5()
            .px_2p5()
            .py_1p5()
            .rounded_md()
            .text_xs()
            .text_color(theme::text_primary())
            .hover(|s| s.bg(theme::bg_active()))
            .child(
                Icon::new(IconName::Folder)
                    .size(px(16.0))
                    .text_color(theme::text_muted()),
            )
            .child(div().child("Open Game Handler"));

        if let Some(cb) = self.on_open_handler {
            open_handler_row = open_handler_row.on_click(move |ev, win, app| cb(ev, win, app));
        }
        panel = panel.child(open_handler_row);

        // Action 2: Open Original Exe Path
        let mut open_exe_row =
            div()
                .id("ctx-open-exe")
                .cursor_pointer()
                .flex()
                .items_center()
                .gap_2p5()
                .px_2p5()
                .py_1p5()
                .rounded_md()
                .text_xs()
                .text_color(if exe_available {
                    theme::text_primary()
                } else {
                    theme::text_muted()
                })
                .hover(|s| s.bg(theme::bg_active()))
                .child(Icon::new(IconName::FolderOpen).size(px(16.0)).text_color(
                    if exe_available {
                        theme::text_muted()
                    } else {
                        theme::text_subtle()
                    },
                ))
                .child(div().child("Open Original Exe Path"));

        if let Some(cb) = self.on_open_exe {
            open_exe_row = open_exe_row.on_click(move |ev, win, app| cb(ev, win, app));
        }
        panel = panel.child(open_exe_row);

        // Action 3: Open UserProfile Save Path
        let mut open_userprofile_row = div()
            .id("ctx-open-userprofile")
            .cursor_pointer()
            .flex()
            .items_center()
            .gap_2p5()
            .px_2p5()
            .py_1p5()
            .rounded_md()
            .text_xs()
            .text_color(theme::text_primary())
            .hover(|s| s.bg(theme::bg_active()))
            .child(
                Icon::new(IconName::Save)
                    .size(px(16.0))
                    .text_color(theme::text_muted()),
            )
            .child(div().child("Open UserProfile Save Path"));

        if let Some(cb) = self.on_open_userprofile {
            open_userprofile_row =
                open_userprofile_row.on_click(move |ev, win, app| cb(ev, win, app));
        }
        panel = panel.child(open_userprofile_row);

        // Action 4: Delete UserProfile Save Path
        let mut delete_userprofile_row = div()
            .id("ctx-delete-userprofile")
            .cursor_pointer()
            .flex()
            .items_center()
            .gap_2p5()
            .px_2p5()
            .py_1p5()
            .rounded_md()
            .text_xs()
            .text_color(theme::warning())
            .hover(|s| s.bg(theme::warning().opacity(0.15)))
            .child(
                Icon::new(IconName::Trash)
                    .size(px(16.0))
                    .text_color(theme::warning()),
            )
            .child(div().child("Delete UserProfile Save Path"));

        if let Some(cb) = self.on_delete_userprofile {
            delete_userprofile_row =
                delete_userprofile_row.on_click(move |ev, win, app| cb(ev, win, app));
        }
        panel = panel.child(delete_userprofile_row);

        // Separator
        let separator = || div().h_px().bg(theme::border_color().opacity(0.5)).my_1();
        panel = panel.child(separator());

        // Action 5: Assign Gamepads By Index
        let mut assign_by_index_row = div()
            .id("ctx-toggle-assign-by-index")
            .cursor_pointer()
            .flex()
            .items_center()
            .justify_between()
            .px_2p5()
            .py_1p5()
            .rounded_md()
            .text_xs()
            .text_color(theme::text_primary())
            .hover(|s| s.bg(theme::bg_active()))
            .child(
                div()
                    .flex()
                    .items_center()
                    .gap_2p5()
                    .child(
                        Icon::new(IconName::Gamepad2)
                            .size(px(16.0))
                            .text_color(theme::text_muted()),
                    )
                    .child(div().child("Assign Gamepads By Index")),
            )
            .child(
                div()
                    .px_2()
                    .py_0p5()
                    .rounded_md()
                    .bg(if assign_by_index {
                        theme::accent()
                    } else {
                        theme::bg_active()
                    })
                    .text_color(if assign_by_index {
                        theme::text_on_accent()
                    } else {
                        theme::text_muted()
                    })
                    .font_weight(FontWeight::BOLD)
                    .child(if assign_by_index { "ON" } else { "OFF" }),
            );

        if let Some(cb) = self.on_toggle_assign_by_index {
            assign_by_index_row =
                assign_by_index_row.on_click(move |ev, win, app| cb(ev, win, app));
        }
        panel = panel.child(assign_by_index_row);

        // Action 6: Assign Gamepads On Button Press (profile)
        let mut assign_on_press_row = div()
            .id("ctx-toggle-assign-on-press")
            .cursor_pointer()
            .flex()
            .items_center()
            .justify_between()
            .px_2p5()
            .py_1p5()
            .rounded_md()
            .text_xs()
            .text_color(theme::text_primary())
            .hover(|s| s.bg(theme::bg_active()))
            .child(
                div()
                    .flex()
                    .items_center()
                    .gap_2p5()
                    .child(
                        Icon::new(IconName::CircleDot)
                            .size(px(16.0))
                            .text_color(theme::text_muted()),
                    )
                    .child(div().child("Assign Gamepads On Button Press (profile)")),
            )
            .child(
                div()
                    .px_2()
                    .py_0p5()
                    .rounded_md()
                    .bg(if assign_on_press {
                        theme::accent()
                    } else {
                        theme::bg_active()
                    })
                    .text_color(if assign_on_press {
                        theme::text_on_accent()
                    } else {
                        theme::text_muted()
                    })
                    .font_weight(FontWeight::BOLD)
                    .child(if assign_on_press { "ON" } else { "OFF" }),
            );

        if let Some(cb) = self.on_toggle_assign_on_press {
            assign_on_press_row =
                assign_on_press_row.on_click(move |ev, win, app| cb(ev, win, app));
        }
        panel = panel.child(assign_on_press_row);

        // Action 7: Keep Instances Content Folder
        let mut keep_content_row = div()
            .id("ctx-toggle-keep-content")
            .cursor_pointer()
            .flex()
            .items_center()
            .justify_between()
            .px_2p5()
            .py_1p5()
            .rounded_md()
            .text_xs()
            .text_color(theme::text_primary())
            .hover(|s| s.bg(theme::bg_active()))
            .child(
                div()
                    .flex()
                    .items_center()
                    .gap_2p5()
                    .child(
                        Icon::new(IconName::Package)
                            .size(px(16.0))
                            .text_color(theme::text_muted()),
                    )
                    .child(div().child("Keep Instances Content Folder")),
            )
            .child(
                div()
                    .px_2()
                    .py_0p5()
                    .rounded_md()
                    .bg(if keep_folder {
                        theme::accent()
                    } else {
                        theme::bg_active()
                    })
                    .text_color(if keep_folder {
                        theme::text_on_accent()
                    } else {
                        theme::text_muted()
                    })
                    .font_weight(FontWeight::BOLD)
                    .child(if keep_folder { "ON" } else { "OFF" }),
            );

        if let Some(cb) = self.on_toggle_keep_content {
            keep_content_row = keep_content_row.on_click(move |ev, win, app| cb(ev, win, app));
        }
        panel = panel.child(keep_content_row);

        // Action 8: Disable Profile
        let mut disable_profile_row = div()
            .id("ctx-toggle-disable-profile")
            .cursor_pointer()
            .flex()
            .items_center()
            .justify_between()
            .px_2p5()
            .py_1p5()
            .rounded_md()
            .text_xs()
            .text_color(theme::text_primary())
            .hover(|s| s.bg(theme::bg_active()))
            .child(
                div()
                    .flex()
                    .items_center()
                    .gap_2p5()
                    .child(
                        Icon::new(IconName::Ban)
                            .size(px(16.0))
                            .text_color(theme::danger()),
                    )
                    .child(div().child("Disable Profile")),
            )
            .child(
                div()
                    .px_2()
                    .py_0p5()
                    .rounded_md()
                    .bg(if disable_prof {
                        theme::danger()
                    } else {
                        theme::bg_active()
                    })
                    .text_color(if disable_prof {
                        theme::text_on_accent()
                    } else {
                        theme::text_muted()
                    })
                    .font_weight(FontWeight::BOLD)
                    .child(if disable_prof { "ON" } else { "OFF" }),
            );

        if let Some(cb) = self.on_toggle_disable_profile {
            disable_profile_row =
                disable_profile_row.on_click(move |ev, win, app| cb(ev, win, app));
        }
        panel = panel.child(disable_profile_row);

        // Action 9: Disable Handler Update
        let mut disable_update_row = div()
            .id("ctx-toggle-disable-update")
            .cursor_pointer()
            .flex()
            .items_center()
            .justify_between()
            .px_2p5()
            .py_1p5()
            .rounded_md()
            .text_xs()
            .text_color(theme::text_primary())
            .hover(|s| s.bg(theme::bg_active()))
            .child(
                div()
                    .flex()
                    .items_center()
                    .gap_2p5()
                    .child(
                        Icon::new(IconName::Lock)
                            .size(px(16.0))
                            .text_color(theme::text_muted()),
                    )
                    .child(div().child("Disable Handler Update")),
            )
            .child(
                div()
                    .px_2()
                    .py_0p5()
                    .rounded_md()
                    .bg(if disable_upd {
                        theme::accent()
                    } else {
                        theme::bg_active()
                    })
                    .text_color(if disable_upd {
                        theme::text_on_accent()
                    } else {
                        theme::text_muted()
                    })
                    .font_weight(FontWeight::BOLD)
                    .child(if disable_upd { "ON" } else { "OFF" }),
            );

        if let Some(cb) = self.on_toggle_disable_update {
            disable_update_row = disable_update_row.on_click(move |ev, win, app| cb(ev, win, app));
        }
        panel = panel.child(disable_update_row);

        // Separator
        panel = panel.child(separator());

        // Action 10: Change Game Icon
        let mut change_icon_row = div()
            .id("ctx-change-icon")
            .cursor_pointer()
            .flex()
            .items_center()
            .gap_2p5()
            .px_2p5()
            .py_1p5()
            .rounded_md()
            .text_xs()
            .text_color(theme::text_primary())
            .hover(|s| s.bg(theme::bg_active()))
            .child(
                Icon::new(IconName::Image)
                    .size(px(16.0))
                    .text_color(theme::text_muted()),
            )
            .child(div().child("Change Game Icon"));

        if let Some(cb) = self.on_change_icon {
            change_icon_row = change_icon_row.on_click(move |ev, win, app| cb(ev, win, app));
        }
        panel = panel.child(change_icon_row);

        // Action 11: Game Assets
        let mut open_assets_row = div()
            .id("ctx-open-assets")
            .cursor_pointer()
            .flex()
            .items_center()
            .gap_2p5()
            .px_2p5()
            .py_1p5()
            .rounded_md()
            .text_xs()
            .text_color(theme::text_primary())
            .hover(|s| s.bg(theme::bg_active()))
            .child(
                Icon::new(IconName::Palette)
                    .size(px(16.0))
                    .text_color(theme::text_muted()),
            )
            .child(div().child("Game Assets (Covers & Artwork)"));

        if let Some(cb) = self.on_open_assets {
            open_assets_row = open_assets_row.on_click(move |ev, win, app| cb(ev, win, app));
        }
        panel = panel.child(open_assets_row);

        // Action 12: Steam Language selector (collapsible dropdown picker)
        let langs = [
            "english",
            "german",
            "french",
            "spanish",
            "italian",
            "russian",
            "japanese",
            "schinese",
            "tchinese",
            "koreana",
            "brazilian",
            "polish",
        ];

        let mut toggle_lang_btn = div()
            .id("ctx-toggle-steam-language")
            .cursor_pointer()
            .flex()
            .items_center()
            .justify_between()
            .px_2p5()
            .py_1p5()
            .rounded_md()
            .text_xs()
            .text_color(theme::text_primary())
            .hover(|s| s.bg(theme::bg_active()))
            .child(
                div()
                    .flex()
                    .items_center()
                    .gap_2p5()
                    .child(
                        Icon::new(IconName::Globe)
                            .size(px(16.0))
                            .text_color(theme::text_muted()),
                    )
                    .child(div().child("Steam Language")),
            )
            .child(
                div()
                    .flex()
                    .items_center()
                    .gap_1p5()
                    .child(
                        div()
                            .px_2()
                            .py_0p5()
                            .rounded_md()
                            .bg(theme::bg_active())
                            .text_color(theme::accent())
                            .font_weight(FontWeight::BOLD)
                            .child(cur_lang.clone()),
                    )
                    .child(
                        div()
                            .text_xs()
                            .text_color(theme::accent())
                            .child(if is_picker_open { "▲" } else { "▼" }),
                    ),
            );

        if let Some(cb) = self.on_toggle_language_picker {
            toggle_lang_btn = toggle_lang_btn.on_click(move |ev, win, app| cb(ev, win, app));
        }

        let mut lang_container = div().flex().flex_col().gap_1().child(toggle_lang_btn);

        if is_picker_open {
            let on_select_rc: Option<Rc<dyn Fn(&str, &mut Window, &mut App)>> =
                self.on_select_language.map(Rc::from);
            let mut chips_grid = div()
                .flex()
                .flex_wrap()
                .gap_1()
                .p_1p5()
                .bg(theme::bg_panel())
                .rounded_md()
                .border_1()
                .border_color(theme::accent().opacity(0.4));

            for (l_idx, &l) in langs.iter().enumerate() {
                let is_active = l.eq_ignore_ascii_case(&cur_lang);
                let mut chip = div()
                    .id(("ctx-lang-opt", l_idx))
                    .cursor_pointer()
                    .px_2()
                    .py_1()
                    .rounded_sm()
                    .text_xs();

                if is_active {
                    chip = chip
                        .bg(theme::accent())
                        .text_color(theme::text_on_accent())
                        .font_weight(FontWeight::BOLD);
                } else {
                    chip = chip
                        .bg(theme::bg_subtle())
                        .text_color(theme::text_muted())
                        .hover(|h| h.bg(theme::bg_hover()).text_color(theme::text_primary()));
                }

                if let Some(ref cb) = on_select_rc {
                    let cb = cb.clone();
                    let l_str = l.to_string();
                    chip = chip.on_click(move |_ev, win, app| {
                        cb(&l_str, win, app);
                    });
                }

                chip = chip.child(l);
                chips_grid = chips_grid.child(chip);
            }

            lang_container = lang_container.child(chips_grid);
        }

        panel = panel.child(lang_container);

        // Separator
        panel = panel.child(separator());

        // Action: Create Desktop Shortcut
        let mut create_shortcut_row = div()
            .id("ctx-create-desktop-shortcut")
            .cursor_pointer()
            .flex()
            .items_center()
            .gap_2p5()
            .px_2p5()
            .py_1p5()
            .rounded_md()
            .text_xs()
            .text_color(theme::text_primary())
            .hover(|s| s.bg(theme::bg_active()))
            .child(
                Icon::new(IconName::Monitor)
                    .size(px(16.0))
                    .text_color(theme::text_muted()),
            )
            .child(div().child("Create Desktop Shortcut"));

        if let Some(cb) = self.on_create_desktop_shortcut {
            create_shortcut_row =
                create_shortcut_row.on_click(move |ev, win, app| cb(ev, win, app));
        }
        panel = panel.child(create_shortcut_row);

        // Action: Copy Launch Command (Playnite / Steam)
        let mut copy_launch_cmd_row = div()
            .id("ctx-copy-launch-cmd")
            .cursor_pointer()
            .flex()
            .items_center()
            .gap_2p5()
            .px_2p5()
            .py_1p5()
            .rounded_md()
            .text_xs()
            .text_color(theme::text_primary())
            .hover(|s| s.bg(theme::bg_active()))
            .child(
                Icon::new(IconName::Copy)
                    .size(px(16.0))
                    .text_color(theme::text_muted()),
            )
            .child(div().child("Copy Launch Command (Playnite / Steam)"));

        if let Some(cb) = self.on_copy_launch_cmd {
            copy_launch_cmd_row =
                copy_launch_cmd_row.on_click(move |ev, win, app| cb(ev, win, app));
        }
        panel = panel.child(copy_launch_cmd_row);

        // Separator
        panel = panel.child(separator());

        // Action 13: Remove Game From List (destructive/danger tone)
        let mut remove_game_row = div()
            .id("ctx-remove-game")
            .cursor_pointer()
            .flex()
            .items_center()
            .gap_2p5()
            .px_2p5()
            .py_1p5()
            .rounded_md()
            .text_xs()
            .font_weight(FontWeight::SEMIBOLD)
            .text_color(theme::danger())
            .hover(|s| s.bg(theme::danger_subtle()))
            .child(
                Icon::new(IconName::Trash)
                    .size(px(16.0))
                    .text_color(theme::danger()),
            )
            .child(div().child("Remove Game From List"));

        if let Some(cb) = self.on_remove_game {
            remove_game_row = remove_game_row.on_click(move |ev, win, app| cb(ev, win, app));
        }
        panel = panel.child(remove_game_row);

        // 3. Assemble full modal overlay
        div()
            .id("context-menu-modal-overlay")
            .absolute()
            .inset_0()
            .flex()
            .items_center()
            .justify_center()
            .p_4()
            .child(backdrop)
            .child(panel)
            .into_any_element()
    }
}

impl IntoElement for CouchContextMenu {
    type Element = AnyElement;

    fn into_element(self) -> Self::Element {
        self.into_any_element()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::models::HandlerItem;

    #[test]
    fn test_couch_context_menu_builder() {
        let handler = HandlerItem::new(
            "test-guid-l4d2",
            "Left 4 Dead 2",
            "left4dead2.exe",
            4,
            "L4D2 Handler",
        );

        let menu = CouchContextMenu::new(handler)
            .handlers_dir("C:\\Games\\NucleusCoop\\handlers")
            .text_editor_path("C:\\Program Files\\Notepad++\\notepad++.exe")
            .language_picker_open(true)
            .on_close(|_, _, _| {})
            .on_open_handler(|_, _, _| {})
            .on_open_exe(|_, _, _| {})
            .on_open_userprofile(|_, _, _| {})
            .on_delete_userprofile(|_, _, _| {})
            .on_toggle_assign_by_index(|_, _, _| {})
            .on_toggle_assign_on_press(|_, _, _| {})
            .on_toggle_keep_content(|_, _, _| {})
            .on_toggle_disable_profile(|_, _, _| {})
            .on_toggle_disable_update(|_, _, _| {})
            .on_change_icon(|_, _, _| {})
            .on_open_assets(|_, _, _| {})
            .on_toggle_language_picker(|_, _, _| {})
            .on_select_language(|_, _, _| {})
            .on_create_desktop_shortcut(|_, _, _| {})
            .on_copy_launch_cmd(|_, _, _| {})
            .on_remove_game(|_, _, _| {});

        assert_eq!(menu.handler.game_name, "Left 4 Dead 2");
        assert_eq!(menu.handlers_dir, "C:\\Games\\NucleusCoop\\handlers");
        assert_eq!(
            menu.text_editor_path,
            "C:\\Program Files\\Notepad++\\notepad++.exe"
        );
        assert!(menu.is_language_picker_open);
        assert!(menu.on_close.is_some());
        assert!(menu.on_open_handler.is_some());
        assert!(menu.on_open_exe.is_some());
        assert!(menu.on_open_userprofile.is_some());
        assert!(menu.on_delete_userprofile.is_some());
        assert!(menu.on_toggle_assign_by_index.is_some());
        assert!(menu.on_toggle_assign_on_press.is_some());
        assert!(menu.on_toggle_keep_content.is_some());
        assert!(menu.on_toggle_disable_profile.is_some());
        assert!(menu.on_toggle_disable_update.is_some());
        assert!(menu.on_change_icon.is_some());
        assert!(menu.on_open_assets.is_some());
        assert!(menu.on_toggle_language_picker.is_some());
        assert!(menu.on_select_language.is_some());
        assert!(menu.on_create_desktop_shortcut.is_some());
        assert!(menu.on_copy_launch_cmd.is_some());
        assert!(menu.on_remove_game.is_some());

        // Verify conversion into AnyElement
        let _elem = menu.into_any_element();
    }

    #[test]
    fn test_couch_context_menu_defaults() {
        let handler = HandlerItem::new(
            "test-guid-portal2",
            "Portal 2",
            "portal2.exe",
            2,
            "Portal 2 Handler",
        );

        let menu = CouchContextMenu::new(handler);

        assert_eq!(menu.handler.game_name, "Portal 2");
        assert_eq!(menu.handlers_dir, "");
        assert_eq!(menu.text_editor_path, "");
        assert!(!menu.is_language_picker_open);
        assert!(menu.on_close.is_none());
        assert!(menu.on_open_handler.is_none());
        assert!(menu.on_open_exe.is_none());
        assert!(menu.on_open_userprofile.is_none());
        assert!(menu.on_delete_userprofile.is_none());
        assert!(menu.on_toggle_assign_by_index.is_none());
        assert!(menu.on_toggle_assign_on_press.is_none());
        assert!(menu.on_toggle_keep_content.is_none());
        assert!(menu.on_toggle_disable_profile.is_none());
        assert!(menu.on_toggle_disable_update.is_none());
        assert!(menu.on_change_icon.is_none());
        assert!(menu.on_open_assets.is_none());
        assert!(menu.on_toggle_language_picker.is_none());
        assert!(menu.on_select_language.is_none());
        assert!(menu.on_create_desktop_shortcut.is_none());
        assert!(menu.on_copy_launch_cmd.is_none());
        assert!(menu.on_remove_game.is_none());

        // Verify conversion into AnyElement for default state
        let _elem = menu.into_any_element();
    }
}
