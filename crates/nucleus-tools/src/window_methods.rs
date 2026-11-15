//! Window manipulation and positioning helper methods.

use nucleus_common::error::NucleusResult;
use nucleus_common::types::{Rect, WindowHandle};
use nucleus_platform::WindowManager;

/// Tool providing window manipulation helpers such as removing window borders,
/// repositioning, sizing to screen bounds, and setting topmost z-order.
#[derive(Debug, Default, Clone)]
pub struct WindowMethodsTool;

impl WindowMethodsTool {
    /// Positions and formats an instance game window with specified bounds, borderless styling, and topmost z-order.
    pub fn position_instance_window(
        window_mgr: &dyn WindowManager,
        handle: WindowHandle,
        bounds: &Rect,
        borderless: bool,
        topmost: bool,
    ) -> NucleusResult<()> {
        if borderless {
            let _ = window_mgr.remove_borders(handle);
        }

        window_mgr.set_window_position(handle, bounds)?;

        if topmost {
            let _ = window_mgr.set_topmost(handle, true);
        }

        let _ = window_mgr.set_foreground(handle);
        Ok(())
    }

    /// Sets a friendly window title (e.g. "Game Name - Player 1").
    pub fn retitle_instance(
        window_mgr: &dyn WindowManager,
        handle: WindowHandle,
        title: &str,
    ) -> NucleusResult<()> {
        window_mgr.set_window_title(handle, title)
    }

    /// Calculates multi-instance viewport layouts (split-screen geometry) given display bounds and player count.
    pub fn calculate_split_layout(
        screen_bounds: &Rect,
        num_players: usize,
        horizontal: bool,
    ) -> Vec<Rect> {
        let x = screen_bounds.x;
        let y = screen_bounds.y;
        let w = screen_bounds.width;
        let h = screen_bounds.height;

        match num_players {
            0 => Vec::new(),
            1 => vec![*screen_bounds],
            2 => {
                if horizontal {
                    let half_h = h / 2;
                    vec![
                        Rect::new(x, y, w, half_h),
                        Rect::new(x, y + half_h, w, h - half_h),
                    ]
                } else {
                    let half_w = w / 2;
                    vec![
                        Rect::new(x, y, half_w, h),
                        Rect::new(x + half_w, y, w - half_w, h),
                    ]
                }
            }
            3 => {
                if horizontal {
                    let half_h = h / 2;
                    let half_w = w / 2;
                    vec![
                        // Player 1: Top full width
                        Rect::new(x, y, w, half_h),
                        // Player 2: Bottom left
                        Rect::new(x, y + half_h, half_w, h - half_h),
                        // Player 3: Bottom right
                        Rect::new(x + half_w, y + half_h, w - half_w, h - half_h),
                    ]
                } else {
                    let half_w = w / 2;
                    let half_h = h / 2;
                    vec![
                        // Player 1: Left full height
                        Rect::new(x, y, half_w, h),
                        // Player 2: Top right
                        Rect::new(x + half_w, y, w - half_w, half_h),
                        // Player 3: Bottom right
                        Rect::new(x + half_w, y + half_h, w - half_w, h - half_h),
                    ]
                }
            }
            4 => {
                // 2x2 Grid
                let half_w = w / 2;
                let half_h = h / 2;
                vec![
                    Rect::new(x, y, half_w, half_h),
                    Rect::new(x + half_w, y, w - half_w, half_h),
                    Rect::new(x, y + half_h, half_w, h - half_h),
                    Rect::new(x + half_w, y + half_h, w - half_w, h - half_h),
                ]
            }
            n => {
                // Generalized NxM grid for larger numbers
                let cols = (n as f64).sqrt().ceil() as i32;
                let rows = ((n as f64) / (cols as f64)).ceil() as i32;
                let cell_w = w / cols;
                let cell_h = h / rows;

                let mut rects = Vec::with_capacity(n);
                for i in 0..n {
                    let col = (i as i32) % cols;
                    let row = (i as i32) / cols;

                    let cur_x = x + col * cell_w;
                    let cur_y = y + row * cell_h;
                    let cur_w = if col == cols - 1 {
                        w - col * cell_w
                    } else {
                        cell_w
                    };
                    let cur_h = if row == rows - 1 {
                        h - row * cell_h
                    } else {
                        cell_h
                    };

                    rects.push(Rect::new(cur_x, cur_y, cur_w, cur_h));
                }
                rects
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_calculate_split_layout_1_player() {
        let screen = Rect::new(0, 0, 1920, 1080);
        let layout = WindowMethodsTool::calculate_split_layout(&screen, 1, false);
        assert_eq!(layout.len(), 1);
        assert_eq!(layout[0], screen);
    }

    #[test]
    fn test_calculate_split_layout_2_players() {
        let screen = Rect::new(0, 0, 1920, 1080);
        // Vertical split (side-by-side)
        let vertical = WindowMethodsTool::calculate_split_layout(&screen, 2, false);
        assert_eq!(vertical.len(), 2);
        assert_eq!(vertical[0], Rect::new(0, 0, 960, 1080));
        assert_eq!(vertical[1], Rect::new(960, 0, 960, 1080));

        // Horizontal split (top / bottom)
        let horizontal = WindowMethodsTool::calculate_split_layout(&screen, 2, true);
        assert_eq!(horizontal.len(), 2);
        assert_eq!(horizontal[0], Rect::new(0, 0, 1920, 540));
        assert_eq!(horizontal[1], Rect::new(0, 540, 1920, 540));
    }

    #[test]
    fn test_calculate_split_layout_4_players() {
        let screen = Rect::new(0, 0, 1920, 1080);
        let grid = WindowMethodsTool::calculate_split_layout(&screen, 4, false);
        assert_eq!(grid.len(), 4);
        assert_eq!(grid[0], Rect::new(0, 0, 960, 540));
        assert_eq!(grid[1], Rect::new(960, 0, 960, 540));
        assert_eq!(grid[2], Rect::new(0, 540, 960, 540));
        assert_eq!(grid[3], Rect::new(960, 540, 960, 540));
    }

    #[test]
    fn test_calculate_split_layout_3_players() {
        let screen = Rect::new(0, 0, 1920, 1080);
        let layout = WindowMethodsTool::calculate_split_layout(&screen, 3, true);
        assert_eq!(layout.len(), 3);
        assert_eq!(layout[0], Rect::new(0, 0, 1920, 540));
        assert_eq!(layout[1], Rect::new(0, 540, 960, 540));
        assert_eq!(layout[2], Rect::new(960, 540, 960, 540));
    }
}
