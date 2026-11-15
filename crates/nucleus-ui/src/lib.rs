#![recursion_limit = "4096"]
//! # nucleus-ui
//!
//! Graphical user interface layer for NucleusCoop built with GPUI Kit components.
//!
//! Provides the primary desktop application views: game library list, player setup
//! and monitor splitting screen, community hub browser, settings panel, and dialogs.

pub mod actions;
pub mod big_picture;
pub mod callbacks;
pub mod components;
pub mod debugger_view;
pub mod device_panel;
pub mod dialogs;
pub mod disk_scanner;
pub mod download_dialog;
pub mod extensions_view;
pub mod game_card;
pub mod game_list;
pub mod handler_details;
pub mod http_client;
pub mod hub_account_modal;
pub mod hub_browser;
pub mod hub_details;
pub mod main_view;
pub mod markdown;
pub mod models;
pub mod nefos;
pub mod osd;
pub mod profile_editor;
pub mod save_slots_modal;
pub mod settings_view;
pub mod setup_screen;
pub mod theme;
pub mod window;

pub use actions::UiAction;
pub use callbacks::{EngineToGui, MainWindowCallbacks};
pub use debugger_view::{render_debugger_view, DebuggerViewState};
pub use device_panel::DevicePanelComponent;
pub use disk_scanner::DiskScannerView;
pub use download_dialog::DownloadDialog;
pub use extensions_view::{render_extensions_view, ExtensionItem, ExtensionType, ExtensionsViewState};
pub use game_card::{render_game_card, GameCardComponent};
pub use game_list::GameListView;
pub use handler_details::render_handler_details_modal;
pub use http_client::NucleusHttpClient;
pub use hub_account_modal::{render_hub_account_modal, HubAccountModalState};
pub use hub_browser::HubBrowserView;
pub use hub_details::{render_hub_details_screen, HubDetailTab};
pub use main_view::MainView;
pub use markdown::{render_markdown, render_markdown_with_id};
pub use models::{
    ConfirmAction, ConfirmDialogState, DeviceItem, HandlerItem, HubInstallProgressMsg,
    HubPackageItem, LaunchPromptRequest,
};
pub use osd::{parse_osd_color, render_hotkey_recording_modal, render_osd_overlay};
pub use profile_editor::ProfileEditorView;
pub use save_slots_modal::{render_save_slots_modal, SaveSlotsModalState};
pub use settings_view::{SettingsCategory, SettingsView};
pub use setup_screen::SetupScreenView;
pub use theme::{Color, Theme};
pub use window::{ActiveTab, MainWindow};
pub use gpui_component::Icon;
pub use gpui_kit_assets::IconName;
pub use gpui_component::scroll::ScrollableElement;
pub use gpui_component::tooltip::Tooltip;
pub use components::*;
pub use nefos::*;

/// Creates a tooltip closure for use with `.tooltip(...)`.
pub fn make_tooltip(text: impl Into<gpui::SharedString>) -> impl Fn(&mut gpui::Window, &mut gpui::App) -> gpui::AnyView {
    let s: gpui::SharedString = text.into();
    move |window, cx| gpui_component::tooltip::Tooltip::new(s.clone()).build(window, cx)
}

/// Initializes GPUI Kit components (Theme, GlobalState, Overlays, Tooltips, TextView).
pub fn init(cx: &mut gpui::App) {
    gpui_component::init(cx);
    gpui_component::Theme::change(gpui_component::ThemeMode::Dark, None, cx);
}

/// Helper to construct a GPUI `ImageSource` from either a remote HTTP URL or a local file path.
///
/// Ensures local file paths on Windows (e.g. `C:\Games\NucleusCoop242\gui\covers\image.jpg`)
/// are converted to `PathBuf` so GPUI's element loader reads directly from disk rather than
/// mistaking it for an embedded resource.
pub fn make_image_source(path_or_url: &str) -> gpui::ImageSource {
    if path_or_url.starts_with("http://") || path_or_url.starts_with("https://") {
        path_or_url.to_string().into()
    } else {
        std::path::PathBuf::from(path_or_url).into()
    }
}

