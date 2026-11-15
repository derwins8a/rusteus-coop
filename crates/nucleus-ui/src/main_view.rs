//! Main application view container and navigation frame.
//!
//! Hosts the primary layout including sidebar navigation, header status bar,
//! and the active view pane (game list, setup screen, hub browser, or settings).

/// State and component representation for the main application view.
#[derive(Default)]
pub struct MainView {
    /// Active tab or view identifier.
    pub active_tab: ActiveTab,
}

/// Active navigation view tab.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum ActiveTab {
    /// Game list / library view.
    #[default]
    GameList,
    /// Player and device setup screen.
    SetupScreen,
    /// Community handler hub browser.
    HubBrowser,
    /// Extensions and add-ons ecosystem manager.
    Extensions,
    /// Step-by-step handler debugger and runtime inspector.
    Debugger,
    /// Application settings panel.
    Settings,
}

impl MainView {
    /// Creates a new `MainView` instance.
    pub fn new() -> Self {
        Self::default()
    }

    /// Switches the currently visible view tab.
    pub fn navigate_to(&mut self, tab: ActiveTab) {
        self.active_tab = tab;
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_main_view_navigation() {
        let mut view = MainView::new();
        assert_eq!(view.active_tab, ActiveTab::GameList);

        view.navigate_to(ActiveTab::SetupScreen);
        assert_eq!(view.active_tab, ActiveTab::SetupScreen);

        view.navigate_to(ActiveTab::HubBrowser);
        assert_eq!(view.active_tab, ActiveTab::HubBrowser);

        view.navigate_to(ActiveTab::Settings);
        assert_eq!(view.active_tab, ActiveTab::Settings);
    }
}
