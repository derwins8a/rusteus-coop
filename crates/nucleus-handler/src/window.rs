//! Window positioning, sizing, and styling setup.

use nucleus_common::error::NucleusResult;
use nucleus_common::types::{Rect, WindowHandle};
use nucleus_platform::WindowManager;

/// Rectangle representing screen bounds for window placement.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct WindowBounds {
    /// X coordinate of the top-left corner.
    pub x: i32,
    /// Y coordinate of the top-left corner.
    pub y: i32,
    /// Width in pixels.
    pub width: i32,
    /// Height in pixels.
    pub height: i32,
}

/// Manages window discovery, repositioning, and style modifications for instances.
pub struct WindowSetup;

impl WindowSetup {
    /// Finds the primary window handle for a given process ID.
    pub fn find_window_by_pid(win_mgr: &dyn WindowManager, pid: u32) -> NucleusResult<Option<usize>> {
        let handles = win_mgr.find_windows_by_process(pid)?;
        Ok(handles.first().map(|h| h.0 as usize))
    }

    /// Positions and resizes the specified window to the assigned bounds.
    pub fn apply_bounds(win_mgr: &dyn WindowManager, hwnd: usize, bounds: WindowBounds) -> NucleusResult<()> {
        Self::apply_bounds_ext(win_mgr, hwnd, bounds, false)
    }

    /// Positions and resizes the specified window with explicit topmost guard control.
    pub fn apply_bounds_ext(
        win_mgr: &dyn WindowManager,
        hwnd: usize,
        bounds: WindowBounds,
        not_top_most: bool,
    ) -> NucleusResult<()> {
        let rect = Rect {
            x: bounds.x,
            y: bounds.y,
            width: bounds.width,
            height: bounds.height,
        };
        win_mgr.set_window_position_ext(WindowHandle(hwnd as u64), &rect, not_top_most)
    }

    /// Calculates centered pillarboxed or letterboxed window bounds to preserve aspect ratio,
    /// implementing 100% C# parity with splitscreenme-nucleus GlobalWindowMethods.cs:1412-1457.
    pub fn apply_aspect_ratio_correction(
        target_bounds: WindowBounds,
        orig_width: i32,
        orig_height: i32,
    ) -> WindowBounds {
        if orig_width <= 0 || orig_height <= 0 || target_bounds.width <= 0 || target_bounds.height <= 0 {
            return target_bounds;
        }

        let orig_ratio = (orig_width as f64) / (orig_height as f64);
        let target_w = target_bounds.width as f64;
        let target_h = target_bounds.height as f64;
        let target_ratio = target_w / target_h;

        let (new_w, new_h) = if target_ratio > orig_ratio {
            // Target slot is wider than original ratio: fit to height, letterbox sides (pillarbox)
            let h = target_h;
            let w = h * orig_ratio;
            (w, h)
        } else {
            // Target slot is taller than original ratio: fit to width, letterbox top/bottom
            let w = target_w;
            let h = w / orig_ratio;
            (w, h)
        };

        let new_w_int = (new_w.round() as i32).max(1);
        let new_h_int = (new_h.round() as i32).max(1);

        // Center horizontally and vertically within the target viewport (letterboxing/pillarboxing)
        let x_offset = target_bounds.x + ((target_bounds.width - new_w_int) / 2);
        let y_offset = target_bounds.y + ((target_bounds.height - new_h_int) / 2);

        WindowBounds {
            x: x_offset,
            y: y_offset,
            width: new_w_int,
            height: new_h_int,
        }
    }

    /// Strips window borders and title bars for borderless fullscreen mode.
    pub fn make_borderless(win_mgr: &dyn WindowManager, hwnd: usize) -> NucleusResult<()> {
        win_mgr.remove_borders(WindowHandle(hwnd as u64))
    }

    /// Calculates regular M rows x N cols grid bounds for the requested player count.
    pub fn calculate_regular_grid(
        screen: Rect,
        rows: usize,
        cols: usize,
        player_count: usize,
    ) -> Vec<WindowBounds> {
        if rows == 0 || cols == 0 || player_count == 0 {
            return Vec::new();
        }
        let cell_w = screen.width / cols as i32;
        let cell_h = screen.height / rows as i32;
        (0..player_count)
            .map(|i| {
                let r = i / cols;
                let c = i % cols;
                WindowBounds {
                    x: screen.x + (c as i32 * cell_w),
                    y: screen.y + (r as i32 * cell_h),
                    width: cell_w,
                    height: cell_h,
                }
            })
            .collect()
    }

    /// Calculates window bounds for all players on a monitor according to the requested split mode.
    pub fn calculate_splits(
        screen: Rect,
        player_count: usize,
        mode: SplitMode,
    ) -> Vec<WindowBounds> {
        if player_count == 0 {
            return Vec::new();
        }
        if player_count == 1 {
            return vec![WindowBounds {
                x: screen.x,
                y: screen.y,
                width: screen.width,
                height: screen.height,
            }];
        }

        let chosen_mode = match mode {
            SplitMode::Auto => {
                if player_count == 2 {
                    SplitMode::Vertical
                } else if player_count == 3 {
                    SplitMode::TopOneBottomTwo
                } else if player_count <= 4 {
                    SplitMode::Grid4
                } else if player_count <= 6 {
                    SplitMode::Grid6
                } else if player_count <= 8 {
                    SplitMode::Grid8
                } else if player_count <= 16 {
                    SplitMode::Grid16
                } else {
                    let cols = (player_count as f64).sqrt().ceil() as usize;
                    let rows = ((player_count as f64) / cols as f64).ceil() as usize;
                    SplitMode::CustomGrid { rows, cols }
                }
            }
            other => other,
        };

        match chosen_mode {
            SplitMode::Horizontal => Self::calculate_regular_grid(screen, player_count, 1, player_count),
            SplitMode::Vertical => Self::calculate_regular_grid(screen, 1, player_count, player_count),
            SplitMode::Grid4 => Self::calculate_regular_grid(screen, 2, 2, player_count),
            SplitMode::Grid6 => Self::calculate_regular_grid(screen, 2, 3, player_count),
            SplitMode::Grid8 => Self::calculate_regular_grid(screen, 2, 4, player_count),
            SplitMode::Grid16 => Self::calculate_regular_grid(screen, 4, 4, player_count),
            SplitMode::CustomGrid { rows, cols } => {
                Self::calculate_regular_grid(screen, rows, cols, player_count)
            }
            SplitMode::TopOneBottomTwo => {
                let h = screen.height / 2;
                let w = screen.width / 2;
                vec![
                    WindowBounds { x: screen.x, y: screen.y, width: screen.width, height: h },
                    WindowBounds { x: screen.x, y: screen.y + h, width: w, height: h },
                    WindowBounds { x: screen.x + w, y: screen.y + h, width: w, height: h },
                ]
                .into_iter()
                .take(player_count)
                .collect()
            }
            SplitMode::TopTwoBottomOne => {
                let h = screen.height / 2;
                let w = screen.width / 2;
                vec![
                    WindowBounds { x: screen.x, y: screen.y, width: w, height: h },
                    WindowBounds { x: screen.x + w, y: screen.y, width: w, height: h },
                    WindowBounds { x: screen.x, y: screen.y + h, width: screen.width, height: h },
                ]
                .into_iter()
                .take(player_count)
                .collect()
            }
            SplitMode::Auto => unreachable!(),
        }
    }
}

/// Strategy for dividing a monitor among player instances.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum SplitMode {
    /// Automatically selects optimal layout based on player count.
    #[default]
    Auto,
    /// Stacked horizontal bands (good for ultrawide).
    Horizontal,
    /// Side-by-side vertical columns.
    Vertical,
    /// 2x2 four-quadrant grid.
    Grid4,
    /// 6 players: 2 rows x 3 columns grid.
    Grid6,
    /// 8 players: 2 rows x 4 columns grid.
    Grid8,
    /// 16 players: 4 rows x 4 columns grid.
    Grid16,
    /// Custom M rows x N columns regular grid.
    CustomGrid { rows: usize, cols: usize },
    /// 3 players: 1 full-width top, 2 half-width bottom.
    TopOneBottomTwo,
    /// 3 players: 2 half-width top, 1 full-width bottom.
    TopTwoBottomOne,
}

/// Parses a UI or profile layout string into its canonical `SplitMode`.
pub fn split_mode_from_name(name: &str) -> SplitMode {
    let lower = name.trim().to_ascii_lowercase();
    if lower.is_empty()
        || lower.contains("auto")
        || lower.contains("multi")
        || lower.contains("1x1")
        || lower.contains("single")
        || lower.contains("fullscreen")
    {
        return SplitMode::Auto;
    }
    if lower.contains("top 1")
        || lower.contains("top1")
        || lower.contains("topone")
        || lower.contains("3-way")
    {
        return SplitMode::TopOneBottomTwo;
    }
    if lower.contains("top 2") || lower.contains("top2") || lower.contains("toptwo") {
        return SplitMode::TopTwoBottomOne;
    }
    if lower.contains("1x2") || lower.contains("stacked") || lower.contains("horizontal") {
        return SplitMode::Horizontal;
    }
    if lower.contains("2x1") || lower.contains("side-by-side") || lower.contains("vertical") {
        return SplitMode::Vertical;
    }
    if lower.contains("4x4") || lower == "16" || lower.contains("grid 16") {
        return SplitMode::Grid16;
    }
    if lower.contains("4x2") || lower == "8" || lower.contains("grid 8") {
        return SplitMode::Grid8;
    }
    if lower.contains("3x2") || lower == "6" || lower.contains("grid 6") {
        return SplitMode::Grid6;
    }
    if lower.contains("2x2") || lower.contains("quad") || lower.contains("grid") {
        return SplitMode::Grid4;
    }
    SplitMode::Auto
}

impl SplitMode {
    /// Parses a UI or profile layout string into its canonical `SplitMode`.
    pub fn from_layout_name(name: &str) -> Self {
        split_mode_from_name(name)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_calculate_splits() {
        let screen = Rect { x: 0, y: 0, width: 1920, height: 1080 };

        // 2 players Auto -> Vertical
        let splits2 = WindowSetup::calculate_splits(screen, 2, SplitMode::Auto);
        assert_eq!(splits2.len(), 2);
        assert_eq!(splits2[0], WindowBounds { x: 0, y: 0, width: 960, height: 1080 });
        assert_eq!(splits2[1], WindowBounds { x: 960, y: 0, width: 960, height: 1080 });

        // 2 players Horizontal
        let splits2_h = WindowSetup::calculate_splits(screen, 2, SplitMode::Horizontal);
        assert_eq!(splits2_h[0], WindowBounds { x: 0, y: 0, width: 1920, height: 540 });
        assert_eq!(splits2_h[1], WindowBounds { x: 0, y: 540, width: 1920, height: 540 });

        // 4 players Auto -> Grid4
        let splits4 = WindowSetup::calculate_splits(screen, 4, SplitMode::Auto);
        assert_eq!(splits4.len(), 4);
        assert_eq!(splits4[0], WindowBounds { x: 0, y: 0, width: 960, height: 540 });
        assert_eq!(splits4[3], WindowBounds { x: 960, y: 540, width: 960, height: 540 });

        // 3 players TopOneBottomTwo
        let splits3 = WindowSetup::calculate_splits(screen, 3, SplitMode::TopOneBottomTwo);
        assert_eq!(splits3.len(), 3);
        assert_eq!(splits3[0], WindowBounds { x: 0, y: 0, width: 1920, height: 540 });
        assert_eq!(splits3[1], WindowBounds { x: 0, y: 540, width: 960, height: 540 });
        assert_eq!(splits3[2], WindowBounds { x: 960, y: 540, width: 960, height: 540 });

        // 6 players Grid6 (2 rows x 3 cols: cell width 640, height 540)
        let splits6 = WindowSetup::calculate_splits(screen, 6, SplitMode::Grid6);
        assert_eq!(splits6.len(), 6);
        assert_eq!(splits6[0], WindowBounds { x: 0, y: 0, width: 640, height: 540 });
        assert_eq!(splits6[1], WindowBounds { x: 640, y: 0, width: 640, height: 540 });
        assert_eq!(splits6[2], WindowBounds { x: 1280, y: 0, width: 640, height: 540 });
        assert_eq!(splits6[3], WindowBounds { x: 0, y: 540, width: 640, height: 540 });
        assert_eq!(splits6[5], WindowBounds { x: 1280, y: 540, width: 640, height: 540 });

        // 8 players Grid8 (2 rows x 4 cols: cell width 480, height 540)
        let splits8 = WindowSetup::calculate_splits(screen, 8, SplitMode::Grid8);
        assert_eq!(splits8.len(), 8);
        assert_eq!(splits8[0], WindowBounds { x: 0, y: 0, width: 480, height: 540 });
        assert_eq!(splits8[7], WindowBounds { x: 1440, y: 540, width: 480, height: 540 });

        // 16 players Grid16 (4 rows x 4 cols: cell width 480, height 270)
        let splits16 = WindowSetup::calculate_splits(screen, 16, SplitMode::Grid16);
        assert_eq!(splits16.len(), 16);
        assert_eq!(splits16[0], WindowBounds { x: 0, y: 0, width: 480, height: 270 });
        assert_eq!(splits16[15], WindowBounds { x: 1440, y: 810, width: 480, height: 270 });

        // CustomGrid (3 rows x 2 cols, 5 players)
        let splits_custom = WindowSetup::calculate_splits(
            screen,
            5,
            SplitMode::CustomGrid { rows: 3, cols: 2 },
        );
        assert_eq!(splits_custom.len(), 5);
        assert_eq!(splits_custom[0], WindowBounds { x: 0, y: 0, width: 960, height: 360 });
        assert_eq!(splits_custom[4], WindowBounds { x: 0, y: 720, width: 960, height: 360 });
    }

    #[test]
    fn test_apply_aspect_ratio_correction() {
        // Monitor is 16:9 (1920x1080)
        // Target slot is vertical split (960x1080)
        let slot = WindowBounds { x: 0, y: 0, width: 960, height: 1080 };
        let corrected = WindowSetup::apply_aspect_ratio_correction(slot, 1920, 1080);
        assert_eq!(corrected.width, 960);
        assert_eq!(corrected.height, 540);
        assert_eq!(corrected.x, 0);
        assert_eq!(corrected.y, 270); // Centered vertically (1080 - 540) / 2 = 270

        // Target slot is horizontal split (1920x540)
        let slot_h = WindowBounds { x: 0, y: 0, width: 1920, height: 540 };
        let corrected_h = WindowSetup::apply_aspect_ratio_correction(slot_h, 1920, 1080);
        assert_eq!(corrected_h.width, 960);
        assert_eq!(corrected_h.height, 540);
        assert_eq!(corrected_h.x, 480); // Centered horizontally (1920 - 960) / 2 = 480
        assert_eq!(corrected_h.y, 0);
    }

    #[test]
    fn test_split_mode_from_layout_name() {
        assert_eq!(split_mode_from_name("Fullscreen (1 Player)"), SplitMode::Auto);
        assert_eq!(split_mode_from_name("1x1"), SplitMode::Auto);
        assert_eq!(split_mode_from_name("single"), SplitMode::Auto);
        assert_eq!(SplitMode::from_layout_name("1x2 (Stacked)"), SplitMode::Horizontal);
        assert_eq!(split_mode_from_name("horizontal"), SplitMode::Horizontal);
        assert_eq!(split_mode_from_name("2x1 (Side-by-Side)"), SplitMode::Vertical);
        assert_eq!(split_mode_from_name("vertical"), SplitMode::Vertical);
        assert_eq!(split_mode_from_name("Top 1 / Bottom 2"), SplitMode::TopOneBottomTwo);
        assert_eq!(split_mode_from_name("top1"), SplitMode::TopOneBottomTwo);
        assert_eq!(split_mode_from_name("3-way"), SplitMode::TopOneBottomTwo);
        assert_eq!(split_mode_from_name("Top 2 / Bottom 1"), SplitMode::TopTwoBottomOne);
        assert_eq!(split_mode_from_name("toptwo"), SplitMode::TopTwoBottomOne);
        assert_eq!(split_mode_from_name("2x2 (Grid)"), SplitMode::Grid4);
        assert_eq!(split_mode_from_name("quad"), SplitMode::Grid4);
        assert_eq!(split_mode_from_name("3x2 (Grid 6)"), SplitMode::Grid6);
        assert_eq!(split_mode_from_name("6"), SplitMode::Grid6);
        assert_eq!(split_mode_from_name("4x2 (Grid 8)"), SplitMode::Grid8);
        assert_eq!(split_mode_from_name("8"), SplitMode::Grid8);
        assert_eq!(split_mode_from_name("4x4 (Grid 16)"), SplitMode::Grid16);
        assert_eq!(split_mode_from_name("16"), SplitMode::Grid16);
        assert_eq!(split_mode_from_name("Multi-Monitor"), SplitMode::Auto);
        assert_eq!(split_mode_from_name("Auto"), SplitMode::Auto);
    }
}

