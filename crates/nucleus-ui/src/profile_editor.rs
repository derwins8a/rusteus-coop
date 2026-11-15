//! Game profile and launch option editor view.
//!
//! Allows fine-tuning per-game arguments, environment variables, Nicknames,
//! and custom handler options for specific game configurations.

/// Editor component for game-specific launch parameters and player profiles.
#[derive(Default)]
pub struct ProfileEditorView {
    /// Identifier of the game profile being edited.
    pub profile_id: Option<String>,
}

impl ProfileEditorView {
    /// Creates a new `ProfileEditorView`.
    pub fn new() -> Self {
        Self::default()
    }
}
