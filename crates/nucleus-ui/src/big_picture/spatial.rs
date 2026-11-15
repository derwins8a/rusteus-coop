use std::cell::RefCell;
use std::collections::HashMap;
use std::rc::Rc;

use gpui::{AnyElement, Hsla, IntoElement, Styled, canvas, rgb};

/// Cardinal directions for 2D geometric spatial navigation.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum SpatialDirection {
    Up,
    Down,
    Left,
    Right,
}

/// Interactive targets across all Big Picture screens.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum SpatialTarget {
    // Stepper Tabs
    StepperTab(usize),

    // Carousel
    HeaderTab(usize),
    CarouselSearch,
    CarouselFilter,
    CarouselSort,
    CarouselPopoverItem(usize),
    CarouselCard(usize),
    CarouselMissingExeCta,

    // Couch Setup - Step 1: Players
    PlayerSlot(usize),
    SlotSwap(usize),
    SlotRemove(usize),
    UnassignedTrayHeader,
    UnassignedItem(usize),
    UnassignedAssign(usize),
    IgnoredItem(usize),
    ScreenPreviewCard,
    KmKeyboardMouse,
    KmRawKeyboard,
    KmRawMouse,

    // Couch Setup - Step 2: Screen layout
    LayoutPrev,
    LayoutNext,
    LayoutCard(usize),
    ChangeDisplayButton,

    // Couch Setup - Step 3: Game options
    OptionRow(usize),
    OptionCombobox(usize),
    LanguageSelector,

    // Couch Setup - Step 4: Launch
    ProfileSaveNew,
    ProfileChip(usize),
    ProfileDelete(usize),
    LaunchButton,

    // Shared Step buttons
    NextStepButton,
    PrevStepButton,

    // Couch Hub
    HubSearch,
    HubCategory(usize),
    HubCard(usize),
    HubMissingExeBrowse,
    HubOptionRow(usize),

    // In-Game Couch OSD (Image 2)
    OsdAction(usize),
    OsdResume,
    OsdDevices,
    OsdGuide,
    OsdSaveProfile,
    OsdEndSession,
    OsdPlayerSlot(usize),
    OsdDropIn,
    OsdDropOut(usize),
}

/// Axis-aligned rectangle in logical screen pixels for spatial hit testing.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct SpatialRect {
    pub x: f32,
    pub y: f32,
    pub w: f32,
    pub h: f32,
}

impl SpatialRect {
    pub const fn new(x: f32, y: f32, w: f32, h: f32) -> Self {
        Self { x, y, w, h }
    }

    pub fn cx(self) -> f32 {
        self.x + self.w * 0.5
    }

    pub fn cy(self) -> f32 {
        self.y + self.h * 0.5
    }

    pub fn right(self) -> f32 {
        self.x + self.w
    }

    pub fn bottom(self) -> f32 {
        self.y + self.h
    }

    pub fn overlap_x(self, other: Self) -> f32 {
        (self.right().min(other.right()) - self.x.max(other.x)).max(0.0)
    }

    pub fn overlap_y(self, other: Self) -> f32 {
        (self.bottom().min(other.bottom()) - self.y.max(other.y)).max(0.0)
    }

    pub fn intersects_axis_x(self, other: Self) -> bool {
        self.overlap_x(other) > 0.0
    }

    pub fn intersects_axis_y(self, other: Self) -> bool {
        self.overlap_y(other) > 0.0
    }
}

/// Returns the dynamic summoner focus border glow color for a given player index:
/// - Player 1 (index 0): Emerald `#22c55e`
/// - Player 2 (index 1): Blue `#3b82f6`
/// - Player 3 (index 2): Amber `#f59e0b`
/// - Player 4+ (index 3+): Rose `#f43f5e`
pub fn summoner_glow_color(player_idx: usize) -> Hsla {
    match player_idx % 4 {
        0 => rgb(0x22c55e).into(),
        1 => rgb(0x3b82f6).into(),
        2 => rgb(0xf59e0b).into(),
        _ => rgb(0xf43f5e).into(),
    }
}

/// Calculates 2D geometric cone distance between two axis-aligned rectangles.
/// Returns `(is_in_cone, distance)` where distance is weighted:
/// $D = \text{Euclidean} \times (1.0 + 2.0 \times \sin\theta)$
pub fn calculate_cone_distance(
    origin: SpatialRect,
    candidate: SpatialRect,
    dir: SpatialDirection,
) -> (bool, f32) {
    let dx = candidate.cx() - origin.cx();
    let dy = candidate.cy() - origin.cy();
    let euclidean = (dx * dx + dy * dy).sqrt();
    if euclidean < 1e-4 {
        return (false, f32::MAX);
    }

    let (along_axis, perp) = match dir {
        SpatialDirection::Right => (dx, dy.abs()),
        SpatialDirection::Left => (-dx, dy.abs()),
        SpatialDirection::Down => (dy, dx.abs()),
        SpatialDirection::Up => (-dy, dx.abs()),
    };

    if along_axis <= 0.0 {
        return (false, f32::MAX);
    }

    let sin_theta = (perp / euclidean).min(1.0);
    let distance = euclidean * (1.0 + 2.0 * sin_theta);
    (true, distance)
}

/// Returns the weighted cone distance between origin and candidate along `dir`.
pub fn weighted_cone_distance(
    origin: SpatialRect,
    candidate: SpatialRect,
    dir: SpatialDirection,
) -> f32 {
    let (in_cone, dist) = calculate_cone_distance(origin, candidate, dir);
    if in_cone { dist } else { f32::MAX }
}

/// Paired target identifier and its bounding rectangle.
#[derive(Debug, Clone, PartialEq)]
pub struct SpatialNode {
    pub target: SpatialTarget,
    pub rect: SpatialRect,
}

/// Live bounds collector populated passively during GPUI prepaint/paint without
/// leasing entity state or calling `cx.notify()`.
#[derive(Debug, Clone, Default)]
pub struct SpatialBoundsRegistry(pub Rc<RefCell<HashMap<SpatialTarget, SpatialRect>>>);

impl SpatialBoundsRegistry {
    pub fn new() -> Self {
        Self(Rc::new(RefCell::new(HashMap::new())))
    }

    pub fn clear(&self) {
        self.0.borrow_mut().clear();
    }

    pub fn record(&self, target: SpatialTarget, rect: SpatialRect) {
        self.0.borrow_mut().insert(target, rect);
    }

    pub fn resolve(&self, target: SpatialTarget) -> Option<SpatialRect> {
        self.0.borrow().get(&target).copied()
    }

    /// Replaces canonical fallback rectangles with live recorded bounds ONLY for
    /// targets present in `fallback` (ensuring stale invisible targets stay excluded).
    pub fn into_nodes(&self, fallback: Vec<SpatialNode>) -> Vec<SpatialNode> {
        let map = self.0.borrow();
        fallback
            .into_iter()
            .map(|mut node| {
                if let Some(live) = map.get(&node.target) {
                    node.rect = *live;
                }
                node
            })
            .collect()
    }
}

/// Wrapper struct storing prebuilt `SpatialNode`s for fluent graph queries.
#[derive(Debug, Clone, Default)]
pub struct SpatialNavGraph {
    nodes: Vec<SpatialNode>,
}

impl SpatialNavGraph {
    pub fn new(nodes: Vec<SpatialNode>) -> Self {
        Self { nodes }
    }

    pub fn nodes(&self) -> &[SpatialNode] {
        &self.nodes
    }

    pub fn find_best_neighbor(
        &self,
        current: SpatialTarget,
        dir: SpatialDirection,
    ) -> Option<SpatialTarget> {
        resolve_spatial_move(current, dir, &self.nodes)
    }
}

/// Pure 2D geometric neighbor resolution across active `SpatialNode`s.
pub fn resolve_spatial_move(
    current: SpatialTarget,
    dir: SpatialDirection,
    nodes: &[SpatialNode],
) -> Option<SpatialTarget> {
    let origin = nodes.iter().find(|n| n.target == current)?.rect;

    let mut best: Option<(f32, SpatialTarget)> = None;

    for node in nodes {
        if node.target == current {
            continue;
        }
        let c = node.rect;

        let valid = match dir {
            SpatialDirection::Right => {
                if origin.intersects_axis_y(c) {
                    c.cx() > origin.cx() + 0.5
                        && c.x > origin.x + 1.0
                        && c.right() > origin.right().min(origin.cx() + 1.0)
                } else {
                    c.cx() > origin.cx() + 0.5
                        && c.x > origin.cx() - 1.0
                        && c.right() > origin.right() + 1.0
                }
            }
            SpatialDirection::Left => {
                if origin.intersects_axis_y(c) {
                    c.cx() < origin.cx() - 0.5
                        && c.right() < origin.right() - 1.0
                        && c.x < origin.x.max(origin.cx() - 1.0)
                } else {
                    c.cx() < origin.cx() - 0.5
                        && c.right() < origin.cx() + 1.0
                        && c.x < origin.x - 1.0
                }
            }
            SpatialDirection::Down => {
                if origin.intersects_axis_x(c) {
                    c.cy() > origin.cy() + 0.5
                        && c.y > origin.y + 1.0
                        && c.bottom() > origin.bottom().min(origin.cy() + 1.0)
                } else {
                    c.cy() > origin.cy() + 0.5
                        && c.y >= origin.bottom() - 1.0
                        && c.bottom() > origin.bottom() + 1.0
                }
            }
            SpatialDirection::Up => {
                if origin.intersects_axis_x(c) {
                    c.cy() < origin.cy() - 0.5
                        && c.bottom() < origin.bottom() - 1.0
                        && c.y < origin.y.max(origin.cy() - 1.0)
                } else {
                    c.cy() < origin.cy() - 0.5
                        && c.bottom() <= origin.y + 1.0
                        && c.y < origin.y - 1.0
                }
            }
        };

        if !valid {
            continue;
        }

        let (primary_gap, perp_gap, center_perp_delta) = match dir {
            SpatialDirection::Right => {
                let pgap = if c.x >= origin.right() {
                    c.x - origin.right()
                } else {
                    (c.cx() - origin.cx()).max(0.0)
                };
                let o_y = origin.overlap_y(c);
                let perp_dist = if o_y > 2.0 {
                    0.0
                } else if c.y >= origin.bottom() {
                    c.y - origin.bottom()
                } else {
                    origin.y - c.bottom()
                };
                (pgap, perp_dist.max(0.0), (c.cy() - origin.cy()).abs())
            }
            SpatialDirection::Left => {
                let pgap = if c.right() <= origin.x {
                    origin.x - c.right()
                } else {
                    (origin.cx() - c.cx()).max(0.0)
                };
                let o_y = origin.overlap_y(c);
                let perp_dist = if o_y > 2.0 {
                    0.0
                } else if c.y >= origin.bottom() {
                    c.y - origin.bottom()
                } else {
                    origin.y - c.bottom()
                };
                (pgap, perp_dist.max(0.0), (c.cy() - origin.cy()).abs())
            }
            SpatialDirection::Down => {
                let pgap = if c.y >= origin.bottom() {
                    c.y - origin.bottom()
                } else {
                    (c.cy() - origin.cy()).max(0.0)
                };
                let o_x = origin.overlap_x(c);
                let perp_dist = if o_x > 2.0 {
                    0.0
                } else if c.x >= origin.right() {
                    c.x - origin.right()
                } else {
                    origin.x - c.right()
                };
                (pgap, perp_dist.max(0.0), (c.cx() - origin.cx()).abs())
            }
            SpatialDirection::Up => {
                let pgap = if c.bottom() <= origin.y {
                    origin.y - c.bottom()
                } else {
                    (origin.cy() - c.cy()).max(0.0)
                };
                let o_x = origin.overlap_x(c);
                let perp_dist = if o_x > 2.0 {
                    0.0
                } else if c.x >= origin.right() {
                    c.x - origin.right()
                } else {
                    origin.x - c.right()
                };
                (pgap, perp_dist.max(0.0), (c.cx() - origin.cx()).abs())
            }
        };

        let score = primary_gap + perp_gap * 3.0 + center_perp_delta * 0.25;
        let is_better = match best {
            None => true,
            Some((best_score, _)) => score + 1e-4 < best_score,
        };
        if is_better {
            best = Some((score, node.target));
        }
    }

    best.map(|(_, target)| target)
}

/// Passive non-leasing GPUI bounds observer anchored inside a `.relative()` parent element.
pub fn track_spatial_bounds(target: SpatialTarget, registry: &SpatialBoundsRegistry) -> AnyElement {
    let map = Rc::clone(&registry.0);
    canvas(
        move |bounds, _win, _cx| {
            let x: f32 = bounds.origin.x.into();
            let y: f32 = bounds.origin.y.into();
            let w: f32 = bounds.size.width.into();
            let h: f32 = bounds.size.height.into();
            if w > 0.0 && h > 0.0 {
                map.borrow_mut()
                    .insert(target, SpatialRect::new(x, y, w, h));
            }
        },
        |_, _, _, _| {},
    )
    .absolute()
    .inset_0()
    .into_any_element()
}

/// Fallback geometric specification for the Big Picture Carousel screen.
pub fn build_carousel_layout_rects(
    tab_count: usize,
    popover_item_count: usize,
    card_count: usize,
    show_missing_exe_cta: bool,
) -> Vec<SpatialNode> {
    let mut nodes = Vec::new();

    for i in 0..tab_count {
        nodes.push(SpatialNode {
            target: SpatialTarget::HeaderTab(i),
            rect: SpatialRect::new(32.0 + (i as f32) * 160.0, 20.0, 148.0, 36.0),
        });
    }

    nodes.push(SpatialNode {
        target: SpatialTarget::CarouselSearch,
        rect: SpatialRect::new(180.0, 84.0, 220.0, 36.0),
    });
    nodes.push(SpatialNode {
        target: SpatialTarget::CarouselFilter,
        rect: SpatialRect::new(520.0, 84.0, 220.0, 36.0),
    });
    nodes.push(SpatialNode {
        target: SpatialTarget::CarouselSort,
        rect: SpatialRect::new(860.0, 84.0, 220.0, 36.0),
    });

    for i in 0..popover_item_count {
        nodes.push(SpatialNode {
            target: SpatialTarget::CarouselPopoverItem(i),
            rect: SpatialRect::new(520.0, 132.0 + (i as f32) * 40.0, 240.0, 34.0),
        });
    }

    let stride = if card_count <= 3 { 340.0 } else { 244.0 };
    let start_x = if card_count <= 3 { 180.0 } else { 64.0 };
    for i in 0..card_count {
        nodes.push(SpatialNode {
            target: SpatialTarget::CarouselCard(i),
            rect: SpatialRect::new(start_x + (i as f32) * stride, 200.0, 220.0, 340.0),
        });
    }

    if show_missing_exe_cta {
        nodes.push(SpatialNode {
            target: SpatialTarget::CarouselMissingExeCta,
            rect: SpatialRect::new(430.0, 572.0, 400.0, 44.0),
        });
    }

    nodes
}

/// Configuration describing visible controls in Couch Setup for fallback layout generation.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct SetupLayoutSpec {
    pub has_km_row: bool,
    pub has_raw_kb: bool,
    pub has_raw_mouse: bool,
    pub player_count: usize,
    pub unassigned_count: usize,
    pub ignored_count: usize,
    pub profile_count: usize,
    pub can_launch: bool,
}

/// Fallback geometric layout for the 2-column Big Picture Couch Setup screen.
pub fn build_setup_layout_rects(spec: &SetupLayoutSpec) -> Vec<SpatialNode> {
    let mut nodes = Vec::new();

    nodes.push(SpatialNode {
        target: SpatialTarget::LayoutPrev,
        rect: SpatialRect::new(32.0, 24.0, 52.0, 44.0),
    });
    nodes.push(SpatialNode {
        target: SpatialTarget::LayoutNext,
        rect: SpatialRect::new(740.0, 24.0, 52.0, 44.0),
    });

    let mut cur_y = 84.0;
    if spec.has_km_row {
        nodes.push(SpatialNode {
            target: SpatialTarget::KmKeyboardMouse,
            rect: SpatialRect::new(32.0, cur_y, 240.0, 40.0),
        });
        let mut x = 288.0;
        if spec.has_raw_kb {
            nodes.push(SpatialNode {
                target: SpatialTarget::KmRawKeyboard,
                rect: SpatialRect::new(x, cur_y, 240.0, 40.0),
            });
            x += 256.0;
        }
        if spec.has_raw_mouse {
            nodes.push(SpatialNode {
                target: SpatialTarget::KmRawMouse,
                rect: SpatialRect::new(x, cur_y, 240.0, 40.0),
            });
        }
        cur_y += 56.0;
    }

    let slots_top = cur_y;
    let cols = if spec.player_count <= 1 { 1 } else { 2 };
    let slot_w = if cols == 1 { 760.0 } else { 372.0 };
    let rows = if spec.player_count == 0 {
        0
    } else {
        (spec.player_count + cols - 1) / cols
    };

    for idx in 0..spec.player_count {
        let row = idx / cols;
        let col = idx % cols;
        nodes.push(SpatialNode {
            target: SpatialTarget::PlayerSlot(idx),
            rect: SpatialRect::new(
                32.0 + (col as f32) * (slot_w + 16.0),
                slots_top + (row as f32) * 164.0,
                slot_w,
                148.0,
            ),
        });
    }

    let mut after_y = slots_top + (rows as f32) * 164.0;

    if spec.unassigned_count > 0 {
        for idx in 0..spec.unassigned_count {
            nodes.push(SpatialNode {
                target: SpatialTarget::UnassignedItem(idx),
                rect: SpatialRect::new(32.0 + (idx as f32) * 192.0, after_y, 180.0, 44.0),
            });
        }
        after_y += 56.0;
    }

    if spec.ignored_count > 0 {
        for idx in 0..spec.ignored_count {
            nodes.push(SpatialNode {
                target: SpatialTarget::IgnoredItem(idx),
                rect: SpatialRect::new(32.0 + (idx as f32) * 192.0, after_y, 180.0, 44.0),
            });
        }
        after_y += 56.0;
    }

    if spec.can_launch {
        nodes.push(SpatialNode {
            target: SpatialTarget::LaunchButton,
            rect: SpatialRect::new(32.0, after_y + 8.0, 760.0, 48.0),
        });
    }

    // Right column: Profiles Manager aligned horizontally with slot rows
    nodes.push(SpatialNode {
        target: SpatialTarget::ProfileSaveNew,
        rect: SpatialRect::new(832.0, slots_top, 416.0, 44.0),
    });

    let chips_top = if rows <= 1 {
        slots_top + 56.0
    } else {
        slots_top + 216.0
    };
    for k in 0..spec.profile_count {
        let cy = chips_top + (k as f32) * 56.0;
        nodes.push(SpatialNode {
            target: SpatialTarget::ProfileChip(k),
            rect: SpatialRect::new(832.0, cy, 360.0, 44.0),
        });
        nodes.push(SpatialNode {
            target: SpatialTarget::ProfileDelete(k),
            rect: SpatialRect::new(1200.0, cy, 48.0, 44.0),
        });
    }

    nodes
}

/// Fallback geometric layout for the Big Picture Hub browser grid.
pub fn build_hub_layout_rects(
    category_count: usize,
    card_count: usize,
    columns: usize,
) -> Vec<SpatialNode> {
    let mut nodes = Vec::new();

    nodes.push(SpatialNode {
        target: SpatialTarget::HubSearch,
        rect: SpatialRect::new(32.0, 84.0, 420.0, 40.0),
    });

    for i in 0..category_count {
        nodes.push(SpatialNode {
            target: SpatialTarget::HubCategory(i),
            rect: SpatialRect::new(32.0 + (i as f32) * 140.0, 144.0, 128.0, 34.0),
        });
    }

    let cols = columns.max(1);
    for idx in 0..card_count {
        let row = idx / cols;
        let col = idx % cols;
        nodes.push(SpatialNode {
            target: SpatialTarget::HubCard(idx),
            rect: SpatialRect::new(
                32.0 + (col as f32) * 396.0,
                204.0 + (row as f32) * 190.0,
                380.0,
                172.0,
            ),
        });
    }

    nodes
}
