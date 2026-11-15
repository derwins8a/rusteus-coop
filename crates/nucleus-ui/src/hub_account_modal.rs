//! Splitscreen.Me Community Hub Account authentication and session modal.

use std::sync::Arc;
use gpui::prelude::*;
use gpui::*;
use gpui_component::Icon;
use gpui_kit_assets::IconName;
use nucleus_profile::hub_account::HubAccountSummary;
use crate::components::badge::{CouchTag, TagTone};
use crate::components::button::{CouchButton, CouchButtonSize};
use crate::components::modal::{CouchModal, ModalWidth};
use crate::theme;

/// State for the Hub account login dialog form.
#[derive(Debug, Clone, Default)]
pub struct HubAccountModalState {
    pub is_open: bool,
    pub username_input: String,
    pub password_input: String,
    pub remember_credentials: bool,
    pub is_authenticating: bool,
    pub error_message: Option<String>,
}

impl HubAccountModalState {
    pub fn new() -> Self {
        Self {
            remember_credentials: true,
            ..Default::default()
        }
    }

    pub fn open(&mut self) {
        self.is_open = true;
        self.error_message = None;
    }

    pub fn close(&mut self) {
        self.is_open = false;
        self.password_input.clear();
        self.error_message = None;
    }
}

/// Renders the Community Hub account management and authentication modal.
pub fn render_hub_account_modal(
    state: &HubAccountModalState,
    account: Option<&HubAccountSummary>,
    on_login: Option<Arc<dyn Fn(&str, &str, bool) + Send + Sync>>,
    on_logout: Option<Arc<dyn Fn() + Send + Sync>>,
    on_close: Arc<dyn Fn(&ClickEvent, &mut Window, &mut App) + 'static>,
) -> AnyElement {
    if !state.is_open {
        return div().into_any_element();
    }

    let is_logged_in = account.is_some();
    let title = if is_logged_in {
        "Splitscreen.Me Hub Account"
    } else {
        "Sign In to Splitscreen.Me Hub"
    };

    let modal_close = on_close.clone();
    let mut modal = CouchModal::new("modal-hub-account", title)
        .icon(IconName::Globe)
        .width(ModalWidth::Md)
        .on_close(move |ev, win, app| modal_close(ev, win, app));

    if is_logged_in {
        modal = modal.tone(TagTone::Success);
    }

    let body = if let Some(acct) = account {
        // Authenticated profile view
        div()
            .flex()
            .flex_col()
            .gap_4()
            .p_4()
            .child(
                div()
                    .flex()
                    .items_center()
                    .justify_between()
                    .p_3()
                    .rounded_md()
                    .bg(theme::bg_panel())
                    .border_1()
                    .border_color(theme::border_color())
                    .child(
                        div()
                            .flex()
                            .flex_col()
                            .gap_1()
                            .child(
                                div()
                                    .text_base()
                                    .font_weight(FontWeight::BOLD)
                                    .text_color(theme::text_primary())
                                    .child(acct.username.clone()),
                            )
                            .child(
                                div()
                                    .text_xs()
                                    .text_color(theme::text_muted())
                                    .child(format!("User ID: {}", acct.user_id)),
                            )
                            .child(
                                div()
                                    .text_xs()
                                    .text_color(theme::text_muted())
                                    .child(format!("Hub URL: {}", acct.hub_base_url)),
                            ),
                    )
                    .child(
                        CouchTag::new("Authenticated")
                            .tone(TagTone::Success)
                            .icon(IconName::CircleCheck),
                    ),
            )
            .child(
                div()
                    .text_xs()
                    .text_color(theme::text_muted())
                    .child("Session credentials are securely stored locally using Windows DPAPI."),
            )
            .child(
                div()
                    .flex()
                    .justify_end()
                    .gap_2()
                    .pt_2()
                    .child(
                        CouchButton::danger("btn-hub-logout", "Sign Out")
                            .size(CouchButtonSize::Sm)
                            .icon(IconName::LogOut)
                            .on_click({
                                let logout_cb = on_logout.clone();
                                let close_cb = on_close.clone();
                                move |ev, win, app| {
                                    if let Some(ref cb) = logout_cb {
                                        cb();
                                    }
                                    close_cb(ev, win, app);
                                }
                            }),
                    ),
            )
    } else {
        // Login form view
        let username_val = state.username_input.clone();
        let password_val = state.password_input.clone();
        let remember_val = state.remember_credentials;
        let login_cb = on_login.clone();
        let close_cb = on_close.clone();

        div()
            .flex()
            .flex_col()
            .gap_4()
            .p_4()
            .child(
                div()
                    .text_xs()
                    .text_color(theme::text_muted())
                    .child("Sign in with your Splitscreen.Me credentials to upload handlers, rate community configs, and submit reports."),
            )
            .when_some(state.error_message.as_ref(), |this, err| {
                this.child(
                    div()
                        .flex()
                        .items_center()
                        .gap_2()
                        .p_2p5()
                        .rounded_md()
                        .bg(theme::danger_subtle())
                        .border_1()
                        .border_color(theme::danger())
                        .text_xs()
                        .text_color(theme::danger())
                        .child(Icon::new(IconName::TriangleAlert).size_4())
                        .child(err.clone()),
                )
            })
            .child(
                div()
                    .flex()
                    .flex_col()
                    .gap_1()
                    .child(
                        div()
                            .text_xs()
                            .font_weight(FontWeight::SEMIBOLD)
                            .text_color(theme::text_muted())
                            .child("Username or Email"),
                    )
                    .child(
                        div()
                            .id("input-hub-username")
                            .flex()
                            .items_center()
                            .h(px(32.0))
                            .px_3()
                            .rounded_md()
                            .bg(theme::bg_panel())
                            .border_1()
                            .border_color(theme::border_color())
                            .text_sm()
                            .text_color(if username_val.is_empty() {
                                theme::text_muted()
                            } else {
                                theme::text_primary()
                            })
                            .child(if username_val.is_empty() {
                                "Enter username or email...".to_string()
                            } else {
                                username_val.clone()
                            }),
                    ),
            )
            .child(
                div()
                    .flex()
                    .flex_col()
                    .gap_1()
                    .child(
                        div()
                            .text_xs()
                            .font_weight(FontWeight::SEMIBOLD)
                            .text_color(theme::text_muted())
                            .child("Password"),
                    )
                    .child(
                        div()
                            .id("input-hub-password")
                            .flex()
                            .items_center()
                            .h(px(32.0))
                            .px_3()
                            .rounded_md()
                            .bg(theme::bg_panel())
                            .border_1()
                            .border_color(theme::border_color())
                            .text_sm()
                            .text_color(if password_val.is_empty() {
                                theme::text_muted()
                            } else {
                                theme::text_primary()
                            })
                            .child(if password_val.is_empty() {
                                "••••••••".to_string()
                            } else {
                                "•".repeat(password_val.len().min(16))
                            }),
                    ),
            )
            .child(
                div()
                    .flex()
                    .items_center()
                    .gap_2()
                    .child(
                        Icon::new(if remember_val {
                            IconName::CircleCheck
                        } else {
                            IconName::CircleAlert
                        })
                        .size_4()
                        .text_color(if remember_val {
                            theme::accent()
                        } else {
                            theme::text_muted()
                        }),
                    )
                    .child(
                        div()
                            .text_xs()
                            .text_color(theme::text_muted())
                            .child("Remember credentials securely on this machine (Windows DPAPI)"),
                    ),
            )
            .child(
                div()
                    .flex()
                    .justify_end()
                    .gap_2()
                    .pt_2()
                    .child(
                        CouchButton::secondary("btn-hub-cancel", "Cancel")
                            .size(CouchButtonSize::Sm)
                            .on_click(move |ev, win, app| {
                                close_cb(ev, win, app);
                            }),
                    )
                    .child(
                        CouchButton::primary("btn-hub-signin", "Sign In")
                            .size(CouchButtonSize::Sm)
                            .icon(IconName::Check)
                            .on_click(move |ev, win, app| {
                                if let Some(ref cb) = login_cb {
                                    cb(&username_val, &password_val, remember_val);
                                }
                                on_close(ev, win, app);
                            }),
                    ),
            )
    };

    modal.body(body.into_any_element()).into_any_element()
}
