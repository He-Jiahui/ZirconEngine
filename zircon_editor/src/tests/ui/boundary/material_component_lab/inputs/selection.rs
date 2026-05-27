use std::fs;

use zircon_runtime::ui::v2::UiZuiAssetLoader;
use zircon_runtime_interface::ui::v2::UiV2NodeDefinition;

use super::super::support::{child_nodes, editor_asset};
use super::{assert_non_dispatchable_child, string_array_prop};

#[test]
fn material_select_sample_covers_closed_open_selected_multi_and_disabled_states() {
    let path = editor_asset("assets/ui/editor/material_components/material_selects.zui");
    let source = fs::read_to_string(&path)
        .unwrap_or_else(|error| panic!("{} should be readable: {error}", path.display()));
    let document = UiZuiAssetLoader::load_zui_str(&source)
        .unwrap_or_else(|error| panic!("{} should load as .zui: {error}", path.display()));
    let sample = document
        .nodes
        .get("sample")
        .unwrap_or_else(|| panic!("{} should define the Select sample", path.display()));

    assert_eq!(sample.component.as_str(), "HorizontalBox");
    assert!(sample
        .classes
        .iter()
        .any(|class| class == "material-lab-sample"));
    assert_eq!(sample.events.len(), 1);
    assert_eq!(sample.events[0].id, "MaterialLab/Selects/Change");
    assert_eq!(
        child_nodes(&document, "sample"),
        vec![
            "select_closed",
            "select_open",
            "select_selected",
            "select_multi_chip",
            "select_disabled",
        ],
        "Select sample should show closed, open, selected, multi-chip, and disabled examples"
    );

    for (
        node_id,
        value,
        value_text,
        variant,
        multiple,
        display_empty,
        popup_open,
        selected_options,
        disabled,
    ) in [
        (
            "select_closed",
            "",
            "",
            "outlined",
            false,
            true,
            false,
            Vec::<&str>::new(),
            false,
        ),
        (
            "select_open",
            "secondary",
            "Secondary",
            "outlined",
            false,
            false,
            true,
            vec!["secondary"],
            false,
        ),
        (
            "select_selected",
            "primary",
            "Primary",
            "filled",
            false,
            false,
            false,
            vec!["primary"],
            false,
        ),
        (
            "select_multi_chip",
            "primary,secondary",
            "Primary, Secondary",
            "outlined",
            true,
            false,
            false,
            vec!["primary", "secondary"],
            false,
        ),
        (
            "select_disabled",
            "disabled",
            "Disabled",
            "standard",
            false,
            false,
            false,
            vec!["disabled"],
            true,
        ),
    ] {
        let node = document
            .nodes
            .get(node_id)
            .unwrap_or_else(|| panic!("{} should define `{node_id}`", path.display()));
        assert_eq!(node.component.as_str(), "Select");
        assert_eq!(
            node.props.get("value").and_then(|value| value.as_str()),
            Some(value),
            "Select child `{node_id}` should freeze selected value metadata"
        );
        assert_eq!(
            node.props
                .get("value_text")
                .and_then(|value| value.as_str()),
            Some(value_text),
            "Select child `{node_id}` should freeze display text metadata"
        );
        assert_eq!(
            node.props.get("variant").and_then(|value| value.as_str()),
            Some(variant),
            "Select child `{node_id}` should freeze variant metadata"
        );
        assert_eq!(
            node.props.get("multiple").and_then(|value| value.as_bool()),
            Some(multiple),
            "Select child `{node_id}` should freeze multiple-selection metadata"
        );
        assert_eq!(
            node.props
                .get("display_empty")
                .and_then(|value| value.as_bool()),
            Some(display_empty),
            "Select child `{node_id}` should freeze empty-display metadata"
        );
        assert_eq!(
            node.props
                .get("popup_open")
                .and_then(|value| value.as_bool()),
            Some(popup_open),
            "Select child `{node_id}` should freeze popup state"
        );
        assert_eq!(
            string_array_prop(node.props.get("selected_options")),
            selected_options,
            "Select child `{node_id}` should freeze selected option ids"
        );
        assert_eq!(
            string_array_prop(node.props.get("disabled_options")),
            vec!["disabled"],
            "Select child `{node_id}` should freeze disabled option ids"
        );
        assert_eq!(
            node.props.get("disabled").and_then(|value| value.as_bool()),
            Some(disabled),
            "Select child `{node_id}` should freeze disabled state"
        );
        assert!(
            node.events.is_empty(),
            "Select child examples should not add extra representative feedback routes"
        );
        assert_non_dispatchable_child(node, node_id, "Select");
    }
}

#[test]
fn material_autocomplete_sample_covers_query_popup_selected_multi_and_disabled_states() {
    let path = editor_asset("assets/ui/editor/material_components/material_autocomplete.zui");
    let source = fs::read_to_string(&path)
        .unwrap_or_else(|error| panic!("{} should be readable: {error}", path.display()));
    let document = UiZuiAssetLoader::load_zui_str(&source)
        .unwrap_or_else(|error| panic!("{} should load as .zui: {error}", path.display()));
    let sample = document
        .nodes
        .get("sample")
        .unwrap_or_else(|| panic!("{} should define the Autocomplete sample", path.display()));

    assert_eq!(sample.component.as_str(), "Autocomplete");
    assert!(sample
        .classes
        .iter()
        .any(|class| class == "material-lab-sample"));
    assert!(sample
        .classes
        .iter()
        .any(|class| class == "MuiAutocomplete-root"));
    assert_eq!(sample.events.len(), 1);
    assert_eq!(sample.events[0].id, "MaterialLab/Autocomplete/Change");
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
        ],
        "Autocomplete sample should mount local MUI input, tag, popup, paper, listbox, and option slots"
    );

    assert_eq!(str_prop(sample, "query"), Some("at"));
    assert_eq!(str_prop(sample, "inputValue"), Some("at"));
    assert_eq!(str_prop(sample, "value"), Some("atlas"));
    assert_eq!(str_prop(sample, "value_text"), Some("Atlas"));
    assert_eq!(str_prop(sample, "size"), Some("small"));
    assert_eq!(bool_prop(sample, "multiple"), Some(true));
    assert_eq!(bool_prop(sample, "free_solo"), Some(false));
    assert_eq!(bool_prop(sample, "freeSolo"), Some(false));
    assert_eq!(bool_prop(sample, "popup_open"), Some(true));
    assert_eq!(bool_prop(sample, "popupOpen"), Some(true));
    assert_eq!(bool_prop(sample, "fullWidth"), Some(true));
    assert_eq!(bool_prop(sample, "disableClearable"), Some(false));
    assert_eq!(bool_prop(sample, "disablePortal"), Some(true));
    assert_eq!(bool_prop(sample, "inputFocused"), Some(true));
    assert_eq!(bool_prop(sample, "loading"), Some(false));
    assert_eq!(bool_prop(sample, "readOnly"), Some(false));
    assert_eq!(str_prop(sample, "forcePopupIcon"), Some("auto"));
    assert_eq!(
        string_array_prop(sample.props.get("selected_options")),
        vec!["atlas"]
    );
    assert_eq!(
        string_array_prop(sample.props.get("selectedOptions")),
        vec!["atlas"]
    );
    assert_eq!(
        string_array_prop(sample.props.get("filtered_options")),
        vec!["atlas", "asset"]
    );
    assert_eq!(
        string_array_prop(sample.props.get("filteredOptions")),
        vec!["atlas", "asset"]
    );
    assert_eq!(
        string_array_prop(sample.props.get("matched_options")),
        vec!["atlas", "asset"]
    );
    assert_eq!(
        string_array_prop(sample.props.get("matchedOptions")),
        vec!["atlas", "asset"]
    );
    assert_eq!(
        string_array_prop(sample.props.get("disabled_options")),
        vec!["disabled"]
    );
    assert_eq!(
        string_array_prop(sample.props.get("disabledOptions")),
        vec!["disabled"]
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

    for (node_id, component, slot_name) in [
        ("autocomplete_input_root", "HorizontalBox", "inputRoot"),
        ("autocomplete_input", "Label", "input"),
        ("autocomplete_tag", "Label", "tag"),
        ("autocomplete_popup_indicator", "Label", "popupIndicator"),
        ("autocomplete_popper", "Label", "popper"),
        ("autocomplete_paper", "Label", "paper"),
        ("autocomplete_listbox", "Label", "listbox"),
        ("autocomplete_option", "Label", "option"),
    ] {
        let node = document
            .nodes
            .get(node_id)
            .unwrap_or_else(|| panic!("{} should define `{node_id}`", path.display()));
        assert_eq!(node.component.as_str(), component);
        assert_eq!(
            child_slot_name(sample, node_id),
            Some(slot_name),
            "Autocomplete child `{node_id}` should mount the expected slot"
        );
        assert!(
            node.events.is_empty(),
            "Autocomplete slot child `{node_id}` should not add feedback routes"
        );
        assert_non_dispatchable_child(node, node_id, "Autocomplete");
    }
}

fn str_prop<'a>(node: &'a UiV2NodeDefinition, name: &str) -> Option<&'a str> {
    node.props.get(name).and_then(|value| value.as_str())
}

fn bool_prop(node: &UiV2NodeDefinition, name: &str) -> Option<bool> {
    node.props.get(name).and_then(|value| value.as_bool())
}

fn slot_class_name<'a>(node: &'a UiV2NodeDefinition, slot: &str) -> Option<&'a str> {
    node.props
        .get("slotProps")
        .and_then(|value| value.as_table())
        .and_then(|slot_props| slot_props.get(slot))
        .and_then(|value| value.as_table())
        .and_then(|props| props.get("className"))
        .and_then(|value| value.as_str())
}

fn child_slot_name<'a>(node: &'a UiV2NodeDefinition, child_id: &str) -> Option<&'a str> {
    node.children
        .iter()
        .find(|child| child.node == child_id)
        .and_then(|child| child.slot.get("name"))
        .and_then(|value| value.as_str())
}
