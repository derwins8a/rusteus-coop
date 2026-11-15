//! Integration and unit tests for uniform option sizing and `CouchOptionCombobox`.

use nucleus_ui::components::{
    CouchOptionCombobox, CouchOptionRow, CouchToggle, OPTION_CARD_MIN_HEIGHT_PX,
    OPTION_CONTROL_HEIGHT_PX, filter_combobox_options, is_boolean_values, should_use_combobox,
    validate_combobox_input,
};
use nucleus_ui::nefos::{
    NefosOptionCombobox, OPTION_CARD_MIN_HEIGHT_PX as NEFOS_OPTION_CARD_MIN_HEIGHT_PX,
    OPTION_CONTROL_HEIGHT_PX as NEFOS_OPTION_CONTROL_HEIGHT_PX,
    filter_combobox_options as nefos_filter_combobox_options,
    should_use_combobox as nefos_should_use_combobox,
    validate_combobox_input as nefos_validate_combobox_input,
};

#[test]
fn test_option_sizing_constants() {
    assert_eq!(OPTION_CARD_MIN_HEIGHT_PX, 56.0);
    assert_eq!(OPTION_CONTROL_HEIGHT_PX, 32.0);
    assert_eq!(NEFOS_OPTION_CARD_MIN_HEIGHT_PX, 56.0);
    assert_eq!(NEFOS_OPTION_CONTROL_HEIGHT_PX, 32.0);
}

#[test]
fn test_couch_toggle_and_option_row_construction() {
    let full_toggle = CouchToggle::new("opt-toggle-full", "Topmost Windows", true)
        .description("Keeps splitscreen game viewports pinned to top of display")
        .focused(true)
        .disabled(false)
        .on_click(|_, _, _| {});
    let _e1 = full_toggle.into_any_element();

    let empty_desc_toggle = CouchToggle::new("opt-toggle-nodesc", "Windowed Mode", false)
        .description("")
        .focused(false);
    let _e2 = empty_desc_toggle.into_any_element();

    let compact_toggle = CouchToggle::new("opt-toggle-compact", "Offline Mode", true)
        .compact(true);
    let _e3 = compact_toggle.into_any_element();

    let option_row = CouchOptionRow::new(
        "opt-row-quality",
        "Shadow Quality",
        "Adjusts real-time shadow map fidelity across instances",
        "High",
    )
    .focused(true)
    .on_prev(|_, _, _| {})
    .on_next(|_, _, _| {});
    let _e4 = option_row.into_any_element();

    let option_row_no_desc = CouchOptionRow::new("opt-row-nodesc", "HUD Scale", "", "100%");
    let _e5 = option_row_no_desc.into_any_element();
}

#[test]
fn test_is_boolean_values_semantics() {
    assert!(is_boolean_values(&["true".into(), "false".into()]));
    assert!(is_boolean_values(&["FALSE".into(), "TRUE".into()]));
    assert!(is_boolean_values(&["Yes".into(), "No".into()]));
    assert!(is_boolean_values(&["no".into(), "yes".into()]));
    assert!(is_boolean_values(&["1".into(), "0".into()]));
    assert!(is_boolean_values(&["0".into(), "1".into()]));
    assert!(is_boolean_values(&["Enabled".into(), "Disabled".into()]));
    assert!(is_boolean_values(&["On".into(), "Off".into()]));

    assert!(!is_boolean_values(&["true".into()]));
    assert!(!is_boolean_values(&["true".into(), "maybe".into()]));
    assert!(!is_boolean_values(&["1".into(), "2".into()]));
    assert!(!is_boolean_values(&[
        "Low".into(),
        "Medium".into(),
        "High".into()
    ]));
}

#[test]
fn test_should_use_combobox_thresholds() {
    // 2 booleans -> false
    assert!(!should_use_combobox(&["true".into(), "false".into()]));
    assert!(!should_use_combobox(&["Yes".into(), "No".into()]));
    assert!(!should_use_combobox(&["1".into(), "0".into()]));

    // 4 choices -> false
    assert!(!should_use_combobox(&[
        "Low".into(),
        "Medium".into(),
        "High".into(),
        "Ultra".into()
    ]));

    // 5 choices -> true
    let five_choices: Vec<String> = vec![
        "1280x720".into(),
        "1600x900".into(),
        "1920x1080".into(),
        "2560x1440".into(),
        "3840x2160".into(),
    ];
    assert!(should_use_combobox(&five_choices));
    assert!(nefos_should_use_combobox(&five_choices));
}

#[test]
fn test_filter_combobox_options_ranking() {
    let values: Vec<String> = vec![
        "Ultra High".into(),
        "High Quality".into(),
        "Medium High".into(),
        "High".into(),
        "Low".into(),
    ];

    // Empty / whitespace query returns all in original order
    let all = filter_combobox_options(&values, "   ");
    let all_strs: Vec<&str> = all.iter().map(|s| s.as_str()).collect();
    assert_eq!(
        all_strs,
        vec!["Ultra High", "High Quality", "Medium High", "High", "Low"]
    );

    // Exact match first, then prefix matches in order, then substring matches in order
    let ranked = filter_combobox_options(&values, " high ");
    let ranked_strs: Vec<&str> = ranked.iter().map(|s| s.as_str()).collect();
    assert_eq!(
        ranked_strs,
        vec!["High", "High Quality", "Ultra High", "Medium High"]
    );

    // Re-export behaves identically
    assert_eq!(nefos_filter_combobox_options(&values, "high"), ranked);

    // No match returns empty
    assert!(filter_combobox_options(&values, "nonexistent").is_empty());
}

#[test]
fn test_validate_combobox_input_strict_matching() {
    let values: Vec<String> = vec!["Low".into(), "Medium".into(), "High".into(), "Ultra".into()];

    assert_eq!(validate_combobox_input("high", &values), Ok("High".to_string()));
    assert_eq!(
        validate_combobox_input("  HIGH ", &values),
        Ok("High".to_string())
    );
    assert_eq!(
        nefos_validate_combobox_input("medium", &values),
        Ok("Medium".to_string())
    );

    assert_eq!(
        validate_combobox_input("  CustomVal ", &values),
        Err("Invalid option 'CustomVal': must match one of 4 available choices".to_string())
    );
    assert_eq!(
        validate_combobox_input("", &values),
        Err("Invalid option '': must match one of 4 available choices".to_string())
    );
}

#[test]
fn test_couch_option_combobox_open_closed_and_validation_states() {
    let values: Vec<String> = vec![
        "1280x720".into(),
        "1600x900".into(),
        "1920x1080".into(),
        "2560x1440".into(),
        "3840x2160".into(),
    ];

    // Closed, valid
    let closed_valid = CouchOptionCombobox::new(
        "cb-res-closed",
        "Render Resolution",
        "Target internal resolution per splitscreen instance",
        "1920x1080",
        values.clone(),
    )
    .is_open(false)
    .focused(false)
    .focus_handle(None);
    let _e1 = closed_valid.into_any_element();

    // Open, with search draft, highlight, and callbacks
    let open_valid = CouchOptionCombobox::new(
        "cb-res-open",
        "Render Resolution",
        "",
        "1920x1080",
        values.clone(),
    )
    .is_open(true)
    .search_draft("1080")
    .highlighted_idx(0)
    .focused(true)
    .on_toggle(|_, _, _| {})
    .on_select(|_, _, _| {})
    .on_key_down(|_, _, _| {})
    .on_clear_draft(|_, _, _| {});
    let _e2 = open_valid.into_any_element();

    // Closed with validation error
    let closed_err = CouchOptionCombobox::new(
        "cb-res-closed-err",
        "Render Resolution",
        "Select one of the supported resolutions",
        "1920x1080",
        values.clone(),
    )
    .is_open(false)
    .validation_error(Some(
        "Invalid option '4K': must match one of 5 available choices".to_string(),
    ));
    let _e3 = closed_err.into_any_element();

    // Open with validation error + Nefos alias
    let open_err = NefosOptionCombobox::new(
        "cb-res-open-err",
        "Render Resolution",
        "Select one of the supported resolutions",
        "1920x1080",
        values,
    )
    .is_open(true)
    .search_draft("4K")
    .validation_error(Some(
        "Invalid option '4K': must match one of 5 available choices".to_string(),
    ));
    let _e4 = open_err.into_any_element();
}
