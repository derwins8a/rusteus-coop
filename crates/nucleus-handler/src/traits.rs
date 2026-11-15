//! Handler lifecycle traits defining execution and coordination stages.

use nucleus_common::NucleusResult;

/// Core interface for orchestrating multi-instance game sessions.
pub trait IGameHandler {
    /// Prepares instance directories, configuration files, and symlinks.
    fn prepare_instances(&mut self) -> NucleusResult<()>;

    /// Launches a game instance for the specified player index.
    fn launch_instance(&mut self, player_index: usize) -> NucleusResult<u32>;

    /// Positions, sizes, and configures the window for the specified player.
    fn setup_window(&mut self, player_index: usize) -> NucleusResult<()>;

    /// Re-applies borderless styles and bounds to an existing instance window.
    fn reposition_instance_window(&self, _player_index: usize) -> NucleusResult<bool> {
        Ok(false)
    }

    /// Dynamically switches/cycles splitscreen layout partitions for active instances.
    fn switch_layout(&mut self) -> NucleusResult<String> {
        Ok("Default".to_string())
    }

    /// Toggles minimizing or restoring all active game windows (Ctrl+T parity).
    fn toggle_minimize_windows(&mut self) -> NucleusResult<bool> {
        Ok(false)
    }

    /// Toggles cutscenes presentation mode (Ctrl+M parity).
    fn toggle_cutscenes_mode(&mut self) -> NucleusResult<bool> {
        Ok(false)
    }

    /// Finalizes instance state once all players have launched (e.g. SetTopMostAtEnd, taskbar hide).
    fn finalize_instances(&self) -> NucleusResult<()> {
        Ok(())
    }

    /// Tears down instances, cleans up symlinks, and restores backups.
    fn stop(&mut self) -> NucleusResult<()>;
}
