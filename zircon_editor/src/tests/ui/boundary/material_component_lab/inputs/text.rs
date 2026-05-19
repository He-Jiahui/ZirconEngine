use std::fs;

use zircon_runtime::ui::v2::UiZuiAssetLoader;

use super::super::support::{child_nodes, editor_asset};
use super::assert_non_dispatchable_child;

#[test]
fn material_text_field_sample_covers_variants_helper_error_and_disabled_states() {
    let path = editor_asset("assets/ui/editor/material_components/material_text_fields.zui");
    let source = fs::read_to_string(&path)
        .unwrap_or_else(|error| panic!("{} should be readable: {error}", path.display()));
    let document = UiZuiAssetLoader::load_zui_str(&source)
        .unwrap_or_else(|error| panic!("{} should load as .zui: {error}", path.display()));
    let sample = document
        .nodes
        .get("sample")
        .unwrap_or_else(|| panic!("{} should define the TextField sample", path.display()));

    assert_eq!(sample.component.as_str(), "HorizontalBox");
    assert!(sample
        .classes
        .iter()
        .any(|class| class == "material-lab-sample"));
    assert_eq!(sample.events.len(), 1);
    assert_eq!(sample.events[0].id, "MaterialLab/TextFields/Change");
    assert_eq!(
        child_nodes(&document, "sample"),
        vec![
            "field_outlined_focus",
            "field_filled_helper",
            "field_standard",
            "field_error",
            "field_disabled",
        ],
        "TextField sample should show the planned variant, helper, error, and disabled examples"
    );

    for (node_id, component, variant, focused, validation_level, disabled) in [
        (
            "field_outlined_focus",
            "TextField",
            "outlined",
            true,
            "normal",
            false,
        ),
        (
            "field_filled_helper",
            "TextField",
            "filled",
            false,
            "normal",
            false,
        ),
        (
            "field_standard",
            "InputField",
            "standard",
            false,
            "normal",
            false,
        ),
        (
            "field_error",
            "TextField",
            "outlined",
            false,
            "error",
            false,
        ),
        (
            "field_disabled",
            "TextField",
            "outlined",
            false,
            "normal",
            true,
        ),
    ] {
        let node = document
            .nodes
            .get(node_id)
            .unwrap_or_else(|| panic!("{} should define `{node_id}`", path.display()));
        assert_eq!(node.component.as_str(), component);
        assert_eq!(
            node.props.get("variant").and_then(|value| value.as_str()),
            Some(variant),
            "TextField child `{node_id}` should freeze variant metadata"
        );
        assert_eq!(
            node.props.get("focused").and_then(|value| value.as_bool()),
            Some(focused),
            "TextField child `{node_id}` should freeze focus state"
        );
        assert_eq!(
            node.props
                .get("validation_level")
                .and_then(|value| value.as_str()),
            Some(validation_level),
            "TextField child `{node_id}` should freeze validation state"
        );
        assert_eq!(
            node.props.get("disabled").and_then(|value| value.as_bool()),
            Some(disabled),
            "TextField child `{node_id}` should freeze disabled state"
        );
        assert!(
            node.props
                .get("label")
                .and_then(|value| value.as_str())
                .is_some_and(|label| !label.is_empty()),
            "TextField child `{node_id}` should expose label metadata"
        );
        assert!(
            node.props
                .get("helper_text")
                .and_then(|value| value.as_str())
                .is_some_and(|helper| !helper.is_empty()),
            "TextField child `{node_id}` should expose helper text metadata"
        );
        assert!(
            node.events.is_empty(),
            "TextField child examples should not add extra representative feedback routes"
        );
        assert_non_dispatchable_child(node, node_id, "TextField");
    }
}

#[test]
fn material_textarea_autosize_sample_covers_min_max_autosize_error_and_disabled_states() {
    let path = editor_asset("assets/ui/editor/material_components/material_textarea_autosize.zui");
    let source = fs::read_to_string(&path)
        .unwrap_or_else(|error| panic!("{} should be readable: {error}", path.display()));
    let document = UiZuiAssetLoader::load_zui_str(&source)
        .unwrap_or_else(|error| panic!("{} should load as .zui: {error}", path.display()));
    let sample = document.nodes.get("sample").unwrap_or_else(|| {
        panic!(
            "{} should define the TextareaAutosize sample",
            path.display()
        )
    });

    assert_eq!(sample.component.as_str(), "HorizontalBox");
    assert!(sample
        .classes
        .iter()
        .any(|class| class == "material-lab-sample"));
    assert_eq!(sample.events.len(), 1);
    assert_eq!(sample.events[0].id, "MaterialLab/TextareaAutosize/Change");
    assert_eq!(
        child_nodes(&document, "sample"),
        vec![
            "textarea_min_rows",
            "textarea_max_rows",
            "textarea_autosize_focus",
            "textarea_error",
            "textarea_disabled",
        ],
        "TextareaAutosize sample should show min rows, max rows, autosize, error, and disabled examples"
    );

    for (node_id, min_rows, max_rows, autosize, focused, validation_level, disabled) in [
        ("textarea_min_rows", 2, 4, true, false, "normal", false),
        ("textarea_max_rows", 2, 4, true, false, "normal", false),
        ("textarea_autosize_focus", 1, 5, true, true, "normal", false),
        ("textarea_error", 2, 4, true, false, "error", false),
        ("textarea_disabled", 2, 4, true, false, "normal", true),
    ] {
        let node = document
            .nodes
            .get(node_id)
            .unwrap_or_else(|| panic!("{} should define `{node_id}`", path.display()));
        assert_eq!(node.component.as_str(), "TextareaAutosize");
        assert_eq!(
            node.props
                .get("min_rows")
                .and_then(|value| value.as_integer()),
            Some(min_rows),
            "TextareaAutosize child `{node_id}` should freeze min_rows metadata"
        );
        assert_eq!(
            node.props
                .get("max_rows")
                .and_then(|value| value.as_integer()),
            Some(max_rows),
            "TextareaAutosize child `{node_id}` should freeze max_rows metadata"
        );
        assert_eq!(
            node.props.get("autosize").and_then(|value| value.as_bool()),
            Some(autosize),
            "TextareaAutosize child `{node_id}` should freeze autosize metadata"
        );
        assert_eq!(
            node.props
                .get("multiline")
                .and_then(|value| value.as_bool()),
            Some(true),
            "TextareaAutosize child `{node_id}` should remain multiline"
        );
        assert_eq!(
            node.props.get("focused").and_then(|value| value.as_bool()),
            Some(focused),
            "TextareaAutosize child `{node_id}` should freeze focus state"
        );
        assert_eq!(
            node.props
                .get("validation_level")
                .and_then(|value| value.as_str()),
            Some(validation_level),
            "TextareaAutosize child `{node_id}` should freeze validation state"
        );
        assert_eq!(
            node.props.get("disabled").and_then(|value| value.as_bool()),
            Some(disabled),
            "TextareaAutosize child `{node_id}` should freeze disabled state"
        );
        assert!(
            node.props
                .get("helper_text")
                .and_then(|value| value.as_str())
                .is_some_and(|helper| !helper.is_empty()),
            "TextareaAutosize child `{node_id}` should expose row/helper metadata"
        );
        assert!(
            node.events.is_empty(),
            "TextareaAutosize child examples should not add extra representative feedback routes"
        );
        assert_non_dispatchable_child(node, node_id, "TextareaAutosize");
    }
}
