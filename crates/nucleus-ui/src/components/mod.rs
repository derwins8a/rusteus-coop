//! # Nucleus UI Custom Component Meta-Framework
//!
//! A tailored domain-specific component layer built on top of `gpui-kit` and `gpui-component` (Longbridge).
//!
//! ## Architecture & Safety Invariants:
//! 1. **Zero-Boilerplate Presets**: High-level builders for 10-foot Couch mode and desktop layouts.
//! 2. **Type Boundary Flattening**: Every component implements `.into_any_element()` returning an `AnyElement`.
//!    This prevents deeply nested generic type tree explosion (`Div<Div<Div<...>>>>`), drastically reducing
//!    incremental compilation times on Windows MSVC.
//! 3. **Guaranteed Event Safety**: Modals and popovers automatically stop mouse event propagation, permanently
//!    preventing accidental click pass-through to underlying cards.
//! 4. **Edge-to-Edge Cover Art**: `GameCover` strictly enforces explicit aspect ratio constraints to eliminate
//!    letterbox/pillarbox gaps caused by default GPUI `img.rs` intrinsic aspect ratio overrides.

pub mod badge;
pub mod button;
pub mod confirm_dialog;
pub mod context_menu;
pub mod cover;
pub mod modal;
pub mod option_combobox;
pub mod option_row;
pub mod partition;
pub mod picker;
pub mod search_bar;
pub mod sidebar_nav;
pub mod slot_card;
pub mod status_bar;
pub mod title_bar;
pub mod toggle;

pub use crate::nefos;
pub use nefos::*;

pub use badge::{CouchTag, TagTone};
pub use button::{CouchButton, CouchButtonSize, CouchButtonVariant};
pub use confirm_dialog::CouchConfirmDialog;
pub use context_menu::CouchContextMenu;
pub use cover::GameCover;
pub use modal::{CouchModal, ModalWidth};
pub use option_combobox::{
    CouchOptionCombobox, filter_combobox_options, is_boolean_values, should_use_combobox,
    validate_combobox_input,
};
pub use option_row::{CouchOptionRow, OPTION_CARD_MIN_HEIGHT_PX, OPTION_CONTROL_HEIGHT_PX};
pub use partition::{PartitionPreview, SplitLayout};
pub use picker::CouchPicker;
pub use search_bar::CouchSearchBar;
pub use sidebar_nav::CouchSidebarNav;
pub use slot_card::CouchSlotCard;
pub use status_bar::CouchStatusBar;
pub use title_bar::CouchTitleBar;
pub use toggle::CouchToggle;
