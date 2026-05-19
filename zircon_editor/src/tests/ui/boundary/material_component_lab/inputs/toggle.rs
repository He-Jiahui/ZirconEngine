use std::fs;

use zircon_runtime::ui::v2::UiZuiAssetLoader;

use super::super::support::{child_nodes, editor_asset};
use super::assert_non_dispatchable_child;

#[test]
fn material_toggle_button_sample_covers_exclusive_multiple_and_disabled_states() {
    let path = editor_asset("assets/ui/editor/material_components/material_toggle_button.zui");
    let source = fs::read_to_string(&path)
        .unwrap_or_else(|error| panic!("{} should be readable: {error}", path.display()));
    let document = UiZuiAssetLoader::load_zui_str(&source)
        .unwrap_or_else(|error| panic!("{} should load as .zui: {error}", path.display()));
    let sample = document
        .nodes
        .get("sample")
        .unwrap_or_else(|| panic!("{} should define the ToggleButton sample", path.display()));

    assert_eq!(sample.component.as_str(), "HorizontalBox");
    assert!(sample
        .classes
        .iter()
        .any(|class| class == "material-lab-sample"));
    assert_eq!(sample.events.len(), 1);
    assert_eq!(sample.events[0].id, "MaterialLab/ToggleButton/Toggle");
    assert_eq!(
        child_nodes(&document, "sample"),
        vec![
            "toggle_exclusive_selected",
            "toggle_exclusive_hover",
            "toggle_multi_checked",
            "toggle_disabled",
        ],
        "ToggleButton sample should show the planned exclusive, multiple, and disabled examples"
    );

    for (node_id, selection_state, selected, checked, disabled) in [
        ("toggle_exclusive_selected", "exclusive", true, true, false),
        ("toggle_exclusive_hover", "exclusive", false, false, false),
        ("toggle_multi_checked", "multiple", true, true, false),
        ("toggle_disabled", "exclusive", false, false, true),
    ] {
        let node = document
            .nodes
            .get(node_id)
            .unwrap_or_else(|| panic!("{} should define `{node_id}`", path.display()));
        assert_eq!(node.component.as_str(), "ToggleButton");
        assert_eq!(
            node.props
                .get("selection_state")
                .and_then(|value| value.as_str()),
            Some(selection_state),
            "ToggleButton `{node_id}` should freeze selection semantics"
        );
        assert_eq!(
            node.props.get("selected").and_then(|value| value.as_bool()),
            Some(selected),
            "ToggleButton `{node_id}` should freeze selected state"
        );
        assert_eq!(
            node.props.get("checked").and_then(|value| value.as_bool()),
            Some(checked),
            "ToggleButton `{node_id}` should freeze checked state"
        );
        assert_eq!(
            node.props.get("disabled").and_then(|value| value.as_bool()),
            Some(disabled),
            "ToggleButton `{node_id}` should freeze disabled state"
        );
        assert!(
            node.events.is_empty(),
            "ToggleButton child examples should not add extra representative feedback routes"
        );
        assert_non_dispatchable_child(node, node_id, "ToggleButton");
    }
}
