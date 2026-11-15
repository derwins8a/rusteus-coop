//! # CouchConfirmDialog Component
//!
//! Standardized confirmation modal dialog for destructive or major actions in Nucleus Co-op
//! (e.g. resetting settings to defaults, removing games from library, deleting profiles,
//! or deleting save paths).
//!
//! ## Invariants & Guarantees:
//! 1. **Backdrop Scrim**: Full-screen overlay absorbing clicks, triggering `on_cancel` when clicking outside.
//! 2. **Event Shielding**: Dialog panel intercepts and stops click propagation to prevent pass-through
//!    to underlying game cards or controls.
//! 3. **Ergonomic Sizing**: Bounded panel width (max 520px) preventing visual distortion across resolutions.
//! 4. **Visual Hierarchy**: Warning iconography (`IconName::TriangleAlert`) and danger buttons (`CouchButton::danger`) for destructive
//!    actions; information iconography (`IconName::Info`) and primary buttons (`CouchButton::primary`) for safe actions.
//! 5. **Type Boundary Flattening**: Implements `into_any_element()` returning `AnyElement` to guarantee bounded
//!    generic types and fast build times.

use std::rc::Rc;

use gpui::{
    AnyElement, App, ClickEvent, FontWeight, InteractiveElement, IntoElement, MouseButton,
    ParentElement, StatefulInteractiveElement, Styled, Window, div, px,
};

use crate::components::button::CouchButton;
use crate::models::ConfirmDialogState;
use crate::theme;
use crate::{Icon, IconName};

/// A standardized confirmation dialog component engineered for Nucleus Co-op.
///
/// Encapsulates modal backdrop scrim, click pass-through prevention, destructive/info visual styling,
/// and standardized footer action buttons.
pub struct CouchConfirmDialog {
    /// State representation for an active user confirmation dialog.
    pub state: ConfirmDialogState,
    on_confirm: Option<Box<dyn Fn(&ClickEvent, &mut Window, &mut App) + 'static>>,
    on_cancel: Option<Box<dyn Fn(&ClickEvent, &mut Window, &mut App) + 'static>>,
}

impl CouchConfirmDialog {
    /// Creates a new `CouchConfirmDialog` with the given confirmation dialog state.
    pub fn new(state: ConfirmDialogState) -> Self {
        Self {
            state,
            on_confirm: None,
            on_cancel: None,
        }
    }

    /// Registers a callback triggered when the confirmation button is clicked.
    pub fn on_confirm<F>(mut self, handler: F) -> Self
    where
        F: Fn(&ClickEvent, &mut Window, &mut App) + 'static,
    {
        self.on_confirm = Some(Box::new(handler));
        self
    }

    /// Registers a callback triggered when the cancel button or backdrop overlay is clicked.
    pub fn on_cancel<F>(mut self, handler: F) -> Self
    where
        F: Fn(&ClickEvent, &mut Window, &mut App) + 'static,
    {
        self.on_cancel = Some(Box::new(handler));
        self
    }

    /// Flattens the confirmation modal and returns an `AnyElement` to guarantee bounded generic types and fast builds.
    pub fn into_any_element(self) -> AnyElement {
        let is_destructive = self.state.is_destructive;
        let title = self.state.title;
        let message = self.state.message;
        let confirm_label = self.state.confirm_label;

        // Wrap on_cancel into Rc so it can be invoked by both the backdrop and the cancel button.
        let on_cancel_shared: Option<Rc<dyn Fn(&ClickEvent, &mut Window, &mut App) + 'static>> =
            self.on_cancel.map(|cb| {
                let rc: Rc<dyn Fn(&ClickEvent, &mut Window, &mut App) + 'static> =
                    Rc::new(move |ev, win, app| cb(ev, win, app));
                rc
            });

        // 1. Full-screen backdrop overlay
        let mut backdrop = div()
            .id("confirm-dialog-backdrop")
            .absolute()
            .inset_0()
            .bg(theme::overlay_scrim())
            .flex()
            .items_center()
            .justify_center();

        if let Some(ref cb) = on_cancel_shared {
            let cb = cb.clone();
            backdrop = backdrop.on_click(move |ev, win, app| {
                cb(ev, win, app);
            });
        }

        // 2. Dialog card panel with click pass-through prevention
        let panel = div()
            .id("confirm-dialog-panel")
            .flex()
            .flex_col()
            .w(px(460.0))
            .max_w(px(520.0))
            .p_5()
            .bg(theme::bg_panel())
            .border_1()
            .border_color(theme::border_focus())
            .rounded_xl()
            .shadow_lg()
            .gap_4()
            .on_click(|_, _, cx| {
                cx.stop_propagation();
            })
            .on_mouse_down(MouseButton::Left, |_, _, cx| {
                cx.stop_propagation();
            });

        // 3. Header row with icon, title, and message
        let header = div()
            .flex()
            .items_start()
            .gap_3()
            .child(
                div().pt_0p5().child(
                    Icon::new(if is_destructive {
                        IconName::TriangleAlert
                    } else {
                        IconName::Info
                    })
                    .size_5()
                    .text_color(if is_destructive {
                        theme::warning()
                    } else {
                        theme::accent()
                    }),
                ),
            )
            .child(
                div()
                    .flex()
                    .flex_col()
                    .gap_1p5()
                    .flex_1()
                    .min_w_0()
                    .child(
                        div()
                            .text_sm()
                            .font_weight(FontWeight::BOLD)
                            .text_color(theme::text_primary())
                            .child(title),
                    )
                    .child(
                        div()
                            .text_xs()
                            .text_color(theme::text_muted())
                            .max_w(px(380.0))
                            .child(message),
                    ),
            );

        // 4. Footer buttons
        let mut cancel_btn = CouchButton::secondary("btn-confirm-cancel", "Cancel");
        if let Some(cb) = on_cancel_shared {
            cancel_btn = cancel_btn.on_click(move |ev, win, app| {
                cb(ev, win, app);
            });
        }

        let mut confirm_btn = if is_destructive {
            CouchButton::danger("btn-confirm-accept", confirm_label)
        } else {
            CouchButton::primary("btn-confirm-accept", confirm_label)
        };
        if let Some(cb) = self.on_confirm {
            confirm_btn = confirm_btn.on_click(move |ev, win, app| {
                cb(ev, win, app);
            });
        }

        let footer = div()
            .flex()
            .items_center()
            .justify_end()
            .gap_2p5()
            .child(cancel_btn)
            .child(confirm_btn);

        // Assemble full modal
        backdrop
            .child(panel.child(header).child(footer))
            .into_any_element()
    }
}

impl IntoElement for CouchConfirmDialog {
    type Element = AnyElement;

    fn into_element(self) -> Self::Element {
        self.into_any_element()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::models::{ConfirmAction, ConfirmDialogState};

    #[test]
    fn test_couch_confirm_dialog_destructive_construction() {
        let state = ConfirmDialogState {
            title: "Reset Preferences".to_string(),
            message: "Are you sure you want to restore default preferences?".to_string(),
            confirm_label: "Reset".to_string(),
            is_destructive: true,
            action: ConfirmAction::ResetDefaults,
        };

        let dialog = CouchConfirmDialog::new(state)
            .on_confirm(|_, _, _| {})
            .on_cancel(|_, _, _| {});

        assert!(dialog.state.is_destructive);
        assert_eq!(dialog.state.title, "Reset Preferences");
        assert_eq!(dialog.state.confirm_label, "Reset");

        let _elem = dialog.into_any_element();
    }

    #[test]
    fn test_couch_confirm_dialog_info_construction() {
        let state = ConfirmDialogState {
            title: "Notice".to_string(),
            message: "An informational confirmation prompt.".to_string(),
            confirm_label: "Proceed".to_string(),
            is_destructive: false,
            action: ConfirmAction::ResetDefaults,
        };

        let dialog = CouchConfirmDialog::new(state);
        assert!(!dialog.state.is_destructive);
        assert_eq!(dialog.state.confirm_label, "Proceed");

        let _elem = dialog.into_any_element();
    }
}
