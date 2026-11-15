//! User interface action definitions and command dispatchers.
//!
//! Encapsulates user interactions, menu shortcuts, launch requests, and navigation
//! commands dispatched from UI views into engine operations.

/// Command or intent triggered by user interface controls.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum UiAction {
    /// Navigate to a specific view tab.
    Navigate(String),
    /// Request start of a game session by handler GUID.
    LaunchGame(String),
    /// Stop the currently active game session.
    StopSession,
    /// Refresh the game library catalog.
    RefreshLibrary,
    /// Initiate a background disk scan for game binaries.
    ScanDisks,
    /// Open the application settings modal.
    OpenSettings,
}

gpui::actions!(
    couch,
    [
        /// Navigate focus upwards in directional menus and grids.
        NavigateUp,
        /// Navigate focus downwards in directional menus and grids.
        NavigateDown,
        /// Navigate focus to the left in directional menus and grids.
        NavigateLeft,
        /// Navigate focus to the right in directional menus and grids.
        NavigateRight,
        /// Confirm selection or activate focused element (Gamepad A / Enter / Space).
        ConfirmAction,
        /// Cancel, go back, or dismiss modal/view (Gamepad B / Escape).
        CancelAction,
        /// Switch to next tab or section (Gamepad RB).
        NextTab,
        /// Switch to previous tab or section (Gamepad LB).
        PrevTab,
        /// Primary context action, e.g. remove slot (Gamepad X).
        ContextAction1,
        /// Secondary context action, e.g. ignore device (Gamepad Y).
        ContextAction2,
        /// Fast launch splitscreen session (Gamepad Start).
        QuickLaunch,
        /// Toggle between Desktop and Couch Big Picture modes (F11 or dedicated button).
        ToggleBigPicture,
        /// Toggle In-Game Couch OSD overlay menu (Back+Start 1.5s hold or Ctrl+Escape).
        ToggleOsd,
    ]
);
