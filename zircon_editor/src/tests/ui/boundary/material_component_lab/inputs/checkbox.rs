use std::fs;

use zircon_runtime::ui::v2::UiZuiAssetLoader;

use super::super::support::{child_nodes, editor_asset};
use super::assert_non_dispatchable_child;

#[test]
fn material_checkbox_sample_covers_unchecked_checked_indeterminate_error_and_disabled_states() {
    let path = editor_asset("assets/ui/editor/material_components/material_checkboxes.zui");
    let source = fs::read_to_string(&path)
        .unwrap_or_else(|error| panic!("{} should be readable: {error}", path.display()));
    let document = UiZuiAssetLoader::load_zui_str(&source)
        .unwrap_or_else(|error| panic!("{} should load as .zui: {error}", path.display()));
    let sample = document
        .nodes
        .get("sample")
        .unwrap_or_else(|| panic!("{} should define the Checkbox sample", path.display()));

    assert_eq!(sample.component.as_str(), "HorizontalBox");
    assert!(sample
        .classes
        .iter()
        .any(|class| class == "material-lab-sample"));
    assert_eq!(sample.events.len(), 1);
    assert_eq!(sample.events[0].id, "MaterialLab/Checkboxes/Toggle");
    assert_eq!(
        child_nodes(&document, "sample"),
        vec![
            "checkbox_unchecked",
            "checkbox_checked",
            "checkbox_indeterminate",
            "checkbox_error",
            "checkbox_disabled",
        ],
        "Checkbox sample should show unchecked, checked, indeterminate, error, and disabled examples"
    );

    for (node_id, checked, indeterminate, selected, validation_level, disabled, text_tone) in [
        (
            "checkbox_unchecked",
            false,
            false,
            false,
            "normal",
            false,
            "secondary",
        ),
        (
            "checkbox_checked",
            true,
            false,
            true,
            "normal",
            false,
            "primary",
        ),
        (
            "checkbox_indeterminate",
            false,
            true,
            true,
            "normal",
            false,
            "primary",
        ),
        (
            "checkbox_error",
            false,
            false,
            false,
            "error",
            false,
            "error",
        ),
        (
            "checkbox_disabled",
            true,
            false,
            false,
            "normal",
            true,
            "disabled",
        ),
    ] {
        let node = document
            .nodes
            .get(node_id)
            .unwrap_or_else(|| panic!("{} should define `{node_id}`", path.display()));
        assert_eq!(node.component.as_str(), "Checkbox");
        assert_eq!(
            node.props.get("checked").and_then(|value| value.as_bool()),
            Some(checked),
            "Checkbox child `{node_id}` should freeze checked state"
        );
        assert_eq!(
            node.props
                .get("indeterminate")
                .and_then(|value| value.as_bool()),
            Some(indeterminate),
            "Checkbox child `{node_id}` should freeze indeterminate state"
        );
        assert_eq!(
            node.props.get("selected").and_then(|value| value.as_bool()),
            Some(selected),
            "Checkbox child `{node_id}` should freeze selected paint state"
        );
        assert_eq!(
            node.props
                .get("validation_level")
                .and_then(|value| value.as_str()),
            Some(validation_level),
            "Checkbox child `{node_id}` should freeze validation state"
        );
        assert_eq!(
            node.props.get("disabled").and_then(|value| value.as_bool()),
            Some(disabled),
            "Checkbox child `{node_id}` should freeze disabled state"
        );
        assert_eq!(
            node.props.get("text_tone").and_then(|value| value.as_str()),
            Some(text_tone),
            "Checkbox child `{node_id}` should freeze label tone"
        );
        assert_eq!(
            node.props
                .get("label_click_toggles")
                .and_then(|value| value.as_bool()),
            Some(true),
            "Checkbox child `{node_id}` should document label-click ownership"
        );
        assert_eq!(
            node.props
                .get("indeterminate_resolves_to_checked")
                .and_then(|value| value.as_bool()),
            Some(true),
            "Checkbox child `{node_id}` should document indeterminate click policy"
        );
        assert!(
            node.events.is_empty(),
            "Checkbox child examples should not add extra representative feedback routes"
        );
        assert_non_dispatchable_child(node, node_id, "Checkbox");
    }
}
