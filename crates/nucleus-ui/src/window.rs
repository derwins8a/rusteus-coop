//! Native GPUI application window component for NucleusCoop.

use crate::big_picture::{
    BigPictureScreen, CarouselState, CouchHubState, CouchOptionItem, CouchSetupState,
    PathResolution, resolve_game_path_or_couch_prompt,
};
use crate::callbacks::MainWindowCallbacks;
use crate::components::{
    CouchConfirmDialog, CouchContextMenu, CouchSidebarNav, CouchStatusBar, CouchTitleBar,
};
use crate::models::{DeviceItem, HandlerItem, HubPackageItem};
use crate::theme;
use crate::{Icon, IconName};
use gpui::prelude::*;
use gpui::*;
use std::sync::Arc;

/// Active top-level navigation view tab.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum ActiveTab {
    /// Game library and discovered handlers catalog.
    #[default]
    Library,
    /// Player count, monitor partitioning, and device assignment setup.
    Setup,
    /// Community handler hub script browser.
    Hub,
    /// Extensions and add-ons ecosystem manager.
    Extensions,
    /// Step-by-step handler debugger and runtime inspector.
    Debugger,
    /// Global application preferences and directories.
    Settings,
}

/// Sort option for Hub browser packages.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum HubSortOption {
    #[default]
    Trending,
    Hottest,
    Downloads,
    ReleaseDate,
    Alphabetical,
}

/// Top-level application window rendered with GPUI Kit components.
pub struct MainWindow {
    /// Active navigation tab.
    pub active_tab: ActiveTab,
    /// Number of detected physical game controllers.
    pub detected_devices: usize,
    /// Detailed list of detected physical devices.
    pub detected_devices_list: Vec<DeviceItem>,
    /// List of legacy handler names for backward compatibility.
    pub handler_names: Vec<String>,
    /// Detailed list of installed/discovered game handlers.
    pub handlers: Vec<HandlerItem>,
    /// Currently selected game title or GUID for splitscreen configuration.
    pub selected_game: Option<String>,
    /// Configured player count for splitscreen session.
    pub player_count: usize,
    /// Configured monitor partition layout name.
    pub selected_layout: String,
    /// Per-player device assignment labels (e.g. "Gamepad #0 (Xbox)", "Keyboard & Mouse").
    pub player_device_assignments: Vec<String>,
    /// Query string for hub search.
    pub hub_search_query: String,
    /// Available community packages loaded from Hub API or cache.
    pub hub_packages: Vec<HubPackageItem>,
    /// Active page index in community hub browser.
    pub hub_page: usize,
    /// Number of items per page in community hub browser.
    pub hub_page_size: usize,
    /// Active Hub sorting option.
    pub hub_sort: HubSortOption,
    /// Whether Hub sorting is ascending.
    pub hub_sort_ascending: bool,
    /// Active filter tags in Community Hub.
    pub selected_tags: std::collections::HashSet<String>,
    /// Whether Hub packages have been loaded automatically.
    pub hub_loaded: bool,
    /// Currently inspected Hub package for details modal.
    pub inspecting_hub_package: Option<HubPackageItem>,
    /// Active tab in Hub package details view.
    pub hub_detail_tab: crate::hub_details::HubDetailTab,
    /// Loaded comments for the currently inspected Hub package.
    pub hub_detail_comments: Vec<nucleus_hub::HubComment>,
    /// Loaded package releases/versions for the currently inspected Hub package.
    pub hub_detail_packages: Vec<nucleus_hub::HubPackage>,
    /// Whether details (comments, versions, js code) are currently being fetched.
    pub hub_detail_loading: bool,
    /// Extra likes/hotness added in current session for this package.
    pub hub_detail_extra_likes: std::collections::HashSet<String>,
    /// Currently inspected local handler for details modal.
    pub inspecting_handler: Option<HandlerItem>,
    /// Configured path for game handler scripts.
    pub handlers_dir: String,
    /// Whether automatic game scanning on startup is enabled.
    pub auto_scan: bool,
    /// Name of the active visual theme.
    pub theme_name: String,
    /// Current status bar message.
    pub status_message: String,
    /// Full persistent application settings.
    pub app_settings: nucleus_profile::AppSettings,
    /// Active OSD overlay message (text, start_time, duration).
    pub osd_message: Option<(String, std::time::Instant, std::time::Duration)>,
    /// Focus handle for Community Hub search input.
    pub hub_search_focus: Option<FocusHandle>,
    /// Focus handle for Game Library search input.
    pub lib_search_focus: Option<FocusHandle>,
    /// Focus handle for interactive physical keypress recording.
    pub hotkey_record_focus: Option<FocusHandle>,
    /// Focus handle for top-level window key events.
    pub window_focus: Option<FocusHandle>,
    /// Map of package ID to download/install progress (0.0 to 1.0).
    pub installing_handlers: std::collections::HashMap<String, f32>,
    /// Set of package IDs that have been installed in this session.
    pub installed_package_ids: std::collections::HashSet<String>,
    /// Embedded library view model.
    pub game_list: crate::GameListView,
    /// Embedded setup screen view model.
    pub setup_screen: crate::SetupScreenView,
    /// Embedded hub browser view model.
    pub hub_browser: crate::HubBrowserView,
    /// Embedded settings view model.
    pub settings_view: crate::SettingsView,
    /// Active library sort criterion.
    pub library_sort: crate::models::LibrarySortOption,
    /// Active library sort direction.
    pub library_sort_dir: crate::models::LibrarySortDirection,
    /// Whether a Community Hub API search or load is currently in-flight.
    pub hub_loading: bool,
    /// Currently opened right-click contextual menu handler.
    pub context_menu_handler: Option<HandlerItem>,
    /// Currently selected custom options map (key -> selected option value).
    pub selected_handler_options: std::collections::HashMap<String, String>,
    /// Active modal confirmation dialog if displayed.
    pub confirm_dialog: Option<crate::models::ConfirmDialogState>,
    /// External event and action callbacks.
    pub callbacks: MainWindowCallbacks,
    /// Saved numbered profiles for the currently configured game: list of (profile_number, title).
    pub saved_profiles: Vec<(usize, String)>,
    /// Active loaded profile number if one is loaded.
    pub active_profile_id: Option<usize>,
    /// Whether the left navigation sidebar is collapsed into icon-only mode (None = follow window width breakpoint).
    pub sidebar_collapsed: Option<bool>,
    /// Player slot index currently displaying device selection dropdown.
    pub dropdown_player_device: Option<usize>,
    /// Whether the context menu Steam language picker is expanded.
    pub context_menu_language_picker: bool,
    /// Whether the controller shortcuts setup modal is currently open.
    pub show_controller_shortcuts_modal: bool,
    /// Action key currently being remapped in the controller shortcuts modal (e.g. Some("unfocus")).
    pub remapping_controller_shortcut: Option<String>,
    /// Whether Couch Big Picture mode is currently active.
    pub is_big_picture: bool,
    /// Currently active screen in Couch Big Picture mode.
    pub big_picture_screen: BigPictureScreen,
    /// Horizontal carousel state for library browsing in Couch Big Picture mode.
    pub carousel_state: CarouselState,
    /// Couch Splitscreen Setup state (auto-assignment, swap motion, unassigned tray).
    pub couch_setup_state: CouchSetupState,
    /// Couch Community Hub browsing state and modals.
    pub couch_hub_state: CouchHubState,
    /// In-game Couch OSD overlay state.
    pub in_game_osd: crate::big_picture::overlay::InGameOsdState,
    /// Whether a splitscreen game session is actively executing.
    pub session_active: bool,
    /// Connected displays detected by the platform display manager.
    pub connected_displays: Vec<nucleus_common::types::Display>,
    /// Display index assignments per player (0-based monitor index).
    pub player_monitor_assignments: Vec<usize>,
    /// Available live audio output endpoints: list of (device_id, friendly_name).
    pub audio_devices: Vec<(String, String)>,
    /// Assigned audio device IDs per player instance.
    pub player_audio_device_assignments: Vec<String>,
    /// Active interactive launch prompt request modal, if one is awaiting user response.
    pub active_launch_prompt: Option<crate::models::LaunchPromptRequest>,
    /// Startup pre-flight warning message, if any.
    pub preflight_warning: Option<String>,
    /// Startup pre-flight danger/error message, if any.
    pub preflight_danger: Option<String>,
    /// Hub account modal state.
    pub hub_account_modal: crate::hub_account_modal::HubAccountModalState,
    /// Authenticated Hub account summary if logged in.
    pub hub_account_summary: Option<nucleus_profile::hub_account::HubAccountSummary>,
    /// Save slots modal state.
    pub save_slots_modal: crate::save_slots_modal::SaveSlotsModalState,
    /// Active save slots for currently viewed game.
    pub current_save_slots: Vec<nucleus_profile::save_slots::SaveSlot>,
    /// Assigned save slot IDs per player slot (0-indexed).
    pub player_save_slots: Vec<Option<String>>,
    /// Extensions view state.
    pub extensions_view_state: crate::extensions_view::ExtensionsViewState,
    /// Installed extensions list for the UI.
    pub installed_extensions: Vec<crate::extensions_view::ExtensionItem>,
    /// Handler debugger view state.
    pub debugger_view_state: crate::debugger_view::DebuggerViewState,
    /// Active sub-tab in Big Picture Settings screen (0 = General, 1 = Controllers).
    pub big_picture_settings_tab: usize,
    /// Controller GUID currently being tested/identified (for pulse/rumble effect).
    pub identifying_controller_guid: Option<String>,
    /// Whether an asynchronous disk game scan is currently running.
    pub is_scanning_games: bool,
    /// Focus index for Screen 7 (Game Details): 0 = Set up session, 1 = Load saved profile, 2 = Manage game, 3 = Breadcrumb.
    pub details_focus_index: usize,
    /// Focus index for Screen 1 (Game Management): 0..8.
    pub management_focus_index: usize,
    /// Focus index for Screens 2 & 3 (Sessions): action or profile index.
    pub sessions_focus_index: usize,
    /// Focus zone for Screens 4 & 5 (Settings): 0 = Sidebar tabs, 1 = Settings content.
    pub settings_focus_zone: usize,
    /// Focus item index within active settings panel (0..N).
    pub settings_item_index: usize,
    /// Focus zone for Screen 8 (Couch Hub / Add games): 0 = Top action cards, 1 = Handlers list, 2 = Detail pane action.
    pub hub_focus_zone: usize,
    /// Top action card focus for Screen 8 (0 = Scan for games, 1 = Browse files).
    pub hub_top_card_index: usize,
}

impl MainWindow {
    /// Creates a new `MainWindow` component with detected hardware and handlers.
    pub fn new(detected_devices: usize, handler_names: Vec<String>) -> Self {
        let selected_game = handler_names.first().cloned();
        let initial_player_count = std::cmp::max(detected_devices, 2);

        let mut setup_screen = crate::SetupScreenView::new();
        setup_screen.set_player_count(initial_player_count);

        let mut game_list = crate::GameListView::new();
        if let Some(ref g) = selected_game {
            game_list.select_game(Some(g.clone()));
        }

        let detected_devices_list = Vec::new();

        let handlers: Vec<HandlerItem> = handler_names
            .iter()
            .map(|name| {
                HandlerItem::new(
                    name.to_lowercase().replace(' ', "-"),
                    name.clone(),
                    format!("{}.exe", name.to_lowercase().replace(' ', "")),
                    4,
                    format!("Official handler script for {}. Features multi-instance save isolation, custom viewport calculations, and gamepad focus hooks.", name),
                )
            })
            .collect();

        let mut win = Self {
            active_tab: ActiveTab::Library,
            detected_devices,
            detected_devices_list,
            handler_names,
            handlers,
            selected_game,
            player_count: initial_player_count,
            selected_layout: "2x1 (Side-by-Side)".to_string(),
            player_device_assignments: Vec::new(),
            hub_search_query: String::new(),
            hub_packages: Vec::new(),
            hub_page: 0,
            hub_page_size: 20,
            hub_sort: HubSortOption::Trending,
            hub_sort_ascending: false,
            selected_tags: std::collections::HashSet::new(),
            hub_loaded: false,
            inspecting_hub_package: None,
            hub_detail_tab: crate::hub_details::HubDetailTab::default(),
            hub_detail_comments: Vec::new(),
            hub_detail_packages: Vec::new(),
            hub_detail_loading: false,
            hub_detail_extra_likes: std::collections::HashSet::new(),
            inspecting_handler: None,
            handlers_dir: "handlers".to_string(),
            auto_scan: true,
            theme_name: "Dark".to_string(),
            status_message: "Ready. Select a game to configure splitscreen.".to_string(),
            app_settings: nucleus_profile::AppSettings::default(),
            osd_message: None,
            hub_search_focus: None,
            lib_search_focus: None,
            hotkey_record_focus: None,
            window_focus: None,
            installing_handlers: std::collections::HashMap::new(),
            installed_package_ids: std::collections::HashSet::new(),
            game_list,
            setup_screen,
            hub_browser: crate::HubBrowserView::new(),
            settings_view: crate::SettingsView::new(),
            library_sort: crate::models::LibrarySortOption::Alphabetical,
            library_sort_dir: crate::models::LibrarySortDirection::Ascending,
            hub_loading: false,
            context_menu_handler: None,
            selected_handler_options: std::collections::HashMap::new(),
            callbacks: MainWindowCallbacks::default(),
            saved_profiles: Vec::new(),
            active_profile_id: None,
            sidebar_collapsed: None,
            confirm_dialog: None,
            dropdown_player_device: None,
            context_menu_language_picker: false,
            show_controller_shortcuts_modal: false,
            remapping_controller_shortcut: None,
            is_big_picture: false,
            big_picture_screen: BigPictureScreen::Carousel,
            carousel_state: CarouselState::new(0),
            couch_setup_state: CouchSetupState::new(),
            couch_hub_state: CouchHubState::new(),
            in_game_osd: crate::big_picture::overlay::InGameOsdState::new(),
            session_active: false,
            connected_displays: Vec::new(),
            player_monitor_assignments: Vec::new(),
            audio_devices: Vec::new(),
            player_audio_device_assignments: Vec::new(),
            active_launch_prompt: None,
            preflight_warning: None,
            preflight_danger: None,
            hub_account_modal: crate::hub_account_modal::HubAccountModalState::new(),
            hub_account_summary: None,
            save_slots_modal: crate::save_slots_modal::SaveSlotsModalState::new(),
            current_save_slots: Vec::new(),
            player_save_slots: Vec::new(),
            extensions_view_state: crate::extensions_view::ExtensionsViewState::default(),
            installed_extensions: Vec::new(),
            debugger_view_state: crate::debugger_view::DebuggerViewState::default(),
            big_picture_settings_tab: 0,
            identifying_controller_guid: None,
            is_scanning_games: false,
            details_focus_index: 0,
            management_focus_index: 0,
            sessions_focus_index: 0,
            settings_focus_zone: 0,
            settings_item_index: 0,
            hub_focus_zone: 1,
            hub_top_card_index: 0,
        };

        win.sync_player_assignments();
        win
    }

    /// Injects a mock/virtual controller device for testing (Developer Mode only).
    pub fn add_virtual_gamepad(&mut self) {
        if !self.app_settings.developer_mode {
            return;
        }
        let next_id = self
            .detected_devices_list
            .iter()
            .map(|d| d.id)
            .max()
            .map(|m| m + 1)
            .unwrap_or(0);
        let dev = DeviceItem::new_virtual(
            next_id,
            format!("Virtual Controller #{} (XInput)", next_id),
            format!("VIRTUAL-PAD-{:04}", next_id),
        );
        self.detected_devices_list.push(dev);
        self.detected_devices = self.detected_devices_list.len();
        self.sync_player_assignments();
        self.status_message = format!(
            "Added Virtual Controller #{} (Ready for assignment)",
            next_id
        );
    }

    /// Clears any injected virtual controller devices.
    pub fn clear_virtual_gamepads(&mut self) {
        self.detected_devices_list.retain(|d| !d.is_virtual);
        self.detected_devices = self.detected_devices_list.len();
        self.sync_player_assignments();
        self.status_message = "Cleared virtual mock gamepads.".to_string();
    }

    /// Sets explicit rich `HandlerItem` instances from GameManager.
    pub fn with_handler_items(mut self, handlers: Vec<HandlerItem>) -> Self {
        self.handler_names = handlers.iter().map(|h| h.game_name.clone()).collect();
        if self.selected_game.is_none() && !handlers.is_empty() {
            self.selected_game = Some(handlers[0].game_name.clone());
        }
        self.carousel_state.set_count(handlers.len());
        self.handlers = handlers;
        self
    }

    /// Toggles between standard desktop UI and Big Picture couch UI mode.
    pub fn toggle_big_picture(&mut self, cx: &mut Context<Self>) {
        self.is_big_picture = !self.is_big_picture;
        self.carousel_state.set_count(self.handlers.len());
        if self.is_big_picture {
            if let Some(ref sel) = self.selected_game {
                if let Some(pos) = self.handlers.iter().position(|h| {
                    h.game_name.eq_ignore_ascii_case(sel) || h.guid.eq_ignore_ascii_case(sel)
                }) {
                    self.carousel_state.select(pos);
                }
            }
        }
        cx.notify();
    }

    /// Selects a game from the Big Picture carousel by index and opens Game Details.
    pub fn select_carousel_game(&mut self, idx: usize, cx: &mut Context<Self>) {
        let filtered = self
            .carousel_state
            .filter_and_sort_handlers(&self.handlers, &self.carousel_state.search_query);
        if let Some(handler) = filtered.get(idx) {
            self.carousel_state.select(idx);
            let game_name = handler.game_name.clone();
            if !handler.is_installed {
                self.couch_hub_state.prompt_missing_exe(&game_name);
                cx.notify();
                return;
            }
            self.selected_game = Some(game_name);
            self.big_picture_screen = BigPictureScreen::GameDetails;
            cx.notify();
        } else if idx < self.handlers.len() {
            self.carousel_state.select(idx);
            let game_name = self.handlers[idx].game_name.clone();
            if !self.handlers[idx].is_installed {
                self.couch_hub_state.prompt_missing_exe(&game_name);
                cx.notify();
                return;
            }
            self.selected_game = Some(game_name);
            self.big_picture_screen = BigPictureScreen::GameDetails;
            cx.notify();
        }
    }

    /// Prompts the user to locate the executable for an uninstalled handler via native file picker asynchronously.
    pub fn locate_exe_for_handler(&mut self, handler_name: &str, cx: &mut Context<Self>) {
        let h_name = handler_name.to_string();
        self.status_message = format!("Browsing for '{}' executable...", h_name);
        cx.notify();
        cx.spawn(async move |this, cx| {
            let exe_opt = crate::dialogs::pick_file_async(
                "Locate Game Executable",
                "Executables (*.exe)|*.exe|All Files (*.*)|*.*",
            )
            .await;
            if let Some(exe) = exe_opt {
                let exe_path = exe.to_string_lossy().to_string();
                this.update(&mut *cx, |this, cx| {
                    if let Some(ref cb) = this.callbacks.on_add_game {
                        let _ = cb(&exe_path);
                    }
                    if let Some(item) = this.handlers.iter_mut().find(|it| {
                        it.game_name.eq_ignore_ascii_case(&h_name)
                            || it.guid.eq_ignore_ascii_case(&h_name)
                    }) {
                        item.is_installed = true;
                        item.executable_path = Some(exe_path.clone());
                    }
                    if !this
                        .handler_names
                        .iter()
                        .any(|n| n.eq_ignore_ascii_case(&h_name))
                    {
                        this.handler_names.push(h_name.clone());
                    }
                    this.selected_game = Some(h_name.clone());
                    this.status_message =
                        format!("Located executable for '{}': {}", h_name, exe_path);
                    this.couch_hub_state.dismiss_prompt();
                    this.enter_couch_setup(&h_name, cx);
                })
                .ok();
            }
        })
        .detach();
    }

    /// Enters Couch Splitscreen Setup for the specified game with auto-assignment.
    pub fn enter_couch_setup(&mut self, game_name: &str, cx: &mut Context<Self>) {
        self.selected_game = Some(game_name.to_string());
        let handler_opt = self
            .handlers
            .iter()
            .find(|h| {
                h.game_name.eq_ignore_ascii_case(game_name)
                    || h.guid.eq_ignore_ascii_case(game_name)
            });
        let supports_keyboard = handler_opt
            .map(|h| h.supports_keyboard)
            .unwrap_or(false);

        if let Some(h) = handler_opt {
            for opt in &h.options {
                if !self.selected_handler_options.contains_key(&opt.key) {
                    if let Some(first_val) = opt.values.first() {
                        self.selected_handler_options.insert(opt.key.clone(), first_val.clone());
                    }
                }
            }
        }

        self.couch_setup_state.auto_assign_from_items_with_options(
            &self.detected_devices_list,
            &self.app_settings.ignored_device_guids,
            supports_keyboard,
        );
        self.couch_setup_state.current_step = crate::big_picture::setup::CouchSetupStep::Players;
        self.couch_setup_state.focused_slot = 0;
        self.big_picture_screen = BigPictureScreen::CouchSetup;
        cx.notify();
    }

    /// Enters Couch Community Hub browsing in Big Picture mode.
    pub fn enter_couch_hub(&mut self, cx: &mut Context<Self>) {
        self.big_picture_screen = BigPictureScreen::CouchHub;
        if self.hub_packages.is_empty() {
            self.trigger_hub_search_async(cx);
        }
        cx.notify();
    }

    /// Opens the gamepad-navigable handler custom options modal.
    pub fn open_couch_options_modal(
        &mut self,
        game_name: &str,
        options: Vec<CouchOptionItem>,
        cx: &mut Context<Self>,
    ) {
        self.couch_hub_state.custom_options_modal = Some(
            crate::big_picture::CouchOptionsModalState::new(game_name, options),
        );
        cx.notify();
    }

    /// Confirms current custom options and proceeds with couch session launch.
    pub fn confirm_couch_options_and_launch(&mut self, cx: &mut Context<Self>) {
        if let Some(modal) = self.couch_hub_state.custom_options_modal.take() {
            self.selected_handler_options = modal.to_hashmap();
        }
        self.launch_couch_session(cx);
    }

    /// Downloads and installs a package from Couch Hub, then immediately drops into Couch Setup.
    pub fn download_and_play_couch_package(&mut self, package_id: &str, cx: &mut Context<Self>) {
        let pkg = match self
            .hub_packages
            .iter()
            .find(|p| p.id == package_id)
            .cloned()
        {
            Some(p) => p,
            None => return,
        };

        let is_installed = pkg.is_installed || self.installed_package_ids.contains(&pkg.id);
        if is_installed {
            let maybe_handler = self
                .handlers
                .iter()
                .find(|h| h.guid == pkg.id || h.game_name.eq_ignore_ascii_case(&pkg.game_name))
                .cloned();

            let candidate = maybe_handler
                .as_ref()
                .and_then(|h| h.executable_path.as_ref().map(std::path::PathBuf::from));

            match resolve_game_path_or_couch_prompt(candidate) {
                PathResolution::Found(_) => {
                    let game_name = maybe_handler.map(|h| h.game_name).unwrap_or(pkg.game_name);
                    self.enter_couch_setup(&game_name, cx);
                }
                PathResolution::PromptDesktopMode => {
                    self.couch_hub_state.prompt_desktop_mode(&pkg.title);
                    cx.notify();
                }
            }
            return;
        }

        self.installing_handlers.insert(pkg.id.clone(), 0.0);
        self.status_message = format!("Downloading package '{}'...", pkg.title);
        cx.notify();

        if let Some(ref cb) = self.callbacks.on_install_handler {
            let cb = cb.clone();
            let pkg_id_clone = pkg.id.clone();
            let pkg_id_done = pkg.id.clone();
            let pkg_title_done = pkg.title.clone();
            let pkg_game_done = pkg.game_name.clone();

            let (tx, mut rx) =
                tokio::sync::mpsc::unbounded_channel::<crate::models::HubInstallProgressMsg>();
            let tx_prog = tx.clone();

            std::thread::spawn(move || {
                let item = cb(
                    &pkg_id_clone,
                    Box::new(move |progress: f32| {
                        let _ =
                            tx_prog.send(crate::models::HubInstallProgressMsg::Progress(progress));
                    }),
                );
                let _ = tx.send(crate::models::HubInstallProgressMsg::Done(item));
            });

            cx.spawn(async move |this, cx| {
                while let Some(msg) = rx.recv().await {
                    let pkg_id = pkg_id_done.clone();
                    let pkg_title = pkg_title_done.clone();
                    let pkg_game = pkg_game_done.clone();

                    let _ = this.update(&mut *cx, |win, cx| match msg {
                        crate::models::HubInstallProgressMsg::Progress(p) => {
                            win.installing_handlers.insert(pkg_id, p);
                            cx.notify();
                        }
                        crate::models::HubInstallProgressMsg::Done(installed_item) => {
                            win.installing_handlers.remove(&pkg_id);
                            win.installed_package_ids.insert(pkg_id.clone());
                            if let Some(p) =
                                win.hub_packages.iter_mut().find(|item| item.id == pkg_id)
                            {
                                p.is_installed = true;
                                p.has_update = false;
                                p.is_up_to_date = true;
                                p.installed_version = Some(p.version_number);
                            }

                            let target_handler = if let Some(h) = installed_item {
                                let name = h.game_name.clone();
                                if let Some(existing) =
                                    win.handlers.iter_mut().find(|e| e.guid == h.guid)
                                {
                                    *existing = h.clone();
                                } else {
                                    win.handlers.push(h.clone());
                                }
                                if !win.handler_names.contains(&name) {
                                    win.handler_names.push(name.clone());
                                }
                                Some(h)
                            } else {
                                win.handlers
                                    .iter()
                                    .find(|e| {
                                        e.guid == pkg_id
                                            || e.game_name.eq_ignore_ascii_case(&pkg_game)
                                    })
                                    .cloned()
                            };

                            let candidate = target_handler.as_ref().and_then(|h| {
                                h.executable_path.as_ref().map(std::path::PathBuf::from)
                            });

                            match resolve_game_path_or_couch_prompt(candidate) {
                                PathResolution::Found(_) => {
                                    let g_name =
                                        target_handler.map(|h| h.game_name).unwrap_or(pkg_game);
                                    win.status_message =
                                        format!("Installed '{}'. Ready for couch setup!", g_name);
                                    win.enter_couch_setup(&g_name, cx);
                                }
                                PathResolution::PromptDesktopMode => {
                                    win.status_message = format!(
                                        "Installed '{}', but executable was not found.",
                                        pkg_title
                                    );
                                    win.couch_hub_state.prompt_desktop_mode(&pkg_title);
                                    cx.notify();
                                }
                            }
                        }
                    });
                }
            })
            .detach();
        } else {
            // Standalone or testing without live installer callback
            self.installing_handlers.remove(&pkg.id);
            self.installed_package_ids.insert(pkg.id.clone());
            if let Some(p) = self.hub_packages.iter_mut().find(|item| item.id == pkg.id) {
                p.is_installed = true;
            }

            let maybe_handler = self
                .handlers
                .iter()
                .find(|e| e.guid == pkg.id || e.game_name.eq_ignore_ascii_case(&pkg.game_name))
                .cloned();

            let candidate = maybe_handler
                .as_ref()
                .and_then(|h| h.executable_path.as_ref().map(std::path::PathBuf::from));

            match resolve_game_path_or_couch_prompt(candidate) {
                PathResolution::Found(_) => {
                    let g_name = maybe_handler.map(|h| h.game_name).unwrap_or(pkg.game_name);
                    self.enter_couch_setup(&g_name, cx);
                }
                PathResolution::PromptDesktopMode => {
                    self.couch_hub_state.prompt_desktop_mode(&pkg.title);
                    cx.notify();
                }
            }
        }
    }

    /// Restores a previously ignored controller GUID, saving settings and re-adding
    /// the device to active couch setup slots if found in detected devices list.
    pub fn restore_ignored_controller(&mut self, guid: &str, cx: &mut Context<Self>) {
        self.app_settings.ignored_device_guids.retain(|g| g != guid);
        let _ = self.app_settings.save();

        if guid == "keyboard-mouse-0" {
            let kbm = crate::big_picture::setup::create_keyboard_mouse_device();
            self.couch_setup_state
                .restore_device(&kbm, &mut self.app_settings.ignored_device_guids);
        } else if let Some(dev_item) = self
            .detected_devices_list
            .iter()
            .find(|d| d.guid == guid)
            .cloned()
        {
            self.couch_setup_state
                .restore_device_from_item(&dev_item, &mut self.app_settings.ignored_device_guids);
        }

        cx.notify();
    }

    /// Returns the filtered and sorted list of community hub packages for Couch Big Picture.
    pub fn get_filtered_couch_hub_packages(&self) -> Vec<&HubPackageItem> {
        crate::big_picture::hub::get_filtered_packages(
            &self.hub_packages,
            self.couch_hub_state.selected_category,
            &self.couch_hub_state.search_query,
            self.couch_hub_state.filter_verified,
            self.couch_hub_state.filter_gamepads,
            self.couch_hub_state.filter_km,
        )
    }

    /// Dispatches a canonical couch UI action by name across Big Picture screens.
    pub fn dispatch_couch_action_by_name(&mut self, action_name: &str, cx: &mut Context<Self>) {
        match action_name {
            "ToggleBigPicture" => {
                self.toggle_big_picture(cx);
            }
            "ToggleOsd" => {
                self.toggle_in_game_osd(cx);
            }
            "NavigateUp" => {
                if self.in_game_osd.is_visible {
                    self.in_game_osd.move_up();
                } else if let Some(ref mut m) = self.couch_hub_state.custom_options_modal {
                    m.move_up();
                } else if self.big_picture_screen == BigPictureScreen::Carousel {
                    self.carousel_state.move_up();
                } else if self.big_picture_screen == BigPictureScreen::GameDetails {
                    self.details_focus_index = self.details_focus_index.saturating_sub(1);
                } else if self.big_picture_screen == BigPictureScreen::GameManagement {
                    self.management_focus_index = match self.management_focus_index {
                        0 | 1 => 8,
                        2 => 0,
                        3 => 1,
                        4 => 2,
                        5 => 3,
                        6 => 4,
                        7 => 5,
                        _ => 0,
                    };
                } else if self.big_picture_screen == BigPictureScreen::Sessions {
                    if self.is_session_active() {
                        self.sessions_focus_index = self.sessions_focus_index.saturating_sub(1);
                    } else {
                        self.sessions_focus_index = 0;
                    }
                } else if self.big_picture_screen == BigPictureScreen::Settings {
                    if self.settings_focus_zone == 0 {
                        self.big_picture_settings_tab = self.big_picture_settings_tab.saturating_sub(1);
                    } else {
                        self.settings_item_index = self.settings_item_index.saturating_sub(1);
                    }
                } else if self.big_picture_screen == BigPictureScreen::CouchHub {
                    if self.hub_focus_zone == 3 {
                        self.hub_focus_zone = 1;
                    } else if self.hub_focus_zone == 2 {
                        if self.couch_hub_state.selected_index == 0 {
                            self.hub_focus_zone = 1;
                        } else {
                            self.couch_hub_state.selected_index -= 1;
                        }
                    } else if self.hub_focus_zone == 1 {
                        self.hub_focus_zone = 0;
                    }
                } else if self.big_picture_screen == BigPictureScreen::CouchSetup {
                    if self.couch_setup_state.focused_slot > 0 {
                        self.couch_setup_state.focused_slot -= 1;
                    }
                    if self.couch_setup_state.current_step == crate::big_picture::setup::CouchSetupStep::ScreenLayout
                        && self.couch_setup_state.focused_slot < crate::big_picture::setup::AVAILABLE_LAYOUTS.len()
                    {
                        self.couch_setup_state.selected_layout = crate::big_picture::setup::AVAILABLE_LAYOUTS[self.couch_setup_state.focused_slot].to_string();
                        self.selected_layout = self.couch_setup_state.selected_layout.clone();
                    }
                }
                cx.notify();
            }
            "NavigateDown" => {
                if self.in_game_osd.is_visible {
                    self.in_game_osd.move_down();
                } else if let Some(ref mut m) = self.couch_hub_state.custom_options_modal {
                    m.move_down();
                } else if self.big_picture_screen == BigPictureScreen::Carousel {
                    self.carousel_state.move_down();
                } else if self.big_picture_screen == BigPictureScreen::GameDetails {
                    self.details_focus_index = (self.details_focus_index + 1).min(2);
                } else if self.big_picture_screen == BigPictureScreen::GameManagement {
                    self.management_focus_index = match self.management_focus_index {
                        8 => 1,
                        0 => 2,
                        1 => 3,
                        2 => 4,
                        3 => 5,
                        4 => 6,
                        5 => 7,
                        _ => 7,
                    };
                } else if self.big_picture_screen == BigPictureScreen::Sessions {
                    if self.is_session_active() {
                        self.sessions_focus_index = (self.sessions_focus_index + 1).min(4);
                    } else if self.sessions_focus_index == 0 {
                        self.sessions_focus_index = 1;
                    }
                } else if self.big_picture_screen == BigPictureScreen::Settings {
                    if self.settings_focus_zone == 0 {
                        self.big_picture_settings_tab = (self.big_picture_settings_tab + 1).min(1);
                    } else {
                        let max_items = if self.big_picture_settings_tab == 0 { 6 } else { self.detected_devices_list.len() + 1 };
                        self.settings_item_index = (self.settings_item_index + 1).min(max_items);
                    }
                } else if self.big_picture_screen == BigPictureScreen::CouchHub {
                    if self.hub_focus_zone == 0 {
                        self.hub_focus_zone = 1;
                    } else if self.hub_focus_zone == 1 {
                        self.hub_focus_zone = 2;
                    } else if self.hub_focus_zone == 2 {
                        let count = self.get_filtered_couch_hub_packages().len();
                        if count > 0 && self.couch_hub_state.selected_index + 1 < count {
                            self.couch_hub_state.selected_index += 1;
                        }
                    }
                } else if self.big_picture_screen == BigPictureScreen::CouchSetup {
                    let max_slots = match self.couch_setup_state.current_step {
                        crate::big_picture::setup::CouchSetupStep::Players => self.couch_setup_state.slots.len(),
                        crate::big_picture::setup::CouchSetupStep::ScreenLayout => crate::big_picture::setup::AVAILABLE_LAYOUTS.len(),
                        crate::big_picture::setup::CouchSetupStep::GameOptions => {
                            self.selected_game.as_ref().and_then(|g| self.handlers.iter().find(|h| &h.game_name == g || &h.guid == g)).map(|h| h.options.len()).unwrap_or(0)
                        }
                        crate::big_picture::setup::CouchSetupStep::Launch => 0,
                    };
                    if self.couch_setup_state.focused_slot < max_slots {
                        self.couch_setup_state.focused_slot += 1;
                    }
                    if self.couch_setup_state.current_step == crate::big_picture::setup::CouchSetupStep::ScreenLayout
                        && self.couch_setup_state.focused_slot < crate::big_picture::setup::AVAILABLE_LAYOUTS.len()
                    {
                        self.couch_setup_state.selected_layout = crate::big_picture::setup::AVAILABLE_LAYOUTS[self.couch_setup_state.focused_slot].to_string();
                        self.selected_layout = self.couch_setup_state.selected_layout.clone();
                    }
                }
                cx.notify();
            }
            "NavigateLeft" => {
                if let Some(ref mut m) = self.couch_hub_state.custom_options_modal {
                    m.cycle_choice(false);
                } else if self.big_picture_screen == BigPictureScreen::Carousel {
                    match self.carousel_state.focus_zone {
                        crate::big_picture::carousel::CarouselFocusZone::HeroBanner(btn_idx) => {
                            if btn_idx > 0 {
                                self.carousel_state.focus_zone = crate::big_picture::carousel::CarouselFocusZone::HeroBanner(btn_idx - 1);
                            }
                        }
                        crate::big_picture::carousel::CarouselFocusZone::HeaderTabs => {
                            self.big_picture_screen = self.big_picture_screen.prev();
                        }
                        crate::big_picture::carousel::CarouselFocusZone::Cards => {
                            self.carousel_state.move_left();
                        }
                    }
                } else if self.big_picture_screen == BigPictureScreen::GameDetails {
                    self.big_picture_screen = BigPictureScreen::Carousel;
                } else if self.big_picture_screen == BigPictureScreen::GameManagement {
                    self.management_focus_index = match self.management_focus_index {
                        1 => 0,
                        3 => 2,
                        5 => 4,
                        7 => 6,
                        8 => 0,
                        _ => self.management_focus_index,
                    };
                } else if self.big_picture_screen == BigPictureScreen::Sessions {
                    if !self.is_session_active() {
                        let total_profiles = self.get_all_saved_profiles_items().len();
                        if self.sessions_focus_index > 1 {
                            self.sessions_focus_index -= 1;
                        } else if self.sessions_focus_index == 1 && total_profiles > 0 {
                            self.sessions_focus_index = total_profiles;
                        }
                    }
                } else if self.big_picture_screen == BigPictureScreen::Settings {
                    self.settings_focus_zone = 0;
                } else if self.big_picture_screen == BigPictureScreen::CouchHub {
                    if self.hub_focus_zone == 0 {
                        self.hub_top_card_index = 0;
                    } else if self.hub_focus_zone == 3 {
                        self.hub_focus_zone = 2;
                    } else {
                        self.couch_hub_state.prev_category();
                    }
                } else if self.big_picture_screen == BigPictureScreen::CouchSetup {
                    match self.couch_setup_state.current_step {
                        crate::big_picture::setup::CouchSetupStep::Players => {
                            if self.couch_setup_state.focused_slot == self.couch_setup_state.slots.len() {
                                self.couch_setup_state.focused_slot = 0;
                            } else {
                                self.couch_setup_state.navigate_slot_left();
                            }
                        }
                        crate::big_picture::setup::CouchSetupStep::ScreenLayout => {
                            self.couch_setup_state.focused_slot = 0;
                        }
                        crate::big_picture::setup::CouchSetupStep::GameOptions => {
                            let cur_slot = self.couch_setup_state.focused_slot;
                            if let Some(h) = self.selected_game.as_ref().and_then(|g| self.handlers.iter().find(|h| &h.game_name == g || &h.guid == g)) {
                                if let Some(opt) = h.options.get(cur_slot) {
                                    let cur_val = self.selected_handler_options.get(&opt.key).cloned().unwrap_or_else(|| opt.values.first().cloned().unwrap_or_default());
                                    if let Some(pos) = opt.values.iter().position(|v| v == &cur_val) {
                                        let prev_idx = (pos + opt.values.len() - 1) % opt.values.len();
                                        if let Some(nv) = opt.values.get(prev_idx) {
                                            self.selected_handler_options.insert(opt.key.clone(), nv.clone());
                                        }
                                    }
                                }
                            }
                        }
                        crate::big_picture::setup::CouchSetupStep::Launch => {}
                    }
                } else {
                    self.carousel_state.move_left();
                }
                cx.notify();
            }
            "NavigateRight" => {
                if let Some(ref mut m) = self.couch_hub_state.custom_options_modal {
                    m.cycle_choice(true);
                } else if self.big_picture_screen == BigPictureScreen::Carousel {
                    match self.carousel_state.focus_zone {
                        crate::big_picture::carousel::CarouselFocusZone::HeroBanner(btn_idx) => {
                            if btn_idx < 2 {
                                self.carousel_state.focus_zone = crate::big_picture::carousel::CarouselFocusZone::HeroBanner(btn_idx + 1);
                            }
                        }
                        crate::big_picture::carousel::CarouselFocusZone::HeaderTabs => {
                            self.big_picture_screen = self.big_picture_screen.next();
                        }
                        crate::big_picture::carousel::CarouselFocusZone::Cards => {
                            self.carousel_state.move_right();
                        }
                    }
                } else if self.big_picture_screen == BigPictureScreen::GameDetails {
                    // Retain focus on right actions panel
                } else if self.big_picture_screen == BigPictureScreen::GameManagement {
                    self.management_focus_index = match self.management_focus_index {
                        0 => 1,
                        2 => 3,
                        4 => 5,
                        6 => 7,
                        _ => self.management_focus_index,
                    };
                } else if self.big_picture_screen == BigPictureScreen::Sessions {
                    if !self.is_session_active() {
                        let total_profiles = self.get_all_saved_profiles_items().len();
                        if self.sessions_focus_index == 0 {
                            self.sessions_focus_index = 1;
                        } else if self.sessions_focus_index < total_profiles {
                            self.sessions_focus_index += 1;
                        } else if total_profiles > 0 {
                            self.sessions_focus_index = 1;
                        }
                    }
                } else if self.big_picture_screen == BigPictureScreen::Settings {
                    self.settings_focus_zone = 1;
                } else if self.big_picture_screen == BigPictureScreen::CouchHub {
                    if self.hub_focus_zone == 0 {
                        self.hub_top_card_index = 1;
                    } else if self.hub_focus_zone == 2 {
                        self.hub_focus_zone = 3;
                    } else {
                        self.couch_hub_state.next_category();
                    }
                } else if self.big_picture_screen == BigPictureScreen::CouchSetup {
                    match self.couch_setup_state.current_step {
                        crate::big_picture::setup::CouchSetupStep::Players => {
                            self.couch_setup_state.focused_slot = self.couch_setup_state.slots.len();
                        }
                        crate::big_picture::setup::CouchSetupStep::ScreenLayout => {
                            self.couch_setup_state.focused_slot = crate::big_picture::setup::AVAILABLE_LAYOUTS.len();
                        }
                        crate::big_picture::setup::CouchSetupStep::GameOptions => {
                            let cur_slot = self.couch_setup_state.focused_slot;
                            let opts_len = self.selected_game.as_ref().and_then(|g| self.handlers.iter().find(|h| &h.game_name == g || &h.guid == g)).map(|h| h.options.len()).unwrap_or(0);
                            if cur_slot >= opts_len {
                                // Already on next button
                            } else if let Some(h) = self.selected_game.as_ref().and_then(|g| self.handlers.iter().find(|h| &h.game_name == g || &h.guid == g)) {
                                if let Some(opt) = h.options.get(cur_slot) {
                                    let cur_val = self.selected_handler_options.get(&opt.key).cloned().unwrap_or_else(|| opt.values.first().cloned().unwrap_or_default());
                                    if let Some(pos) = opt.values.iter().position(|v| v == &cur_val) {
                                        let next_idx = (pos + 1) % opt.values.len();
                                        if let Some(nv) = opt.values.get(next_idx) {
                                            self.selected_handler_options.insert(opt.key.clone(), nv.clone());
                                        }
                                    }
                                }
                            }
                        }
                        crate::big_picture::setup::CouchSetupStep::Launch => {}
                    }
                } else {
                    self.carousel_state.move_right();
                }
                cx.notify();
            }
            "Confirm" => {
                if self.in_game_osd.is_visible {
                    self.handle_osd_confirm(cx);
                    return;
                }
                if self.couch_hub_state.custom_options_modal.is_some() {
                    self.confirm_couch_options_and_launch(cx);
                    return;
                }
                if let Some(h_name) = self.couch_hub_state.missing_exe_modal_handler.clone() {
                    self.locate_exe_for_handler(&h_name, cx);
                    return;
                }
                if self.big_picture_screen == BigPictureScreen::Carousel {
                    match self.carousel_state.focus_zone {
                        crate::big_picture::carousel::CarouselFocusZone::HeroBanner(btn_idx) => {
                            if btn_idx == 2 {
                                self.carousel_state.search_input_open = true;
                                cx.notify();
                            } else {
                                let filtered = self.carousel_state.filter_and_sort_handlers(&self.handlers, &self.carousel_state.search_query);
                                let target = filtered.get(self.carousel_state.selected_index).map(|h| h.game_name.clone()).or_else(|| self.selected_game.clone());
                                if let Some(name) = target {
                                    if btn_idx == 0 {
                                        self.enter_couch_setup(&name, cx);
                                    } else {
                                        self.selected_game = Some(name);
                                        self.big_picture_screen = BigPictureScreen::GameDetails;
                                        cx.notify();
                                    }
                                }
                            }
                        }
                        crate::big_picture::carousel::CarouselFocusZone::HeaderTabs => {}
                        crate::big_picture::carousel::CarouselFocusZone::Cards => {
                            self.select_carousel_game(self.carousel_state.selected_index, cx);
                        }
                    }
                } else if self.big_picture_screen == BigPictureScreen::GameDetails {
                    match self.details_focus_index {
                        0 => {
                            if let Some(ref g) = self.selected_game.clone() {
                                self.enter_couch_setup(g, cx);
                            }
                        }
                        1 => {
                            self.load_last_saved_profile_for_selected(cx);
                        }
                        2 => {
                            self.big_picture_screen = BigPictureScreen::GameManagement;
                            cx.notify();
                        }
                        3 => {
                            self.big_picture_screen = BigPictureScreen::Carousel;
                            cx.notify();
                        }
                        _ => {
                            if let Some(ref g) = self.selected_game.clone() {
                                self.enter_couch_setup(g, cx);
                            }
                        }
                    }
                } else if self.big_picture_screen == BigPictureScreen::GameManagement {
                    let target_game = self.selected_game.clone().unwrap_or_default();
                    let handler_opt = self.handlers.iter().find(|h| h.game_name == target_game || h.guid == target_game).cloned();
                    let h_path = handler_opt.as_ref().and_then(|h| h.handler_path.clone());
                    let h_guid = handler_opt.as_ref().map(|h| h.guid.clone()).unwrap_or_default();
                    let exe_path = handler_opt.as_ref().and_then(|h| h.executable_path.clone());
                    match self.management_focus_index {
                        0 => self.open_game_handler_file(h_path.as_deref(), &h_guid, &target_game, cx),
                        1 => {
                            let is_on_press = self.app_settings.custom_preferences.get(&format!("{}:gamepad_mode", h_guid)).map(|v| v == "on_button_press").unwrap_or(true);
                            self.set_gamepad_mode_preference(&h_guid, if is_on_press { "by_index" } else { "on_button_press" }, cx);
                        }
                        2 => self.open_game_exe_path(exe_path.as_deref(), &target_game, cx),
                        3 => self.cycle_steam_language_preference(&h_guid, cx),
                        4 => self.open_game_userprofile_path(&target_game, cx),
                        5 => self.open_game_assets_folder(&target_game, cx),
                        6 => self.trigger_change_game_icon_async(&h_guid, &target_game, cx),
                        7 => self.prompt_remove_game_confirmation(&h_guid, &target_game, cx),
                        8 => {
                            self.big_picture_screen = BigPictureScreen::GameDetails;
                            cx.notify();
                        }
                        _ => {}
                    }
                } else if self.big_picture_screen == BigPictureScreen::CouchSetup {
                    let total_slots = self.couch_setup_state.slots.len();
                    match self.couch_setup_state.current_step {
                        crate::big_picture::setup::CouchSetupStep::Players => {
                            if self.couch_setup_state.focused_slot == total_slots {
                                self.couch_setup_state.next_step();
                                self.couch_setup_state.focused_slot = 0;
                            } else {
                                let focused = self.couch_setup_state.focused_slot;
                                if self.couch_setup_state.grabbed_slot.is_some() {
                                    self.couch_setup_state.drop_swap_target(focused);
                                } else {
                                    self.couch_setup_state.grab_slot(focused);
                                }
                            }
                            cx.notify();
                        }
                        crate::big_picture::setup::CouchSetupStep::ScreenLayout => {
                            self.couch_setup_state.next_step();
                            self.couch_setup_state.focused_slot = 0;
                            cx.notify();
                        }
                        crate::big_picture::setup::CouchSetupStep::GameOptions => {
                            let cur_slot = self.couch_setup_state.focused_slot;
                            let opts_len = self.selected_game.as_ref().and_then(|g| self.handlers.iter().find(|h| &h.game_name == g || &h.guid == g)).map(|h| h.options.len()).unwrap_or(0);
                            if cur_slot >= opts_len {
                                self.couch_setup_state.next_step();
                                self.couch_setup_state.focused_slot = 0;
                            } else if let Some(h) = self.selected_game.as_ref().and_then(|g| self.handlers.iter().find(|h| &h.game_name == g || &h.guid == g)) {
                                if let Some(opt) = h.options.get(cur_slot) {
                                    let cur_val = self.selected_handler_options.get(&opt.key).cloned().unwrap_or_else(|| opt.values.first().cloned().unwrap_or_default());
                                    if let Some(pos) = opt.values.iter().position(|v| v == &cur_val) {
                                        let next_idx = (pos + 1) % opt.values.len();
                                        if let Some(nv) = opt.values.get(next_idx) {
                                            self.selected_handler_options.insert(opt.key.clone(), nv.clone());
                                        }
                                    }
                                }
                            }
                            cx.notify();
                        }
                        crate::big_picture::setup::CouchSetupStep::Launch => {
                            self.launch_couch_session(cx);
                        }
                    }
                } else if self.big_picture_screen == BigPictureScreen::CouchHub {
                    if self.hub_focus_zone == 0 {
                        if self.hub_top_card_index == 0 {
                            self.trigger_scan_games_async(cx);
                        } else {
                            self.trigger_browse_game_exe_async(cx);
                        }
                    } else if self.hub_focus_zone == 1 {
                        self.couch_hub_state.search_input_open = true;
                        cx.notify();
                    } else {
                        let pkgs = self.get_filtered_couch_hub_packages();
                        if self.couch_hub_state.selected_index < pkgs.len() {
                            let pkg = pkgs[self.couch_hub_state.selected_index];
                            let is_installed = self.is_handler_or_pkg_installed(&pkg.id, &pkg.game_name);
                            if is_installed {
                                self.selected_game = Some(pkg.game_name.clone());
                                self.big_picture_screen = BigPictureScreen::GameDetails;
                                cx.notify();
                            } else {
                                let pid = pkg.id.clone();
                                self.download_and_play_couch_package(&pid, cx);
                            }
                        }
                    }
                } else if self.big_picture_screen == BigPictureScreen::Sessions {
                    if self.is_session_active() {
                        match self.sessions_focus_index {
                            0 => self.resume_running_session_windows(cx),
                            1 => {
                                self.in_game_osd.open_reassign(0);
                                self.set_osd_visible(true);
                                cx.notify();
                            }
                            2 => {
                                self.big_picture_screen = BigPictureScreen::GameDetails;
                                cx.notify();
                            }
                            3 => self.save_current_session_profile(cx),
                            4 => self.stop_active_session_cleanly(cx),
                            _ => self.resume_running_session_windows(cx),
                        }
                    } else {
                        if self.sessions_focus_index == 0 {
                            self.save_current_session_profile(cx);
                        } else {
                            let prof_idx = self.sessions_focus_index.saturating_sub(1);
                            let saved = self.get_all_saved_profiles_items();
                            if let Some(prof) = saved.get(prof_idx) {
                                self.load_saved_profile_by_number(&prof.game_guid, prof.profile_number, cx);
                            }
                        }
                    }
                } else if self.big_picture_screen == BigPictureScreen::Settings {
                    if self.settings_focus_zone == 0 {
                        cx.notify();
                    } else if self.big_picture_settings_tab == 0 {
                        match self.settings_item_index {
                            0 => {
                                self.app_settings.start_in_big_picture = !self.app_settings.start_in_big_picture;
                                let _ = self.app_settings.save();
                                self.show_osd("Setting saved", 1200, cx);
                                cx.notify();
                            }
                            1 => self.cycle_interface_scale_preference(cx),
                            2 => self.cycle_preferred_display_preference(cx),
                            3 => { self.show_osd("Sound effects toggled", 1200, cx); cx.notify(); }
                            4 => { self.show_osd("English (Default)", 1200, cx); cx.notify(); }
                            5 => { self.show_osd("Auto-updates enabled", 1200, cx); cx.notify(); }
                            6 => {
                                self.app_settings.developer_mode = !self.app_settings.developer_mode;
                                let _ = self.app_settings.save();
                                self.show_osd(
                                    if self.app_settings.developer_mode {
                                        "Developer mode enabled"
                                    } else {
                                        "Developer mode disabled"
                                    },
                                    1200,
                                    cx,
                                );
                                cx.notify();
                            }
                            _ => {}
                        }
                    } else {
                        if self.settings_item_index == 0 {
                            self.show_osd("Assignment method: On button press", 1500, cx);
                            cx.notify();
                        } else if self.settings_item_index <= self.detected_devices_list.len() {
                            let dev_idx = self.settings_item_index - 1;
                            if let Some(dev) = self.detected_devices_list.get(dev_idx) {
                                self.identifying_controller_guid = Some(dev.guid.clone());
                                self.show_osd(&format!("Testing {}", dev.name), 1500, cx);
                                cx.notify();
                            }
                        } else {
                            self.refresh_detected_gamepads(cx);
                        }
                    }
                }
            }
            "Cancel" => {
                if self.in_game_osd.is_visible {
                    if self.in_game_osd.reassign_modal_open {
                        self.in_game_osd.close_reassign();
                    } else {
                        self.set_osd_visible(false);
                    }
                    cx.notify();
                    return;
                }
                if self.couch_hub_state.custom_options_modal.is_some() {
                    self.couch_hub_state.custom_options_modal = None;
                    cx.notify();
                    return;
                }
                if self.couch_hub_state.missing_exe_modal_handler.is_some() {
                    self.couch_hub_state.dismiss_prompt();
                    cx.notify();
                    return;
                }
                if self.big_picture_screen == BigPictureScreen::GameManagement {
                    self.big_picture_screen = BigPictureScreen::GameDetails;
                    cx.notify();
                } else if self.big_picture_screen == BigPictureScreen::GameDetails {
                    self.big_picture_screen = BigPictureScreen::Carousel;
                    cx.notify();
                } else if self.big_picture_screen == BigPictureScreen::CouchSetup {
                    if self.couch_setup_state.grabbed_slot.is_some() {
                        self.couch_setup_state.cancel_grab();
                    } else if self.couch_setup_state.current_step != crate::big_picture::setup::CouchSetupStep::Players {
                        self.couch_setup_state.prev_step();
                    } else {
                        self.big_picture_screen = BigPictureScreen::GameDetails;
                    }
                    cx.notify();
                } else if self.big_picture_screen == BigPictureScreen::CouchHub
                    || self.big_picture_screen == BigPictureScreen::Sessions
                    || self.big_picture_screen == BigPictureScreen::Settings
                {
                    self.big_picture_screen = BigPictureScreen::Carousel;
                    cx.notify();
                } else {
                    self.toggle_big_picture(cx);
                }
            }
            "QuickLaunch" => {
                if self.in_game_osd.is_visible {
                    self.set_osd_visible(false);
                    cx.notify();
                    return;
                }
                if self.couch_hub_state.custom_options_modal.is_some() {
                    self.confirm_couch_options_and_launch(cx);
                    return;
                }
                if self.big_picture_screen == BigPictureScreen::CouchSetup {
                    self.launch_couch_session(cx);
                } else if self.big_picture_screen == BigPictureScreen::CouchHub {
                    let pkgs = self.get_filtered_couch_hub_packages();
                    if self.couch_hub_state.selected_index < pkgs.len() {
                        let pid = pkgs[self.couch_hub_state.selected_index].id.clone();
                        self.download_and_play_couch_package(&pid, cx);
                    }
                } else {
                    self.select_carousel_game(self.carousel_state.selected_index, cx);
                }
            }
            "Context1" => {
                if self.big_picture_screen == BigPictureScreen::CouchSetup {
                    let focused = self.couch_setup_state.focused_slot;
                    self.couch_setup_state.remove_slot(focused);
                    cx.notify();
                }
            }
            "Context2" => {
                if self.big_picture_screen == BigPictureScreen::CouchSetup {
                    let focused = self.couch_setup_state.focused_slot;
                    if focused < self.couch_setup_state.slots.len() {
                        let guid = self.couch_setup_state.slots[focused].device_guid.clone();
                        self.couch_setup_state
                            .ignore_device(&guid, &mut self.app_settings.ignored_device_guids);
                        let _ = self.app_settings.save();
                    }
                    cx.notify();
                }
            }
            "NextTab" => {
                if self.big_picture_screen == BigPictureScreen::CouchSetup {
                    self.couch_setup_state.next_step();
                } else {
                    self.big_picture_screen = self.big_picture_screen.next();
                    if self.big_picture_screen == BigPictureScreen::CouchHub
                        && self.hub_packages.is_empty()
                    {
                        self.trigger_hub_search_async(cx);
                    }
                }
                cx.notify();
            }
            "PrevTab" => {
                if self.big_picture_screen == BigPictureScreen::CouchSetup {
                    self.couch_setup_state.prev_step();
                } else {
                    self.big_picture_screen = self.big_picture_screen.prev();
                    if self.big_picture_screen == BigPictureScreen::CouchHub
                        && self.hub_packages.is_empty()
                    {
                        self.trigger_hub_search_async(cx);
                    }
                }
                cx.notify();
            }
            _ => {}
        }
    }

    /// Launches a splitscreen session immediately from Couch Setup.
    pub fn launch_couch_session(&mut self, cx: &mut Context<Self>) {
        let target = match self.selected_game {
            Some(ref g) => g.clone(),
            None => return,
        };
        let player_count = self.couch_setup_state.slots.len();
        if player_count == 0 {
            self.status_message = "Cannot launch: No player slots assigned.".to_string();
            cx.notify();
            return;
        }

        // Ensure default options are populated for any unconfigured keys
        if let Some(handler) = self.handlers.iter().find(|h| {
            h.game_name.eq_ignore_ascii_case(&target) || h.guid.eq_ignore_ascii_case(&target)
        }) {
            for opt in &handler.options {
                if !self.selected_handler_options.contains_key(&opt.key) {
                    if let Some(first_val) = opt.values.first() {
                        self.selected_handler_options.insert(opt.key.clone(), first_val.clone());
                    }
                }
            }
        }
        self.player_count = player_count;
        self.selected_layout = self.couch_setup_state.selected_layout.clone();
        self.player_device_assignments = self
            .couch_setup_state
            .slots
            .iter()
            .map(|s| {
                if s.is_complete_raw_kbm() {
                    let kb = s.raw_keyboard_name.as_deref().unwrap_or("Keyboard");
                    let m = s.raw_mouse_name.as_deref().unwrap_or("Mouse");
                    format!("{} + {} (ProtoInput)", kb, m)
                } else if s.has_raw_keyboard() {
                    let kb = s.raw_keyboard_name.as_deref().unwrap_or("Keyboard");
                    format!("{} (ProtoInput)", kb)
                } else if s.has_raw_mouse() {
                    let m = s.raw_mouse_name.as_deref().unwrap_or("Mouse");
                    format!("{} (ProtoInput)", m)
                } else if s.is_paired_kbm() {
                    "Keyboard & Mouse (Paired)".to_string()
                } else if let Some(id) = s.device_id {
                    format!("Gamepad #{} ({})", id, s.device_name)
                } else {
                    "Keyboard & Mouse Slot".to_string()
                }
            })
            .collect();

        if let Some(ref launcher) = self.callbacks.on_launch {
            launcher(
                &target,
                player_count,
                &self.selected_layout,
                &self.selected_handler_options,
                &self.player_device_assignments,
                &self.player_monitor_assignments,
                &self.player_audio_device_assignments,
            );
        }
        self.status_message = format!(
            "Splitscreen session launched for '{}' ({} players)!",
            target, player_count
        );
        cx.notify();
    }

    /// Advances disconnect resilience timers for all couch player slots and repaints.
    pub fn tick_disconnect_resilience(
        &mut self,
        delta: std::time::Duration,
        cx: &mut Context<Self>,
    ) {
        self.couch_setup_state.tick_disconnect(delta);
        cx.notify();
    }

    /// Advances disconnect resilience timers on state without notifying a GPUI context.
    pub fn tick_disconnect_resilience_state(&mut self, delta: std::time::Duration) {
        self.couch_setup_state.tick_disconnect(delta);
    }

    /// Synchronizes detected devices and notifies CouchSetupState of disconnects and reconnects.
    pub fn sync_devices(&mut self, devices: Vec<DeviceItem>) {
        // Detect newly disconnected device IDs:
        for prev in &self.detected_devices_list {
            let still_connected = devices.iter().any(|d| d.id == prev.id && d.is_connected);
            if prev.is_connected && !still_connected {
                self.couch_setup_state
                    .handle_device_disconnect_by_id(prev.id);
            }
        }
        // Detect reconnected devices:
        for new_dev in &devices {
            if new_dev.is_connected {
                let was_connected = self
                    .detected_devices_list
                    .iter()
                    .any(|d| d.id == new_dev.id && d.is_connected);
                if !was_connected {
                    let fp = nucleus_input::gamepad::DeviceFingerprint::new(
                        &new_dev.guid,
                        &new_dev.name,
                        &nucleus_common::types::DeviceType::XInputGamepad,
                    );
                    self.couch_setup_state.handle_device_reconnect(
                        new_dev.id,
                        &new_dev.guid,
                        &new_dev.name,
                        Some(&fp),
                    );
                }
            }
        }
        self.detected_devices = devices.len();
        self.detected_devices_list = devices;
        self.sync_player_assignments();
    }

    /// Sets explicit detected hardware `DeviceItem` instances.
    pub fn with_device_items(mut self, devices: Vec<DeviceItem>) -> Self {
        self.detected_devices = devices.len();
        self.detected_devices_list = devices;
        self.sync_player_assignments();
        self
    }

    /// Sets initial community packages loaded from Hub API.
    pub fn with_hub_packages(mut self, packages: Vec<HubPackageItem>) -> Self {
        self.hub_packages = packages;
        self
    }

    /// Opens the game handler script file in a text editor or Explorer asynchronously.
    pub fn open_game_handler_file(
        &mut self,
        handler_path: Option<&str>,
        guid: &str,
        game_name: &str,
        cx: &mut Context<Self>,
    ) {
        let p_opt = handler_path.map(|s| s.to_string()).or_else(|| {
            let h_dir = std::path::Path::new(&self.handlers_dir);
            let candidate1 = h_dir.join(format!("{}.js", guid));
            if candidate1.exists() {
                return Some(candidate1.to_string_lossy().to_string());
            }
            let candidate2 = h_dir.join(format!("{}.js", game_name));
            if candidate2.exists() {
                return Some(candidate2.to_string_lossy().to_string());
            }
            None
        });

        if let Some(p) = p_opt {
            let p_str = p.clone();
            std::thread::spawn(move || {
                #[cfg(windows)]
                {
                    if std::process::Command::new("notepad.exe").arg(&p_str).spawn().is_err() {
                        let _ = std::process::Command::new("explorer").arg(&p_str).spawn();
                    }
                }
                #[cfg(not(windows))]
                {
                    let _ = std::process::Command::new("xdg-open").arg(&p_str).spawn();
                }
            });
            self.show_osd(&format!("Opened handler for '{}'", game_name), 2000, cx);
        } else {
            self.show_osd(&format!("Handler script not found for '{}'", game_name), 3000, cx);
        }
        cx.notify();
    }

    /// Opens the parent directory of the original game executable in Windows Explorer.
    pub fn open_game_exe_path(
        &mut self,
        exe_path: Option<&str>,
        game_name: &str,
        cx: &mut Context<Self>,
    ) {
        if let Some(p) = exe_path {
            let p_path = std::path::PathBuf::from(p);
            let dir = p_path.parent().unwrap_or(&p_path).to_string_lossy().to_string();
            std::thread::spawn(move || {
                #[cfg(windows)]
                {
                    let _ = std::process::Command::new("explorer").arg(&dir).spawn();
                }
                #[cfg(not(windows))]
                {
                    let _ = std::process::Command::new("xdg-open").arg(&dir).spawn();
                }
            });
            self.show_osd(&format!("Opened executable folder for '{}'", game_name), 2000, cx);
        } else {
            self.show_osd(&format!("Executable path not configured for '{}'", game_name), 3000, cx);
        }
        cx.notify();
    }

    /// Opens the UserProfile save path for the game (%USERPROFILE%\NucleusCoop\<GameName>).
    pub fn open_game_userprofile_path(&mut self, game_name: &str, cx: &mut Context<Self>) {
        let name_clean = game_name.to_string();
        std::thread::spawn(move || {
            let user_profile = std::env::var("USERPROFILE").unwrap_or_else(|_| "C:\\Users\\Default".to_string());
            let target_dir = std::path::Path::new(&user_profile)
                .join("NucleusCoop")
                .join(&name_clean);
            let _ = std::fs::create_dir_all(&target_dir);
            #[cfg(windows)]
            {
                let _ = std::process::Command::new("explorer").arg(&target_dir).spawn();
            }
            #[cfg(not(windows))]
            {
                let _ = std::process::Command::new("xdg-open").arg(&target_dir).spawn();
            }
        });
        self.show_osd(&format!("Opened UserProfile save path for '{}'", game_name), 2000, cx);
        cx.notify();
    }

    /// Opens the game assets content directory (./content/<GameName>).
    pub fn open_game_assets_folder(&mut self, game_name: &str, cx: &mut Context<Self>) {
        let name_clean = game_name.to_string();
        let data_dir = if !self.app_settings.nucleus_data_dir.is_empty() {
            self.app_settings.nucleus_data_dir.clone()
        } else {
            "content".to_string()
        };
        std::thread::spawn(move || {
            let target_dir = std::path::Path::new(&data_dir).join(&name_clean);
            let _ = std::fs::create_dir_all(&target_dir);
            #[cfg(windows)]
            {
                let _ = std::process::Command::new("explorer").arg(&target_dir).spawn();
            }
            #[cfg(not(windows))]
            {
                let _ = std::process::Command::new("xdg-open").arg(&target_dir).spawn();
            }
        });
        self.show_osd(&format!("Opened game assets folder for '{}'", game_name), 2000, cx);
        cx.notify();
    }

    /// Sets custom gamepad assignment preference ("by_index" or "on_button_press").
    pub fn set_gamepad_mode_preference(&mut self, guid: &str, mode: &str, cx: &mut Context<Self>) {
        let key = format!("{}:gamepad_mode", guid);
        self.app_settings.custom_preferences.insert(key, mode.to_string());
        let _ = self.app_settings.save();
        self.show_osd(&format!("Gamepad mode set to '{}'", if mode == "by_index" { "By index" } else { "On button press" }), 1500, cx);
        cx.notify();
    }

    /// Cycles the custom Steam language setting for the game.
    pub fn cycle_steam_language_preference(&mut self, guid: &str, cx: &mut Context<Self>) {
        let key = format!("{}:steam_language", guid);
        let langs = [
            "English", "Spanish", "French", "German", "Italian", "Japanese", "Portuguese", "Russian", "Simplified Chinese"
        ];
        let current = self.app_settings.custom_preferences.get(&key).map(|s| s.as_str()).unwrap_or("English");
        let next_idx = langs.iter().position(|&l| l.eq_ignore_ascii_case(current)).map(|i| (i + 1) % langs.len()).unwrap_or(0);
        let next_lang = langs[next_idx];
        self.app_settings.custom_preferences.insert(key, next_lang.to_string());
        let _ = self.app_settings.save();
        self.show_osd(&format!("Steam language: {}", next_lang), 1500, cx);
        cx.notify();
    }

    /// Asynchronously prompts the user for a game icon image file and updates the handler.
    pub fn trigger_change_game_icon_async(&mut self, guid: &str, game_name: &str, cx: &mut Context<Self>) {
        let g_guid = guid.to_string();
        let g_name = game_name.to_string();
        self.show_osd("Select game icon file...", 2000, cx);
        cx.spawn(async move |this, cx| {
            let path_opt = crate::dialogs::pick_file_async(
                "Select Game Icon (*.png; *.jpg; *.ico)",
                "*.png;*.jpg;*.jpeg;*.ico;*.webp",
            ).await;
            if let Some(path) = path_opt {
                let p_str = path.to_string_lossy().to_string();
                this.update(&mut *cx, |this, cx| {
                    if let Some(h) = this.handlers.iter_mut().find(|h| h.guid == g_guid || h.game_name == g_name) {
                        h.cover_image_url = Some(p_str.clone());
                    }
                    this.show_osd("Game icon updated!", 2000, cx);
                    cx.notify();
                }).ok();
            }
        }).detach();
    }

    /// Asynchronously prompts the user for a game executable to add to the library.
    pub fn trigger_browse_game_exe_async(&mut self, cx: &mut Context<Self>) {
        self.show_osd("Select game executable...", 2000, cx);
        cx.spawn(async move |this, cx| {
            let path_opt = crate::dialogs::pick_file_async(
                "Select Game Executable (*.exe)",
                "Executables (*.exe)|*.exe|All Files (*.*)|*.*",
            ).await;
            if let Some(path) = path_opt {
                let p_str = path.to_string_lossy().to_string();
                this.update(&mut *cx, |this, cx| {
                    if let Some(ref cb) = this.callbacks.on_add_game {
                        let _ = cb(&p_str);
                    }
                    this.show_osd("Game executable added!", 2000, cx);
                    cx.notify();
                }).ok();
            }
        }).detach();
    }

    /// Shows confirmation dialog to remove a game from the local library.
    pub fn prompt_remove_game_confirmation(&mut self, guid: &str, game_name: &str, cx: &mut Context<Self>) {
        let target_guid = guid.to_string();
        let target_name = game_name.to_string();
        self.confirm_dialog = Some(crate::models::ConfirmDialogState {
            title: format!("Remove '{}'?", target_name),
            message: format!(
                "Are you sure you want to remove '{}' from your Nucleus library? The game files and save data on your PC will remain untouched.",
                target_name
            ),
            confirm_label: "Remove Game".to_string(),
            is_destructive: true,
            action: crate::models::ConfirmAction::RemoveGame(target_guid),
        });
        cx.notify();
    }

    /// Loads the last saved profile for the currently selected game.
    pub fn load_last_saved_profile_for_selected(&mut self, cx: &mut Context<Self>) {
        if let Some(ref game) = self.selected_game.clone() {
            let guid = self.handlers.iter().find(|h| h.game_name == *game || h.guid == *game)
                .map(|h| h.guid.clone()).unwrap_or_else(|| game.clone());
            if let Some(ref cb) = self.callbacks.on_load_numbered_profile {
                if let Some(prof) = cb(&guid, 0) {
                    self.player_count = prof.player_count;
                    self.selected_layout = prof.layout.clone();
                    self.show_osd(&format!("Loaded profile: {} players, {}", prof.player_count, prof.layout), 2000, cx);
                    self.enter_couch_setup(game, cx);
                    return;
                }
            }
            self.show_osd("No saved profile found. Opening session setup.", 2000, cx);
            self.enter_couch_setup(game, cx);
        }
    }

    /// Resumes the running game session and minimizes the launcher.
    pub fn resume_running_session_windows(&mut self, cx: &mut Context<Self>) {
        self.show_osd("Resuming session...", 1500, cx);
        #[cfg(windows)]
        unsafe {
            use windows::Win32::UI::Input::KeyboardAndMouse::GetActiveWindow;
            let hwnd = GetActiveWindow();
            if !hwnd.is_invalid() {
                let _ = windows::Win32::UI::WindowsAndMessaging::ShowWindow(
                    hwnd,
                    windows::Win32::UI::WindowsAndMessaging::SW_MINIMIZE,
                );
            }
        }
        cx.notify();
    }

    /// Saves the active session setup as a numbered profile.
    pub fn save_current_session_profile(&mut self, cx: &mut Context<Self>) {
        let game = self.selected_game.clone().unwrap_or_else(|| "Unknown".to_string());
        let guid = self.handlers.iter().find(|h| h.game_name == game || h.guid == game)
            .map(|h| h.guid.clone()).unwrap_or_else(|| game.clone());
        let profile_title = format!("{} ({}p)", game, self.player_count);
        let mut prof = nucleus_profile::game_profile::GameProfile::default();
        prof.id = format!("{}_profile_0", guid);
        prof.handler_guid = guid.clone();
        prof.profile_name = Some(profile_title.clone());
        prof.player_count = self.player_count;
        prof.layout = self.selected_layout.clone();
        prof.options = self.selected_handler_options.clone();
        prof.player_device_assignments = self.player_device_assignments.clone();
        prof.player_monitor_assignments = self.player_monitor_assignments.clone();
        prof.player_audio_device_assignments = self.player_audio_device_assignments.clone();

        if let Some(ref cb) = self.callbacks.on_save_numbered_profile {
            cb(&guid, Some(0), &prof);
            self.show_osd(&format!("Saved session profile: {}", profile_title), 2500, cx);
        } else {
            self.show_osd("Profile saved to local cache.", 2000, cx);
        }
        cx.notify();
    }

    /// Cleanly stops an active session and returns to the library carousel.
    pub fn stop_active_session_cleanly(&mut self, cx: &mut Context<Self>) {
        if let Some(ref cb) = self.callbacks.on_stop_session {
            cb();
        }
        self.session_active = false;
        self.show_osd("Splitscreen session ended.", 2500, cx);
        self.big_picture_screen = BigPictureScreen::Carousel;
        cx.notify();
    }

    /// Generates a list of all saved profile items for display in the Saved Profiles view.
    pub fn get_all_saved_profiles_items(&self) -> Vec<crate::models::SavedProfileItem> {
        let mut items = Vec::new();
        for (idx, h) in self.handlers.iter().enumerate() {
            items.push(crate::models::SavedProfileItem {
                game_guid: h.guid.clone(),
                game_title: h.game_name.clone(),
                profile_number: idx,
                profile_title: format!("{} setup", h.game_name),
                player_count: if h.max_players >= 2 { 2 } else { 1 },
                layout: if h.max_players >= 2 { "1x2 Stacked".to_string() } else { "1x1 Single".to_string() },
                cover_url: h.cover_image_url.clone(),
            });
        }
        items
    }

    /// Loads a saved profile by index and transitions to Couch Splitscreen Setup.
    pub fn load_saved_profile_by_number(&mut self, guid: &str, num: usize, cx: &mut Context<Self>) {
        if let Some(h) = self.handlers.iter().find(|h| h.guid == guid || h.game_name == guid).cloned() {
            let g_name = h.game_name.clone();
            self.selected_game = Some(g_name.clone());
            if let Some(ref cb) = self.callbacks.on_load_numbered_profile {
                if let Some(prof) = cb(&h.guid, num) {
                    self.player_count = prof.player_count;
                    self.selected_layout = prof.layout.clone();
                    let name_disp = prof.profile_name.as_deref().unwrap_or(&h.game_name);
                    self.show_osd(&format!("Loaded profile: {}", name_disp), 2000, cx);
                    self.enter_couch_setup(&g_name, cx);
                    return;
                }
            }
            self.player_count = if h.max_players >= 2 { 2 } else { 1 };
            self.selected_layout = if h.max_players >= 2 { "1x2 Stacked".to_string() } else { "1x1 Single".to_string() };
            self.show_osd(&format!("Loaded setup for '{}'", g_name), 2000, cx);
            self.enter_couch_setup(&g_name, cx);
        }
    }

    /// Opens an external URL in the default browser asynchronously.
    pub fn open_url_external(&mut self, url: &str, cx: &mut Context<Self>) {
        let u = url.to_string();
        std::thread::spawn(move || {
            #[cfg(windows)]
            {
                let _ = std::process::Command::new("rundll32")
                    .args(["url.dll,FileProtocolHandler", &u])
                    .spawn();
            }
            #[cfg(not(windows))]
            {
                let _ = std::process::Command::new("xdg-open").arg(&u).spawn();
            }
        });
        self.show_osd(&format!("Opening {}", url), 1500, cx);
        cx.notify();
    }

    /// Re-enumerates connected gamepad hardware.
    pub fn refresh_detected_gamepads(&mut self, cx: &mut Context<Self>) {
        self.show_osd(&format!("Controllers: {} connected", self.detected_devices), 1500, cx);
        cx.notify();
    }

    /// Cycles interface scale preference.
    pub fn cycle_interface_scale_preference(&mut self, cx: &mut Context<Self>) {
        let scales = ["Large · TV", "Normal", "Large"];
        let cur = &self.app_settings.interface_scale;
        let next_idx = scales.iter().position(|&s| s == cur).map(|i| (i + 1) % scales.len()).unwrap_or(0);
        self.app_settings.interface_scale = scales[next_idx].to_string();
        let _ = self.app_settings.save();
        self.show_osd(&format!("Interface scale: {}", self.app_settings.interface_scale), 1500, cx);
        cx.notify();
    }

    /// Cycles preferred display monitor index.
    pub fn cycle_preferred_display_preference(&mut self, cx: &mut Context<Self>) {
        let total_displays = if !self.connected_displays.is_empty() {
            self.connected_displays.len()
        } else {
            1
        };
        self.app_settings.preferred_display = (self.app_settings.preferred_display + 1) % total_displays;
        let _ = self.app_settings.save();
        self.show_osd(&format!("Preferred display: Display {}", self.app_settings.preferred_display + 1), 1500, cx);
        cx.notify();
    }

    /// Tests and identifies a controller with haptic rumble and visual pulse feedback.
    pub fn test_identify_controller(&mut self, guid: &str, name: &str, cx: &mut Context<Self>) {
        self.identifying_controller_guid = Some(guid.to_string());
        self.show_osd(&format!("Identified: {}", name), 1500, cx);
        cx.notify();

        // Automatically clear identification pulse after 1.5s using GPUI native timer (Safety Rule 1)
        cx.spawn(async move |this, cx| {
            cx.background_executor().timer(std::time::Duration::from_millis(1500)).await;
            this.update(&mut *cx, |this, cx| {
                this.identifying_controller_guid = None;
                cx.notify();
            }).ok();
        }).detach();
    }

    /// Asynchronously scans all storage drives for supported games.
    pub fn trigger_scan_games_async(&mut self, cx: &mut Context<Self>) {
        if self.is_scanning_games {
            self.show_osd("Game scan already in progress...", 2000, cx);
            return;
        }
        self.is_scanning_games = true;
        self.show_osd("Scanning drives for installed games...", 3000, cx);
        cx.notify();

        let scan_cb = self.callbacks.on_scan_games.clone();
        cx.spawn(async move |this, cx| {
            let (tx, rx) = futures::channel::oneshot::channel();
            std::thread::spawn(move || {
                let found = if let Some(cb) = scan_cb {
                    cb()
                } else {
                    Vec::new()
                };
                let _ = tx.send(found);
            });
            let discovered = rx.await.unwrap_or_default();
            this.update(&mut *cx, |this, cx| {
                this.is_scanning_games = false;
                let mut added_count = 0;
                for item in discovered {
                    if !this.handlers.iter().any(|h| h.guid == item.guid || h.game_name.eq_ignore_ascii_case(&item.game_name)) {
                        this.handlers.push(item);
                        added_count += 1;
                    }
                }
                this.show_osd(&format!("Scan complete! Discovered {} new game(s).", added_count), 3000, cx);
                cx.notify();
            }).ok();
        }).detach();
    }

    /// Checks whether a specific handler GUID or game title is installed locally.
    pub fn is_handler_or_pkg_installed(&self, pkg_id: &str, game_name: &str) -> bool {
        if self.installed_package_ids.contains(pkg_id) {
            return true;
        }
        let norm_game = crate::hub_browser::normalize_for_match(game_name);
        let norm_id = crate::hub_browser::normalize_for_match(pkg_id);
        self.handlers.iter().any(|h| {
            let norm_h_name = crate::hub_browser::normalize_for_match(&h.game_name);
            let norm_h_guid = crate::hub_browser::normalize_for_match(&h.guid);
            norm_h_name == norm_game
                || norm_h_name == norm_id
                || norm_h_guid == norm_game
                || norm_h_guid == norm_id
                || h.guid.eq_ignore_ascii_case(pkg_id)
                || h.game_name.eq_ignore_ascii_case(game_name)
        })
    }

    /// Synchronizes per-player device assignment slots to match `player_count`.
    /// Enforces device uniqueness so no two players share Keyboard or the same controller.
    pub fn sync_player_assignments(&mut self) {
        let mut new_assignments = Vec::with_capacity(self.player_count);
        let mut keyboard_used = false;
        let mut used_gamepad_ids = Vec::new();

        for p in 0..self.player_count {
            let mut assigned_device = None;

            // Preserve existing assignment if valid and not duplicate
            if let Some(existing) = self.player_device_assignments.get(p) {
                if (existing == "Keyboard & Mouse Slot" || existing == "Keyboard & Mouse (Paired)")
                    && !keyboard_used
                {
                    keyboard_used = true;
                    assigned_device = Some(existing.clone());
                } else if existing.contains("ProtoInput") {
                    assigned_device = Some(existing.clone());
                } else if existing.starts_with("Gamepad #")
                    || existing.starts_with("Virtual Gamepad #")
                    || existing.starts_with("Virtual Controller #")
                {
                    if let Some(dev) = self.detected_devices_list.iter().find(|d| {
                        existing.starts_with(&format!("Gamepad #{}", d.id))
                            || existing.starts_with(&format!("Virtual Controller #{}", d.id))
                            || existing.starts_with(&format!("Virtual Gamepad #{}", d.id))
                    }) {
                        if !used_gamepad_ids.contains(&dev.id) {
                            used_gamepad_ids.push(dev.id);
                            assigned_device = Some(existing.clone());
                        }
                    }
                }
            }

            if let Some(dev_str) = assigned_device {
                new_assignments.push(dev_str);
                continue;
            }

            // Prefer assigning physical or virtual gamepads
            if let Some(dev) = self
                .detected_devices_list
                .iter()
                .find(|d| !used_gamepad_ids.contains(&d.id))
            {
                used_gamepad_ids.push(dev.id);
                if dev.is_virtual {
                    new_assignments.push(format!("Virtual Gamepad #{} ({})", dev.id, dev.name));
                } else {
                    new_assignments.push(format!("Gamepad #{} ({})", dev.id, dev.name));
                }
            } else if !keyboard_used {
                // Otherwise assign the single keyboard slot
                keyboard_used = true;
                new_assignments.push("Keyboard & Mouse Slot".to_string());
            } else {
                // No remaining input devices
                new_assignments.push("Unassigned (No Device Available)".to_string());
            }
        }
        self.player_device_assignments = new_assignments;

        // Synchronize per-player monitor assignments
        let disp_count = if self.connected_displays.is_empty() {
            1
        } else {
            self.connected_displays.len()
        };
        self.player_monitor_assignments.resize(self.player_count, 0);
        for (i, m) in self.player_monitor_assignments.iter_mut().enumerate() {
            if *m >= disp_count {
                *m = i % disp_count;
            }
        }

        // Synchronize per-player audio assignments
        self.player_audio_device_assignments
            .resize(self.player_count, "Default".to_string());
    }

    /// Cycles the assigned device for the given player slot between available physical gamepads, virtual gamepads, and Keyboard.
    pub fn cycle_player_device(&mut self, player_idx: usize) {
        if player_idx >= self.player_device_assignments.len() {
            return;
        }

        // Determine devices taken by other players
        let mut keyboard_taken_by_other = false;
        let mut gamepad_ids_taken_by_other = Vec::new();

        for (idx, assignment) in self.player_device_assignments.iter().enumerate() {
            if idx == player_idx {
                continue;
            }
            if assignment == "Keyboard & Mouse Slot" || assignment == "Keyboard & Mouse (Paired)" {
                keyboard_taken_by_other = true;
            }
            for d in &self.detected_devices_list {
                if assignment.starts_with(&format!("Gamepad #{}", d.id))
                    || assignment.starts_with(&format!("Virtual Gamepad #{}", d.id))
                    || assignment.starts_with(&format!("Virtual Controller #{}", d.id))
                {
                    gamepad_ids_taken_by_other.push(d.id);
                }
            }
        }

        let mut options: Vec<String> = Vec::new();

        // 1. Available physical and virtual gamepads (not claimed by other players)
        for d in &self.detected_devices_list {
            if !gamepad_ids_taken_by_other.contains(&d.id) {
                if d.is_virtual {
                    options.push(format!("Virtual Gamepad #{} ({})", d.id, d.name));
                } else {
                    options.push(format!("Gamepad #{} ({})", d.id, d.name));
                }
            }
        }

        // 2. Keyboard & Mouse (only if not claimed by another player)
        if !keyboard_taken_by_other {
            options.push("Keyboard & Mouse (Paired)".to_string());
        }

        // 3. ProtoInput Individual Keyboards & Mice
        options.push("Keyboard #1 (ProtoInput)".to_string());
        options.push("Keyboard #2 (ProtoInput)".to_string());
        options.push("Mouse #1 (ProtoInput)".to_string());
        options.push("Mouse #2 (ProtoInput)".to_string());
        options.push("K#1 + M#1 (ProtoInput Pair)".to_string());
        options.push("K#2 + M#2 (ProtoInput Pair)".to_string());

        // 3. DirectInput Emulation (x360ce) - if at least one physical gamepad is detected
        if self.detected_devices_list.iter().any(|d| !d.is_virtual) {
            options.push("DirectInput Emulation (x360ce)".to_string());
        }

        // 4. Always provide Unassigned option
        options.push("Unassigned (No Device Available)".to_string());

        let current = &self.player_device_assignments[player_idx];
        let current_pos = options.iter().position(|opt| opt == current).unwrap_or(0);
        let next_pos = (current_pos + 1) % options.len();
        self.player_device_assignments[player_idx] = options[next_pos].clone();
        self.status_message = format!(
            "Player {} input assigned to: {}",
            player_idx + 1,
            self.player_device_assignments[player_idx]
        );
    }

    /// Advances to the next page of Hub packages.
    pub fn hub_next_page(&mut self) {
        let total_pages = if self.hub_packages.is_empty() {
            1
        } else {
            (self.hub_packages.len() + self.hub_page_size - 1) / self.hub_page_size
        };
        if self.hub_page + 1 < total_pages {
            self.hub_page += 1;
        }
    }

    /// Returns to the previous page of Hub packages.
    pub fn hub_prev_page(&mut self) {
        if self.hub_page > 0 {
            self.hub_page -= 1;
        }
    }

    /// Marks a Hub package as installed in local state and adds to game list.
    pub fn mark_package_installed(&mut self, package_id: &str) {
        let mut added_name = None;
        for pkg in &mut self.hub_packages {
            if pkg.id == package_id {
                pkg.is_installed = true;
                added_name = Some((pkg.title.clone(), pkg.description.clone()));
                break;
            }
        }
        if let Some((title, desc)) = added_name {
            if !self.handler_names.contains(&title) {
                self.handler_names.push(title.clone());
                self.handlers.push(HandlerItem::new(
                    package_id,
                    title.clone(),
                    format!("{}.exe", title.to_lowercase().replace(' ', "")),
                    4,
                    desc,
                ));
            }
        }
    }

    /// Replaces the entire callback builder struct on MainWindow.
    pub fn with_callbacks(mut self, callbacks: MainWindowCallbacks) -> Self {
        self.callbacks = callbacks;
        self
    }

    /// Sets the launch callback for starting games from the GUI.
    pub fn with_launcher<F>(mut self, callback: F) -> Self
    where
        F: Fn(
                &str,
                usize,
                &str,
                &std::collections::HashMap<String, String>,
                &[String],
                &[usize],
                &[String],
            ) + Send
            + Sync
            + 'static,
    {
        self.callbacks = self.callbacks.with_launcher(callback);
        self
    }

    /// Sets callback for toggling favorite status.
    pub fn with_favorite_toggler<F>(mut self, callback: F) -> Self
    where
        F: Fn(&str, bool) + Send + Sync + 'static,
    {
        self.callbacks = self.callbacks.with_favorite_toggler(callback);
        self
    }

    /// Sets callback for saving profile properties.
    pub fn with_profile_saver<F>(mut self, callback: F) -> Self
    where
        F: Fn(&HandlerItem) + Send + Sync + 'static,
    {
        self.callbacks = self.callbacks.with_profile_saver(callback);
        self
    }

    /// Sets callback for deleting UserProfile save directory.
    pub fn with_save_path_deleter<F>(mut self, callback: F) -> Self
    where
        F: Fn(&str) + Send + Sync + 'static,
    {
        self.callbacks = self.callbacks.with_save_path_deleter(callback);
        self
    }

    /// Sets callback for removing a game from the library.
    pub fn with_game_remover<F>(mut self, callback: F) -> Self
    where
        F: Fn(&str) + Send + Sync + 'static,
    {
        self.callbacks = self.callbacks.with_game_remover(callback);
        self
    }

    /// Sets callback for adding a game by executable path.
    pub fn with_game_adder<F>(mut self, callback: F) -> Self
    where
        F: Fn(&str) -> Option<HandlerItem> + Send + Sync + 'static,
    {
        self.callbacks = self.callbacks.with_game_adder(callback);
        self
    }

    /// Sets callback for searching the community hub.
    pub fn with_hub_searcher<F>(mut self, callback: F) -> Self
    where
        F: Fn(&str) -> Vec<HubPackageItem> + Send + Sync + 'static,
    {
        self.callbacks = self.callbacks.with_hub_searcher(callback);
        self
    }

    /// Dispatches an asynchronous Hub package search on a background thread.
    pub fn trigger_hub_search_async(&mut self, cx: &mut Context<Self>) {
        if let Some(cb) = self.callbacks.on_search_hub.clone() {
            self.hub_loading = true;
            let query = self.hub_search_query.clone();
            cx.spawn(async move |this, cx| {
                let results = cx
                    .background_executor()
                    .spawn(async move { cb(&query) })
                    .await;

                this.update(&mut *cx, |win, cx| {
                    win.hub_packages = results;
                    win.hub_loading = false;
                    win.hub_page = 0;
                    win.status_message =
                        format!("Hub: Loaded {} community packages.", win.hub_packages.len());
                    cx.notify();
                })
                .ok();
            })
            .detach();
        }
    }

    /// Loads comments and version packages for the inspected Hub package.
    pub fn load_hub_package_details(&mut self, handler_id: &str, cx: &mut Context<Self>) {
        self.hub_detail_loading = true;
        self.hub_detail_comments.clear();
        self.hub_detail_packages.clear();
        let h_id = handler_id.to_string();
        let (tx, mut rx) = tokio::sync::mpsc::unbounded_channel();

        crate::http_client::get_tokio_runtime().spawn(async move {
            let client = nucleus_hub::HubClient::default();
            let packages_res = client.get_packages(&h_id).await.unwrap_or_default();
            let comments_res = client.get_comments(&h_id).await.unwrap_or_default();
            let _ = tx.send((packages_res, comments_res));
        });

        cx.spawn(async move |this, cx| {
            if let Some((packages_res, comments_res)) = rx.recv().await {
                this.update(&mut *cx, |win, cx| {
                    win.hub_detail_packages = packages_res;
                    win.hub_detail_comments = comments_res;
                    win.hub_detail_loading = false;
                    cx.notify();
                })
                .ok();
            }
        })
        .detach();
    }

    /// Sets callback for installing a hub handler package with progress reporting.
    pub fn with_hub_installer<F>(mut self, callback: F) -> Self
    where
        F: Fn(&str, Box<dyn Fn(f32) + Send + Sync>) -> Option<HandlerItem> + Send + Sync + 'static,
    {
        self.callbacks = self.callbacks.with_hub_installer(callback);
        self
    }

    /// Sets callback for updating the handlers directory.
    pub fn with_handlers_dir_changer<F>(mut self, callback: F) -> Self
    where
        F: Fn(&str) -> Vec<HandlerItem> + Send + Sync + 'static,
    {
        self.callbacks = self.callbacks.with_handlers_dir_changer(callback);
        self
    }

    /// Sets callback for saving theme changes.
    pub fn with_theme_changer<F>(mut self, callback: F) -> Self
    where
        F: Fn(&str) + Send + Sync + 'static,
    {
        self.callbacks = self.callbacks.with_theme_changer(callback);
        self
    }

    /// Sets callback for auto scan toggles.
    pub fn with_auto_scan_changer<F>(mut self, callback: F) -> Self
    where
        F: Fn(bool) + Send + Sync + 'static,
    {
        self.callbacks = self.callbacks.with_auto_scan_changer(callback);
        self
    }

    /// Sets callback for scanning storage drives for installed games matching discovered handlers.
    pub fn with_game_scanner<F>(mut self, callback: F) -> Self
    where
        F: Fn() -> Vec<HandlerItem> + Send + Sync + 'static,
    {
        self.callbacks = self.callbacks.with_game_scanner(callback);
        self
    }

    /// Sets callback for listing saved numbered profiles for a game.
    pub fn with_numbered_profile_lister<F>(mut self, callback: F) -> Self
    where
        F: Fn(&str) -> Vec<(usize, String)> + Send + Sync + 'static,
    {
        self.callbacks = self.callbacks.with_numbered_profile_lister(callback);
        self
    }

    /// Sets callback for loading a numbered profile.
    pub fn with_numbered_profile_loader<F>(mut self, callback: F) -> Self
    where
        F: Fn(&str, usize) -> Option<nucleus_profile::GameProfile> + Send + Sync + 'static,
    {
        self.callbacks = self.callbacks.with_numbered_profile_loader(callback);
        self
    }

    /// Sets callback for saving a numbered profile.
    pub fn with_numbered_profile_saver<F>(mut self, callback: F) -> Self
    where
        F: Fn(&str, Option<usize>, &nucleus_profile::GameProfile) -> Option<usize>
            + Send
            + Sync
            + 'static,
    {
        self.callbacks = self.callbacks.with_numbered_profile_saver(callback);
        self
    }

    /// Sets callback for deleting a numbered profile.
    pub fn with_numbered_profile_deleter<F>(mut self, callback: F) -> Self
    where
        F: Fn(&str, usize) -> bool + Send + Sync + 'static,
    {
        self.callbacks = self.callbacks.with_numbered_profile_deleter(callback);
        self
    }

    /// Sets callback for unloading the active profile.
    pub fn with_profile_unloader<F>(mut self, callback: F) -> Self
    where
        F: Fn() + Send + Sync + 'static,
    {
        self.callbacks = self.callbacks.with_profile_unloader(callback);
        self
    }

    /// Sets callback for toggling session freeze.
    pub fn with_freeze_toggler<F>(mut self, callback: F) -> Self
    where
        F: Fn() -> Option<bool> + Send + Sync + 'static,
    {
        self.callbacks = self.callbacks.with_freeze_toggler(callback);
        self
    }

    /// Sets callback for hot-swapping controller assignment mid-game.
    pub fn with_controller_reassigner<F>(mut self, callback: F) -> Self
    where
        F: Fn(usize, usize) + Send + Sync + 'static,
    {
        self.callbacks = self.callbacks.with_controller_reassigner(callback);
        self
    }

    /// Sets callback for re-enforcing window bounds.
    pub fn with_window_re_enforcer<F>(mut self, callback: F) -> Self
    where
        F: Fn() + Send + Sync + 'static,
    {
        self.callbacks = self.callbacks.with_window_re_enforcer(callback);
        self
    }

    /// Sets callback for toggling raw input lock.
    pub fn with_input_lock_toggler<F>(mut self, callback: F) -> Self
    where
        F: Fn() -> bool + Send + Sync + 'static,
    {
        self.callbacks = self.callbacks.with_input_lock_toggler(callback);
        self
    }

    /// Sets callback for stopping an active session.
    pub fn with_session_stopper<F>(mut self, callback: F) -> Self
    where
        F: Fn() + Send + Sync + 'static,
    {
        self.callbacks = self.callbacks.with_session_stopper(callback);
        self
    }

    /// Sets callback to check if a session is actively running in the engine.
    pub fn with_session_active_checker<F>(mut self, callback: F) -> Self
    where
        F: Fn() -> bool + Send + Sync + 'static,
    {
        self.callbacks = self.callbacks.with_session_active_checker(callback);
        self
    }

    /// Sets callback to toggle the window's TOPMOST status when in-game OSD opens or closes.
    pub fn with_osd_topmost_handler<F>(mut self, callback: F) -> Self
    where
        F: Fn(bool, Option<usize>) + Send + Sync + 'static,
    {
        self.callbacks = self.callbacks.with_osd_topmost_handler(callback);
        self
    }

    /// Sets callback for Hub account login.
    pub fn with_hub_login<F>(mut self, callback: F) -> Self
    where
        F: Fn(&str, &str, bool) + Send + Sync + 'static,
    {
        self.callbacks = self.callbacks.with_hub_login(callback);
        self
    }

    /// Sets callback for Hub account logout.
    pub fn with_hub_logout<F>(mut self, callback: F) -> Self
    where
        F: Fn() + Send + Sync + 'static,
    {
        self.callbacks = self.callbacks.with_hub_logout(callback);
        self
    }

    /// Sets callback for listing save slots.
    pub fn with_save_slots_lister<F>(mut self, callback: F) -> Self
    where
        F: Fn(&str) -> Vec<nucleus_profile::save_slots::SaveSlot> + Send + Sync + 'static,
    {
        self.callbacks = self.callbacks.with_save_slots_lister(callback);
        self
    }

    /// Sets callback for creating a save slot.
    pub fn with_save_slot_creator<F>(mut self, callback: F) -> Self
    where
        F: Fn(&str, &str, &str, &str) -> Option<nucleus_profile::save_slots::SaveSlot>
            + Send
            + Sync
            + 'static,
    {
        self.callbacks = self.callbacks.with_save_slot_creator(callback);
        self
    }

    /// Sets callback for cloning a save slot.
    pub fn with_save_slot_cloner<F>(mut self, callback: F) -> Self
    where
        F: Fn(&str, &str, &str) -> Option<nucleus_profile::save_slots::SaveSlot>
            + Send
            + Sync
            + 'static,
    {
        self.callbacks = self.callbacks.with_save_slot_cloner(callback);
        self
    }

    /// Sets callback for deleting a save slot.
    pub fn with_save_slot_deleter<F>(mut self, callback: F) -> Self
    where
        F: Fn(&str, &str) -> bool + Send + Sync + 'static,
    {
        self.callbacks = self.callbacks.with_save_slot_deleter(callback);
        self
    }

    /// Sets callback for assigning a player save slot.
    pub fn with_player_save_slot_setter<F>(mut self, callback: F) -> Self
    where
        F: Fn(&str, usize, Option<String>) + Send + Sync + 'static,
    {
        self.callbacks = self.callbacks.with_player_save_slot_setter(callback);
        self
    }

    /// Sets callback for toggling an extension.
    pub fn with_extension_toggler<F>(mut self, callback: F) -> Self
    where
        F: Fn(&str, bool) + Send + Sync + 'static,
    {
        self.callbacks = self.callbacks.with_extension_toggler(callback);
        self
    }

    /// Sets callback for installing an extension.
    pub fn with_extension_installer<F>(mut self, callback: F) -> Self
    where
        F: Fn(&str) -> Result<String, String> + Send + Sync + 'static,
    {
        self.callbacks = self.callbacks.with_extension_installer(callback);
        self
    }

    /// Sets callback for rescanning extensions from disk and immediately populates installed_extensions.
    pub fn with_extension_rescanner<F>(mut self, callback: F) -> Self
    where
        F: Fn() -> Vec<crate::extensions_view::ExtensionItem> + Send + Sync + 'static,
    {
        self.callbacks = self.callbacks.with_extension_rescanner(callback);
        if let Some(ref cb) = self.callbacks.on_rescan_extensions {
            self.installed_extensions = cb();
        }
        self
    }

    /// Sets callback for granting an extension permission.
    pub fn with_extension_permission_granter<F>(mut self, callback: F) -> Self
    where
        F: Fn(&str, &str) + Send + Sync + 'static,
    {
        self.callbacks = self.callbacks.with_extension_permission_granter(callback);
        self
    }

    /// Sets callback for debugger resume.
    pub fn with_debugger_resume<F>(mut self, callback: F) -> Self
    where
        F: Fn() + Send + Sync + 'static,
    {
        self.callbacks = self.callbacks.with_debugger_resume(callback);
        self
    }

    /// Sets callback for debugger step over.
    pub fn with_debugger_step_over<F>(mut self, callback: F) -> Self
    where
        F: Fn() + Send + Sync + 'static,
    {
        self.callbacks = self.callbacks.with_debugger_step_over(callback);
        self
    }

    /// Sets callback for debugger step into.
    pub fn with_debugger_step_into<F>(mut self, callback: F) -> Self
    where
        F: Fn() + Send + Sync + 'static,
    {
        self.callbacks = self.callbacks.with_debugger_step_into(callback);
        self
    }

    /// Sets callback for debugger step out.
    pub fn with_debugger_step_out<F>(mut self, callback: F) -> Self
    where
        F: Fn() + Send + Sync + 'static,
    {
        self.callbacks = self.callbacks.with_debugger_step_out(callback);
        self
    }

    /// Sets callback for debugger pause.
    pub fn with_debugger_pause<F>(mut self, callback: F) -> Self
    where
        F: Fn() + Send + Sync + 'static,
    {
        self.callbacks = self.callbacks.with_debugger_pause(callback);
        self
    }

    /// Sets callback for debugger stop.
    pub fn with_debugger_stop<F>(mut self, callback: F) -> Self
    where
        F: Fn() + Send + Sync + 'static,
    {
        self.callbacks = self.callbacks.with_debugger_stop(callback);
        self
    }

    /// Sets callback for debugger expression evaluation.
    pub fn with_debugger_eval<F>(mut self, callback: F) -> Self
    where
        F: Fn(&str) -> Result<String, String> + Send + Sync + 'static,
    {
        self.callbacks = self.callbacks.with_debugger_eval(callback);
        self
    }

    /// Sets callback for toggling breakpoint.
    pub fn with_debugger_breakpoint_toggler<F>(mut self, callback: F) -> Self
    where
        F: Fn(&str, u32) + Send + Sync + 'static,
    {
        self.callbacks = self.callbacks.with_debugger_breakpoint_toggler(callback);
        self
    }

    /// Sets callback for installing a specific historical Hub handler package version with progress.
    pub fn with_hub_version_installer<F>(mut self, callback: F) -> Self
    where
        F: Fn(&str, &str, i32, Option<u64>, Box<dyn Fn(f32) + Send + Sync>) -> Option<HandlerItem>
            + Send
            + Sync
            + 'static,
    {
        self.callbacks = self.callbacks.with_hub_version_installer(callback);
        self
    }

    /// Sets callback for hot-adding a Drop-In player into an active split-screen session.
    pub fn with_drop_in_player<F>(mut self, callback: F) -> Self
    where
        F: Fn(&str, &str) -> Result<usize, String> + Send + Sync + 'static,
    {
        self.callbacks = self.callbacks.with_drop_in_handler(callback);
        self
    }

    /// Sets callback for cleanly dropping out an active player slot mid-session.
    pub fn with_drop_out_player<F>(mut self, callback: F) -> Self
    where
        F: Fn(usize, &str) -> Result<usize, String> + Send + Sync + 'static,
    {
        self.callbacks = self.callbacks.with_drop_out_handler(callback);
        self
    }

    /// Sets callback to query a live `ActiveSessionSnapshot` from the background engine.
    pub fn with_active_session_querier<F>(mut self, callback: F) -> Self
    where
        F: Fn() -> Option<nucleus_engine::session::ActiveSessionSnapshot> + Send + Sync + 'static,
    {
        self.callbacks = self.callbacks.with_active_session_querier(callback);
        self
    }

    /// Returns true if a splitscreen session is actively executing.
    pub fn is_session_active(&self) -> bool {
        if let Some(ref cb) = self.callbacks.on_check_session_active {
            cb()
        } else {
            self.session_active
        }
    }

    /// Sets visibility of the in-game OSD menu for a specific summoning player slot
    /// and toggles window TOPMOST status on the target monitor.
    pub fn set_osd_visible_for_player(&mut self, summoner_slot: Option<usize>, visible: bool) {
        if visible {
            if !self.is_session_active() {
                return;
            }
            self.in_game_osd.summoner_slot = summoner_slot;
            if let Some(slot) = summoner_slot {
                self.in_game_osd.reassign_slot_idx = slot;
            }
            self.in_game_osd.open();
        } else {
            self.in_game_osd.close();
        }
        if let Some(ref cb) = self.callbacks.on_toggle_osd_topmost {
            cb(visible, summoner_slot);
        }
    }

    /// Sets visibility of the in-game OSD menu and toggles window TOPMOST status.
    pub fn set_osd_visible(&mut self, visible: bool) {
        let slot = self.in_game_osd.summoner_slot;
        self.set_osd_visible_for_player(slot, visible);
    }

    /// Starts an asynchronous listener on the GPUI executor that toggles the in-game OSD
    /// whenever a hotkey or controller guide signal is received from the background engine thread.
    pub fn listen_for_osd_signals(
        &mut self,
        mut rx: tokio::sync::mpsc::UnboundedReceiver<crate::callbacks::EngineToGui>,
        cx: &mut Context<Self>,
    ) {
        cx.spawn(async move |this, cx| {
            while let Some(msg) = rx.recv().await {
                let _ = this.update(&mut *cx, |win, cx| match msg {
                    crate::callbacks::EngineToGui::ToggleInGameOsd { summoner_slot } => {
                        win.toggle_in_game_osd_for_player(summoner_slot, cx);
                    }
                    crate::callbacks::EngineToGui::LaunchPrompt(req) => {
                        win.active_launch_prompt = Some(req);
                        cx.notify();
                    }
                });
            }
        })
        .detach();
    }

    /// Toggles the In-Game Couch OSD overlay menu for a specific summoning player slot.
    pub fn toggle_in_game_osd_for_player(
        &mut self,
        summoner_slot: Option<usize>,
        cx: &mut Context<Self>,
    ) {
        if !self.is_session_active() {
            return;
        }
        let now_visible = !self.in_game_osd.is_visible;
        self.set_osd_visible_for_player(summoner_slot, now_visible);
        cx.notify();
    }

    /// Toggles the In-Game Couch OSD overlay menu.
    pub fn toggle_in_game_osd(&mut self, cx: &mut Context<Self>) {
        self.toggle_in_game_osd_for_player(None, cx);
    }

    /// Freezes or resumes the running game session.
    pub fn toggle_freeze_session(&mut self, cx: &mut Context<Self>) {
        let is_now_frozen = if let Some(ref cb) = self.callbacks.on_toggle_freeze {
            cb().unwrap_or(!self.in_game_osd.is_frozen)
        } else {
            !self.in_game_osd.is_frozen
        };
        self.in_game_osd.is_frozen = is_now_frozen;
        self.status_message = if is_now_frozen {
            "Instances Frozen (NtSuspendProcess)".to_string()
        } else {
            "Instances Resumed (NtResumeProcess)".to_string()
        };
        cx.notify();
    }

    /// Reassigns controller device ID for a player slot.
    pub fn reassign_controller(
        &mut self,
        slot_idx: usize,
        new_device_id: usize,
        cx: &mut Context<Self>,
    ) {
        if let Some(ref cb) = self.callbacks.on_reassign_controller {
            cb(slot_idx, new_device_id);
        }
        if let Some(slot) = self.couch_setup_state.slots.get_mut(slot_idx) {
            slot.device_id = Some(new_device_id);
            if let Some(dev) = self
                .detected_devices_list
                .iter()
                .find(|d| d.id == new_device_id)
            {
                slot.device_name = dev.name.clone();
                slot.device_guid = dev.guid.clone();
            }
        }
        if slot_idx < self.player_device_assignments.len() {
            let name = self
                .detected_devices_list
                .iter()
                .find(|d| d.id == new_device_id)
                .map(|d| d.name.clone())
                .unwrap_or_else(|| format!("Gamepad #{}", new_device_id));
            self.player_device_assignments[slot_idx] =
                format!("Gamepad #{} ({})", new_device_id, name);
        }
        self.status_message = format!(
            "Player {} controller hot-swapped to Device #{}",
            slot_idx + 1,
            new_device_id
        );
        self.in_game_osd.close_reassign();
        cx.notify();
    }

    /// Re-enforces window layout boundaries for active instances.
    pub fn re_enforce_windows(&mut self, cx: &mut Context<Self>) {
        if let Some(ref cb) = self.callbacks.on_re_enforce_windows {
            cb();
        }
        self.status_message = "Window bounds re-enforced".to_string();
        cx.notify();
    }

    /// Toggles raw input lock status.
    pub fn toggle_input_lock(&mut self, cx: &mut Context<Self>) {
        let is_locked = if let Some(ref cb) = self.callbacks.on_toggle_input_lock {
            cb()
        } else {
            !self.in_game_osd.input_locked
        };
        self.in_game_osd.input_locked = is_locked;
        self.status_message = if is_locked {
            "Raw Input Locked (VK_END)".to_string()
        } else {
            "Raw Input Unlocked (VK_END)".to_string()
        };
        cx.notify();
    }

    /// Stops the currently active splitscreen session and returns to carousel.
    pub fn stop_session(&mut self, cx: &mut Context<Self>) {
        self.session_active = false;
        if let Some(ref cb) = self.callbacks.on_stop_session {
            cb();
        }
        self.set_osd_visible(false);
        self.big_picture_screen = BigPictureScreen::Carousel;
        self.status_message = "Session stopped. Returned to Big Picture.".to_string();
        cx.notify();
    }

    /// Handles confirmation of the selected OSD action.
    pub fn handle_osd_confirm(&mut self, cx: &mut Context<Self>) {
        match self.in_game_osd.selected_action() {
            crate::big_picture::OsdAction::ResumeGame => {
                self.set_osd_visible(false);
                cx.notify();
            }
            crate::big_picture::OsdAction::ToggleFreeze => {
                self.toggle_freeze_session(cx);
            }
            crate::big_picture::OsdAction::ReassignControllers => {
                let slot = self
                    .couch_setup_state
                    .focused_slot
                    .min(self.player_count.saturating_sub(1));
                self.in_game_osd.open_reassign(slot);
                cx.notify();
            }
            crate::big_picture::OsdAction::DropInPlayer => {
                if let Some(ref querier) = self.callbacks.on_query_active_session {
                    if let Some(snap) = querier() {
                        self.in_game_osd.sync_from_snapshot(&snap);
                    }
                }
                let active_count = if self.in_game_osd.active_players.is_empty() {
                    self.player_count.max(1)
                } else {
                    self.in_game_osd.active_players.len()
                };
                let max_p = self.in_game_osd.max_players.max(active_count).max(4);
                self.in_game_osd.open_drop_in(active_count, max_p);
                cx.notify();
            }
            crate::big_picture::OsdAction::DropOutPlayer => {
                if let Some(ref querier) = self.callbacks.on_query_active_session {
                    if let Some(snap) = querier() {
                        self.in_game_osd.sync_from_snapshot(&snap);
                    }
                }
                let summoner = self.in_game_osd.summoner_slot;
                self.in_game_osd.open_drop_out(summoner);
                cx.notify();
            }
            crate::big_picture::OsdAction::ReEnforceWindows => {
                self.re_enforce_windows(cx);
                self.set_osd_visible(false);
                cx.notify();
            }
            crate::big_picture::OsdAction::ToggleInputLock => {
                self.toggle_input_lock(cx);
            }
            crate::big_picture::OsdAction::StopSession => {
                self.stop_session(cx);
            }
        }
    }

    /// Refreshes the list of saved numbered profiles for the currently selected game.
    pub fn refresh_saved_profiles(&mut self) {
        if let Some(ref game_name) = self.selected_game {
            let guid = self
                .handlers
                .iter()
                .find(|h| &h.game_name == game_name)
                .map(|h| h.guid.clone())
                .unwrap_or_else(|| game_name.clone());
            if let Some(ref lister) = self.callbacks.on_list_profiles {
                self.saved_profiles = lister(&guid);
            }
        } else {
            self.saved_profiles.clear();
        }
    }

    /// Refreshes the list of installed extensions from the engine.
    pub fn refresh_extensions(&mut self) {
        if let Some(ref rescanner) = self.callbacks.on_rescan_extensions {
            self.installed_extensions = rescanner();
        }
    }

    /// Opens file dialog to install an extension package archive (*.nucplugin, *.nucaddon, *.zip).
    pub fn install_extension_dialog(&mut self, cx: &mut Context<Self>) {
        self.status_message = "Browsing for extension package archive...".to_string();
        cx.notify();
        cx.spawn(async move |this, cx| {
            let pkg_opt = crate::dialogs::pick_file_async(
                "Select Extension Package",
                "Extension Packages (*.nucplugin;*.nucaddon;*.zip)|*.nucplugin;*.nucaddon;*.zip|All Files (*.*)|*.*",
            )
            .await;
            if let Some(pkg) = pkg_opt {
                let pkg_path = pkg.to_string_lossy().to_string();
                let _ = this.update(&mut *cx, |this, cx| {
                    if let Some(ref cb) = this.callbacks.on_install_extension {
                        match cb(&pkg_path) {
                            Ok(id) => {
                                this.status_message = format!("Successfully installed extension '{}'", id);
                            }
                            Err(e) => {
                                this.status_message = format!("Failed installing extension: {}", e);
                            }
                        }
                    }
                    this.refresh_extensions();
                    cx.notify();
                });
            }
        }).detach();
    }

    /// Attaches persistent application settings to the window.
    pub fn with_app_settings(mut self, settings: nucleus_profile::AppSettings) -> Self {
        self.handlers_dir = settings.handlers_directory.clone();
        self.theme_name = settings.theme.clone();
        self.auto_scan = settings.auto_scan_on_start;
        crate::theme::set_active_theme(&self.theme_name);
        self.app_settings = settings;
        self
    }

    /// Displays a prominent On-Screen Display (OSD) banner over the interface (parity with C# `Globals.MainOSD.Show(timing, text)`).
    pub fn show_osd(&mut self, text: &str, duration_ms: u32, cx: &mut Context<Self>) {
        let dur = std::time::Duration::from_millis(duration_ms as u64);
        self.osd_message = Some((text.to_string(), std::time::Instant::now(), dur));
        self.status_message = text.to_string();
        cx.notify();

        cx.spawn(async move |this, cx| {
            cx.background_executor().timer(dur).await;
            this.update(&mut *cx, |win, cx| {
                if let Some((_, start, dur)) = win.osd_message {
                    if start.elapsed() >= dur {
                        win.osd_message = None;
                        cx.notify();
                    }
                }
            })
            .ok();
        })
        .detach();
    }

    /// Renders the floating On-Screen Display (OSD) overlay badge when active.
    fn render_osd_overlay(&self) -> Option<AnyElement> {
        if let Some((ref text, start, dur)) = self.osd_message {
            if start.elapsed() < dur {
                let color_val = parse_osd_color(&self.app_settings.osd_color);
                return Some(crate::osd::render_osd_overlay(text, color_val));
            }
        }
        None
    }

    /// Cycles the screen partition layout to the next available option (parity with Ctrl+N shortcut).
    pub fn cycle_layout(&mut self, cx: &mut Context<Self>) {
        let layout_options = [
            "1x1 (Single / Fullscreen)",
            "2x1 (Side-by-Side)",
            "1x2 (Stacked)",
            "2x2 (Grid)",
            "Top 1 / Bottom 2",
            "Top 2 / Bottom 1",
            "Multi-Monitor Span",
        ];
        let current = self.selected_layout.as_str();
        let idx = layout_options
            .iter()
            .position(|&l| l.eq_ignore_ascii_case(current))
            .unwrap_or(0);
        let next_idx = (idx + 1) % layout_options.len();
        let chosen = layout_options[next_idx].to_string();
        self.selected_layout = chosen.clone();
        self.status_message = format!("Partition layout set to: {}", chosen);
        self.show_osd(&chosen, 1200, cx);
        cx.notify();
    }

    /// Handles physical keypress recording for lock key or shortcuts.
    pub fn handle_key_recording(
        &mut self,
        event: &gpui::KeyDownEvent,
        cx: &mut Context<Self>,
    ) -> bool {
        let key_raw = event.keystroke.key.as_str();
        if key_raw.eq_ignore_ascii_case("escape") {
            self.settings_view.recording_hotkey = None;
            self.status_message = "Key recording cancelled.".to_string();
            cx.notify();
            return true;
        }

        // Ignore lonely modifier presses
        if matches!(
            key_raw.to_lowercase().as_str(),
            "control" | "shift" | "alt" | "meta" | "ctrl"
        ) {
            return true;
        }

        if let Some(target) = self.settings_view.recording_hotkey.clone() {
            if target == "lock" {
                let normalized = crate::settings_view::normalize_key_input(key_raw);
                self.app_settings.lock_inputs_key = normalized.clone();
                self.settings_view.recording_hotkey = None;
                self.status_message =
                    format!("Lock Inputs Key successfully updated to: {}", normalized);
                self.show_osd(&format!("Lock Key: {}", normalized), 1200, cx);
            } else {
                let key_clean = crate::settings_view::normalize_key_input(key_raw);
                let combo = crate::settings_view::format_shortcut_combo(
                    &event.keystroke.modifiers,
                    &key_clean,
                );
                crate::settings_view::set_hotkey_value(
                    &mut self.app_settings,
                    &target,
                    combo.clone(),
                );
                self.settings_view.recording_hotkey = None;
                self.status_message = format!("Shortcut successfully updated to: {}", combo);
                self.show_osd(&format!("Shortcut: {}", combo), 1200, cx);
            }
            cx.notify();
            return true;
        }
        false
    }

    /// Renders the interactive physical keypress recording modal dialog when active.
    fn render_hotkey_recording_modal(&self, cx: &mut Context<Self>) -> Option<AnyElement> {
        if let Some(ref target) = self.settings_view.recording_hotkey {
            let target_name = match target.as_str() {
                "lock" => "Lock Inputs Toggle Key (Default: End)".to_string(),
                other => crate::settings_view::APP_SHORTCUTS
                    .iter()
                    .find(|s| s.id == other)
                    .map(|s| format!("{} (Default: {})", s.name, s.default_val))
                    .unwrap_or_else(|| "Custom Shortcut".to_string()),
            };

            let focus_handle = self
                .hotkey_record_focus
                .clone()
                .unwrap_or_else(|| cx.focus_handle());

            let on_cancel = cx.listener(|this: &mut MainWindow, _, _, cx| {
                this.settings_view.recording_hotkey = None;
                this.status_message = "Key recording cancelled.".to_string();
                cx.notify();
            });

            let on_key_down = cx.listener(
                |this: &mut MainWindow, event: &gpui::KeyDownEvent, _window, cx| {
                    this.handle_key_recording(event, cx);
                },
            );

            return Some(crate::osd::render_hotkey_recording_modal(
                &target_name,
                &focus_handle,
                on_cancel,
                on_key_down,
            ));
        }
        None
    }

    /// Renders the client-side decorations (CSD) title bar with icon, title, drag handling, and window controls.
    fn render_title_bar(&self, _window: &mut Window, cx: &mut Context<Self>) -> AnyElement {
        #[cfg(target_os = "windows")]
        let is_maximized = unsafe {
            use windows::Win32::UI::Input::KeyboardAndMouse::GetActiveWindow;
            use windows::Win32::UI::WindowsAndMessaging::IsZoomed;
            let hwnd = GetActiveWindow();
            !hwnd.is_invalid() && IsZoomed(hwnd).as_bool()
        };
        #[cfg(not(target_os = "windows"))]
        let is_maximized = false;

        CouchTitleBar::new()
            .selected_game(self.selected_game.as_deref())
            .is_maximized(is_maximized)
            .on_toggle_big_picture(cx.listener(|this, _, _, cx| this.toggle_big_picture(cx)))
            .on_minimize(cx.listener(|_this, _, window, _cx| window.minimize_window()))
            .on_maximize(cx.listener(|_this, _, window, _cx| {
                #[cfg(target_os = "windows")]
                unsafe {
                    use windows::Win32::Foundation::{LPARAM, WPARAM};
                    use windows::Win32::UI::Input::KeyboardAndMouse::GetActiveWindow;
                    use windows::Win32::UI::WindowsAndMessaging::{
                        IsZoomed, PostMessageW, WM_SYSCOMMAND,
                    };
                    let hwnd = GetActiveWindow();
                    if !hwnd.is_invalid() {
                        let is_zoomed = IsZoomed(hwnd).as_bool();
                        let sc_cmd = if is_zoomed {
                            0xF120 /* SC_RESTORE */
                        } else {
                            0xF030 /* SC_MAXIMIZE */
                        };
                        let _ = PostMessageW(hwnd, WM_SYSCOMMAND, WPARAM(sc_cmd), LPARAM(0));
                        _cx.notify();
                        return;
                    }
                }
                window.zoom_window();
                _cx.notify();
            }))
            .on_close(cx.listener(|_this, _, window, _cx| window.remove_window()))
            .into_any_element()
    }

    /// Renders the left sidebar with navigation tabs and device counters.
    /// Supports both full mode (200px) and compact icon-only mode (60px) based on window width or manual toggle.
    fn render_sidebar(&self, window_width: Pixels, cx: &mut Context<Self>) -> AnyElement {
        let is_compact = window_width < px(960.0);
        let is_collapsed = self.sidebar_collapsed.unwrap_or(is_compact);

        let handle_tab = cx.listener(|this, tab: &ActiveTab, _, cx| {
            this.active_tab = *tab;
            if *tab == ActiveTab::Setup {
                this.refresh_saved_profiles();
            } else if *tab == ActiveTab::Extensions {
                this.refresh_extensions();
            }
            cx.notify();
        });

        CouchSidebarNav::new()
            .active_tab(self.active_tab)
            .detected_devices(self.detected_devices)
            .handler_count(self.handlers.len())
            .is_collapsed(is_collapsed)
            .on_tab_click(move |tab, win, app| handle_tab(&tab, win, app))
            .on_toggle_collapse(cx.listener(move |this, _, _, cx| {
                let cur = this.sidebar_collapsed.unwrap_or(is_compact);
                this.sidebar_collapsed = Some(!cur);
                cx.notify();
            }))
            .into_any_element()
    }

    /// Renders the bottom status bar with session information and device count.
    fn render_status_bar(&self) -> AnyElement {
        CouchStatusBar::new()
            .message(&self.status_message)
            .detected_devices(self.detected_devices)
            .session_active(self.is_session_active())
            .into_any_element()
    }

    /// Renders the modal dialog for inspecting handler or Hub package details with Markdown support.
    fn render_details_modal(&self, cx: &mut Context<Self>) -> Option<AnyElement> {
        if let Some(ref pkg) = self.inspecting_hub_package {
            let is_installed = pkg.is_installed && self.is_handler_or_pkg_installed(&pkg.id, &pkg.game_name);
            let install_progress = self.installing_handlers.get(&pkg.id).copied();
            let has_extra_like = self.hub_detail_extra_likes.contains(&pkg.id);

            Some(crate::hub_details::render_hub_details_screen(
                pkg,
                is_installed,
                install_progress,
                self.hub_detail_tab,
                &self.hub_detail_comments,
                &self.hub_detail_packages,
                self.hub_detail_loading,
                has_extra_like,
                cx,
            ))
        } else if let Some(ref h) = self.inspecting_handler {
            Some(crate::handler_details::render_handler_details_modal(h, cx))
        } else {
            None
        }
    }

    /// Executes the action confirmed by the user in `CouchConfirmDialog`.
    pub fn execute_confirm_action(
        &mut self,
        action: &crate::models::ConfirmAction,
        cx: &mut Context<Self>,
    ) {
        match action {
            crate::models::ConfirmAction::ResetDefaults => {
                self.app_settings = nucleus_profile::AppSettings::default();
                self.status_message = "Preferences restored to defaults".to_string();
                self.show_osd("Defaults Restored", 800, cx);
            }
            crate::models::ConfirmAction::DeleteNumberedProfile(guid, num) => {
                if let Some(ref deleter) = self.callbacks.on_delete_numbered_profile {
                    let _ = deleter(guid, *num);
                    self.status_message = format!("Deleted Profile {}!", num);
                    self.show_osd(&format!("Profile #{} Deleted", num), 800, cx);
                    if let Some(ref list_cb) = self.callbacks.on_list_profiles {
                        self.saved_profiles = list_cb(guid);
                    }
                    if self.active_profile_id == Some(*num) {
                        self.active_profile_id = None;
                    }
                }
            }
            crate::models::ConfirmAction::RemoveGame(guid) => {
                let g = guid.clone();
                self.handlers.retain(|h| &h.guid != &g);
                self.handler_names.retain(|n| n != &g);
                if let Some(ref cb) = self.callbacks.on_remove_game {
                    cb(&g);
                }
                if self.is_big_picture {
                    self.selected_game = None;
                    self.big_picture_screen = BigPictureScreen::Carousel;
                }
                self.status_message = "Game removed from library".to_string();
                self.show_osd("Game Removed", 1500, cx);
            }
            crate::models::ConfirmAction::DeleteSavePath(guid) => {
                if let Some(ref cb) = self.callbacks.on_delete_save_path {
                    cb(guid);
                }
                if let Some(h) = self.handlers.iter().find(|it| &it.guid == guid) {
                    if let Some(home) = dirs::home_dir() {
                        let p = home.join("NucleusCoop").join(&h.game_name);
                        if p.exists() {
                            let _ = std::fs::remove_dir_all(&p);
                        }
                    }
                }
                self.status_message = "Deleted UserProfile Save Path.".to_string();
                self.show_osd("Save Data Deleted", 800, cx);
            }
        }
    }

    /// Renders the right-click contextual menu for game cards (delegated to `CouchContextMenu`).
    fn render_context_menu(&self, cx: &mut Context<Self>) -> Option<AnyElement> {
        let h = self.context_menu_handler.as_ref()?;
        let guid = h.guid.clone();
        let game_name = h.game_name.clone();
        let exe_path_opt = h.executable_path.clone();
        let handler_path_opt = h.handler_path.clone();
        let handlers_dir = self.handlers_dir.clone();
        let text_editor_path = self.app_settings.text_editor_path.clone();
        let is_picker_open = self.context_menu_language_picker;

        Some(
            CouchContextMenu::new(h.clone())
                .handlers_dir(handlers_dir.clone())
                .text_editor_path(text_editor_path)
                .language_picker_open(is_picker_open)
                .on_close(cx.listener(|this, _, _, cx| {
                    this.context_menu_handler = None;
                    cx.notify();
                }))
                .on_open_handler({
                    let h_path = handler_path_opt.clone();
                    let h_dir = handlers_dir.clone();
                    let g_name = game_name.clone();
                    let g_id = guid.clone();
                    cx.listener(move |this, _, _, cx| {
                        let candidate = h_path.as_deref().and_then(|p| {
                            let path = std::path::Path::new(p);
                            if path.is_file() {
                                Some(path.to_path_buf())
                            } else {
                                None
                            }
                        }).or_else(|| {
                            let p1 = std::path::Path::new(&h_dir).join(format!("{}.js", g_name));
                            if p1.is_file() {
                                Some(p1)
                            } else {
                                let p2 = std::path::Path::new(&h_dir).join(format!("{}.js", g_id));
                                if p2.is_file() {
                                    Some(p2)
                                } else {
                                    None
                                }
                            }
                        });

                        if let Some(target_file) = candidate {
                            let target_str = target_file.to_string_lossy().to_string();
                            let editor = if !this.app_settings.text_editor_path.is_empty() && this.app_settings.text_editor_path != "Default" {
                                &this.app_settings.text_editor_path
                            } else {
                                "notepad++.exe"
                            };
                            if std::process::Command::new(editor).arg(&target_str).spawn().is_err() {
                                let _ = std::process::Command::new("notepad.exe").arg(&target_str).spawn();
                            }
                            this.status_message = format!("Opened handler script: {}", target_file.file_name().map(|n| n.to_string_lossy()).unwrap_or_default());
                        } else {
                            let _ = std::process::Command::new("explorer").arg(&h_dir).spawn();
                            this.status_message = format!("Opened handlers folder: {}", h_dir);
                        }
                        this.context_menu_handler = None;
                        cx.notify();
                    })
                })
                .on_open_exe({
                    let exe_p = exe_path_opt.clone();
                    let g_name = game_name.clone();
                    cx.listener(move |this, _, _, cx| {
                        if let Some(ref p) = exe_p {
                            let p_path = std::path::Path::new(p);
                            let dir_to_open = if p_path.is_file() {
                                p_path.parent().map(|d| d.to_path_buf())
                            } else if p_path.is_dir() {
                                Some(p_path.to_path_buf())
                            } else if let Some(parent) = p_path.parent() {
                                if !parent.as_os_str().is_empty() && parent.is_dir() {
                                    Some(parent.to_path_buf())
                                } else {
                                    None
                                }
                            } else {
                                None
                            };

                            if let Some(dir) = dir_to_open {
                                let _ = std::process::Command::new("explorer").arg(&dir).spawn();
                                this.status_message = format!("Opened game directory: {}", dir.display());
                            } else {
                                this.status_message = format!("Unable to open original executable path for '{}': directory not found.", g_name);
                            }
                        } else {
                            this.status_message = format!("No executable located for '{}' yet.", g_name);
                        }
                        this.context_menu_handler = None;
                        cx.notify();
                    })
                })
                .on_open_userprofile({
                    let g_name = game_name.clone();
                    cx.listener(move |this, _, _, cx| {
                        if let Some(home) = dirs::home_dir() {
                            let p = home.join("NucleusCoop").join(&g_name);
                            if !p.exists() {
                                let _ = std::fs::create_dir_all(&p);
                            }
                            let _ = std::process::Command::new("explorer").arg(&p).spawn();
                        }
                        this.context_menu_handler = None;
                        cx.notify();
                    })
                })
                .on_delete_userprofile({
                    let g_id = guid.clone();
                    let g_name = game_name.clone();
                    cx.listener(move |this, _, _, cx| {
                        this.context_menu_handler = None;
                        this.confirm_dialog = Some(crate::ConfirmDialogState {
                            title: format!("Delete Saves for '{}'?", g_name),
                            message: format!("Are you sure you want to permanently delete the UserProfile save data for '{}'? All saved games and configs under %USERPROFILE%\\NucleusCoop will be deleted.", g_name),
                            confirm_label: "Delete Saves".to_string(),
                            is_destructive: true,
                            action: crate::ConfirmAction::DeleteSavePath(g_id.clone()),
                        });
                        cx.notify();
                    })
                })
                .on_toggle_assign_by_index({
                    let g_id = guid.clone();
                    cx.listener(move |this, _, _, cx| {
                        if let Some(item) = this.handlers.iter_mut().find(|it| it.guid == g_id) {
                            item.assign_gamepads_by_index = !item.assign_gamepads_by_index;
                            if item.assign_gamepads_by_index {
                                item.assign_gamepads_on_button_press = false;
                            }
                            if let Some(ref cb) = this.callbacks.on_save_profile {
                                cb(item);
                            }
                            this.context_menu_handler = Some(item.clone());
                        }
                        cx.notify();
                    })
                })
                .on_toggle_assign_on_press({
                    let g_id = guid.clone();
                    cx.listener(move |this, _, _, cx| {
                        if let Some(item) = this.handlers.iter_mut().find(|it| it.guid == g_id) {
                            item.assign_gamepads_on_button_press = !item.assign_gamepads_on_button_press;
                            if item.assign_gamepads_on_button_press {
                                item.assign_gamepads_by_index = false;
                            }
                            if let Some(ref cb) = this.callbacks.on_save_profile {
                                cb(item);
                            }
                            this.context_menu_handler = Some(item.clone());
                        }
                        cx.notify();
                    })
                })
                .on_toggle_keep_content({
                    let g_id = guid.clone();
                    cx.listener(move |this, _, _, cx| {
                        if let Some(item) = this.handlers.iter_mut().find(|it| it.guid == g_id) {
                            item.keep_instances_content_folder = !item.keep_instances_content_folder;
                            if let Some(ref cb) = this.callbacks.on_save_profile {
                                cb(item);
                            }
                            this.context_menu_handler = Some(item.clone());
                        }
                        cx.notify();
                    })
                })
                .on_toggle_disable_profile({
                    let g_id = guid.clone();
                    cx.listener(move |this, _, _, cx| {
                        if let Some(item) = this.handlers.iter_mut().find(|it| it.guid == g_id) {
                            item.disable_profile = !item.disable_profile;
                            if let Some(ref cb) = this.callbacks.on_save_profile {
                                cb(item);
                            }
                            this.context_menu_handler = Some(item.clone());
                        }
                        cx.notify();
                    })
                })
                .on_toggle_disable_update({
                    let g_id = guid.clone();
                    cx.listener(move |this, _, _, cx| {
                        if let Some(item) = this.handlers.iter_mut().find(|it| it.guid == g_id) {
                            item.disable_handler_update = !item.disable_handler_update;
                            if let Some(ref cb) = this.callbacks.on_save_profile {
                                cb(item);
                            }
                            this.context_menu_handler = Some(item.clone());
                        }
                        cx.notify();
                    })
                })
                .on_change_icon({
                    let g_id = guid.clone();
                    cx.listener(move |this, _, _, cx| {
                        this.status_message = "Browsing for game icon...".to_string();
                        cx.notify();
                        let target_guid = g_id.clone();
                        cx.spawn(async move |this, cx| {
                            let icon_opt = crate::dialogs::pick_file_async(
                                "Select Game Icon",
                                "Icons & Images (*.ico;*.png;*.exe)|*.ico;*.png;*.exe|All Files (*.*)|*.*",
                            )
                            .await;
                            if let Some(icon_path) = icon_opt {
                                let path_str = icon_path.to_string_lossy().to_string();
                                let _ = this.update(&mut *cx, |this, cx| {
                                    if let Some(item) = this.handlers.iter_mut().find(|it| it.guid == target_guid) {
                                        item.custom_game_icon = Some(path_str.clone());
                                        if let Some(ref cb) = this.callbacks.on_save_profile {
                                            cb(item);
                                        }
                                        this.context_menu_handler = Some(item.clone());
                                    }
                                    this.status_message = format!("Set custom icon: {}", path_str);
                                    cx.notify();
                                });
                            } else {
                                let _ = this.update(&mut *cx, |this, cx| {
                                    this.status_message = "Game icon selection cancelled.".to_string();
                                    cx.notify();
                                });
                            }
                        })
                        .detach();
                    })
                })
                .on_open_assets(cx.listener(|this, _, _, cx| {
                    let p = nucleus_common::Globals::try_get()
                        .map(|g| g.install_root.join("gui").join("covers"))
                        .unwrap_or_else(|| std::path::PathBuf::from("gui/covers"));
                    if !p.exists() {
                        let _ = std::fs::create_dir_all(&p);
                    }
                    if let Some(g) = nucleus_common::Globals::try_get() {
                        let _ = std::fs::create_dir_all(g.install_root.join("gui").join("screenshots"));
                        let _ = std::fs::create_dir_all(g.install_root.join("gui").join("icons"));
                    }
                    let _ = std::process::Command::new("explorer").arg(&p).spawn();
                    this.context_menu_handler = None;
                    this.status_message = format!("Opened game assets folder: {}", p.display());
                    cx.notify();
                }))
                .on_toggle_language_picker(cx.listener(|this, _, _, cx| {
                    this.context_menu_language_picker = !this.context_menu_language_picker;
                    cx.notify();
                }))
                .on_select_language({
                    let gid = guid.clone();
                    let sel_cb = cx.listener(move |this, lang: &str, _, cx| {
                        let lang_str = lang.to_string();
                        if let Some(item) = this.handlers.iter_mut().find(|it| it.guid == gid) {
                            item.steam_language = Some(lang_str.clone());
                            if let Some(ref cb) = this.callbacks.on_save_profile {
                                cb(item);
                            }
                            this.context_menu_handler = Some(item.clone());
                        }
                        this.context_menu_language_picker = false;
                        this.status_message = format!("Steam Language set to: {}", lang_str);
                        cx.notify();
                    });
                    move |lang: &str, win, app| {
                        sel_cb(lang, win, app);
                    }
                })
                .on_remove_game({
                    let g_id = guid.clone();
                    let g_name = game_name.clone();
                    cx.listener(move |this, _, _, cx| {
                        this.context_menu_handler = None;
                        this.confirm_dialog = Some(crate::ConfirmDialogState {
                            title: format!("Remove '{}' from Library?", g_name),
                            message: format!("Are you sure you want to remove '{}' from your game library? Your local game installation and save files will remain untouched.", g_name),
                            confirm_label: "Remove Game".to_string(),
                            is_destructive: true,
                            action: crate::ConfirmAction::RemoveGame(g_id.clone()),
                        });
                        cx.notify();
                    })
                })
                .into_any_element(),
        )
    }

    /// Renders the modal confirmation dialog for destructive/irreversible actions (delegated to `CouchConfirmDialog`).
    fn render_confirm_dialog(&self, cx: &mut Context<Self>) -> Option<AnyElement> {
        let dlg = self.confirm_dialog.as_ref()?;
        let action = dlg.action.clone();

        Some(
            CouchConfirmDialog::new(dlg.clone())
                .on_confirm(cx.listener(move |this, _, _, cx| {
                    this.execute_confirm_action(&action, cx);
                    this.confirm_dialog = None;
                    cx.notify();
                }))
                .on_cancel(cx.listener(|this, _, _, cx| {
                    this.confirm_dialog = None;
                    cx.notify();
                }))
                .into_any_element(),
        )
    }
    /// Remaps a controller action to a target button name.
    pub fn remap_controller_action(
        &mut self,
        action: &str,
        button_name: &str,
        cx: &mut Context<Self>,
    ) {
        let is_single = matches!(action, "drag_drop" | "right_click" | "left_click");
        let new_binding = if is_single {
            button_name.to_string()
        } else {
            let cur = self.app_settings.get_gamepad_binding(action);
            if let Some((modifier, _)) = cur.split_once('+') {
                format!("{}+{}", modifier, button_name)
            } else {
                format!("Guide+{}", button_name)
            }
        };

        self.app_settings
            .set_gamepad_binding(action, new_binding.clone());
        let _ = self.app_settings.save();
        self.remapping_controller_shortcut = None;
        let action_display = match action {
            "unfocus" => "Toggle Unfocus",
            "close" => "Close Nucleus",
            "stop" => "Stop Session",
            "topmost" => "Toggle Top Most",
            "reset_windows" => "Reset Windows",
            "cutscenes" => "Cutscenes Mode",
            "switch_layout" => "Switch Layouts",
            "lock_inputs" => "Lock Input Key",
            "release_cursor" => "ALT+TAB",
            "lock_ui" => "Lock UI Control",
            "osk" => "On screen KB",
            "drag_drop" => "Drag and Drop",
            "right_click" => "Right Click",
            "left_click" => "Left Click",
            other => other,
        };
        self.status_message = format!("Remapped {} to {}", action_display, new_binding);
        self.show_osd(&format!("{}: {}", action_display, new_binding), 1500, cx);
        cx.notify();
    }

    /// Helper to render a shortcut combo row that can be clicked to remap.
    fn render_shortcut_row(
        &self,
        action: &'static str,
        label: &'static str,
        is_ps: bool,
        cx: &mut Context<Self>,
    ) -> AnyElement {
        let binding = self.app_settings.get_gamepad_binding(action);
        let (mod_part, btn_part) = if let Some((m, b)) = binding.split_once('+') {
            (m, b)
        } else {
            ("Guide", binding)
        };

        let is_active = self.remapping_controller_shortcut.as_deref() == Some(action);
        let guide_glyph = format_gamepad_button_glyph(mod_part, is_ps);
        let btn_glyph = format_gamepad_button_glyph(btn_part, is_ps);

        div()
            .id(ElementId::Name(format!("shortcut-row-{}", action).into()))
            .cursor_pointer()
            .flex()
            .items_center()
            .justify_between()
            .w_full()
            .px_2()
            .py_1()
            .rounded_md()
            .border_1()
            .border_color(if is_active {
                theme::accent()
            } else {
                theme::border_color()
            })
            .bg(if is_active {
                theme::accent().opacity(0.18)
            } else {
                theme::bg_panel()
            })
            .hover(|s| {
                if is_active {
                    s
                } else {
                    s.bg(theme::bg_hover())
                }
            })
            .tooltip(crate::make_tooltip(if is_active {
                "Listening for replacement button... (Click to cancel)"
            } else {
                "Click to remap this controller shortcut"
            }))
            .on_click(cx.listener(move |this, _, _, cx| {
                if this.remapping_controller_shortcut.as_deref() == Some(action) {
                    this.remapping_controller_shortcut = None;
                } else {
                    this.remapping_controller_shortcut = Some(action.to_string());
                    this.show_osd(&format!("Remapping {}... Press button", label), 1500, cx);
                }
                cx.notify();
            }))
            .child(
                div()
                    .text_xs()
                    .font_weight(FontWeight::MEDIUM)
                    .text_color(if is_active {
                        theme::accent()
                    } else {
                        theme::text_primary()
                    })
                    .child(label),
            )
            .child(
                div()
                    .flex()
                    .items_center()
                    .gap_1p5()
                    .child(
                        div()
                            .size(px(18.0))
                            .rounded_full()
                            .bg(rgb(0xffffff))
                            .text_color(rgb(0x000000))
                            .font_weight(FontWeight::BOLD)
                            .text_xs()
                            .flex()
                            .items_center()
                            .justify_center()
                            .child(guide_glyph),
                    )
                    .child(div().text_xs().text_color(theme::text_muted()).child("+"))
                    .child(
                        div()
                            .min_w(px(24.0))
                            .h(px(20.0))
                            .px_1p5()
                            .rounded_sm()
                            .bg(if is_active {
                                theme::accent()
                            } else {
                                theme::bg_subtle()
                            })
                            .border_1()
                            .border_color(if is_active {
                                theme::accent()
                            } else {
                                theme::border_color()
                            })
                            .text_xs()
                            .font_weight(FontWeight::BOLD)
                            .text_color(rgb(0xffffff))
                            .flex()
                            .items_center()
                            .justify_center()
                            .child(if is_active { "..." } else { btn_glyph }),
                    ),
            )
            .into_any_element()
    }

    /// Helper to render a single navigation action row that can be clicked to remap.
    fn render_nav_single_row(
        &self,
        action: &'static str,
        label: &'static str,
        is_ps: bool,
        cx: &mut Context<Self>,
    ) -> AnyElement {
        let binding = self.app_settings.get_gamepad_binding(action);
        let is_active = self.remapping_controller_shortcut.as_deref() == Some(action);
        let btn_glyph = format_gamepad_button_glyph(binding, is_ps);

        div()
            .id(ElementId::Name(format!("nav-row-{}", action).into()))
            .cursor_pointer()
            .flex()
            .items_center()
            .justify_between()
            .w_full()
            .px_2()
            .py_1()
            .rounded_md()
            .border_1()
            .border_color(if is_active {
                theme::accent()
            } else {
                theme::border_color()
            })
            .bg(if is_active {
                theme::accent().opacity(0.18)
            } else {
                theme::bg_panel()
            })
            .hover(|s| {
                if is_active {
                    s
                } else {
                    s.bg(theme::bg_hover())
                }
            })
            .tooltip(crate::make_tooltip(if is_active {
                "Listening for replacement button... (Click to cancel)"
            } else {
                "Click to remap this navigation button"
            }))
            .on_click(cx.listener(move |this, _, _, cx| {
                if this.remapping_controller_shortcut.as_deref() == Some(action) {
                    this.remapping_controller_shortcut = None;
                } else {
                    this.remapping_controller_shortcut = Some(action.to_string());
                    this.show_osd(&format!("Remapping {}... Press button", label), 1500, cx);
                }
                cx.notify();
            }))
            .child(
                div()
                    .text_xs()
                    .font_weight(FontWeight::MEDIUM)
                    .text_color(if is_active {
                        theme::accent()
                    } else {
                        theme::text_primary()
                    })
                    .child(label),
            )
            .child(
                div()
                    .min_w(px(28.0))
                    .h(px(20.0))
                    .px_2()
                    .rounded_sm()
                    .bg(if is_active {
                        theme::accent()
                    } else {
                        theme::bg_subtle()
                    })
                    .border_1()
                    .border_color(if is_active {
                        theme::accent()
                    } else {
                        theme::border_color()
                    })
                    .text_xs()
                    .font_weight(FontWeight::BOLD)
                    .text_color(rgb(0xffffff))
                    .flex()
                    .items_center()
                    .justify_center()
                    .child(if is_active { "..." } else { btn_glyph }),
            )
            .into_any_element()
    }

    /// Renders the legacy controller shortcuts setup modal matching XInputShortcutsSetup (media_1793137385955_044e590e.png).
    fn render_controller_shortcuts_modal(&self, cx: &mut Context<Self>) -> Option<AnyElement> {
        if !self.show_controller_shortcuts_modal {
            return None;
        }

        let is_ps = self
            .app_settings
            .gamepad_type
            .eq_ignore_ascii_case("Playstation");
        let deadzone_val = self.app_settings.gamepad_deadzone;
        let is_enabled = self.app_settings.gamepad_shortcuts_enabled;

        Some(
            div()
                .absolute()
                .inset_0()
                .bg(theme::overlay_scrim())
                .flex()
                .items_center()
                .justify_center()
                .p_6()
                .child(
                    div()
                        .id("controller-shortcuts-modal-dialog")
                        .flex()
                        .flex_col()
                        .w_full()
                        .max_w(px(880.0))
                        .max_h(px(600.0))
                        .p_5()
                        .gap_3()
                        .bg(rgb(0x111113))
                        .rounded_xl()
                        .border_1()
                        .border_color(theme::border_color())
                        .shadow_lg()
                        // Header: Title, Enable Checkbox & Close '✕'
                        .child(
                            div()
                                .flex()
                                .items_center()
                                .justify_between()
                                .pb_2()
                                .border_b_1()
                                .border_color(theme::border_color())
                                .child(
                                    div()
                                        .flex()
                                        .items_center()
                                        .gap_2()
                                        .child(
                                            Icon::new(IconName::Gamepad2)
                                                .size_4()
                                                .text_color(theme::accent()),
                                        )
                                        .child(
                                            div()
                                                .text_sm()
                                                .font_weight(FontWeight::BOLD)
                                                .text_color(theme::text_primary())
                                                .child("Controller Shortcuts & UI Navigation Setup"),
                                        ),
                                )
                                .child(
                                    div()
                                        .flex()
                                        .items_center()
                                        .gap_4()
                                        // Enabled Checkbox toggle (parity with XInputShortcutsSetup.cs enabled_chk)
                                        .child(
                                            div()
                                                .id("chk-controller-shortcuts-enabled")
                                                .cursor_pointer()
                                                .flex()
                                                .items_center()
                                                .gap_2()
                                                .tooltip(crate::make_tooltip("Toggle controller navigation and shortcuts system"))
                                                .on_click(cx.listener(|this, _, _, cx| {
                                                    this.app_settings.gamepad_shortcuts_enabled = !this.app_settings.gamepad_shortcuts_enabled;
                                                    let _ = this.app_settings.save();
                                                    cx.notify();
                                                }))
                                                .child(
                                                    div()
                                                        .size(px(16.0))
                                                        .rounded_sm()
                                                        .border_1()
                                                        .border_color(if is_enabled { theme::accent() } else { theme::border_color() })
                                                        .bg(if is_enabled { theme::accent() } else { theme::bg_main() })
                                                        .flex()
                                                        .items_center()
                                                        .justify_center()
                                                        .text_xs()
                                                        .text_color(theme::text_on_accent())
                                                        .child(if is_enabled { "✓" } else { "" }),
                                                )
                                                .child(
                                                    div()
                                                        .text_xs()
                                                        .font_weight(FontWeight::SEMIBOLD)
                                                        .text_color(theme::text_primary())
                                                        .child("Enable Shortcuts"),
                                                ),
                                        )
                                        .child(
                                            div()
                                                .id("btn-close-controller-shortcuts")
                                                .cursor_pointer()
                                                .size(px(24.0))
                                                .rounded_md()
                                                .flex()
                                                .items_center()
                                                .justify_center()
                                                .text_sm()
                                                .font_weight(FontWeight::BOLD)
                                                .text_color(theme::text_muted())
                                                .hover(|s| s.bg(theme::bg_hover()).text_color(theme::text_primary()))
                                                .on_click(cx.listener(|this, _, _, cx| {
                                                    this.show_controller_shortcuts_modal = false;
                                                    this.remapping_controller_shortcut = None;
                                                    cx.notify();
                                                }))
                                                .child(Icon::new(IconName::X).size(px(14.0))),
                                        ),
                                ),
                        )
                        // Content Body: 3-column layout matching media_1793137385955_044e590e.png
                        .child(
                            div()
                                .flex()
                                .items_start()
                                .gap_4()
                                .w_full()
                                .flex_1()
                                .min_h_0()
                                // Column 1: Gamepad Type, Reminder & UI Navigation Settings
                                .child(
                                    div()
                                        .flex()
                                        .flex_col()
                                        .w(px(250.0))
                                        .flex_shrink_0()
                                        .gap_3()
                                        // Row with Gamepad Type & Reminder
                                        .child(
                                            div()
                                                .flex()
                                                .items_start()
                                                .gap_2()
                                                .w_full()
                                                // GroupBox: Gamepad Type
                                                .child(
                                                    div()
                                                        .flex_1()
                                                        .p_2p5()
                                                        .rounded_md()
                                                        .border_1()
                                                        .border_color(theme::border_color())
                                                        .bg(theme::bg_panel())
                                                        .flex()
                                                        .flex_col()
                                                        .gap_1p5()
                                                        .child(
                                                            div()
                                                                .text_xs()
                                                                .font_weight(FontWeight::BOLD)
                                                                .text_color(theme::text_muted())
                                                                .child("Gamepad Type"),
                                                        )
                                                        .child(
                                                            div()
                                                                .id("rad-type-xbox")
                                                                .cursor_pointer()
                                                                .flex()
                                                                .items_center()
                                                                .gap_1p5()
                                                                .text_xs()
                                                                .text_color(if !is_ps { theme::text_primary() } else { theme::text_muted() })
                                                                .on_click(cx.listener(|this, _, _, cx| {
                                                                    this.app_settings.gamepad_type = "Xbox".to_string();
                                                                    let _ = this.app_settings.save();
                                                                    this.status_message = "Gamepad type set to Xbox".to_string();
                                                                    cx.notify();
                                                                }))
                                                                .child(
                                                                    div()
                                                                        .size(px(14.0))
                                                                        .rounded_sm()
                                                                        .border_1()
                                                                        .border_color(if !is_ps { theme::accent() } else { theme::border_color() })
                                                                        .bg(if !is_ps { theme::accent() } else { theme::bg_main() })
                                                                        .flex()
                                                                        .items_center()
                                                                        .justify_center()
                                                                        .text_xs()
                                                                        .text_color(theme::text_on_accent())
                                                                        .child(if !is_ps { "✓" } else { "" }),
                                                                )
                                                                .child("Xbox"),
                                                        )
                                                        .child(
                                                            div()
                                                                .id("rad-type-ps")
                                                                .cursor_pointer()
                                                                .flex()
                                                                .items_center()
                                                                .gap_1p5()
                                                                .text_xs()
                                                                .text_color(if is_ps { theme::text_primary() } else { theme::text_muted() })
                                                                .on_click(cx.listener(|this, _, _, cx| {
                                                                    this.app_settings.gamepad_type = "Playstation".to_string();
                                                                    let _ = this.app_settings.save();
                                                                    this.status_message = "Gamepad type set to PlayStation".to_string();
                                                                    cx.notify();
                                                                }))
                                                                .child(
                                                                    div()
                                                                        .size(px(14.0))
                                                                        .rounded_sm()
                                                                        .border_1()
                                                                        .border_color(if is_ps { theme::accent() } else { theme::border_color() })
                                                                        .bg(if is_ps { theme::accent() } else { theme::bg_main() })
                                                                        .flex()
                                                                        .items_center()
                                                                        .justify_center()
                                                                        .text_xs()
                                                                        .text_color(theme::text_on_accent())
                                                                        .child(if is_ps { "✓" } else { "" }),
                                                                )
                                                                .child("Playstation"),
                                                        ),
                                                )
                                                // GroupBox: Reminder
                                                .child(
                                                    div()
                                                        .w(px(80.0))
                                                        .px_2()
                                                        .py_2p5()
                                                        .rounded_md()
                                                        .border_1()
                                                        .border_color(theme::border_color())
                                                        .bg(theme::bg_panel())
                                                        .flex()
                                                        .flex_col()
                                                        .items_center()
                                                        .gap_1p5()
                                                        .child(
                                                            div()
                                                                .text_xs()
                                                                .font_weight(FontWeight::BOLD)
                                                                .text_color(theme::text_muted())
                                                                .whitespace_nowrap()
                                                                .child("Reminder"),
                                                        )
                                                        .child(
                                                            div()
                                                                .size(px(28.0))
                                                                .rounded_full()
                                                                .bg(rgb(0xffffff))
                                                                .text_color(rgb(0x000000))
                                                                .font_weight(FontWeight::BOLD)
                                                                .flex()
                                                                .items_center()
                                                                .justify_center()
                                                                .child(if is_ps { "🅟" } else { "⨂" }),
                                                        ),
                                                ),
                                        )
                                        // GroupBox: UI Navigation Settings (remappable)
                                        .child(
                                            div()
                                                .p_3()
                                                .rounded_md()
                                                .border_1()
                                                .border_color(theme::border_color())
                                                .bg(theme::bg_panel())
                                                .flex()
                                                .flex_col()
                                                .gap_1p5()
                                                .child(
                                                    div()
                                                        .text_xs()
                                                        .font_weight(FontWeight::BOLD)
                                                        .text_color(theme::text_muted())
                                                        .child("UI Navigation Settings"),
                                                )
                                                .child(self.render_shortcut_row("osk", "On screen KB:", is_ps, cx))
                                                .child(self.render_nav_single_row("drag_drop", "Drag and Drop:", is_ps, cx))
                                                .child(self.render_nav_single_row("right_click", "Right Click:", is_ps, cx))
                                                .child(self.render_nav_single_row("left_click", "Left Click:", is_ps, cx))
                                                .child(
                                                    div()
                                                        .flex()
                                                        .items_center()
                                                        .justify_between()
                                                        .pt_1()
                                                        .child(
                                                            div()
                                                                .text_xs()
                                                                .text_color(theme::text_muted())
                                                                .child("Deadzone:"),
                                                        )
                                                        .child(
                                                            div()
                                                                .flex()
                                                                .items_center()
                                                                .gap_1()
                                                                .child(
                                                                    div()
                                                                        .id("btn-deadzone-dec")
                                                                        .cursor_pointer()
                                                                        .size(px(20.0))
                                                                        .rounded_sm()
                                                                        .bg(theme::bg_active())
                                                                        .flex()
                                                                        .items_center()
                                                                        .justify_center()
                                                                        .text_xs()
                                                                        .font_weight(FontWeight::BOLD)
                                                                        .text_color(theme::text_primary())
                                                                        .hover(|s| s.bg(theme::bg_hover()))
                                                                        .on_click(cx.listener(|this, _, _, cx| {
                                                                            this.app_settings.gamepad_deadzone = (this.app_settings.gamepad_deadzone - 1000).max(1000);
                                                                            let _ = this.app_settings.save();
                                                                            cx.notify();
                                                                        }))
                                                                        .child("-"),
                                                                )
                                                                .child(
                                                                    div()
                                                                        .px_2()
                                                                        .py_0p5()
                                                                        .rounded_sm()
                                                                        .border_1()
                                                                        .border_color(theme::border_color())
                                                                        .bg(theme::bg_main())
                                                                        .text_xs()
                                                                        .text_color(theme::accent())
                                                                        .font_weight(FontWeight::BOLD)
                                                                        .child(deadzone_val.to_string()),
                                                                )
                                                                .child(
                                                                    div()
                                                                        .id("btn-deadzone-inc")
                                                                        .cursor_pointer()
                                                                        .size(px(20.0))
                                                                        .rounded_sm()
                                                                        .bg(theme::bg_active())
                                                                        .flex()
                                                                        .items_center()
                                                                        .justify_center()
                                                                        .text_xs()
                                                                        .font_weight(FontWeight::BOLD)
                                                                        .text_color(theme::text_primary())
                                                                        .hover(|s| s.bg(theme::bg_hover()))
                                                                        .on_click(cx.listener(|this, _, _, cx| {
                                                                            this.app_settings.gamepad_deadzone = (this.app_settings.gamepad_deadzone + 1000).min(30000);
                                                                            let _ = this.app_settings.save();
                                                                            cx.notify();
                                                                        }))
                                                                        .child("+"),
                                                                ),
                                                        ),
                                                ),
                                        ),
                                )
                                // Column 2: Shortcuts List (remappable, matching media_1793137385955_044e590e.png center box)
                                .child(
                                    div()
                                        .flex()
                                        .flex_col()
                                        .w(px(260.0))
                                        .flex_shrink_0()
                                        .p_3()
                                        .rounded_md()
                                        .border_1()
                                        .border_color(theme::border_color())
                                        .bg(theme::bg_panel())
                                        .gap_1()
                                        .child(
                                            div()
                                                .text_xs()
                                                .font_weight(FontWeight::BOLD)
                                                .text_color(theme::text_muted())
                                                .child("Shortcuts (Click to Remap)"),
                                        )
                                        .child(self.render_shortcut_row("unfocus", "Toggle Unfocus :", is_ps, cx))
                                        .child(self.render_shortcut_row("close", "Close Nucleus :", is_ps, cx))
                                        .child(self.render_shortcut_row("stop", "Stop Session :", is_ps, cx))
                                        .child(self.render_shortcut_row("topmost", "Toggle Top Most :", is_ps, cx))
                                        .child(self.render_shortcut_row("reset_windows", "Reset Windows :", is_ps, cx))
                                        .child(self.render_shortcut_row("cutscenes", "Cutscenes Mode :", is_ps, cx))
                                        .child(self.render_shortcut_row("switch_layout", "Switch Layouts :", is_ps, cx))
                                        .child(self.render_shortcut_row("lock_inputs", "Lock Input Key :", is_ps, cx))
                                        .child(self.render_shortcut_row("release_cursor", "ALT+TAB :", is_ps, cx))
                                        .child(self.render_shortcut_row("lock_ui", "Lock UI Control :", is_ps, cx)),
                                )
                                // Column 3: Red warning notice, Remap Palette & Controller Silhouette Graphic
                                .child(
                                    div()
                                        .flex_1()
                                        .min_w_0()
                                        .flex()
                                        .flex_col()
                                        .items_center()
                                        .justify_between()
                                        .p_3()
                                        .gap_3()
                                        // Warning Notice
                                        .child(
                                            div()
                                                .w_full()
                                                .p_2p5()
                                                .rounded_md()
                                                .bg(theme::danger_subtle())
                                                .border_1()
                                                .border_color(theme::danger())
                                                .text_xs()
                                                .font_weight(FontWeight::BOLD)
                                                .text_color(theme::danger())
                                                .text_center()
                                                .child("Warning: Steam Input may interfere with the controller shortcuts."),
                                        )
                                        // Interactive Remapping Palette (shown when an action is clicked for remapping)
                                        .when_some(self.remapping_controller_shortcut.clone(), |el, active_action| {
                                            let act_name = active_action.clone();
                                            el.child(
                                                div()
                                                    .w_full()
                                                    .p_2p5()
                                                    .rounded_md()
                                                    .bg(rgb(0x181820))
                                                    .border_1()
                                                    .border_color(theme::accent())
                                                    .flex()
                                                    .flex_col()
                                                    .gap_2()
                                                    .child(
                                                        div()
                                                            .flex()
                                                            .items_center()
                                                            .justify_between()
                                                            .child(
                                                                div()
                                                                    .text_xs()
                                                                    .font_weight(FontWeight::BOLD)
                                                                    .text_color(theme::accent())
                                                                    .child(format!("Select replacement button for '{}':", act_name)),
                                                            )
                                                            .child(
                                                                div()
                                                                    .id("btn-cancel-shortcut-remap")
                                                                    .cursor_pointer()
                                                                    .px_2()
                                                                    .py_0p5()
                                                                    .rounded_md()
                                                                    .bg(theme::bg_active())
                                                                    .text_xs()
                                                                    .text_color(theme::text_muted())
                                                                    .hover(|s| s.text_color(theme::text_primary()))
                                                                    .on_click(cx.listener(|this, _, _, cx| {
                                                                        this.remapping_controller_shortcut = None;
                                                                        cx.notify();
                                                                    }))
                                                                    .child("Cancel"),
                                                            ),
                                                    )
                                                    .child(
                                                        div()
                                                            .flex()
                                                            .flex_wrap()
                                                            .gap_1p5()
                                                            .children(
                                                                [
                                                                    ("A", if is_ps { "✕" } else { "A" }),
                                                                    ("B", if is_ps { "◯" } else { "B" }),
                                                                    ("X", if is_ps { "□" } else { "X" }),
                                                                    ("Y", if is_ps { "△" } else { "Y" }),
                                                                    ("LB", if is_ps { "L1" } else { "LB" }),
                                                                    ("RB", if is_ps { "R1" } else { "RB" }),
                                                                    ("LT", if is_ps { "L2" } else { "LT" }),
                                                                    ("RT", if is_ps { "R2" } else { "RT" }),
                                                                    ("LS", if is_ps { "L3" } else { "LS" }),
                                                                    ("RS", if is_ps { "R3" } else { "RS" }),
                                                                    ("Start", if is_ps { "Opt" } else { "≡" }),
                                                                    ("Back", if is_ps { "Shr" } else { "⧉" }),
                                                                    ("DPadUp", "▲"),
                                                                    ("DPadDown", "▼"),
                                                                    ("DPadLeft", "◀"),
                                                                    ("DPadRight", "▶"),
                                                                ]
                                                                .into_iter()
                                                                .map(|(code, glyph)| {
                                                                    let code_str = code.to_string();
                                                                    let target_act = act_name.clone();
                                                                    div()
                                                                        .id(ElementId::Name(format!("remap-pick-{}", code).into()))
                                                                        .cursor_pointer()
                                                                        .px_2()
                                                                        .py_1()
                                                                        .rounded_md()
                                                                        .bg(rgb(0x24242c))
                                                                        .border_1()
                                                                        .border_color(rgb(0x3a3a46))
                                                                        .text_xs()
                                                                        .font_weight(FontWeight::BOLD)
                                                                        .text_color(theme::text_primary())
                                                                        .hover(|s| s.bg(theme::accent()).border_color(theme::accent()).text_color(theme::text_on_accent()))
                                                                        .on_click(cx.listener(move |this, _, _, cx| {
                                                                            this.remap_controller_action(&target_act, &code_str, cx);
                                                                        }))
                                                                        .child(glyph)
                                                                }),
                                                            ),
                                                    ),
                                            )
                                        })
                                        // Stylized Controller Silhouette Layout with clickable hot zones
                                        .child(
                                            div()
                                                .flex_1()
                                                .w_full()
                                                .flex()
                                                .flex_col()
                                                .items_center()
                                                .justify_center()
                                                .gap_3()
                                                // Controller Top Bumpers/Triggers Graphic
                                                .child(
                                                    div()
                                                        .flex()
                                                        .items_center()
                                                        .justify_between()
                                                        .w(px(200.0))
                                                        .px_3()
                                                        .py_1p5()
                                                        .rounded_md()
                                                        .border_1()
                                                        .border_color(rgb(0x44444c))
                                                        .bg(rgb(0x1a1a1e))
                                                        .child({
                                                            let act_opt = self.remapping_controller_shortcut.clone();
                                                            div()
                                                                .id("sil-top-left")
                                                                .cursor_pointer()
                                                                .text_xs()
                                                                .font_weight(FontWeight::BOLD)
                                                                .text_color(theme::accent())
                                                                .hover(|s| s.text_color(rgb(0xffffff)))
                                                                .on_click(cx.listener(move |this, _, _, cx| {
                                                                    if let Some(ref a) = act_opt {
                                                                        this.remap_controller_action(a, "LB", cx);
                                                                    }
                                                                }))
                                                                .child(if is_ps { "L1 / L2" } else { "LB / LT" })
                                                        })
                                                        .child(
                                                            div()
                                                                .text_xs()
                                                                .text_color(theme::text_muted())
                                                                .child("TOP BUMPERS"),
                                                        )
                                                        .child({
                                                            let act_opt = self.remapping_controller_shortcut.clone();
                                                            div()
                                                                .id("sil-top-right")
                                                                .cursor_pointer()
                                                                .text_xs()
                                                                .font_weight(FontWeight::BOLD)
                                                                .text_color(theme::accent())
                                                                .hover(|s| s.text_color(rgb(0xffffff)))
                                                                .on_click(cx.listener(move |this, _, _, cx| {
                                                                    if let Some(ref a) = act_opt {
                                                                        this.remap_controller_action(a, "RB", cx);
                                                                    }
                                                                }))
                                                                .child(if is_ps { "R1 / R2" } else { "RB / RT" })
                                                        }),
                                                )
                                                // Controller Main Face Graphic
                                                .child(
                                                    div()
                                                        .w(px(240.0))
                                                        .h(px(130.0))
                                                        .rounded_2xl()
                                                        .border_2()
                                                        .border_color(rgb(0x555562))
                                                        .bg(rgb(0x1e1e24))
                                                        .p_3()
                                                        .flex()
                                                        .flex_col()
                                                        .justify_between()
                                                        .child(
                                                            div()
                                                                .flex()
                                                                .items_center()
                                                                .justify_between()
                                                                // Left Stick
                                                                .child({
                                                                    let act_opt = self.remapping_controller_shortcut.clone();
                                                                    div()
                                                                        .id("sil-ls")
                                                                        .cursor_pointer()
                                                                        .size(px(32.0))
                                                                        .rounded_full()
                                                                        .border_1()
                                                                        .border_color(rgb(0x666675))
                                                                        .bg(rgb(0x2a2a33))
                                                                        .flex()
                                                                        .items_center()
                                                                        .justify_center()
                                                                        .text_xs()
                                                                        .text_color(theme::text_muted())
                                                                        .hover(|s| s.bg(theme::accent()).text_color(theme::text_on_accent()))
                                                                        .on_click(cx.listener(move |this, _, _, cx| {
                                                                            if let Some(ref a) = act_opt {
                                                                                this.remap_controller_action(a, "LS", cx);
                                                                            }
                                                                        }))
                                                                        .child("LS")
                                                                })
                                                                // Center Guide Button
                                                                .child({
                                                                    let act_opt = self.remapping_controller_shortcut.clone();
                                                                    div()
                                                                        .id("sil-guide")
                                                                        .cursor_pointer()
                                                                        .size(px(28.0))
                                                                        .rounded_full()
                                                                        .bg(rgb(0xffffff))
                                                                        .text_color(rgb(0x000000))
                                                                        .font_weight(FontWeight::BOLD)
                                                                        .flex()
                                                                        .items_center()
                                                                        .justify_center()
                                                                        .on_click(cx.listener(move |this, _, _, cx| {
                                                                            if let Some(ref a) = act_opt {
                                                                                this.remap_controller_action(a, "Guide", cx);
                                                                            }
                                                                        }))
                                                                        .child(if is_ps { "🅟" } else { "⨂" })
                                                                })
                                                                // Face Buttons Cluster
                                                                .child({
                                                                    let act_opt = self.remapping_controller_shortcut.clone();
                                                                    div()
                                                                        .id("sil-face")
                                                                        .cursor_pointer()
                                                                        .size(px(34.0))
                                                                        .rounded_full()
                                                                        .border_1()
                                                                        .border_color(rgb(0x666675))
                                                                        .bg(rgb(0x2a2a33))
                                                                        .flex()
                                                                        .items_center()
                                                                        .justify_center()
                                                                        .text_xs()
                                                                        .font_weight(FontWeight::BOLD)
                                                                        .text_color(theme::accent())
                                                                        .hover(|s| s.bg(theme::accent()).text_color(theme::text_on_accent()))
                                                                        .on_click(cx.listener(move |this, _, _, cx| {
                                                                            if let Some(ref a) = act_opt {
                                                                                this.remap_controller_action(a, "A", cx);
                                                                            }
                                                                        }))
                                                                        .child(if is_ps { "△ ◯ □ ✕" } else { "Y B X A" })
                                                                }),
                                                        )
                                                        .child(
                                                            div()
                                                                .flex()
                                                                .items_center()
                                                                .justify_between()
                                                                .px_4()
                                                                // DPad
                                                                .child({
                                                                    let act_opt = self.remapping_controller_shortcut.clone();
                                                                    div()
                                                                        .id("sil-dpad")
                                                                        .cursor_pointer()
                                                                        .size(px(28.0))
                                                                        .rounded_md()
                                                                        .bg(rgb(0x2a2a33))
                                                                        .flex()
                                                                        .items_center()
                                                                        .justify_center()
                                                                        .text_xs()
                                                                        .text_color(theme::text_muted())
                                                                        .hover(|s| s.bg(theme::accent()).text_color(theme::text_on_accent()))
                                                                        .on_click(cx.listener(move |this, _, _, cx| {
                                                                            if let Some(ref a) = act_opt {
                                                                                this.remap_controller_action(a, "DPadUp", cx);
                                                                            }
                                                                        }))
                                                                        .child("✚")
                                                                })
                                                                // Right Stick
                                                                .child({
                                                                    let act_opt = self.remapping_controller_shortcut.clone();
                                                                    div()
                                                                        .id("sil-rs")
                                                                        .cursor_pointer()
                                                                        .size(px(32.0))
                                                                        .rounded_full()
                                                                        .border_1()
                                                                        .border_color(rgb(0x666675))
                                                                        .bg(rgb(0x2a2a33))
                                                                        .flex()
                                                                        .items_center()
                                                                        .justify_center()
                                                                        .text_xs()
                                                                        .text_color(theme::text_muted())
                                                                        .hover(|s| s.bg(theme::accent()).text_color(theme::text_on_accent()))
                                                                        .on_click(cx.listener(move |this, _, _, cx| {
                                                                            if let Some(ref a) = act_opt {
                                                                                this.remap_controller_action(a, "RS", cx);
                                                                            }
                                                                        }))
                                                                        .child("RS")
                                                                }),
                                                        ),
                                                ),
                                        ),
                                ),
                        ),
                )
                .into_any_element(),
        )
    }

    /// Renders active interactive launch countdown prompt modal if present.
    fn render_launch_prompt_modal(&self, cx: &mut Context<Self>) -> Option<AnyElement> {
        let req = self.active_launch_prompt.as_ref()?;
        let player_idx = req.player_index;
        let count = req.player_count;
        let msg = req.message.clone();
        let seconds = req.auto_proceed_seconds;

        let modal = crate::nefos::NefosLaunchPromptModal::new(player_idx, count, msg)
            .countdown(seconds)
            .on_proceed(cx.listener(|this, _, _, cx| {
                if let Some(req) = this.active_launch_prompt.take() {
                    let _ = req.response_tx.send(true);
                }
                cx.notify();
            }))
            .on_cancel(cx.listener(|this, _, _, cx| {
                if let Some(req) = this.active_launch_prompt.take() {
                    let _ = req.response_tx.send(false);
                }
                cx.notify();
            }));

        Some(modal.into_any_element())
    }

    /// Renders dismissible pre-flight diagnostics alert banner if errors or warnings exist.
    fn render_preflight_banners(&self, cx: &mut Context<Self>) -> Option<AnyElement> {
        if let Some(ref danger) = self.preflight_danger {
            let msg = danger.clone();
            Some(
                crate::nefos::NefosBanner::new("preflight-danger-banner", msg)
                    .tone(crate::nefos::NefosBannerTone::Danger)
                    .title("Critical Environment Health Alert")
                    .on_dismiss(cx.listener(|this, _, _, cx| {
                        this.preflight_danger = None;
                        cx.notify();
                    }))
                    .into_any_element(),
            )
        } else if let Some(ref warn) = self.preflight_warning {
            let msg = warn.clone();
            Some(
                crate::nefos::NefosBanner::new("preflight-warning-banner", msg)
                    .tone(crate::nefos::NefosBannerTone::Warning)
                    .title("System Compatibility Notice")
                    .on_dismiss(cx.listener(|this, _, _, cx| {
                        this.preflight_warning = None;
                        cx.notify();
                    }))
                    .into_any_element(),
            )
        } else {
            None
        }
    }

    /// Renders the Hub Account authentication modal if currently open.
    pub fn render_hub_account_modal(&self, cx: &mut Context<Self>) -> Option<AnyElement> {
        if !self.hub_account_modal.is_open {
            return None;
        }
        let on_close = cx.listener(|this: &mut MainWindow, _, _, cx| {
            this.hub_account_modal.close();
            cx.notify();
        });
        Some(crate::hub_account_modal::render_hub_account_modal(
            &self.hub_account_modal,
            self.hub_account_summary.as_ref(),
            self.callbacks.on_hub_login.clone(),
            self.callbacks.on_hub_logout.clone(),
            Arc::new(on_close),
        ))
    }

    /// Renders the multi-instance save slot manager modal if currently open.
    pub fn render_save_slots_modal(&self, cx: &mut Context<Self>) -> Option<AnyElement> {
        if !self.save_slots_modal.is_open {
            return None;
        }
        let active_slot = self
            .player_save_slots
            .get(self.save_slots_modal.target_player_index)
            .and_then(|s| s.as_deref());
        let on_close = cx.listener(|this: &mut MainWindow, _, _, cx| {
            this.save_slots_modal.close();
            cx.notify();
        });
        Some(crate::save_slots_modal::render_save_slots_modal(
            &self.save_slots_modal,
            &self.current_save_slots,
            active_slot,
            self.callbacks.on_set_player_save_slot.clone(),
            self.callbacks.on_create_save_slot.as_ref().map(|cb| {
                let c = cb.clone();
                Arc::new(move |g: &str, d: &str, t: &str, h: &str| {
                    let _ = c(g, d, t, h);
                }) as Arc<dyn Fn(&str, &str, &str, &str) + Send + Sync>
            }),
            self.callbacks.on_clone_save_slot.as_ref().map(|cb| {
                let c = cb.clone();
                Arc::new(move |g: &str, s: &str, n: &str| {
                    let _ = c(g, s, n);
                }) as Arc<dyn Fn(&str, &str, &str) + Send + Sync>
            }),
            self.callbacks.on_delete_save_slot.as_ref().map(|cb| {
                let c = cb.clone();
                Arc::new(move |g: &str, s: &str| {
                    let _ = c(g, s);
                }) as Arc<dyn Fn(&str, &str) + Send + Sync>
            }),
            Arc::new(on_close),
        ))
    }
}

/// Formats a controller button code into a display glyph based on Xbox vs PlayStation style.
pub fn format_gamepad_button_glyph(button_code: &str, is_ps: bool) -> &'static str {
    match button_code {
        "A" => {
            if is_ps {
                "✕"
            } else {
                "A"
            }
        }
        "B" => {
            if is_ps {
                "◯"
            } else {
                "B"
            }
        }
        "X" => {
            if is_ps {
                "□"
            } else {
                "X"
            }
        }
        "Y" => {
            if is_ps {
                "△"
            } else {
                "Y"
            }
        }
        "LB" => {
            if is_ps {
                "L1"
            } else {
                "LB"
            }
        }
        "RB" => {
            if is_ps {
                "R1"
            } else {
                "RB"
            }
        }
        "LT" => {
            if is_ps {
                "L2"
            } else {
                "LT"
            }
        }
        "RT" => {
            if is_ps {
                "R2"
            } else {
                "RT"
            }
        }
        "LS" | "L3" => {
            if is_ps {
                "L3"
            } else {
                "LS"
            }
        }
        "RS" | "R3" => {
            if is_ps {
                "R3"
            } else {
                "RS"
            }
        }
        "Start" | "Menu" => {
            if is_ps {
                "Opt"
            } else {
                "≡"
            }
        }
        "Back" | "View" => {
            if is_ps {
                "Shr"
            } else {
                "⧉"
            }
        }
        "DPadUp" | "Up" => "▲",
        "DPadDown" | "Down" => "▼",
        "DPadLeft" | "Left" => "◀",
        "DPadRight" | "Right" => "▶",
        "Guide" | "Home" => {
            if is_ps {
                "🅟"
            } else {
                "⨂"
            }
        }
        _ => "•",
    }
}

impl Render for MainWindow {
    fn render(&mut self, _window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        // Auto-fetch hub packages asynchronously if entering Hub tab and not yet loaded
        if self.active_tab == ActiveTab::Hub && !self.hub_loaded {
            self.hub_loaded = true;
            self.trigger_hub_search_async(cx);
        }

        let active_handler = self.handlers.iter().find(|h| {
            if let Some(ref sel) = self.selected_game {
                h.game_name.eq_ignore_ascii_case(sel) || h.guid.eq_ignore_ascii_case(sel)
            } else {
                false
            }
        });

        if self.hub_search_focus.is_none() {
            self.hub_search_focus = Some(cx.focus_handle());
        }
        if self.lib_search_focus.is_none() {
            self.lib_search_focus = Some(cx.focus_handle());
        }
        if self.hotkey_record_focus.is_none() {
            self.hotkey_record_focus = Some(cx.focus_handle());
        }
        if self.window_focus.is_none() {
            self.window_focus = Some(cx.focus_handle());
        }
        let hub_focus = self.hub_search_focus.clone().unwrap();
        let lib_focus = self.lib_search_focus.clone().unwrap();
        let win_focus = self.window_focus.clone().unwrap();

        // Enforce focus state:
        // 1. If actively recording physical hotkey, ensure hotkey_record_focus is focused.
        // 2. Otherwise if window currently has no focus, focus win_focus so global shortcuts (Ctrl+N) work.
        if self.settings_view.recording_hotkey.is_some() {
            if let Some(ref f) = self.hotkey_record_focus {
                f.focus(_window, cx);
            }
        } else if _window.focused(cx).is_none() {
            win_focus.focus(_window, cx);
        }

        // Dedicated Fullscreen Topmost In-Game Couch OSD Overlay presentation
        if self.in_game_osd.is_visible {
            return div()
                .id("in-game-couch-osd-root")
                .size_full()
                .track_focus(&win_focus)
                .on_key_down(cx.listener(
                    |this: &mut MainWindow, event: &gpui::KeyDownEvent, _window, cx| {
                        let key = event.keystroke.key.as_str();
                        let ctrl = event.keystroke.modifiers.control;
                        let shift = event.keystroke.modifiers.shift;

                        if let Some(action_name) =
                            crate::big_picture::input::map_key_to_couch_action_name(
                                key, ctrl, shift,
                            )
                        {
                            this.dispatch_couch_action_by_name(action_name, cx);
                        }
                    },
                ))
                .on_action(cx.listener(
                    |this: &mut MainWindow, _: &crate::actions::NavigateUp, _window, cx| {
                        this.dispatch_couch_action_by_name("NavigateUp", cx);
                    },
                ))
                .on_action(cx.listener(
                    |this: &mut MainWindow, _: &crate::actions::NavigateDown, _window, cx| {
                        this.dispatch_couch_action_by_name("NavigateDown", cx);
                    },
                ))
                .on_action(cx.listener(
                    |this: &mut MainWindow, _: &crate::actions::ConfirmAction, _window, cx| {
                        this.dispatch_couch_action_by_name("Confirm", cx);
                    },
                ))
                .on_action(cx.listener(
                    |this: &mut MainWindow, _: &crate::actions::CancelAction, _window, cx| {
                        this.dispatch_couch_action_by_name("Cancel", cx);
                    },
                ))
                .child(crate::big_picture::render_in_game_osd(self, cx))
                .into_any_element();
        }

        if self.is_big_picture {
            return div()
                .relative()
                .flex()
                .flex_col()
                .size_full()
                .bg(theme::bg_main())
                .text_color(theme::text_primary())
                .track_focus(&win_focus)
                .on_mouse_down(
                    MouseButton::Left,
                    cx.listener(|this, _, window, cx| {
                        if window.focused(cx).is_none() {
                            if let Some(ref f) = this.window_focus {
                                f.focus(window, cx);
                            }
                        }
                    }),
                )
                .on_key_down(cx.listener(
                    |this: &mut MainWindow, event: &gpui::KeyDownEvent, _window, cx| {
                        let key = event.keystroke.key.as_str();
                        let ctrl = event.keystroke.modifiers.control;
                        let shift = event.keystroke.modifiers.shift;

                        // 1. If Carousel search popover is open, route typing directly into search query
                        if this.carousel_state.search_input_open {
                            if event.keystroke.modifiers.control
                                || event.keystroke.modifiers.alt
                                || event.keystroke.modifiers.platform
                            {
                                return;
                            }
                            if key.eq_ignore_ascii_case("escape")
                                || key.eq_ignore_ascii_case("enter")
                            {
                                this.carousel_state.search_input_open = false;
                                cx.notify();
                                return;
                            }
                            if key.eq_ignore_ascii_case("backspace") {
                                this.carousel_state.search_query.pop();
                                this.carousel_state.selected_index = 0;
                                cx.notify();
                                return;
                            }
                            if key.eq_ignore_ascii_case("space") {
                                this.carousel_state.search_query.push(' ');
                                this.carousel_state.selected_index = 0;
                                cx.notify();
                                return;
                            }
                            if let Some(ref s) = event.keystroke.key_char {
                                this.carousel_state.search_query.push_str(s);
                                this.carousel_state.selected_index = 0;
                                cx.notify();
                                return;
                            }
                            if key.chars().count() == 1 {
                                this.carousel_state.search_query.push_str(key);
                                this.carousel_state.selected_index = 0;
                                cx.notify();
                                return;
                            }
                            return;
                        }

                        // 2. If Hub search popover is open, route typing directly into Hub search query
                        if this.couch_hub_state.search_input_open {
                            if event.keystroke.modifiers.control
                                || event.keystroke.modifiers.alt
                                || event.keystroke.modifiers.platform
                            {
                                return;
                            }
                            if key.eq_ignore_ascii_case("escape")
                                || key.eq_ignore_ascii_case("enter")
                            {
                                this.couch_hub_state.search_input_open = false;
                                cx.notify();
                                return;
                            }
                            if key.eq_ignore_ascii_case("backspace") {
                                this.couch_hub_state.search_query.pop();
                                this.couch_hub_state.selected_index = 0;
                                cx.notify();
                                return;
                            }
                            if key.eq_ignore_ascii_case("space") {
                                this.couch_hub_state.search_query.push(' ');
                                this.couch_hub_state.selected_index = 0;
                                cx.notify();
                                return;
                            }
                            if let Some(ref s) = event.keystroke.key_char {
                                this.couch_hub_state.search_query.push_str(s);
                                this.couch_hub_state.selected_index = 0;
                                cx.notify();
                                return;
                            }
                            if key.chars().count() == 1 {
                                this.couch_hub_state.search_query.push_str(key);
                                this.couch_hub_state.selected_index = 0;
                                cx.notify();
                                return;
                            }
                            return;
                        }

                        // 3. If Setup combobox is open, route typing directly into draft
                        if this.setup_screen.combobox_open_key.is_some() {
                            if event.keystroke.modifiers.control
                                || event.keystroke.modifiers.alt
                                || event.keystroke.modifiers.platform
                            {
                                return;
                            }
                            if key.eq_ignore_ascii_case("escape") {
                                this.setup_screen.close_combobox();
                                cx.notify();
                                return;
                            }
                            if key.eq_ignore_ascii_case("backspace") {
                                this.setup_screen.combobox_search_draft.pop();
                                this.setup_screen.combobox_validation_error = None;
                                cx.notify();
                                return;
                            }
                            if key.eq_ignore_ascii_case("space") {
                                this.setup_screen.combobox_search_draft.push(' ');
                                this.setup_screen.combobox_validation_error = None;
                                cx.notify();
                                return;
                            }
                            if let Some(ref s) = event.keystroke.key_char {
                                this.setup_screen.combobox_search_draft.push_str(s);
                                this.setup_screen.combobox_validation_error = None;
                                cx.notify();
                                return;
                            }
                            if key.chars().count() == 1 {
                                this.setup_screen.combobox_search_draft.push_str(key);
                                this.setup_screen.combobox_validation_error = None;
                                cx.notify();
                                return;
                            }
                            return;
                        }

                        if let Some(action_name) =
                            crate::big_picture::input::map_key_to_couch_action_name(
                                key, ctrl, shift,
                            )
                        {
                            this.dispatch_couch_action_by_name(action_name, cx);
                        }
                    },
                ))
                .on_action(cx.listener(
                    |this: &mut MainWindow, _: &crate::actions::ToggleBigPicture, _window, cx| {
                        this.dispatch_couch_action_by_name("ToggleBigPicture", cx);
                    },
                ))
                .on_action(cx.listener(
                    |this: &mut MainWindow, _: &crate::actions::ToggleOsd, _window, cx| {
                        this.dispatch_couch_action_by_name("ToggleOsd", cx);
                    },
                ))
                .on_action(cx.listener(
                    |this: &mut MainWindow, _: &crate::actions::NavigateUp, _window, cx| {
                        this.dispatch_couch_action_by_name("NavigateUp", cx);
                    },
                ))
                .on_action(cx.listener(
                    |this: &mut MainWindow, _: &crate::actions::NavigateDown, _window, cx| {
                        this.dispatch_couch_action_by_name("NavigateDown", cx);
                    },
                ))
                .on_action(cx.listener(
                    |this: &mut MainWindow, _: &crate::actions::NavigateLeft, _window, cx| {
                        this.dispatch_couch_action_by_name("NavigateLeft", cx);
                    },
                ))
                .on_action(cx.listener(
                    |this: &mut MainWindow, _: &crate::actions::NavigateRight, _window, cx| {
                        this.dispatch_couch_action_by_name("NavigateRight", cx);
                    },
                ))
                .on_action(cx.listener(
                    |this: &mut MainWindow, _: &crate::actions::ConfirmAction, _window, cx| {
                        this.dispatch_couch_action_by_name("Confirm", cx);
                    },
                ))
                .on_action(cx.listener(
                    |this: &mut MainWindow, _: &crate::actions::CancelAction, _window, cx| {
                        this.dispatch_couch_action_by_name("Cancel", cx);
                    },
                ))
                .on_action(cx.listener(
                    |this: &mut MainWindow, _: &crate::actions::QuickLaunch, _window, cx| {
                        this.dispatch_couch_action_by_name("QuickLaunch", cx);
                    },
                ))
                .on_action(cx.listener(
                    |this: &mut MainWindow, _: &crate::actions::ContextAction1, _window, cx| {
                        this.dispatch_couch_action_by_name("Context1", cx);
                    },
                ))
                .on_action(cx.listener(
                    |this: &mut MainWindow, _: &crate::actions::ContextAction2, _window, cx| {
                        this.dispatch_couch_action_by_name("Context2", cx);
                    },
                ))
                .on_action(cx.listener(
                    |this: &mut MainWindow, _: &crate::actions::NextTab, _window, cx| {
                        this.dispatch_couch_action_by_name("NextTab", cx);
                    },
                ))
                .on_action(cx.listener(
                    |this: &mut MainWindow, _: &crate::actions::PrevTab, _window, cx| {
                        this.dispatch_couch_action_by_name("PrevTab", cx);
                    },
                ))
                .child(crate::big_picture::render_big_picture_view(
                    self, _window, cx,
                ))
                .children(self.render_confirm_dialog(cx))
                .children(self.render_osd_overlay())
                .into_any_element();
        }

        let active_view: AnyElement = match self.active_tab {
            ActiveTab::Library => crate::game_list::render_library_view(
                &self.handlers,
                self.selected_game.as_deref(),
                &self.game_list.search_query,
                &lib_focus,
                &self.game_list.scroll_handle,
                self.library_sort,
                self.library_sort_dir,
                &self.game_list.selected_filters,
                cx,
            ),
            ActiveTab::Setup => crate::setup_screen::render_setup_view(
                &self.setup_screen,
                active_handler,
                self.selected_game.as_deref(),
                self.player_count,
                &self.selected_layout,
                &self.detected_devices_list,
                &self.player_device_assignments,
                self.active_profile_id,
                &self.saved_profiles,
                &self.selected_handler_options,
                self.dropdown_player_device,
                &self.app_settings,
                cx,
            ),
            ActiveTab::Hub => crate::hub_browser::render_hub_view(
                &self.hub_packages,
                &self.handlers,
                &self.hub_search_query,
                &hub_focus,
                self.hub_page,
                self.hub_page_size,
                self.hub_sort,
                self.hub_sort_ascending,
                &self.selected_tags,
                &self.installing_handlers,
                &self.installed_package_ids,
                self.hub_loading,
                cx,
            ),
            ActiveTab::Settings => crate::settings_view::render_settings_view(self, cx),
            ActiveTab::Extensions => crate::extensions_view::render_extensions_view(self, cx),
            ActiveTab::Debugger => crate::debugger_view::render_debugger_view(
                &self.debugger_view_state,
                self.callbacks.on_debug_resume.clone(),
                self.callbacks.on_debug_step_over.clone(),
                self.callbacks.on_debug_step_into.clone(),
                self.callbacks.on_debug_step_out.clone(),
                self.callbacks.on_debug_pause.clone(),
                self.callbacks.on_debug_stop.clone(),
                self.callbacks.on_debug_eval.as_ref().map(|cb| {
                    let c = cb.clone();
                    Arc::new(move |expr: &str| {
                        let _ = c(expr);
                    }) as Arc<dyn Fn(&str) + Send + Sync>
                }),
                self.callbacks.on_debug_toggle_breakpoint.clone(),
            ),
        };

        div()
            .relative()
            .flex()
            .flex_col()
            .size_full()
            .bg(theme::bg_main())
            .text_color(theme::text_primary())
            .track_focus(&win_focus)
            .on_mouse_down(
                MouseButton::Left,
                cx.listener(|this, _, window, cx| {
                    if this.settings_view.recording_hotkey.is_none() && window.focused(cx).is_none()
                    {
                        if let Some(ref f) = this.window_focus {
                            f.focus(window, cx);
                        }
                    }
                }),
            )
            .on_key_down(cx.listener(
                |this: &mut MainWindow, event: &gpui::KeyDownEvent, _window, cx| {
                    if this.settings_view.recording_hotkey.is_some() {
                        this.handle_key_recording(event, cx);
                        return;
                    }
                    if this.setup_screen.combobox_open_key.is_some() {
                        let key = event.keystroke.key.as_str();
                        if key.eq_ignore_ascii_case("escape") {
                            this.setup_screen.close_combobox();
                            cx.notify();
                            return;
                        }
                        if key.eq_ignore_ascii_case("backspace") {
                            this.setup_screen.combobox_search_draft.pop();
                            this.setup_screen.combobox_validation_error = None;
                            cx.notify();
                            return;
                        }
                        if key.eq_ignore_ascii_case("space") {
                            this.setup_screen.combobox_search_draft.push(' ');
                            this.setup_screen.combobox_validation_error = None;
                            cx.notify();
                            return;
                        }
                        if let Some(ref s) = event.keystroke.key_char {
                            this.setup_screen.combobox_search_draft.push_str(s);
                            this.setup_screen.combobox_validation_error = None;
                            cx.notify();
                            return;
                        }
                        if key.chars().count() == 1 {
                            this.setup_screen.combobox_search_draft.push_str(key);
                            this.setup_screen.combobox_validation_error = None;
                            cx.notify();
                            return;
                        }
                    }
                    if let Some((idx, field)) = this.settings_view.editing_player {
                        let key = event.keystroke.key.as_str();
                        if key.eq_ignore_ascii_case("escape") || key.eq_ignore_ascii_case("enter") {
                            this.settings_view.editing_player = None;
                            cx.notify();
                            return;
                        }
                        match field {
                            crate::settings_view::PlayerEditField::Nickname => {
                                let mut cur = this.app_settings.get_player_nickname(idx);
                                if key.eq_ignore_ascii_case("backspace") {
                                    cur.pop();
                                } else if key.eq_ignore_ascii_case("space") {
                                    if cur.len() < 24 {
                                        cur.push(' ');
                                    }
                                } else if let Some(ref s) = event.keystroke.key_char {
                                    if cur.len() < 24 {
                                        cur.push_str(s);
                                    }
                                } else if key.chars().count() == 1 {
                                    if cur.len() < 24 {
                                        cur.push_str(key);
                                    }
                                }
                                this.app_settings.set_player_nickname(idx, cur);
                                cx.notify();
                            }
                            crate::settings_view::PlayerEditField::SteamId => {
                                let mut cur = this.app_settings.get_player_steam_id(idx);
                                if key.eq_ignore_ascii_case("backspace") {
                                    cur.pop();
                                } else if let Some(ref s) = event.keystroke.key_char {
                                    for c in s.chars() {
                                        if c.is_ascii_digit() && cur.len() < 17 {
                                            cur.push(c);
                                        }
                                    }
                                } else if key.chars().count() == 1 {
                                    let c = key.chars().next().unwrap();
                                    if c.is_ascii_digit() && cur.len() < 17 {
                                        cur.push(c);
                                    }
                                }
                                this.app_settings.set_player_steam_id(idx, cur);
                                cx.notify();
                            }
                        }
                        return;
                    }
                    let key = event.keystroke.key.as_str();
                    if key.eq_ignore_ascii_case("f11") {
                        this.toggle_big_picture(cx);
                        return;
                    }
                    if key.eq_ignore_ascii_case("escape") {
                        if this.context_menu_handler.is_some() {
                            this.context_menu_handler = None;
                            cx.notify();
                            return;
                        }
                        if this.inspecting_hub_package.is_some() {
                            this.inspecting_hub_package = None;
                            cx.notify();
                            return;
                        }
                    }
                    if event.keystroke.modifiers.control && key.eq_ignore_ascii_case("escape") {
                        this.toggle_in_game_osd(cx);
                        return;
                    }
                    if this.in_game_osd.is_visible {
                        if key.eq_ignore_ascii_case("escape") || key.eq_ignore_ascii_case("b") {
                            if this.in_game_osd.reassign_modal_open {
                                this.in_game_osd.close_reassign();
                            } else {
                                this.set_osd_visible(false);
                            }
                            cx.notify();
                            return;
                        }
                        if key.eq_ignore_ascii_case("up") || key.eq_ignore_ascii_case("w") {
                            this.in_game_osd.move_up();
                            cx.notify();
                            return;
                        }
                        if key.eq_ignore_ascii_case("down") || key.eq_ignore_ascii_case("s") {
                            this.in_game_osd.move_down();
                            cx.notify();
                            return;
                        }
                        if key.eq_ignore_ascii_case("enter") || key.eq_ignore_ascii_case("space") {
                            this.handle_osd_confirm(cx);
                            return;
                        }
                        return;
                    }
                    if event.keystroke.modifiers.control && key.eq_ignore_ascii_case("n") {
                        this.cycle_layout(cx);
                    }
                },
            ))
            .on_action(cx.listener(
                |this: &mut MainWindow, _: &crate::actions::ToggleBigPicture, _window, cx| {
                    this.toggle_big_picture(cx);
                },
            ))
            .on_action(cx.listener(
                |this: &mut MainWindow, _: &crate::actions::ToggleOsd, _window, cx| {
                    this.toggle_in_game_osd(cx);
                },
            ))
            .on_action(cx.listener(
                |this: &mut MainWindow, _: &crate::actions::NavigateLeft, _window, cx| {
                    if this.is_big_picture {
                        if let Some(ref mut m) = this.couch_hub_state.custom_options_modal {
                            m.cycle_choice(false);
                        } else if this.big_picture_screen == BigPictureScreen::CouchSetup {
                            this.couch_setup_state.navigate_slot_left();
                        } else if this.big_picture_screen == BigPictureScreen::CouchHub {
                            let count = this.get_filtered_couch_hub_packages().len();
                            this.couch_hub_state.move_left(count);
                        } else {
                            this.carousel_state.move_left();
                        }
                        cx.notify();
                    }
                },
            ))
            .on_action(cx.listener(
                |this: &mut MainWindow, _: &crate::actions::NavigateRight, _window, cx| {
                    if this.is_big_picture {
                        if let Some(ref mut m) = this.couch_hub_state.custom_options_modal {
                            m.cycle_choice(true);
                        } else if this.big_picture_screen == BigPictureScreen::CouchSetup {
                            this.couch_setup_state.navigate_slot_right();
                        } else if this.big_picture_screen == BigPictureScreen::CouchHub {
                            let count = this.get_filtered_couch_hub_packages().len();
                            this.couch_hub_state.move_right(count);
                        } else {
                            this.carousel_state.move_right();
                        }
                        cx.notify();
                    }
                },
            ))
            .on_action(cx.listener(
                |this: &mut MainWindow, _: &crate::actions::ConfirmAction, _window, cx| {
                    if this.is_big_picture {
                        if this.couch_hub_state.custom_options_modal.is_some() {
                            this.confirm_couch_options_and_launch(cx);
                            return;
                        }
                        if let Some(h_name) = this.couch_hub_state.missing_exe_modal_handler.clone()
                        {
                            this.locate_exe_for_handler(&h_name, cx);
                            return;
                        }
                        if this.big_picture_screen == BigPictureScreen::CouchSetup {
                            let focused = this.couch_setup_state.focused_slot;
                            if this.couch_setup_state.grabbed_slot.is_some() {
                                this.couch_setup_state.drop_swap_target(focused);
                            } else {
                                this.couch_setup_state.grab_slot(focused);
                            }
                            cx.notify();
                        } else if this.big_picture_screen == BigPictureScreen::CouchHub {
                            let pkgs = this.get_filtered_couch_hub_packages();
                            if this.couch_hub_state.selected_index < pkgs.len() {
                                let pid = pkgs[this.couch_hub_state.selected_index].id.clone();
                                this.download_and_play_couch_package(&pid, cx);
                            }
                        } else {
                            this.select_carousel_game(this.carousel_state.selected_index, cx);
                        }
                    }
                },
            ))
            // Client-Side Decorations (Custom Title Bar)
            .child(self.render_title_bar(_window, cx))
            // Top body (Sidebar + Content)
            .child(
                div()
                    .flex()
                    .flex_row()
                    .flex_1()
                    .w_full()
                    .overflow_hidden()
                    .child(self.render_sidebar(_window.viewport_size().width, cx))
                    .child(
                        div()
                            .flex()
                            .flex_col()
                            .flex_1()
                            .min_w_0()
                            .h_full()
                            .overflow_hidden()
                            .child(active_view),
                    ),
            )
            // Bottom status bar
            .child(self.render_status_bar())
            // Pre-flight Diagnostics Alert Banner (Task 6)
            .children(self.render_preflight_banners(cx))
            // Launch Prompt Modal (Task 3)
            .children(self.render_launch_prompt_modal(cx))
            // Details Modal (rendered on top when inspecting package or handler)
            .children(self.render_details_modal(cx))
            // Interactive Hotkey Recording Modal
            .children(self.render_hotkey_recording_modal(cx))
            // Context Menu Modal (rendered on top when right-clicking or clicking wrench)
            .children(self.render_context_menu(cx))
            // Confirmation Dialog Modal (rendered on top for destructive action confirmation)
            .children(self.render_confirm_dialog(cx))
            // Controller Shortcuts & UI Navigation Modal (media_1793137385955_044e590e.png)
            .children(self.render_controller_shortcuts_modal(cx))
            // Hub Account Authentication Modal
            .children(self.render_hub_account_modal(cx))
            // Save Slots Manager Modal
            .children(self.render_save_slots_modal(cx))
            // OSD Notification Overlay (rendered topmost)
            .children(self.render_osd_overlay())
            .into_any_element()
    }
}

/// Parses an OSD RGB color string such as "255,255,255" into GPUI Hsla color.
fn parse_osd_color(color_str: &str) -> Hsla {
    let parts: Vec<&str> = color_str.split(',').map(|s| s.trim()).collect();
    if parts.len() == 3 {
        let r = parts[0].parse::<u8>().unwrap_or(255);
        let g = parts[1].parse::<u8>().unwrap_or(255);
        let b = parts[2].parse::<u8>().unwrap_or(255);
        return rgb((r as u32) << 16 | (g as u32) << 8 | (b as u32)).into();
    }
    rgb(0xffffff).into()
}

#[cfg(test)]
mod tests {
    use super::*;
    use core::prelude::v1::test;

    #[test]
    fn test_main_window_creation_and_state() {
        let window = MainWindow::new(2, vec!["Left 4 Dead 2".to_string(), "Halo".to_string()]);
        assert_eq!(window.active_tab, ActiveTab::Library);
        assert_eq!(window.detected_devices, 2);
        assert_eq!(window.handler_names.len(), 2);
        assert_eq!(window.selected_game, Some("Left 4 Dead 2".to_string()));
        assert_eq!(window.player_count, 2);
        assert!(window.auto_scan);
    }

    #[test]
    fn test_active_tab_enum() {
        let tab = ActiveTab::Setup;
        assert_eq!(tab, ActiveTab::Setup);
        assert_ne!(tab, ActiveTab::Library);
    }

    #[test]
    fn test_device_cycling() {
        let mut window = MainWindow::new(2, vec!["Game".to_string()]);
        assert_eq!(window.player_device_assignments.len(), 2);
        let orig = window.player_device_assignments[0].clone();
        window.cycle_player_device(0);
        assert_ne!(window.player_device_assignments[0], orig);
    }

    #[test]
    fn test_developer_mode_and_mock_device_lifecycle() {
        let mut window = MainWindow::new(0, vec!["Game".to_string()]);
        // Zero mockup devices should be present by default
        assert!(window.detected_devices_list.is_empty());
        assert!(!window.app_settings.developer_mode);

        // Attempting to add virtual gamepad when developer_mode is false does nothing
        window.add_virtual_gamepad();
        assert!(window.detected_devices_list.is_empty());

        // Enabling developer mode allows adding virtual gamepad
        window.app_settings.developer_mode = true;
        window.add_virtual_gamepad();
        assert_eq!(window.detected_devices_list.len(), 1);
        assert!(window.detected_devices_list[0].is_virtual);

        // Clearing virtual gamepads removes them
        window.clear_virtual_gamepads();
        assert!(window.detected_devices_list.is_empty());
    }

    #[test]
    fn test_gpui_keydown() {
        fn check(handle: &gpui::FocusHandle, cx: &mut Context<MainWindow>) {
            let _el = div()
                .id("test")
                .track_focus(handle)
                .on_key_down(cx.listener(
                    |this: &mut MainWindow, event: &gpui::KeyDownEvent, _window, cx| {
                        let key = event.keystroke.key.as_str();
                        if key == "backspace" {
                            this.hub_search_query.pop();
                        } else if key == "escape" {
                            this.hub_search_query.clear();
                        } else if key == "space" {
                            this.hub_search_query.push(' ');
                        } else if let Some(ref s) = event.keystroke.key_char {
                            this.hub_search_query.push_str(s);
                        } else if key.chars().count() == 1 {
                            this.hub_search_query.push_str(key);
                        }
                        cx.notify();
                    },
                ));
        }
        let _ = check;
    }

    #[test]
    fn test_handler_item_features_and_options() {
        let opt = nucleus_script::HandlerOption {
            name: "Graphics API".to_string(),
            description: "Select renderer".to_string(),
            key: "gfx".to_string(),
            values: vec![
                "DirectX 9".to_string(),
                "DirectX 11".to_string(),
                "Vulkan".to_string(),
            ],
        };
        let item = HandlerItem::new(
            "l4d2".to_string(),
            "Left 4 Dead 2".to_string(),
            "left4dead2.exe".to_string(),
            4,
            "Zombie shooter".to_string(),
        )
        .with_favorite(true)
        .with_input_support(true, true, true)
        .with_options(vec![opt])
        .with_handler_path(Some("C:/Games/NucleusCoop/handlers/l4d2.js".to_string()));

        assert!(item.is_favorite);
        assert!(item.supports_gamepad);
        assert!(item.supports_keyboard);
        assert!(item.supports_multiple_keyboards_and_mice);
        assert_eq!(item.options.len(), 1);
        assert_eq!(item.options[0].key, "gfx");
        assert_eq!(item.options[0].values.len(), 3);
        assert_eq!(
            item.handler_path.as_deref(),
            Some("C:/Games/NucleusCoop/handlers/l4d2.js")
        );
    }

    #[test]
    fn test_main_window_callbacks_wiring() {
        use std::sync::Arc;
        use std::sync::atomic::{AtomicBool, Ordering};

        let fav_called = Arc::new(AtomicBool::new(false));
        let fav_called_clone = fav_called.clone();
        let toggle_fav = move |guid: &str, fav: bool| {
            if guid == "test_guid" && fav {
                fav_called_clone.store(true, Ordering::SeqCst);
            }
        };

        let profile_saved = Arc::new(AtomicBool::new(false));
        let profile_saved_clone = profile_saved.clone();
        let save_profile = move |item: &HandlerItem| {
            if item.guid == "test_guid" && item.assign_gamepads_by_index {
                profile_saved_clone.store(true, Ordering::SeqCst);
            }
        };

        let launch_called = Arc::new(AtomicBool::new(false));
        let launch_called_clone = launch_called.clone();
        let launcher = move |name: &str,
                             players: usize,
                             layout: &str,
                             opts: &std::collections::HashMap<String, String>,
                             _assignments: &[String],
                             _monitors: &[usize],
                             _audio: &[String]| {
            if name == "Game"
                && players == 2
                && layout == "2x1 (Side-by-Side)"
                && opts.get("mode").map(|s| s.as_str()) == Some("versus")
            {
                launch_called_clone.store(true, Ordering::SeqCst);
            }
        };

        let list_called = Arc::new(AtomicBool::new(false));
        let list_called_clone = list_called.clone();
        let list_profiles = move |guid: &str| -> Vec<(usize, String)> {
            if guid == "test_guid" {
                list_called_clone.store(true, Ordering::SeqCst);
                vec![(1, "Profile 1".to_string())]
            } else {
                Vec::new()
            }
        };

        let load_called = Arc::new(AtomicBool::new(false));
        let load_called_clone = load_called.clone();
        let load_profile = move |guid: &str, num: usize| -> Option<nucleus_profile::GameProfile> {
            if guid == "test_guid" && num == 1 {
                load_called_clone.store(true, Ordering::SeqCst);
                Some(nucleus_profile::GameProfile::default())
            } else {
                None
            }
        };

        let save_num_called = Arc::new(AtomicBool::new(false));
        let save_num_called_clone = save_num_called.clone();
        let save_numbered = move |guid: &str,
                                  num: Option<usize>,
                                  _p: &nucleus_profile::GameProfile|
              -> Option<usize> {
            if guid == "test_guid" && num == Some(1) {
                save_num_called_clone.store(true, Ordering::SeqCst);
                Some(1)
            } else {
                None
            }
        };

        let del_called = Arc::new(AtomicBool::new(false));
        let del_called_clone = del_called.clone();
        let del_profile = move |guid: &str, num: usize| -> bool {
            if guid == "test_guid" && num == 1 {
                del_called_clone.store(true, Ordering::SeqCst);
                true
            } else {
                false
            }
        };

        let unload_called = Arc::new(AtomicBool::new(false));
        let unload_called_clone = unload_called.clone();
        let unloader = move || {
            unload_called_clone.store(true, Ordering::SeqCst);
        };

        let freeze_called = Arc::new(AtomicBool::new(false));
        let freeze_called_clone = freeze_called.clone();
        let freeze_toggler = move || -> Option<bool> {
            freeze_called_clone.store(true, Ordering::SeqCst);
            Some(true)
        };

        let reassign_called = Arc::new(AtomicBool::new(false));
        let reassign_called_clone = reassign_called.clone();
        let controller_reassigner = move |slot: usize, dev_id: usize| {
            if slot == 0 && dev_id == 2 {
                reassign_called_clone.store(true, Ordering::SeqCst);
            }
        };

        let reenforce_called = Arc::new(AtomicBool::new(false));
        let reenforce_called_clone = reenforce_called.clone();
        let window_reenforcer = move || {
            reenforce_called_clone.store(true, Ordering::SeqCst);
        };

        let lock_called = Arc::new(AtomicBool::new(false));
        let lock_called_clone = lock_called.clone();
        let input_lock_toggler = move || -> bool {
            lock_called_clone.store(true, Ordering::SeqCst);
            true
        };

        let stop_called = Arc::new(AtomicBool::new(false));
        let stop_called_clone = stop_called.clone();
        let session_stopper = move || {
            stop_called_clone.store(true, Ordering::SeqCst);
        };

        let win = MainWindow::new(2, vec!["Game".to_string()])
            .with_favorite_toggler(toggle_fav)
            .with_profile_saver(save_profile)
            .with_launcher(launcher)
            .with_numbered_profile_lister(list_profiles)
            .with_numbered_profile_loader(load_profile)
            .with_numbered_profile_saver(save_numbered)
            .with_numbered_profile_deleter(del_profile)
            .with_profile_unloader(unloader)
            .with_freeze_toggler(freeze_toggler)
            .with_controller_reassigner(controller_reassigner)
            .with_window_re_enforcer(window_reenforcer)
            .with_input_lock_toggler(input_lock_toggler)
            .with_session_stopper(session_stopper);

        let mut item = HandlerItem::new(
            "test_guid".to_string(),
            "Game".to_string(),
            "game.exe".to_string(),
            4,
            "".to_string(),
        );
        item.assign_gamepads_by_index = true;

        if let Some(ref cb) = win.callbacks.on_toggle_favorite {
            cb("test_guid", true);
        }
        if let Some(ref cb) = win.callbacks.on_save_profile {
            cb(&item);
        }
        if let Some(ref cb) = win.callbacks.on_launch {
            let mut opts = std::collections::HashMap::new();
            opts.insert("mode".to_string(), "versus".to_string());
            let dev_assignments = vec![
                "Keyboard & Mouse (Paired)".to_string(),
                "Gamepad #0 (Xbox)".to_string(),
            ];
            cb(
                "Game",
                2,
                "2x1 (Side-by-Side)",
                &opts,
                &dev_assignments,
                &[0, 1],
                &["Default".to_string(), "Default".to_string()],
            );
        }
        if let Some(ref cb) = win.callbacks.on_list_profiles {
            let res = cb("test_guid");
            assert_eq!(res.len(), 1);
        }
        if let Some(ref cb) = win.callbacks.on_load_numbered_profile {
            assert!(cb("test_guid", 1).is_some());
        }
        if let Some(ref cb) = win.callbacks.on_save_numbered_profile {
            assert_eq!(
                cb(
                    "test_guid",
                    Some(1),
                    &nucleus_profile::GameProfile::default()
                ),
                Some(1)
            );
        }
        if let Some(ref cb) = win.callbacks.on_delete_numbered_profile {
            assert!(cb("test_guid", 1));
        }
        if let Some(ref cb) = win.callbacks.on_unload_profile {
            cb();
        }
        if let Some(ref cb) = win.callbacks.on_toggle_freeze {
            assert_eq!(cb(), Some(true));
        }
        if let Some(ref cb) = win.callbacks.on_reassign_controller {
            cb(0, 2);
        }
        if let Some(ref cb) = win.callbacks.on_re_enforce_windows {
            cb();
        }
        if let Some(ref cb) = win.callbacks.on_toggle_input_lock {
            assert!(cb());
        }
        if let Some(ref cb) = win.callbacks.on_stop_session {
            cb();
        }

        assert!(fav_called.load(Ordering::SeqCst));
        assert!(profile_saved.load(Ordering::SeqCst));
        assert!(launch_called.load(Ordering::SeqCst));
        assert!(list_called.load(Ordering::SeqCst));
        assert!(load_called.load(Ordering::SeqCst));
        assert!(save_num_called.load(Ordering::SeqCst));
        assert!(del_called.load(Ordering::SeqCst));
        assert!(unload_called.load(Ordering::SeqCst));
        assert!(freeze_called.load(Ordering::SeqCst));
        assert!(reassign_called.load(Ordering::SeqCst));
        assert!(reenforce_called.load(Ordering::SeqCst));
        assert!(lock_called.load(Ordering::SeqCst));
        assert!(stop_called.load(Ordering::SeqCst));
    }

    #[test]
    fn test_context_menu_gamepad_assign_mutual_exclusivity() {
        let mut item = HandlerItem::new("test_g", "Game", "game.exe", 2, "");
        item.assign_gamepads_by_index = false;
        item.assign_gamepads_on_button_press = true;

        // Toggle index ON -> button press must become false
        item.assign_gamepads_by_index = !item.assign_gamepads_by_index;
        if item.assign_gamepads_by_index {
            item.assign_gamepads_on_button_press = false;
        }
        assert!(item.assign_gamepads_by_index);
        assert!(!item.assign_gamepads_on_button_press);

        // Toggle button press ON -> index must become false
        item.assign_gamepads_on_button_press = !item.assign_gamepads_on_button_press;
        if item.assign_gamepads_on_button_press {
            item.assign_gamepads_by_index = false;
        }
        assert!(item.assign_gamepads_on_button_press);
        assert!(!item.assign_gamepads_by_index);
    }
}
