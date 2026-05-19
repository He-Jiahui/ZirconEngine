use std::fs;

use zircon_runtime::ui::v2::UiZuiAssetLoader;

use super::super::support::{child_nodes, editor_asset, numeric_prop};
use super::assert_non_dispatchable_child;

#[test]
fn material_number_field_sample_covers_step_drag_error_and_disabled_states() {
    let path = editor_asset("assets/ui/editor/material_components/material_number_field.zui");
    let source = fs::read_to_string(&path)
        .unwrap_or_else(|error| panic!("{} should be readable: {error}", path.display()));
    let document = UiZuiAssetLoader::load_zui_str(&source)
        .unwrap_or_else(|error| panic!("{} should load as .zui: {error}", path.display()));
    let sample = document
        .nodes
        .get("sample")
        .unwrap_or_else(|| panic!("{} should define the NumberField sample", path.display()));

    assert_eq!(sample.component.as_str(), "HorizontalBox");
    assert!(sample
        .classes
        .iter()
        .any(|class| class == "material-lab-sample"));
    assert_eq!(sample.events.len(), 1);
    assert_eq!(sample.events[0].id, "MaterialLab/NumberField/Change");
    assert_eq!(
        child_nodes(&document, "sample"),
        vec![
            "number_stepper",
            "number_clamped",
            "number_drag_active",
            "number_error",
            "number_disabled",
        ],
        "NumberField sample should show stepper, clamp, drag-active, error, and disabled examples"
    );

    for (node_id, value, min, max, step, large_step, dragging, validation_level, disabled) in [
        (
            "number_stepper",
            42.0,
            0.0,
            100.0,
            1.0,
            10.0,
            false,
            "normal",
            false,
        ),
        (
            "number_clamped",
            100.0,
            0.0,
            100.0,
            5.0,
            25.0,
            false,
            "normal",
            false,
        ),
        (
            "number_drag_active",
            64.0,
            0.0,
            100.0,
            0.5,
            5.0,
            true,
            "normal",
            false,
        ),
        (
            "number_error",
            8.0,
            0.0,
            100.0,
            1.0,
            10.0,
            false,
            "error",
            false,
        ),
        (
            "number_disabled",
            12.0,
            0.0,
            100.0,
            1.0,
            10.0,
            false,
            "normal",
            true,
        ),
    ] {
        let node = document
            .nodes
            .get(node_id)
            .unwrap_or_else(|| panic!("{} should define `{node_id}`", path.display()));
        assert_eq!(node.component.as_str(), "NumberField");
        for (prop, expected) in [
            ("value", value),
            ("min", min),
            ("max", max),
            ("step", step),
            ("large_step", large_step),
        ] {
            assert_eq!(
                numeric_prop(node.props.get(prop)),
                Some(expected),
                "NumberField child `{node_id}` should freeze numeric `{prop}` metadata"
            );
        }
        assert_eq!(
            node.props.get("dragging").and_then(|value| value.as_bool()),
            Some(dragging),
            "NumberField child `{node_id}` should freeze drag-active state"
        );
        assert_eq!(
            node.props
                .get("validation_level")
                .and_then(|value| value.as_str()),
            Some(validation_level),
            "NumberField child `{node_id}` should freeze validation state"
        );
        assert_eq!(
            node.props.get("disabled").and_then(|value| value.as_bool()),
            Some(disabled),
            "NumberField child `{node_id}` should freeze disabled state"
        );
        assert!(
            node.props
                .get("value_text")
                .and_then(|value| value.as_str())
                .is_none(),
            "NumberField child `{node_id}` should rely on numeric value projection instead of string-only state"
        );
        assert!(
            node.events.is_empty(),
            "NumberField child examples should not add extra representative feedback routes"
        );
        assert_non_dispatchable_child(node, node_id, "NumberField");
    }
}
