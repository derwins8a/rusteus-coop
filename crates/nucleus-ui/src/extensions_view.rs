//! Extensions, Add-ons & Plugins view managing sandboxed WASM extensions and JS add-ons.

use crate::components::badge::{CouchTag, TagTone};
use crate::components::button::{CouchButton, CouchButtonSize};
use crate::theme;
use gpui::prelude::*;
use gpui::*;
use gpui_kit_assets::IconName;

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
    this: &crate::window::MainWindow,
    cx: &mut Context<crate::window::MainWindow>,
) -> AnyElement {
    let extensions = &this.installed_extensions;

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
                                .on_click(cx.listener(|_this, _, _, _cx| {
                                    let dir_to_open = if let Some(globals) = nucleus_common::Globals::try_get() {
                                        globals.app_data_path.join("addons")
                                    } else if let Ok(app_data) = std::env::var("APPDATA") {
                                        std::path::PathBuf::from(app_data).join("NucleusCoop").join("addons")
                                    } else {
                                        std::path::PathBuf::from("addons")
                                    };
                                    let _ = std::fs::create_dir_all(&dir_to_open);
                                    let _ = std::process::Command::new("explorer")
                                        .arg(dir_to_open)
                                        .spawn();
                                })),
                        )
                        .child(
                            CouchButton::secondary("btn-rescan-ext", "Rescan")
                                .size(CouchButtonSize::Sm)
                                .icon(IconName::SlidersHorizontal)
                                .on_click(cx.listener(|this, _, _, cx| {
                                    this.refresh_extensions();
                                    cx.notify();
                                })),
                        )
                        .child(
                            CouchButton::primary("btn-install-ext", "Install Extension")
                                .size(CouchButtonSize::Sm)
                                .icon(IconName::Plus)
                                .on_click(cx.listener(|this, _, _, cx| {
                                    this.install_extension_dialog(cx);
                                })),
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
                                            div()
                                                .flex()
                                                .items_center()
                                                .gap_2()
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
                                                    .on_click(cx.listener({
                                                        let id_str = ext.id.clone();
                                                        let next_enabled = !is_enabled;
                                                        move |this, _, _, cx| {
                                                            if let Some(ref cb) = this.callbacks.on_toggle_extension {
                                                                cb(&id_str, next_enabled);
                                                            }
                                                            if let Some(item) = this.installed_extensions.iter_mut().find(|e| e.id == id_str) {
                                                                item.is_enabled = next_enabled;
                                                            }
                                                            cx.notify();
                                                        }
                                                    })),
                                                )
                                                .child(
                                                    CouchButton::danger(
                                                        ElementId::Name(format!("btn-uninstall-ext-{}", ext.id).into()),
                                                        "Uninstall",
                                                    )
                                                    .size(CouchButtonSize::Sm)
                                                    .icon(IconName::Trash)
                                                    .on_click(cx.listener({
                                                        let id_str = ext.id.clone();
                                                        move |this, _, _, cx| {
                                                            if let Some(ref cb) = this.callbacks.on_uninstall_extension {
                                                                let _ = cb(&id_str);
                                                            }
                                                            this.installed_extensions.retain(|e| e.id != id_str);
                                                            this.status_message = format!("Extension uninstalled: {}", id_str);
                                                            this.show_osd("Extension Uninstalled", 1500, cx);
                                                            cx.notify();
                                                        }
                                                    })),
                                                ),
                                        ),
                                )
                                .child(
                                    div()
                                        .text_xs()
                                        .text_color(theme::text_muted())
                                        .child(ext.description.clone()),
                                )
                                .when(!ext.permissions.is_empty(), |this_card| {
                                    this_card.child(
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
                                                let is_granted = *granted;

                                                div()
                                                    .id(ElementId::Name(format!("perm-{}-{}", ext_id, perm_name).into()))
                                                    .cursor_pointer()
                                                    .on_click(cx.listener({
                                                        let e_id = e_id.clone();
                                                        let p_name = p_name.clone();
                                                        move |this, _, _, cx| {
                                                            if !is_granted {
                                                                if let Some(ref cb) = this.callbacks.on_grant_extension_permission {
                                                                    cb(&e_id, &p_name);
                                                                }
                                                                this.refresh_extensions();
                                                                cx.notify();
                                                            }
                                                        }
                                                    }))
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
