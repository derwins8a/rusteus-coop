//! Extensions, Add-ons & Plugins view managing sandboxed WASM extensions and JS add-ons.

use crate::components::badge::{CouchTag, TagTone};
use crate::components::button::{CouchButton, CouchButtonSize};
use crate::theme;
use gpui::prelude::*;
use gpui::*;
use gpui_kit_assets::IconName;
use std::sync::Arc;

/// Presentation model for an installed extension entry in the UI.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ExtensionItem {
    pub id: String,
    pub name: String,
    pub version: String,
    pub author: String,
    pub description: String,
    pub is_enabled: bool,
    pub extension_type: ExtensionType,
    pub permissions: Vec<(String, bool)>, // (Permission name, is_granted)
}

/// Category/type of extension.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ExtensionType {
    WasmPlugin,
    JsAddon,
    LegacyPlugin,
}

impl ExtensionType {
    pub fn label(self) -> &'static str {
        match self {
            Self::WasmPlugin => "WASM Sandboxed",
            Self::JsAddon => "JS Add-on",
            Self::LegacyPlugin => "Legacy Plugin",
        }
    }

    pub fn tone(self) -> TagTone {
        match self {
            Self::WasmPlugin => TagTone::Accent,
            Self::JsAddon => TagTone::Neutral,
            Self::LegacyPlugin => TagTone::Warning,
        }
    }
}

/// View state for the Extensions management panel.
#[derive(Debug, Clone, Default)]
pub struct ExtensionsViewState {
    pub search_query: String,
    pub selected_filter: Option<ExtensionType>,
    pub status_message: Option<String>,
}

/// Renders the complete Extensions & Add-ons Ecosystem view pane.
pub fn render_extensions_view(
    _state: &ExtensionsViewState,
    extensions: &[ExtensionItem],
    on_toggle_extension: Option<Arc<dyn Fn(&str, bool) + Send + Sync>>,
    on_install_extension: Option<Arc<dyn Fn() + Send + Sync>>,
    on_rescan_extensions: Option<Arc<dyn Fn() + Send + Sync>>,
    on_open_folder: Option<Arc<dyn Fn() + Send + Sync>>,
    on_grant_permission: Option<Arc<dyn Fn(&str, &str) + Send + Sync>>,
) -> AnyElement {
    let install_cb = on_install_extension.clone();
    let rescan_cb = on_rescan_extensions.clone();
    let open_cb = on_open_folder.clone();

    div()
        .flex()
        .flex_col()
        .size_full()
        .p_6()
        .gap_6()
        .bg(theme::bg_main())
        .child(
            // Top action bar
            div()
                .flex()
                .items_center()
                .justify_between()
                .border_b_1()
                .border_color(theme::border_color())
                .pb_4()
                .child(
                    div()
                        .flex()
                        .flex_col()
                        .gap_1()
                        .child(
                            div()
                                .text_xl()
                                .font_weight(FontWeight::BOLD)
                                .text_color(theme::text_primary())
                                .child("Extensions & Add-ons Ecosystem"),
                        )
                        .child(
                            div()
                                .text_xs()
                                .text_color(theme::text_muted())
                                .child("Sandboxed WebAssembly plugins and JavaScript runtime hooks extending Nucleus Co-op capabilities."),
                        ),
                )
                .child(
                    div()
                        .flex()
                        .items_center()
                        .gap_2()
                        .child(
                            CouchButton::secondary("btn-open-ext-dir", "Open Folder")
                                .size(CouchButtonSize::Sm)
                                .icon(IconName::FolderOpen)
                                .on_click(move |_ev, _win, _app| {
                                    if let Some(ref cb) = open_cb {
                                        cb();
                                    }
                                }),
                        )
                        .child(
                            CouchButton::secondary("btn-rescan-ext", "Rescan")
                                .size(CouchButtonSize::Sm)
                                .icon(IconName::SlidersHorizontal)
                                .on_click(move |_ev, _win, _app| {
                                    if let Some(ref cb) = rescan_cb {
                                        cb();
                                    }
                                }),
                        )
                        .child(
                            CouchButton::primary("btn-install-ext", "Install Extension")
                                .size(CouchButtonSize::Sm)
                                .icon(IconName::Plus)
                                .on_click(move |_ev, _win, _app| {
                                    if let Some(ref cb) = install_cb {
                                        cb();
                                    }
                                }),
                        ),
                ),
        )
        .child(
            // Extensions list
            div()
                .id("extensions-list-scroll")
                .flex()
                .flex_col()
                .gap_3()
                .overflow_y_scroll()
                .children(if extensions.is_empty() {
                    vec![div()
                        .flex()
                        .flex_col()
                        .items_center()
                        .justify_center()
                        .p_12()
                        .rounded_lg()
                        .bg(theme::bg_panel())
                        .border_1()
                        .border_color(theme::border_color())
                        .gap_2()
                        .child(
                            div()
                                .text_base()
                                .font_weight(FontWeight::SEMIBOLD)
                                .text_color(theme::text_primary())
                                .child("No Extensions Installed"),
                        )
                        .child(
                            div()
                                .text_xs()
                                .text_color(theme::text_muted())
                                .child("Install .nucplugin or .nucaddon packages to add custom hooks, emulators, or tools."),
                        )
                        .into_any_element()]
                } else {
                    extensions
                        .iter()
                        .map(|ext| {
                            let ext_id = ext.id.clone();
                            let is_enabled = ext.is_enabled;
                            let toggle_cb = on_toggle_extension.clone();
                            let grant_cb = on_grant_permission.clone();

                            div()
                                .id(ElementId::Name(format!("ext-card-{}", ext.id).into()))
                                .flex()
                                .flex_col()
                                .gap_3()
                                .p_4()
                                .rounded_lg()
                                .bg(theme::bg_panel())
                                .border_1()
                                .border_color(if is_enabled {
                                    theme::border_color()
                                } else {
                                    theme::border_subtle()
                                })
                                .child(
                                    div()
                                        .flex()
                                        .items_center()
                                        .justify_between()
                                        .child(
                                            div()
                                                .flex()
                                                .items_center()
                                                .gap_3()
                                                .child(
                                                    div()
                                                        .text_base()
                                                        .font_weight(FontWeight::BOLD)
                                                        .text_color(theme::text_primary())
                                                        .child(ext.name.clone()),
                                                )
                                                .child(
                                                    div()
                                                        .text_xs()
                                                        .text_color(theme::text_muted())
                                                        .child(format!("v{}", ext.version)),
                                                )
                                                .child(
                                                    CouchTag::new(ext.extension_type.label())
                                                        .tone(ext.extension_type.tone()),
                                                ),
                                        )
                                        .child(
                                            (if is_enabled {
                                                CouchButton::primary(
                                                    ElementId::Name(format!("btn-toggle-ext-{}", ext.id).into()),
                                                    "Enabled",
                                                )
                                            } else {
                                                CouchButton::secondary(
                                                    ElementId::Name(format!("btn-toggle-ext-{}", ext.id).into()),
                                                    "Disabled",
                                                )
                                            })
                                            .size(CouchButtonSize::Sm)
                                            .icon(if is_enabled {
                                                IconName::Check
                                            } else {
                                                IconName::X
                                            })
                                            .on_click({
                                                let id_str = ext_id.clone();
                                                move |_ev, _win, _app| {
                                                    if let Some(ref cb) = toggle_cb {
                                                        cb(&id_str, !is_enabled);
                                                    }
                                                }
                                            }),
                                        ),
                                )
                                .child(
                                    div()
                                        .text_xs()
                                        .text_color(theme::text_muted())
                                        .child(ext.description.clone()),
                                )
                                .when(!ext.permissions.is_empty(), |this| {
                                    this.child(
                                        div()
                                            .flex()
                                            .items_center()
                                            .gap_2()
                                            .pt_1()
                                            .child(
                                                div()
                                                    .text_xs()
                                                    .font_weight(FontWeight::SEMIBOLD)
                                                    .text_color(theme::text_muted())
                                                    .child("Permissions:"),
                                            )
                                            .children(ext.permissions.iter().map(|(perm_name, granted)| {
                                                let p_name = perm_name.clone();
                                                let e_id = ext_id.clone();
                                                let g_cb = grant_cb.clone();
                                                let is_granted = *granted;

                                                div()
                                                    .id(ElementId::Name(format!("perm-{}-{}", ext_id, perm_name).into()))
                                                    .cursor_pointer()
                                                    .on_click(move |_ev, _win, _app| {
                                                        if !is_granted {
                                                            if let Some(ref cb) = g_cb {
                                                                cb(&e_id, &p_name);
                                                            }
                                                        }
                                                    })
                                                    .child(
                                                        CouchTag::new(format!(
                                                            "{}{}",
                                                            perm_name,
                                                            if is_granted { "" } else { " (Click to Grant)" }
                                                        ))
                                                        .tone(if is_granted {
                                                            TagTone::Success
                                                        } else {
                                                            TagTone::Warning
                                                        })
                                                        .icon(if is_granted {
                                                            IconName::CircleCheck
                                                        } else {
                                                            IconName::CircleAlert
                                                        }),
                                                    )
                                                    .into_any_element()
                                            })),
                                    )
                                })
                                .into_any_element()
                        })
                        .collect()
                }),
        )
        .into_any_element()
}
