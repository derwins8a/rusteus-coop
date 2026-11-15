//! # Nefos UI Design System
//!
//! Apple-style sleek, ergonomic, and GPU-accelerated design system for Nucleus Co-op.
//! Provides unified surface elevation, segmented navigation, and canonical re-exports.

pub mod banner;
pub mod card;
pub mod launch_prompt;
pub mod segmented_control;

pub use banner::*;
pub use card::*;
pub use launch_prompt::*;
pub use segmented_control::*;

// Canonical re-exports from crate::components::*
pub use crate::components::button::{
    CouchButton as NefosButton, CouchButtonSize as NefosButtonSize,
    CouchButtonVariant as NefosButtonVariant,
};
pub use crate::components::badge::{CouchTag as NefosTag, TagTone as NefosTagTone};
pub use crate::components::modal::{CouchModal as NefosModal, ModalWidth as NefosModalWidth};
pub use crate::components::option_combobox::{
    CouchOptionCombobox as NefosOptionCombobox, filter_combobox_options, is_boolean_values,
    should_use_combobox, validate_combobox_input,
};
pub use crate::components::option_row::{
    CouchOptionRow as NefosOptionRow, OPTION_CARD_MIN_HEIGHT_PX, OPTION_CONTROL_HEIGHT_PX,
};
pub use crate::components::picker::CouchPicker as NefosPicker;
pub use crate::components::search_bar::CouchSearchBar as NefosSearchBar;
pub use crate::components::sidebar_nav::CouchSidebarNav as NefosSidebarNav;
pub use crate::components::status_bar::CouchStatusBar as NefosStatusBar;
pub use crate::components::title_bar::CouchTitleBar as NefosTitleBar;
pub use crate::components::toggle::CouchToggle as NefosToggle;
pub use crate::components::slot_card::CouchSlotCard as NefosSlotCard;
pub use crate::components::partition::{
    PartitionPreview as NefosPartitionPreview, SplitLayout as NefosSplitLayout,
};
pub use crate::components::cover::GameCover as NefosCover;
pub use crate::components::confirm_dialog::CouchConfirmDialog as NefosConfirmDialog;
pub use crate::components::context_menu::CouchContextMenu as NefosContextMenu;
