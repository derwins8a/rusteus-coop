use gpui::{AnyElement, FontWeight, IntoElement, ParentElement, Pixels, Styled, div, px};

use crate::components::badge::CouchTag;
use crate::theme;

/// Monitor partition visualizer that renders player screen boundaries
/// for splitscreen layouts (1x1, 2x1, 1x2, 2x2, Top 1 / Bottom 2, Top 2 / Bottom 1).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum SplitLayout {
    #[default]
    Single, // 1 Player full screen
    SideBySide,      // 2 Players horizontal split (2 columns)
    Stacked,         // 2 Players vertical split (2 rows)
    Grid4,           // 4 Players 2x2 grid
    TopOneBottomTwo, // 3 Players: Top 1 full, Bottom 2 side-by-side
    TopTwoBottomOne, // 3 Players: Top 2 side-by-side, Bottom 1 full
    Grid6,           // 6 Players: 3 columns x 2 rows
    Grid8,           // 8 Players: 4 columns x 2 rows
    Grid16,          // 16 Players: 4 columns x 4 rows
    MultiMonitor {
        monitor_count: usize,
    }, // Multi-monitor span across displays
}

pub struct PartitionPreview {
    layout: SplitLayout,
    player_count: usize,
    width: Pixels,
    height: Pixels,
}

impl PartitionPreview {
    pub fn new(layout: SplitLayout) -> Self {
        Self {
            layout,
            player_count: match layout {
                SplitLayout::Single => 1,
                SplitLayout::SideBySide | SplitLayout::Stacked => 2,
                SplitLayout::TopOneBottomTwo | SplitLayout::TopTwoBottomOne => 3,
                SplitLayout::Grid4 => 4,
                SplitLayout::Grid6 => 6,
                SplitLayout::Grid8 => 8,
                SplitLayout::Grid16 => 16,
                SplitLayout::MultiMonitor { monitor_count } => monitor_count.max(2),
            },
            width: px(320.0),
            height: px(180.0), // 16:9 monitor aspect ratio
        }
    }

    pub fn size(mut self, width: Pixels, height: Pixels) -> Self {
        self.width = width;
        self.height = height;
        self
    }

    pub fn player_count(mut self, count: usize) -> Self {
        self.player_count = count;
        self
    }

    fn render_slot_box(&self, player_idx: usize, label: &'static str) -> AnyElement {
        div()
            .flex_1()
            .flex()
            .flex_col()
            .items_center()
            .justify_center()
            .m_1()
            .rounded_lg()
            .bg(theme::bg_panel())
            .border_1()
            .border_color(theme::accent().opacity(0.4))
            .child(CouchTag::player(player_idx, format!("P{}", player_idx + 1)).into_any_element())
            .child(
                div()
                    .text_xs()
                    .font_weight(FontWeight::MEDIUM)
                    .text_color(theme::text_muted())
                    .child(label),
            )
            .into_any_element()
    }

    pub fn into_any_element(self) -> AnyElement {
        let content: AnyElement = match self.layout {
            SplitLayout::Single => div()
                .flex()
                .size_full()
                .child(self.render_slot_box(0, "100% Fullscreen"))
                .into_any_element(),

            SplitLayout::SideBySide => div()
                .flex()
                .flex_row()
                .size_full()
                .child(self.render_slot_box(0, "Left"))
                .child(self.render_slot_box(1, "Right"))
                .into_any_element(),

            SplitLayout::Stacked => div()
                .flex()
                .flex_col()
                .size_full()
                .child(self.render_slot_box(0, "Top"))
                .child(self.render_slot_box(1, "Bottom"))
                .into_any_element(),

            SplitLayout::Grid4 => div()
                .flex()
                .flex_col()
                .size_full()
                .child(
                    div()
                        .flex_1()
                        .flex()
                        .flex_row()
                        .child(self.render_slot_box(0, "Top Left"))
                        .child(self.render_slot_box(1, "Top Right")),
                )
                .child(
                    div()
                        .flex_1()
                        .flex()
                        .flex_row()
                        .child(self.render_slot_box(2, "Bottom Left"))
                        .child(self.render_slot_box(3, "Bottom Right")),
                )
                .into_any_element(),

            SplitLayout::TopOneBottomTwo => div()
                .flex()
                .flex_col()
                .size_full()
                .child(
                    div()
                        .flex_1()
                        .flex()
                        .child(self.render_slot_box(0, "Top Wide")),
                )
                .child(
                    div()
                        .flex_1()
                        .flex()
                        .flex_row()
                        .child(self.render_slot_box(1, "Bottom Left"))
                        .child(self.render_slot_box(2, "Bottom Right")),
                )
                .into_any_element(),

            SplitLayout::TopTwoBottomOne => div()
                .flex()
                .flex_col()
                .size_full()
                .child(
                    div()
                        .flex_1()
                        .flex()
                        .flex_row()
                        .child(self.render_slot_box(0, "Top Left"))
                        .child(self.render_slot_box(1, "Top Right")),
                )
                .child(
                    div()
                        .flex_1()
                        .flex()
                        .child(self.render_slot_box(2, "Bottom Wide")),
                )
                .into_any_element(),

            SplitLayout::Grid6 => div()
                .flex()
                .flex_col()
                .size_full()
                .child(
                    div()
                        .flex_1()
                        .flex()
                        .flex_row()
                        .child(self.render_slot_box(0, "Top Left"))
                        .child(self.render_slot_box(1, "Top Center"))
                        .child(self.render_slot_box(2, "Top Right")),
                )
                .child(
                    div()
                        .flex_1()
                        .flex()
                        .flex_row()
                        .child(self.render_slot_box(3, "Bottom Left"))
                        .child(self.render_slot_box(4, "Bottom Center"))
                        .child(self.render_slot_box(5, "Bottom Right")),
                )
                .into_any_element(),

            SplitLayout::Grid8 => div()
                .flex()
                .flex_col()
                .size_full()
                .child(
                    div()
                        .flex_1()
                        .flex()
                        .flex_row()
                        .child(self.render_slot_box(0, "T-L"))
                        .child(self.render_slot_box(1, "T-ML"))
                        .child(self.render_slot_box(2, "T-MR"))
                        .child(self.render_slot_box(3, "T-R")),
                )
                .child(
                    div()
                        .flex_1()
                        .flex()
                        .flex_row()
                        .child(self.render_slot_box(4, "B-L"))
                        .child(self.render_slot_box(5, "B-ML"))
                        .child(self.render_slot_box(6, "B-MR"))
                        .child(self.render_slot_box(7, "B-R")),
                )
                .into_any_element(),

            SplitLayout::Grid16 => {
                let mut col = div().flex().flex_col().size_full();
                for r in 0..4 {
                    let mut row = div().flex_1().flex().flex_row();
                    for c in 0..4 {
                        let idx = r * 4 + c;
                        row = row.child(self.render_slot_box(idx, ""));
                    }
                    col = col.child(row);
                }
                col.into_any_element()
            }

            SplitLayout::MultiMonitor { monitor_count } => {
                let count = monitor_count.max(2);
                let mut row = div().flex().flex_row().size_full().gap_1();
                for m in 0..count {
                    let p_idx = m;
                    row = row.child(
                        div()
                            .flex_1()
                            .flex()
                            .flex_col()
                            .items_center()
                            .justify_center()
                            .m_0p5()
                            .p_1()
                            .rounded_lg()
                            .bg(theme::bg_panel())
                            .border_1()
                            .border_color(theme::accent().opacity(0.4))
                            .child(
                                CouchTag::player(p_idx, format!("Disp {}", m + 1))
                                    .into_any_element(),
                            )
                            .child(
                                div()
                                    .text_xs()
                                    .font_weight(FontWeight::MEDIUM)
                                    .text_color(theme::text_muted())
                                    .child(format!("Monitor {}", m + 1)),
                            )
                            .into_any_element(),
                    );
                }
                row.into_any_element()
            }
        };

        // Outer monitor chassis with stand indicator
        div()
            .flex()
            .flex_col()
            .items_center()
            .child(
                div()
                    .w(self.width)
                    .h(self.height)
                    .p_1()
                    .rounded_xl()
                    .bg(theme::bg_subtle())
                    .border_2()
                    .border_color(theme::border_color())
                    .shadow_lg()
                    .child(content),
            )
            .child(
                div()
                    .w(px(60.0))
                    .h(px(8.0))
                    .rounded_b_md()
                    .bg(theme::border_color()),
            )
            .into_any_element()
    }
}

impl IntoElement for PartitionPreview {
    type Element = AnyElement;

    fn into_element(self) -> Self::Element {
        self.into_any_element()
    }
}
