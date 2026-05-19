use std::fs;

use zircon_runtime::ui::v2::UiZuiAssetLoader;

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

    assert_eq!(sample.component.as_str(), "HorizontalBox");
    assert!(sample
        .classes
        .iter()
        .any(|class| class == "material-lab-sample"));
    assert_eq!(sample.events.len(), 1);
    assert_eq!(sample.events[0].id, "MaterialLab/Autocomplete/Change");
    assert_eq!(
        child_nodes(&document, "sample"),
        vec![
            "autocomplete_query",
            "autocomplete_open",
            "autocomplete_selected",
            "autocomplete_multi_chip",
            "autocomplete_disabled",
        ],
        "Autocomplete sample should show query, popup, selected, multi-chip, and disabled examples"
    );

    for (
        node_id,
        query,
        value,
        value_text,
        multiple,
        free_solo,
        popup_open,
        selected_options,
        filtered_options,
        matched_options,
        disabled,
    ) in [
        (
            "autocomplete_query",
            "at",
            "",
            "",
            false,
            false,
            true,
            Vec::<&str>::new(),
            vec!["atlas", "asset"],
            vec!["atlas", "asset"],
            false,
        ),
        (
            "autocomplete_open",
            "as",
            "",
            "",
            false,
            false,
            true,
            Vec::<&str>::new(),
            vec!["asset"],
            vec!["asset"],
            false,
        ),
        (
            "autocomplete_selected",
            "",
            "atlas",
            "Atlas",
            false,
            false,
            false,
            vec!["atlas"],
            vec!["atlas", "asset"],
            Vec::<&str>::new(),
            false,
        ),
        (
            "autocomplete_multi_chip",
            "a",
            "atlas,asset",
            "Atlas, Asset",
            true,
            false,
            false,
            vec!["atlas", "asset"],
            vec!["atlas", "asset"],
            vec!["atlas", "asset"],
            false,
        ),
        (
            "autocomplete_disabled",
            "disabled",
            "disabled",
            "Disabled",
            false,
            false,
            false,
            vec!["disabled"],
            vec!["disabled"],
            vec!["disabled"],
            true,
        ),
    ] {
        let node = document
            .nodes
            .get(node_id)
            .unwrap_or_else(|| panic!("{} should define `{node_id}`", path.display()));
        assert_eq!(node.component.as_str(), "Autocomplete");
        assert_eq!(
            node.props.get("query").and_then(|value| value.as_str()),
            Some(query),
            "Autocomplete child `{node_id}` should freeze query metadata"
        );
        assert_eq!(
            node.props.get("value").and_then(|value| value.as_str()),
            Some(value),
            "Autocomplete child `{node_id}` should freeze selected value metadata"
        );
        assert_eq!(
            node.props
                .get("value_text")
                .and_then(|value| value.as_str()),
            Some(value_text),
            "Autocomplete child `{node_id}` should freeze display text metadata"
        );
        assert_eq!(
            node.props.get("multiple").and_then(|value| value.as_bool()),
            Some(multiple),
            "Autocomplete child `{node_id}` should freeze multiple-selection metadata"
        );
        assert_eq!(
            node.props
                .get("free_solo")
                .and_then(|value| value.as_bool()),
            Some(free_solo),
            "Autocomplete child `{node_id}` should freeze free-solo metadata"
        );
        assert_eq!(
            node.props
                .get("popup_open")
                .and_then(|value| value.as_bool()),
            Some(popup_open),
            "Autocomplete child `{node_id}` should freeze popup state"
        );
        assert_eq!(
            string_array_prop(node.props.get("selected_options")),
            selected_options,
            "Autocomplete child `{node_id}` should freeze selected option ids"
        );
        assert_eq!(
            string_array_prop(node.props.get("filtered_options")),
            filtered_options,
            "Autocomplete child `{node_id}` should freeze filtered option ids"
        );
        assert_eq!(
            string_array_prop(node.props.get("matched_options")),
            matched_options,
            "Autocomplete child `{node_id}` should freeze matched option ids"
        );
        assert_eq!(
            string_array_prop(node.props.get("disabled_options")),
            vec!["disabled"],
            "Autocomplete child `{node_id}` should freeze disabled option ids"
        );
        assert_eq!(
            node.props.get("disabled").and_then(|value| value.as_bool()),
            Some(disabled),
            "Autocomplete child `{node_id}` should freeze disabled state"
        );
        assert!(
            node.events.is_empty(),
            "Autocomplete child examples should not add extra representative feedback routes"
        );
        assert_non_dispatchable_child(node, node_id, "Autocomplete");
    }
}
