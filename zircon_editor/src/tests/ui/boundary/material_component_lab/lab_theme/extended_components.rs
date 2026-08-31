use super::*;

#[test]
fn material_component_lab_typography_sample_uses_runtime_descriptor_and_theme_selectors() {
    let path =
        editor_asset("assets/ui/editor/material_components/data_display/material_typography.zui");
    let source = fs::read_to_string(&path)
        .unwrap_or_else(|error| panic!("{} should be readable: {error}", path.display()));
    let document = UiZuiAssetLoader::load_zui_str(&source)
        .unwrap_or_else(|error| panic!("{} should load as .zui: {error}", path.display()));
    let sample = document
        .nodes
        .get("sample")
        .unwrap_or_else(|| panic!("{} should define a sample node", path.display()));

    assert_component(&document, "sample", "Typography");
    assert_node_class(&document, "sample", "MuiTypography-root");
    assert_eq!(
        str_prop(sample, "className"),
        Some("material-typography-sample")
    );
    assert_eq!(str_prop(sample, "component"), Some("h6"));
    assert_eq!(str_prop(sample, "variant"), Some("h6"));
    assert_eq!(str_prop(sample, "align"), Some("center"));
    assert_eq!(bool_prop(sample, "gutterBottom"), Some(true));
    assert_eq!(bool_prop(sample, "noWrap"), Some(true));
    assert_eq!(table_str_prop(sample, "variantMapping", "h6"), Some("h2"));
    assert_eq!(table_str_prop(sample, "variantMapping", "body2"), Some("p"));

    let selectors = editor_material_theme_selectors();
    for selector in TYPOGRAPHY_THEME_SELECTORS {
        assert!(
            selectors.contains(*selector),
            "Editor Material theme should style Typography selector `{selector}`"
        );
    }
}

#[test]
fn material_component_lab_timeline_sample_uses_runtime_descriptor_and_theme_selectors() {
    let path =
        editor_asset("assets/ui/editor/material_components/data_display/material_timeline.zui");
    let source = fs::read_to_string(&path)
        .unwrap_or_else(|error| panic!("{} should be readable: {error}", path.display()));
    let document = UiZuiAssetLoader::load_zui_str(&source)
        .unwrap_or_else(|error| panic!("{} should load as .zui: {error}", path.display()));
    let sample = document
        .nodes
        .get("sample")
        .unwrap_or_else(|| panic!("{} should define a sample node", path.display()));

    assert_component(&document, "sample", "Timeline");
    assert_node_class(&document, "sample", "MuiTimeline-root");
    assert_eq!(
        str_prop(sample, "className"),
        Some("material-timeline-sample")
    );
    assert_eq!(str_prop(sample, "component"), Some("ul"));
    assert_eq!(str_prop(sample, "position"), Some("alternate-reverse"));
    assert_eq!(numeric_prop(sample.props.get("time")), Some(12.0));
    assert_eq!(numeric_prop(sample.props.get("duration")), Some(48.0));
    for (slot, expected_class) in [
        ("items", "material-timeline-item"),
        ("content", "material-timeline-content"),
        ("separator", "material-timeline-separator"),
        ("connector", "material-timeline-connector"),
        ("dot", "material-timeline-dot"),
    ] {
        assert_eq!(
            slot_class_name(sample, slot),
            Some(expected_class),
            "Timeline sample slotProps.{slot}.className should stay theme-addressable"
        );
    }

    assert_eq!(child_nodes(&document, "sample"), vec!["timeline_item"]);
    assert_component(&document, "timeline_item", "TimelineItem");
    assert_node_class(&document, "timeline_item", "MuiTimelineItem-root");
    let item = node(&document, "timeline_item");
    assert_eq!(str_prop(item, "position"), Some("alternate-reverse"));
    assert_eq!(bool_prop(item, "hasOppositeContent"), Some(false));
    assert_eq!(
        child_nodes(&document, "timeline_item"),
        vec!["timeline_separator", "timeline_content"]
    );

    assert_component(&document, "timeline_separator", "TimelineSeparator");
    assert_node_class(&document, "timeline_separator", "MuiTimelineSeparator-root");
    assert_eq!(
        child_nodes(&document, "timeline_separator"),
        vec!["timeline_dot", "timeline_connector"]
    );

    assert_component(&document, "timeline_dot", "TimelineDot");
    assert_node_class(&document, "timeline_dot", "MuiTimelineDot-root");
    let dot = node(&document, "timeline_dot");
    assert_eq!(str_prop(dot, "variant"), Some("outlined"));
    assert_eq!(str_prop(dot, "color"), Some("secondary"));
    assert_eq!(str_prop(dot, "className"), Some("material-timeline-dot"));

    assert_component(&document, "timeline_connector", "TimelineConnector");
    assert_node_class(&document, "timeline_connector", "MuiTimelineConnector-root");

    assert_component(&document, "timeline_content", "TimelineContent");
    assert_node_class(&document, "timeline_content", "MuiTimelineContent-root");
    let content = node(&document, "timeline_content");
    assert_eq!(str_prop(content, "position"), Some("right"));
    assert_eq!(
        str_prop(content, "className"),
        Some("material-timeline-content")
    );

    let selectors = editor_material_theme_selectors();
    for selector in TIMELINE_THEME_SELECTORS {
        assert!(
            selectors.contains(*selector),
            "Editor Material theme should style Timeline selector `{selector}`"
        );
    }
}

#[test]
fn material_component_lab_transfer_list_sample_uses_runtime_descriptor_and_theme_selectors() {
    let path =
        editor_asset("assets/ui/editor/material_components/navigation/material_transfer_list.zui");
    let source = fs::read_to_string(&path)
        .unwrap_or_else(|error| panic!("{} should be readable: {error}", path.display()));
    let document = UiZuiAssetLoader::load_zui_str(&source)
        .unwrap_or_else(|error| panic!("{} should load as .zui: {error}", path.display()));
    let sample = document
        .nodes
        .get("sample")
        .unwrap_or_else(|| panic!("{} should define a sample node", path.display()));

    assert_component(&document, "sample", "TransferList");
    assert_node_class(&document, "sample", "MuiTransferList-root");
    assert_eq!(
        str_prop(sample, "className"),
        Some("material-transfer-list-sample")
    );
    assert_eq!(
        string_array_prop(sample, "source_items"),
        vec!["Scene", "Materials", "Lighting"]
    );
    assert_eq!(string_array_prop(sample, "target_items"), vec!["Export"]);
    assert_eq!(
        string_array_prop(sample, "source_selected_items"),
        vec!["Materials"]
    );
    assert_eq!(
        string_array_prop(sample, "target_selected_items"),
        vec!["Export"]
    );
    assert_eq!(
        string_array_prop(sample, "disabled_items"),
        vec!["Lighting"]
    );
    assert_eq!(
        string_array_prop(sample, "disabled_actions"),
        vec!["move_all_left"]
    );
    for (slot, expected_class) in [
        ("source", "material-transfer-list-source"),
        ("target", "material-transfer-list-target"),
        ("actions", "material-transfer-list-actions"),
    ] {
        assert_eq!(
            slot_class_name(sample, slot),
            Some(expected_class),
            "TransferList sample slotProps.{slot}.className should stay theme-addressable"
        );
    }

    assert_eq!(
        child_nodes(&document, "sample"),
        vec!["transfer_source", "transfer_actions", "transfer_target"]
    );
    assert_component(&document, "transfer_source", "List");
    assert_node_class(
        &document,
        "transfer_source",
        "material-transfer-list-source",
    );
    assert_component(&document, "transfer_actions", "Button");
    assert_node_class(
        &document,
        "transfer_actions",
        "material-transfer-list-actions",
    );
    assert_component(&document, "transfer_target", "List");
    assert_node_class(
        &document,
        "transfer_target",
        "material-transfer-list-target",
    );
    for child_id in ["transfer_source", "transfer_actions", "transfer_target"] {
        assert_non_dispatchable_child(node(&document, child_id), child_id, "TransferList");
    }

    let selectors = editor_material_theme_selectors();
    for selector in TRANSFER_LIST_THEME_SELECTORS {
        assert!(
            selectors.contains(*selector),
            "Editor Material theme should style TransferList selector `{selector}`"
        );
    }
}

#[test]
fn material_component_lab_autocomplete_sample_uses_runtime_descriptor_and_theme_selectors() {
    let path =
        editor_asset("assets/ui/editor/material_components/inputs/material_autocomplete.zui");
    let source = fs::read_to_string(&path)
        .unwrap_or_else(|error| panic!("{} should be readable: {error}", path.display()));
    let document = UiZuiAssetLoader::load_zui_str(&source)
        .unwrap_or_else(|error| panic!("{} should load as .zui: {error}", path.display()));
    let sample = document
        .nodes
        .get("sample")
        .unwrap_or_else(|| panic!("{} should define a sample node", path.display()));

    assert_component(&document, "sample", "Autocomplete");
    assert_node_class(&document, "sample", "MuiAutocomplete-root");
    assert_eq!(
        str_prop(sample, "className"),
        Some("material-autocomplete-sample")
    );
    assert_eq!(str_prop(sample, "query"), Some("at"));
    assert_eq!(str_prop(sample, "inputValue"), Some("at"));
    assert_eq!(str_prop(sample, "value"), Some("atlas"));
    assert_eq!(bool_prop(sample, "multiple"), Some(true));
    assert_eq!(bool_prop(sample, "popup_open"), Some(true));
    assert_eq!(bool_prop(sample, "popupOpen"), Some(true));
    assert_eq!(bool_prop(sample, "fullWidth"), Some(true));
    assert_eq!(bool_prop(sample, "disablePortal"), Some(true));
    assert_eq!(bool_prop(sample, "inputFocused"), Some(true));
    assert_eq!(string_array_prop(sample, "selected_options"), vec!["atlas"]);
    assert_eq!(
        string_array_prop(sample, "filtered_options"),
        vec!["atlas", "asset"]
    );
    assert_eq!(
        string_array_prop(sample, "matched_options"),
        vec!["atlas", "asset"]
    );
    for (slot, expected_class) in [
        ("inputRoot", "material-autocomplete-input-root"),
        ("input", "material-autocomplete-input"),
        ("tag", "material-autocomplete-tag"),
        ("popupIndicator", "material-autocomplete-popup-indicator"),
        ("popper", "material-autocomplete-popper"),
        ("paper", "material-autocomplete-paper"),
        ("listbox", "material-autocomplete-listbox"),
        ("option", "material-autocomplete-option"),
    ] {
        assert_eq!(
            slot_class_name(sample, slot),
            Some(expected_class),
            "Autocomplete sample slotProps.{slot}.className should stay theme-addressable"
        );
    }

    assert_eq!(
        child_nodes(&document, "sample"),
        vec![
            "autocomplete_input_root",
            "autocomplete_input",
            "autocomplete_tag",
            "autocomplete_popup_indicator",
            "autocomplete_popper",
            "autocomplete_paper",
            "autocomplete_listbox",
            "autocomplete_option",
        ]
    );
    for (child_id, expected_component, expected_slot) in [
        ("autocomplete_input_root", "HorizontalBox", "inputRoot"),
        ("autocomplete_input", "Label", "input"),
        ("autocomplete_tag", "Label", "tag"),
        ("autocomplete_popup_indicator", "Label", "popupIndicator"),
        ("autocomplete_popper", "Label", "popper"),
        ("autocomplete_paper", "Label", "paper"),
        ("autocomplete_listbox", "Label", "listbox"),
        ("autocomplete_option", "Label", "option"),
    ] {
        assert_component(&document, child_id, expected_component);
        assert_eq!(
            child_slot_name(sample, child_id),
            Some(expected_slot),
            "Autocomplete child `{child_id}` should mount the expected slot"
        );
        assert_non_dispatchable_child(node(&document, child_id), child_id, "Autocomplete");
    }

    let selectors = editor_material_theme_selectors();
    for selector in AUTOCOMPLETE_THEME_SELECTORS {
        assert!(
            selectors.contains(*selector),
            "Editor Material theme should style Autocomplete selector `{selector}`"
        );
    }
}
