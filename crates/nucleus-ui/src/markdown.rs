//! Markdown renderer for GPUI Kit using TextView.
//!
//! Provides native text selection, rich styling, headings, bullet lists, code blocks,
//! and fast rendering without DOM-exploding per-word divs.

use gpui::prelude::*;
use gpui::*;
use gpui_component::text::TextView;

/// Opens an external URL in the default browser safely.
pub fn open_url(url: &str) {
    let clean_url = url.trim().to_string();
    if clean_url.starts_with("http://") || clean_url.starts_with("https://") {
        #[cfg(windows)]
        {
            let _ = std::process::Command::new("rundll32")
                .args(["url.dll,FileProtocolHandler", &clean_url])
                .spawn();
        }
        #[cfg(not(windows))]
        {
            let _ = std::process::Command::new("xdg-open")
                .arg(&clean_url)
                .spawn();
        }
    }
}

/// Renders a markdown string using gpui-component's TextView with native text selection.
pub fn render_markdown(text: &str) -> AnyElement {
    render_markdown_with_id("markdown-view", text)
}

/// Renders a markdown string with a specific ElementId using TextView.
pub fn render_markdown_with_id(id: impl Into<ElementId>, text: &str) -> AnyElement {
    let clean_text = text.replace("\r\n", "\n").replace('\r', "\n");
    div()
        .w_full()
        .child(
            TextView::markdown(id.into(), clean_text)
                .selectable(true)
                .on_link_click(|url, _, _, _| {
                    open_url(url.as_ref());
                }),
        )
        .into_any_element()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_markdown_elements_detection() {
        let sample = "+ Fully CLOSE Steam before using this.\n+ [Split-screen setup guide](https://www.splitscreen.me/docs/usage).\n+ Host a **multiplayer game** with *friends*.\n```\nlet x = 10;\n```";
        let _elem = render_markdown(sample);
    }

    #[test]
    fn test_markdown_with_newlines() {
        let sample = "Line 1\n\nLine 2 with special chars: 🎮 & <>\n\nParagraph 3";
        let _elem = render_markdown_with_id("test-md", sample);
    }
}
