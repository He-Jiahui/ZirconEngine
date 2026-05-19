use std::fs;

use zircon_runtime::ui::v2::UiZuiAssetLoader;

use super::super::support::{child_nodes, editor_asset};
use super::{assert_non_dispatchable_child, string_array_prop};

#[test]
fn material_radio_group_sample_covers_exclusive_disabled_and_error_states() {
    let path = editor_asset("assets/ui/editor/material_components/material_radio_buttons.zui");
    let source = fs::read_to_string(&path)
        .unwrap_or_else(|error| panic!("{} should be readable: {error}", path.display()));
    let document = UiZuiAssetLoader::load_zui_str(&source)
        .unwrap_or_else(|error| panic!("{} should load as .zui: {error}", path.display()));
    let sample = document
        .nodes
        .get("sample")
        .unwrap_or_else(|| panic!("{} should define the Radio sample", path.display()));

    assert_eq!(sample.component.as_str(), "HorizontalBox");
    assert!(sample
        .classes
        .iter()
        .any(|class| class == "material-lab-sample"));
    assert_eq!(sample.events.len(), 1);
    assert_eq!(sample.events[0].id, "MaterialLab/RadioButtons/Change");
    assert_eq!(
        child_nodes(&document, "sample"),
        vec![
            "radio_selected",
            "radio_unselected",
            "radio_disabled",
            "radio_error",
        ],
        "Radio sample should show selected, unselected, disabled, and error examples"
    );

    for (node_id, option_id, checked, selected, validation_level, disabled, text_tone) in [
        (
            "radio_selected",
            "editor",
            true,
            true,
            "normal",
            false,
            "primary",
        ),
        (
            "radio_unselected",
            "runtime",
            false,
            false,
            "normal",
            false,
            "secondary",
        ),
        (
            "radio_disabled",
            "disabled",
            false,
            false,
            "normal",
            true,
            "disabled",
        ),
        ("radio_error", "qa", false, false, "error", false, "error"),
    ] {
        let node = document
            .nodes
            .get(node_id)
            .unwrap_or_else(|| panic!("{} should define `{node_id}`", path.display()));
        assert_eq!(node.component.as_str(), "Radio");
        assert_eq!(
            node.props
                .get("group_value")
                .and_then(|value| value.as_str()),
            Some("editor"),
            "Radio child `{node_id}` should freeze the group-selected value"
        );
        assert_eq!(
            node.props.get("option_id").and_then(|value| value.as_str()),
            Some(option_id),
            "Radio child `{node_id}` should freeze option identity"
        );
        assert_eq!(
            node.props.get("checked").and_then(|value| value.as_bool()),
            Some(checked),
            "Radio child `{node_id}` should freeze checked state"
        );
        assert_eq!(
            node.props.get("selected").and_then(|value| value.as_bool()),
            Some(selected),
            "Radio child `{node_id}` should freeze selected paint state"
        );
        assert_eq!(
            node.props
                .get("validation_level")
                .and_then(|value| value.as_str()),
            Some(validation_level),
            "Radio child `{node_id}` should freeze validation state"
        );
        assert_eq!(
            node.props.get("disabled").and_then(|value| value.as_bool()),
            Some(disabled),
            "Radio child `{node_id}` should freeze disabled state"
        );
        assert_eq!(
            node.props.get("text_tone").and_then(|value| value.as_str()),
            Some(text_tone),
            "Radio child `{node_id}` should freeze label tone"
        );
        assert_eq!(
            string_array_prop(node.props.get("options")),
            vec!["editor", "runtime", "disabled", "qa"],
            "Radio child `{node_id}` should freeze the group option ids"
        );
        assert_eq!(
            string_array_prop(node.props.get("disabled_options")),
            vec!["disabled"],
            "Radio child `{node_id}` should freeze disabled option ids"
        );
        for prop in [
            "label_click_selects",
            "exclusive_group",
            "keyboard_navigation",
        ] {
            assert_eq!(
                node.props.get(prop).and_then(|value| value.as_bool()),
                Some(true),
                "Radio child `{node_id}` should document `{prop}` ownership"
            );
        }
        assert!(
            node.events.is_empty(),
            "Radio child examples should not add extra representative feedback routes"
        );
        assert_non_dispatchable_child(node, node_id, "Radio");
    }
}
