use std::fs;

use zircon_runtime::ui::v2::UiZuiAssetLoader;

use super::super::support::{child_nodes, editor_asset};
use super::assert_non_dispatchable_child;

#[test]
fn material_switch_sample_covers_on_off_small_disabled_and_error_states() {
    let path = editor_asset("assets/ui/editor/material_components/material_switches.zui");
    let source = fs::read_to_string(&path)
        .unwrap_or_else(|error| panic!("{} should be readable: {error}", path.display()));
    let document = UiZuiAssetLoader::load_zui_str(&source)
        .unwrap_or_else(|error| panic!("{} should load as .zui: {error}", path.display()));
    let sample = document
        .nodes
        .get("sample")
        .unwrap_or_else(|| panic!("{} should define the Switch sample", path.display()));

    assert_eq!(sample.component.as_str(), "HorizontalBox");
    assert!(sample
        .classes
        .iter()
        .any(|class| class == "material-lab-sample"));
    assert_eq!(sample.events.len(), 1);
    assert_eq!(sample.events[0].id, "MaterialLab/Switches/Toggle");
    assert_eq!(
        child_nodes(&document, "sample"),
        vec![
            "switch_on",
            "switch_off",
            "switch_small",
            "switch_disabled",
            "switch_error",
        ],
        "Switch sample should show on, off, small, disabled, and error examples"
    );

    for (
        node_id,
        checked,
        switch_size,
        switch_color,
        selected,
        disabled,
        validation_level,
        text_tone,
    ) in [
        (
            "switch_on",
            true,
            "medium",
            "primary",
            true,
            false,
            "normal",
            "primary",
        ),
        (
            "switch_off",
            false,
            "medium",
            "default",
            false,
            false,
            "normal",
            "secondary",
        ),
        (
            "switch_small",
            true,
            "small",
            "primary",
            true,
            false,
            "normal",
            "primary",
        ),
        (
            "switch_disabled",
            true,
            "medium",
            "primary",
            false,
            true,
            "normal",
            "disabled",
        ),
        (
            "switch_error",
            false,
            "medium",
            "error",
            false,
            false,
            "error",
            "error",
        ),
    ] {
        let node = document
            .nodes
            .get(node_id)
            .unwrap_or_else(|| panic!("{} should define `{node_id}`", path.display()));
        assert_eq!(node.component.as_str(), "Switch");
        assert_eq!(
            node.props.get("checked").and_then(|value| value.as_bool()),
            Some(checked),
            "Switch child `{node_id}` should freeze checked state"
        );
        assert_eq!(
            node.props
                .get("switch_size")
                .and_then(|value| value.as_str()),
            Some(switch_size),
            "Switch child `{node_id}` should freeze size metadata"
        );
        assert_eq!(
            node.props
                .get("switch_color")
                .and_then(|value| value.as_str()),
            Some(switch_color),
            "Switch child `{node_id}` should freeze color metadata"
        );
        assert_eq!(
            node.props.get("selected").and_then(|value| value.as_bool()),
            Some(selected),
            "Switch child `{node_id}` should freeze selected paint state"
        );
        assert_eq!(
            node.props.get("disabled").and_then(|value| value.as_bool()),
            Some(disabled),
            "Switch child `{node_id}` should freeze disabled state"
        );
        assert_eq!(
            node.props
                .get("validation_level")
                .and_then(|value| value.as_str()),
            Some(validation_level),
            "Switch child `{node_id}` should freeze validation state"
        );
        assert_eq!(
            node.props.get("text_tone").and_then(|value| value.as_str()),
            Some(text_tone),
            "Switch child `{node_id}` should freeze label tone"
        );
        for prop in [
            "label_click_toggles",
            "track_click_toggles",
            "thumb_draggable",
        ] {
            let expected = prop != "thumb_draggable";
            assert_eq!(
                node.props.get(prop).and_then(|value| value.as_bool()),
                Some(expected),
                "Switch child `{node_id}` should document `{prop}` ownership"
            );
        }
        assert!(
            node.events.is_empty(),
            "Switch child examples should not add extra representative feedback routes"
        );
        assert_non_dispatchable_child(node, node_id, "Switch");
    }
}
