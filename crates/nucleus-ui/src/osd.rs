//! OSD overlay and key recording modal helpers.

use gpui::{
    div, px, rgb, FontWeight, InteractiveElement, IntoElement, MouseButton, ParentElement,
    StatefulInteractiveElement, Styled,
};

use crate::theme;
use crate::{Icon, IconName};

/// Parses an RGB comma-separated string (e.g. "255, 255, 255") into a GPUI `Hsla` color.
pub fn parse_osd_color(color_str: &str) -> gpui::Hsla {
    let parts: Vec<&str> = color_str.split(',').map(|s| s.trim()).collect();
    if parts.len() == 3 {
        let r = parts[0].parse::<u8>().unwrap_or(255);
        let g = parts[1].parse::<u8>().unwrap_or(255);
        let b = parts[2].parse::<u8>().unwrap_or(255);
        return rgb((r as u32) << 16 | (g as u32) << 8 | (b as u32)).into();
    }
    rgb(0xffffff).into()
}

/// Stateless On-Screen Display (OSD) overlay badge.
///
/// Parity with legacy Nucleus C# `Globals.MainOSD.Show(timing, text)`.
/// Renders an absolute inset-0 scrim with a centered badge (`#osd-badge`) displaying
/// the text styled with `color_val`.
pub fn render_osd_overlay(text: &str, color_val: gpui::Hsla) -> gpui::AnyElement {
    div()
        .id("osd-overlay")
        .absolute()
        .inset_0()
        .flex()
        .items_center()
        .justify_center()
        .child(
            div()
                .id("osd-badge")
                .px_8()
                .py_4()
                .bg(theme::overlay_scrim())
                .rounded_lg()
                .border_2()
                .border_color(color_val)
                .shadow_lg()
                .child(
                    div()
                        .text_xl()
                        .font_weight(FontWeight::BOLD)
                        .text_color(color_val)
                        .child(format!("  {}  ", text)),
                ),
        )
        .into_any_element()
}

/// Renders the interactive physical keypress recording modal overlay.
///
/// Displays a modal overlay (`#hotkey-recording-modal-overlay`) with a keyboard icon (⌨️),
/// target shortcut description, instructions ("Press any key or shortcut combination..."),
/// and a Cancel button calling `on_cancel`.
pub fn render_hotkey_recording_modal<F, K>(
    target_name: &str,
    focus_handle: &gpui::FocusHandle,
    on_cancel: F,
    on_key_down: K,
) -> gpui::AnyElement
where
    F: Fn(&gpui::ClickEvent, &mut gpui::Window, &mut gpui::App) + 'static,
    K: Fn(&gpui::KeyDownEvent, &mut gpui::Window, &mut gpui::App) + 'static,
{
    let focus_handle_click1 = focus_handle.clone();
    let focus_handle_click2 = focus_handle.clone();

    div()
        .id("hotkey-recording-modal-overlay")
        .absolute()
        .inset_0()
        .bg(theme::overlay_scrim())
        .flex()
        .items_center()
        .justify_center()
        .p_8()
        .track_focus(focus_handle)
        .on_mouse_down(MouseButton::Left, move |_, window, cx| {
            focus_handle_click1.focus(window, cx);
        })
        .on_key_down(on_key_down)
        .child(
            div()
                .id("hotkey-recording-dialog")
                .on_mouse_down(MouseButton::Left, move |_, window, cx| {
                    focus_handle_click2.focus(window, cx);
                })
                .flex()
                .flex_col()
                .w_full()
                .max_w(px(520.0))
                .p_6()
                .gap_4()
                .bg(theme::bg_panel())
                .rounded_xl()
                .border_2()
                .border_color(theme::border_focus())
                .shadow_lg()
                .child(
                    div()
                        .flex()
                        .items_center()
                        .gap_3()
                        .child(Icon::new(IconName::Keyboard).size_6().text_color(theme::accent()))
                        .child(
                            div()
                                .flex()
                                .flex_col()
                                .gap_0p5()
                                .child(
                                    div()
                                        .text_base()
                                        .font_weight(FontWeight::BOLD)
                                        .text_color(theme::text_primary())
                                        .child("Recording Physical Keypress"),
                                )
                                .child(
                                    div()
                                        .text_xs()
                                        .text_color(theme::accent())
                                        .font_weight(FontWeight::MEDIUM)
                                        .child(format!("Target: {}", target_name)),
                                ),
                        ),
                )
                .child(
                    div()
                        .p_4()
                        .rounded_lg()
                        .bg(theme::bg_subtle())
                        .border_1()
                        .border_color(theme::border_subtle())
                        .flex()
                        .flex_col()
                        .items_center()
                        .gap_2()
                        .child(
                            div()
                                .text_sm()
                                .font_weight(FontWeight::SEMIBOLD)
                                .text_color(theme::text_primary())
                                .child("Press any key or shortcut combination on your physical keyboard now..."),
                        )
                        .child(
                            div()
                                .text_xs()
                                .text_color(theme::text_muted())
                                .child("Examples: End, F10, Home, Space, Ctrl+Q, Alt+W, Shift+F1"),
                        ),
                )
                .child(
                    div()
                        .flex()
                        .items_center()
                        .justify_between()
                        .child(
                            div()
                                .text_xs()
                                .text_color(theme::text_muted())
                                .child("Press Escape on your keyboard to cancel"),
                        )
                        .child(
                            div()
                                .id("btn-modal-cancel-recording")
                                .cursor_pointer()
                                .px_4()
                                .py_1p5()
                                .rounded_md()
                                .border_1()
                                .border_color(theme::border_color())
                                .bg(theme::bg_subtle())
                                .text_xs()
                                .font_weight(FontWeight::MEDIUM)
                                .text_color(theme::text_primary())
                                .hover(|s| s.bg(theme::bg_hover()))
                                .on_click(on_cancel)
                                .child("Cancel (Esc)"),
                        ),
                ),
        )
        .into_any_element()
}

/// Renders floating in-game input lock status pill indicator.
pub fn render_input_lock_status_pill<F>(
    is_locked: bool,
    hotkey_name: &str,
    on_toggle: F,
) -> gpui::AnyElement
where
    F: Fn(&gpui::ClickEvent, &mut gpui::Window, &mut gpui::App) + 'static,
{
    let (bg, border, text_color, label, icon) = if is_locked {
        (
            rgb(0x10b981).opacity(0.2),
            rgb(0x10b981).opacity(0.5),
            rgb(0x34d399),
            format!("Input Locked (Press [{}] to release)", hotkey_name),
            IconName::Lock,
        )
    } else {
        (
            rgb(0xf59e0b).opacity(0.2),
            rgb(0xf59e0b).opacity(0.5),
            rgb(0xfbbf24),
            format!("Input Free (Press [{}] to lock to game)", hotkey_name),
            IconName::Lock,
        )
    };

    div()
        .id("osd-input-lock-pill")
        .cursor_pointer()
        .flex()
        .items_center()
        .gap_2()
        .px_3()
        .py_1p5()
        .rounded_full()
        .bg(bg)
        .border_1()
        .border_color(border)
        .shadow_md()
        .on_click(on_toggle)
        .child(Icon::new(icon).size_3p5().text_color(text_color))
        .child(
            div()
                .text_xs()
                .font_weight(FontWeight::SEMIBOLD)
                .text_color(text_color)
                .child(label),
        )
        .into_any_element()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_render_osd_overlay_construction() {
        let color = rgb(0x00ff00).into();
        let el = render_osd_overlay("Test OSD Banner", color);
        drop(el);
    }

    #[test]
    fn test_parse_osd_color() {
        let color = parse_osd_color("255, 0, 128");
        assert_eq!(color, rgb(0xff0080).into());

        let fallback = parse_osd_color("invalid");
        assert_eq!(fallback, rgb(0xffffff).into());
    }
}
