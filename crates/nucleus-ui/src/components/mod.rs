//! # Nucleus UI Custom Component Meta-Framework
//!
//! A tailored domain-specific component layer built on top of `gpui-kit` and `gpui-component`.
//!
//! ## Architecture & Safety Invariants:
//! 1. **Zero-Boilerplate Presets**: High-level builders for both desktop power-user and 10-foot couch modes.
//! 2. **Type Boundary Flattening**: Every component implements `.into_any_element()` returning an `AnyElement`.
//!    This prevents deeply nested generic type tree explosion, drastically reducing incremental compile times.
//! 3. **Guaranteed Event Safety**: Modals, overlays, and popovers automatically stop mouse event propagation,
//!    preventing accidental click pass-through to underlying cards or inputs.
//! 4. **Edge-to-Edge Cover Art**: `GameCover` strictly enforces explicit aspect ratio constraints to eliminate
//!    letterbox/pillarbox gaps.

pub mod badge;
pub mod banner;
pub mod button;
pub mod card;
pub mod confirm_dialog;
pub mod context_menu;
pub mod cover;
pub mod launch_prompt;
pub mod modal;
pub mod option_combobox;
pub mod option_row;
pub mod partition;
pub mod picker;
pub mod search_bar;
pub mod segmented_control;
pub mod sidebar_nav;
pub mod slot_card;
pub mod status_bar;
pub mod title_bar;
pub mod toggle;

pub use badge::{CouchTag, TagTone};
pub use banner::{NefosBanner, NefosBannerTone};
pub use button::{CouchButton, CouchButtonSize, CouchButtonVariant};
pub use card::{NefosCard, NefosCardVariant};
pub use confirm_dialog::CouchConfirmDialog;
pub use context_menu::CouchContextMenu;
pub use cover::GameCover;
pub use launch_prompt::{LaunchPromptModal, NefosLaunchPromptModal};
pub use modal::{CouchModal, ModalWidth};
pub use option_combobox::{
    CouchOptionCombobox, filter_combobox_options, is_boolean_values, should_use_combobox,
    validate_combobox_input,
};
pub use option_row::{CouchOptionRow, OPTION_CARD_MIN_HEIGHT_PX, OPTION_CONTROL_HEIGHT_PX};
pub use partition::{PartitionPreview, SplitLayout};
pub use picker::CouchPicker;
pub use search_bar::CouchSearchBar;
pub use segmented_control::{NefosSegmentedControl, SegmentClickHandler, SegmentSelectHandler};
pub use sidebar_nav::CouchSidebarNav;
pub use slot_card::CouchSlotCard;
pub use status_bar::CouchStatusBar;
pub use title_bar::CouchTitleBar;
pub use toggle::CouchToggle;
