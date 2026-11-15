//! Couch "Big Picture" 10-foot television interface modules.
//!
//! Provides gamepad navigation, library carousel, auto-assigned controller slots,
//! and in-game couch OSD overlay.

pub mod carousel;
pub mod hub;
pub mod input;
pub mod overlay;
pub mod setup;
pub mod spatial;

pub use carousel::{
    BigPictureScreen, CarouselFilterMode, CarouselSortMode, CarouselState,
    MISSING_EXE_METADATA_CALLOUT, UNINSTALLED_BADGE_TEXT, carousel_card_opacity,
    render_big_picture_view, render_carousel_view,
};
pub use hub::{
    CouchHubState, CouchOptionItem, CouchOptionsModalState, PathResolution, get_filtered_packages,
    render_couch_hub_view, render_custom_options_modal, render_missing_exe_modal,
    resolve_game_path_or_couch_prompt,
};
pub use input::{
    DirectionalRepeatState, GamepadInputDispatcher, StickDirection, map_key_to_couch_action_name,
};
pub use overlay::{
    InGameOsdState, OsdAction, OsdSubView, format_duration_hhmmss, render_in_game_osd,
};
pub use setup::{
    AVAILABLE_LAYOUTS, CouchPlayerSlot, CouchSetupState, CouchSetupStep, DisconnectStatus,
    PartitionLayoutKind, create_keyboard_mouse_device, is_keyboard_device, render_couch_setup_view,
    render_disconnect_toast_banner, render_ignored_controllers_tray,
    render_virtual_monitor_partition_box,
};
pub use spatial::{
    SetupLayoutSpec, SpatialBoundsRegistry, SpatialDirection, SpatialNavGraph, SpatialNode,
    SpatialRect, SpatialTarget, build_carousel_layout_rects, build_hub_layout_rects,
    build_setup_layout_rects, calculate_cone_distance, resolve_spatial_move, summoner_glow_color,
    track_spatial_bounds, weighted_cone_distance,
};
