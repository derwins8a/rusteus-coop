//! CouchStatusBar meta-component.
//!
//! Provides a standardized bottom status bar showing system status,
//! session state, connected gamepad counts, and runtime platform.

use gpui::prelude::FluentBuilder;
use gpui::{
    div, px, AnyElement, IntoElement, ParentElement, Styled,
};

use crate::theme;

/// A standardized bottom status bar for 10-foot Couch and desktop views.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CouchStatusBar {
    pub status_message: String,
    pub detected_devices: usize,
    pub session_active: bool,
}

impl Default for CouchStatusBar {
    fn default() -> Self {
        Self::new()
    }
}

impl CouchStatusBar {
    /// Creates a new status bar with default initial values.
    pub fn new() -> Self {
        Self {
            status_message: "Ready".to_string(),
            detected_devices: 0,
            session_active: false,
        }
    }

    /// Sets the status message displayed on the left.
    pub fn message(mut self, msg: impl Into<String>) -> Self {
        self.status_message = msg.into();
        self
    }

    /// Sets the number of detected gamepad devices.
    pub fn detected_devices(mut self, count: usize) -> Self {
        self.detected_devices = count;
        self
    }

    /// Sets whether a game session is actively running.
    pub fn session_active(mut self, active: bool) -> Self {
        self.session_active = active;
        self
    }

    /// Flattens and renders the component as an [`AnyElement`].
    pub fn into_any_element(self) -> AnyElement {
        let session_label = if self.session_active {
            "Session: Active"
        } else {
            "Session: Idle"
        };

        div()
            .flex()
            .flex_row()
            .items_center()
            .justify_between()
            .h(px(32.0))
            .px_4()
            .gap_4()
            .bg(theme::bg_panel())
            .border_t_1()
            .border_color(theme::border_color())
            .text_xs()
            .child(
                div()
                    .flex()
                    .items_center()
                    .gap_2()
                    .min_w_0()
                    .overflow_hidden()
                    .child(
                        div()
                            .flex_shrink_0()
                            .size(px(6.0))
                            .rounded_full()
                            .bg(theme::accent()),
                    )
                    .child(
                        div()
                            .text_color(theme::text_primary())
                            .truncate()
                            .overflow_hidden()
                            .child(self.status_message),
                    ),
            )
            .child(
                div()
                    .flex()
                    .flex_shrink_0()
                    .items_center()
                    .gap_4()
                    .text_color(theme::text_muted())
                    .child(
                        div()
                            .child(session_label)
                            .when(self.session_active, |d| d.text_color(theme::success())),
                    )
                    .child(div().child("|"))
                    .child(div().child(format!("Gamepads: {}", self.detected_devices)))
                    .child(div().child("|"))
                    .child(div().child("Windows Native (x64)")),
            )
            .into_any_element()
    }
}

impl IntoElement for CouchStatusBar {
    type Element = AnyElement;

    fn into_element(self) -> Self::Element {
        self.into_any_element()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_couch_status_bar_construction() {
        let bar = CouchStatusBar::new()
            .message("Testing status")
            .detected_devices(4)
            .session_active(true);

        assert_eq!(bar.status_message, "Testing status");
        assert_eq!(bar.detected_devices, 4);
        assert!(bar.session_active);
    }

    #[test]
    fn test_couch_status_bar_defaults() {
        let bar = CouchStatusBar::default();
        assert_eq!(bar.status_message, "Ready");
        assert_eq!(bar.detected_devices, 0);
        assert!(!bar.session_active);
    }
}
