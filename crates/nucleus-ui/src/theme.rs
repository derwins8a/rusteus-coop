//! UI themes, styling definitions, colors, and design tokens.
//!
//! Provides color palettes (dark and light themes), font metrics, border radii,
//! padding constants, and visual styles matching the GPUI design system.

use gpui::{Hsla, rgb};

/// Spectrum 2 Dark main canvas background (#1d1d1f).
pub const COLOR_BG_MAIN: u32 = 0x1d1d1f;
/// Spectrum 2 Layer 1 card/panel surface background (#262628).
pub const COLOR_BG_PANEL: u32 = 0x262628;
/// Spectrum 2 hover state background (#333336).
pub const COLOR_BG_HOVER: u32 = 0x333336;
/// Spectrum 2 active/selected state background (#3f3f44).
pub const COLOR_BG_ACTIVE: u32 = 0x3f3f44;
/// Spectrum 2 standard border stroke (#3a3a3e).
pub const COLOR_BORDER: u32 = 0x3a3a3e;
/// Spectrum 2 high-contrast primary text (#fdfdfd).
pub const COLOR_TEXT_PRIMARY: u32 = 0xfdfdfd;
/// Spectrum 2 secondary/muted text (#b0b0b8).
pub const COLOR_TEXT_MUTED: u32 = 0xb0b0b8;
/// Spectrum 2 blue primary accent color (#2680eb).
pub const COLOR_ACCENT: u32 = 0x2680eb;
/// Spectrum 2 blue accent hover state (#378ef0).
pub const COLOR_ACCENT_HOVER: u32 = 0x378ef0;
/// Spectrum 2 celery/green success indicator (#27ab83).
pub const COLOR_SUCCESS: u32 = 0x27ab83;
/// Spectrum 2 amber/orange warning indicator (#f29423).
pub const COLOR_WARNING: u32 = 0xf29423;
/// Spectrum 2 red danger indicator (#e34850).
pub const COLOR_DANGER: u32 = 0xe34850;

use std::sync::atomic::{AtomicU8, Ordering};

static CURRENT_THEME: AtomicU8 = AtomicU8::new(0); // 0 = Spectrum 2 Dark, 1 = Slate Dark, 2 = Spectrum 2 Light
static CURRENT_ACCENT: AtomicU8 = AtomicU8::new(0); // 0 = Blue, 1 = Purple, 2 = Emerald, 3 = Rose, 4 = Amber, 5 = Cyan, 6 = Indigo

/// Sets the active global UI theme (0: Spectrum 2 Dark, 1: Slate, 2: Light).
pub fn set_active_theme(theme_name: &str) {
    let lower = theme_name.to_lowercase();
    let mode = if lower.contains("light") {
        2
    } else if lower.contains("slate") || lower.contains("zinc") {
        1
    } else {
        0
    };
    CURRENT_THEME.store(mode, Ordering::Relaxed);
}

/// Sets the active global accent color palette.
pub fn set_active_accent(accent_name: &str) {
    let lower = accent_name.to_lowercase();
    let code = match lower.as_str() {
        "purple" | "violet" => 1,
        "emerald" | "green" => 2,
        "rose" | "red" | "crimson" => 3,
        "amber" | "orange" | "yellow" => 4,
        "cyan" | "teal" => 5,
        "indigo" => 6,
        _ => 0, // default blue
    };
    CURRENT_ACCENT.store(code, Ordering::Relaxed);
}

/// Returns the currently active accent color name.
pub fn active_accent_name() -> &'static str {
    match CURRENT_ACCENT.load(Ordering::Relaxed) {
        1 => "Purple",
        2 => "Emerald",
        3 => "Rose",
        4 => "Amber",
        5 => "Cyan",
        6 => "Indigo",
        _ => "Blue",
    }
}

/// Returns GPUI HSLA color for deepest dark / scrim overlays.
pub fn bg_dark() -> Hsla {
    rgb(0x000000).into()
}

/// Returns GPUI HSLA color for main window background.
pub fn bg_main() -> Hsla {
    match CURRENT_THEME.load(Ordering::Relaxed) {
        2 => rgb(0xf5f5f7).into(),      // Spectrum 2 Light canvas
        1 => rgb(0x0f172a).into(),      // Slate dark
        _ => rgb(COLOR_BG_MAIN).into(), // Spectrum 2 dark canvas
    }
}

/// Returns GPUI HSLA color for panel/card background (Layer 1).
pub fn bg_panel() -> Hsla {
    match CURRENT_THEME.load(Ordering::Relaxed) {
        2 => rgb(0xffffff).into(),       // Light surface
        1 => rgb(0x1e293b).into(),       // Slate dark
        _ => rgb(COLOR_BG_PANEL).into(), // Spectrum 2 dark surface
    }
}

/// Returns GPUI HSLA color for panel/card hover state background.
pub fn bg_panel_hover() -> Hsla {
    match CURRENT_THEME.load(Ordering::Relaxed) {
        2 => rgb(0xf0f0f4).into(),
        1 => rgb(0x243247).into(),
        _ => rgb(0x2e2e32).into(),
    }
}

/// Returns GPUI HSLA color for elevated panels, dropdowns, and modals (Layer 2).
pub fn bg_elevated() -> Hsla {
    match CURRENT_THEME.load(Ordering::Relaxed) {
        2 => rgb(0xffffff).into(),
        1 => rgb(0x283548).into(),
        _ => rgb(0x2f2f33).into(),
    }
}

/// Returns GPUI HSLA color for subtle wells, text inputs, and table backgrounds.
pub fn bg_subtle() -> Hsla {
    match CURRENT_THEME.load(Ordering::Relaxed) {
        2 => rgb(0xeeeff2).into(),
        1 => rgb(0x141d2e).into(),
        _ => rgb(0x18181a).into(),
    }
}

/// Returns GPUI HSLA color for hover state background.
pub fn bg_hover() -> Hsla {
    match CURRENT_THEME.load(Ordering::Relaxed) {
        2 => rgb(0xeaebef).into(),
        1 => rgb(0x334155).into(),
        _ => rgb(COLOR_BG_HOVER).into(),
    }
}

/// Returns GPUI HSLA color for active/selected state background.
pub fn bg_active() -> Hsla {
    match CURRENT_THEME.load(Ordering::Relaxed) {
        2 => rgb(0xdcdee3).into(),
        1 => rgb(0x475569).into(),
        _ => rgb(COLOR_BG_ACTIVE).into(),
    }
}

/// Returns GPUI HSLA color for standard borders.
pub fn border_color() -> Hsla {
    match CURRENT_THEME.load(Ordering::Relaxed) {
        2 => rgb(0xd5d6dc).into(),
        1 => rgb(0x334155).into(),
        _ => rgb(COLOR_BORDER).into(),
    }
}

/// Returns GPUI HSLA color for subtle separators and inner dividers.
pub fn border_subtle() -> Hsla {
    match CURRENT_THEME.load(Ordering::Relaxed) {
        2 => rgb(0xe5e6ec).into(),
        1 => rgb(0x243042).into(),
        _ => rgb(0x2e2e32).into(),
    }
}

/// Returns GPUI HSLA color for high-contrast focus rings and active outlines.
pub fn border_focus() -> Hsla {
    accent()
}

/// Returns GPUI HSLA color for focus glow effects (accent with 0.35 opacity).
pub fn focus_glow() -> Hsla {
    accent().opacity(0.35)
}

/// Returns GPUI HSLA color for subtle focus glow with custom opacity.
pub fn focus_glow_opacity(opacity: f32) -> Hsla {
    accent().opacity(opacity)
}

/// Returns GPUI HSLA color for primary text.
pub fn text_primary() -> Hsla {
    match CURRENT_THEME.load(Ordering::Relaxed) {
        2 => rgb(0x1d1d21).into(), // High contrast text on light (> 14:1)
        1 => rgb(0xf8fafc).into(),
        _ => rgb(COLOR_TEXT_PRIMARY).into(), // High contrast text on dark (> 15:1)
    }
}

/// Returns GPUI HSLA color for muted/secondary text.
pub fn text_muted() -> Hsla {
    match CURRENT_THEME.load(Ordering::Relaxed) {
        2 => rgb(0x5f606a).into(), // Accessible secondary text (> 4.5:1)
        1 => rgb(0x94a3b8).into(),
        _ => rgb(COLOR_TEXT_MUTED).into(),
    }
}

/// Returns GPUI HSLA color for subtle/tertiary text, placeholders, and timestamps.
pub fn text_subtle() -> Hsla {
    match CURRENT_THEME.load(Ordering::Relaxed) {
        2 => rgb(0x878894).into(),
        1 => rgb(0x64748b).into(),
        _ => rgb(0x787882).into(),
    }
}

/// Returns GPUI HSLA color for text on primary solid buttons/badges.
pub fn text_on_accent() -> Hsla {
    rgb(0xffffff).into()
}

/// Returns GPUI HSLA color for accent highlights and active controls.
pub fn accent() -> Hsla {
    match CURRENT_ACCENT.load(Ordering::Relaxed) {
        1 => rgb(0x8b5cf6).into(), // Purple
        2 => rgb(0x10b981).into(), // Emerald
        3 => rgb(0xf43f5e).into(), // Rose
        4 => rgb(0xf59e0b).into(), // Amber
        5 => rgb(0x06b6d4).into(), // Cyan
        6 => rgb(0x6366f1).into(), // Indigo
        _ => match CURRENT_THEME.load(Ordering::Relaxed) {
            2 => rgb(0x0265dc).into(),     // Spectrum Light Blue
            1 => rgb(0x38bdf8).into(),     // Sky blue for slate
            _ => rgb(COLOR_ACCENT).into(), // Spectrum 2 Blue
        },
    }
}

/// Returns GPUI HSLA color for hovered accent controls.
pub fn accent_hover() -> Hsla {
    match CURRENT_ACCENT.load(Ordering::Relaxed) {
        1 => rgb(0x9d74f8).into(),
        2 => rgb(0x34d399).into(),
        3 => rgb(0xfb7185).into(),
        4 => rgb(0xfbbf24).into(),
        5 => rgb(0x22d3ee).into(),
        6 => rgb(0x818cf8).into(),
        _ => match CURRENT_THEME.load(Ordering::Relaxed) {
            2 => rgb(0x0054b6).into(),
            1 => rgb(0x0ea5e9).into(),
            _ => rgb(COLOR_ACCENT_HOVER).into(),
        },
    }
}

/// Returns GPUI HSLA color for subtle tinted accent backgrounds.
pub fn accent_subtle() -> Hsla {
    accent().opacity(0.16)
}

/// Returns GPUI HSLA color for success badges and online status.
pub fn success() -> Hsla {
    match CURRENT_THEME.load(Ordering::Relaxed) {
        2 => rgb(0x0f875f).into(),
        _ => rgb(COLOR_SUCCESS).into(),
    }
}

/// Returns GPUI HSLA color for subtle tinted success backgrounds.
pub fn success_subtle() -> Hsla {
    success().opacity(0.16)
}

/// Returns GPUI HSLA color for warning banners and cautions.
pub fn warning() -> Hsla {
    match CURRENT_THEME.load(Ordering::Relaxed) {
        2 => rgb(0xc46900).into(),
        _ => rgb(COLOR_WARNING).into(),
    }
}

/// Returns GPUI HSLA color for subtle tinted warning backgrounds.
pub fn warning_subtle() -> Hsla {
    warning().opacity(0.16)
}

/// Returns GPUI HSLA color for error badges and destructive indicators.
pub fn danger() -> Hsla {
    match CURRENT_THEME.load(Ordering::Relaxed) {
        2 => rgb(0xd7373f).into(),
        _ => rgb(COLOR_DANGER).into(),
    }
}

/// Returns GPUI HSLA color for subtle tinted danger backgrounds.
pub fn danger_subtle() -> Hsla {
    danger().opacity(0.16)
}

/// Returns GPUI HSLA color for purple/magenta highlights.
pub fn purple() -> Hsla {
    match CURRENT_THEME.load(Ordering::Relaxed) {
        2 => rgb(0x8a3ffc).into(),
        _ => rgb(0xa855f7).into(),
    }
}

/// Returns GPUI HSLA color for subtle tinted purple backgrounds.
pub fn purple_subtle() -> Hsla {
    purple().opacity(0.16)
}

/// Returns the primary border/accent color for player partition index (0 = Player 1).
pub fn player_color(index: usize) -> Hsla {
    match index % 4 {
        0 => accent(),
        1 => success(),
        2 => warning(),
        _ => purple(),
    }
}

/// Returns the tinted translucent background color for player partition index.
pub fn player_color_subtle(index: usize) -> Hsla {
    match index % 4 {
        0 => accent_subtle(),
        1 => success_subtle(),
        2 => warning_subtle(),
        _ => purple_subtle(),
    }
}

/// Returns accessible label text color for player partition index.
pub fn player_color_text(index: usize) -> Hsla {
    match CURRENT_THEME.load(Ordering::Relaxed) {
        2 => match index % 4 {
            0 => rgb(0x0265dc).into(),
            1 => rgb(0x0f875f).into(),
            2 => rgb(0xc46900).into(),
            _ => rgb(0x8a3ffc).into(),
        },
        _ => match index % 4 {
            0 => rgb(0x93c5fd).into(),
            1 => rgb(0x86efac).into(),
            2 => rgb(0xfde68a).into(),
            _ => rgb(0xe9d5ff).into(),
        },
    }
}

/// Returns GPUI HSLA color for dialog/modal backdrop scrim.
pub fn overlay_scrim() -> Hsla {
    match CURRENT_THEME.load(Ordering::Relaxed) {
        2 => rgb(0x000000).opacity(0.38).into(),
        _ => rgb(0x000000).opacity(0.68).into(),
    }
}

/// Color specification in RGBA format.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Color {
    /// Red component (0-255).
    pub r: u8,
    /// Green component (0-255).
    pub g: u8,
    /// Blue component (0-255).
    pub b: u8,
    /// Alpha component (0-255).
    pub a: u8,
}

impl Color {
    /// Creates a new `Color` from RGBA components.
    pub const fn rgba(r: u8, g: u8, b: u8, a: u8) -> Self {
        Self { r, g, b, a }
    }

    /// Creates a fully opaque `Color` from RGB components.
    pub const fn rgb(r: u8, g: u8, b: u8) -> Self {
        Self { r, g, b, a: 255 }
    }
}

/// Active color scheme and styling rules for UI components.
#[derive(Debug, Clone)]
pub struct Theme {
    /// Background color of primary application windows.
    pub background: Color,
    /// Surface card and container color.
    pub surface: Color,
    /// Primary accent color (buttons, active tabs, highlights).
    pub primary: Color,
    /// Text color for primary content.
    pub text_primary: Color,
    /// Text color for secondary or muted content.
    pub text_secondary: Color,
    /// Border stroke color.
    pub border: Color,
}

impl Default for Theme {
    fn default() -> Self {
        Self::dark()
    }
}

impl Theme {
    /// Creates the standard dark theme palette.
    pub fn dark() -> Self {
        Self {
            background: Color::rgb(24, 24, 27),
            surface: Color::rgb(39, 39, 42),
            primary: Color::rgb(59, 130, 246),
            text_primary: Color::rgb(244, 244, 245),
            text_secondary: Color::rgb(161, 161, 170),
            border: Color::rgb(63, 63, 70),
        }
    }

    /// Creates the standard light theme palette.
    pub fn light() -> Self {
        Self {
            background: Color::rgb(255, 255, 255),
            surface: Color::rgb(244, 244, 245),
            primary: Color::rgb(37, 99, 235),
            text_primary: Color::rgb(24, 24, 27),
            text_secondary: Color::rgb(113, 113, 122),
            border: Color::rgb(228, 228, 231),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_theme_colors() {
        let dark = Theme::dark();
        assert_eq!(dark.background.a, 255);
        assert_eq!(dark.primary, Color::rgb(59, 130, 246));

        let light = Theme::light();
        assert_eq!(light.background, Color::rgb(255, 255, 255));
    }

    #[test]
    fn test_set_active_theme_modes() {
        set_active_theme("Spectrum 2 Light");
        assert_eq!(CURRENT_THEME.load(Ordering::Relaxed), 2);
        let light_bg = bg_main();
        assert_eq!(light_bg, rgb(0xf5f5f7).into());

        set_active_theme("Spectrum 2 Dark");
        assert_eq!(CURRENT_THEME.load(Ordering::Relaxed), 0);
        let dark_bg = bg_main();
        assert_eq!(dark_bg, rgb(COLOR_BG_MAIN).into());

        set_active_theme("Slate Dark");
        assert_eq!(CURRENT_THEME.load(Ordering::Relaxed), 1);
        let slate_bg = bg_main();
        assert_eq!(slate_bg, rgb(0x0f172a).into());
    }

    #[test]
    fn test_set_active_accent() {
        set_active_accent("Emerald");
        assert_eq!(active_accent_name(), "Emerald");
        assert_eq!(accent(), rgb(0x10b981).into());

        set_active_accent("Purple");
        assert_eq!(active_accent_name(), "Purple");
        assert_eq!(accent(), rgb(0x8b5cf6).into());

        set_active_accent("Blue");
        assert_eq!(active_accent_name(), "Blue");
    }
}
