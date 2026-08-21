use zircon_runtime::ui::v2::UiV2AssetLoader;

use super::{
    UI_ASSET_EDITOR_ACTION_BAR_V2_TOML, UI_ASSET_EDITOR_PROJECTION_V2_TOML,
    WORKBENCH_CAPTION_V2_TOML, WORKBENCH_LABEL_V2_TOML,
};

const WORKBENCH_CHIP_V2_TOML: &str = include_str!(concat!(
    env!("CARGO_MANIFEST_DIR"),
    "/assets/ui/editor/components/workbench/primitives/chrome/workbench_chip.zui"
));
const WORKBENCH_AXIS_VALUE_FIELD_V2_TOML: &str = include_str!(concat!(
    env!("CARGO_MANIFEST_DIR"),
    "/assets/ui/editor/components/workbench/primitives/chrome/workbench_axis_value_field.zui"
));
const WORKBENCH_INSPECTOR_PANEL_V2_TOML: &str = include_str!(concat!(
    env!("CARGO_MANIFEST_DIR"),
    "/assets/ui/editor/components/workbench/shell/workbench_inspector_panel.zui"
));

#[test]
fn workbench_text_primitives_use_runtime_density_height_tokens() {
    for (source, primitive_name, expected_tokens) in [
        (
            WORKBENCH_CAPTION_V2_TOML,
            "WorkbenchCaption",
            [
                "$editor.density.caption.min_height",
                "$editor.density.caption.preferred_height",
                "$editor.density.caption.max_height",
            ],
        ),
        (
            WORKBENCH_LABEL_V2_TOML,
            "WorkbenchLabel",
            [
                "$editor.density.label.min_height",
                "$editor.density.label.preferred_height",
                "$editor.density.label.max_height",
            ],
        ),
    ] {
        let source: toml::Value =
            toml::from_str(source).expect("text primitive source should stay valid TOML");
        let height = source
            .get("nodes")
            .and_then(toml::Value::as_table)
            .and_then(|nodes| nodes.get("root"))
            .and_then(toml::Value::as_table)
            .and_then(|root| root.get("layout"))
            .and_then(toml::Value::as_table)
            .and_then(|layout| layout.get("height"))
            .and_then(toml::Value::as_table)
            .unwrap_or_else(|| panic!("{primitive_name} must declare tokenized text height"));

        for (field, expected_token) in ["min", "preferred", "max"].into_iter().zip(expected_tokens)
        {
            assert_eq!(
                height.get(field).and_then(toml::Value::as_str),
                Some(expected_token),
                "{primitive_name} must resolve its {field} height through the Runtime density cascade"
            );
        }
    }
}

#[test]
fn workbench_chip_uses_runtime_density_width_tokens() {
    let source: toml::Value = toml::from_str(WORKBENCH_CHIP_V2_TOML)
        .expect("chip primitive source should stay valid TOML");
    let width = source
        .get("nodes")
        .and_then(toml::Value::as_table)
        .and_then(|nodes| nodes.get("root"))
        .and_then(toml::Value::as_table)
        .and_then(|root| root.get("layout"))
        .and_then(toml::Value::as_table)
        .and_then(|layout| layout.get("width"))
        .and_then(toml::Value::as_table)
        .expect("WorkbenchChip must declare a responsive width");

    for (field, expected_token) in ["min", "preferred", "max"].into_iter().zip([
        "$editor.density.chip.min_width",
        "$editor.density.chip.preferred_width",
        "$editor.density.chip.max_width",
    ]) {
        assert_eq!(
            width.get(field).and_then(toml::Value::as_str),
            Some(expected_token),
            "WorkbenchChip must resolve its {field} width through the density cascade"
        );
    }
    let props = source
        .get("nodes")
        .and_then(toml::Value::as_table)
        .and_then(|nodes| nodes.get("root"))
        .and_then(toml::Value::as_table)
        .and_then(|root| root.get("props"))
        .and_then(toml::Value::as_table)
        .expect("WorkbenchChip must retain its text properties");
    assert_eq!(
        props.get("text_overflow").and_then(toml::Value::as_str),
        Some("ellipsis"),
        "a bounded WorkbenchChip must ellipsize dynamic status text instead of clipping it"
    );
}

#[test]
fn workbench_axis_value_field_uses_runtime_density_width_tokens() {
    let source: toml::Value = toml::from_str(WORKBENCH_AXIS_VALUE_FIELD_V2_TOML)
        .expect("axis value field primitive source should stay valid TOML");
    let width = source
        .get("nodes")
        .and_then(toml::Value::as_table)
        .and_then(|nodes| nodes.get("root"))
        .and_then(toml::Value::as_table)
        .and_then(|root| root.get("layout"))
        .and_then(toml::Value::as_table)
        .and_then(|layout| layout.get("width"))
        .and_then(toml::Value::as_table)
        .expect("WorkbenchAxisValueField must declare a responsive width");

    for (field, expected_token) in ["min", "preferred", "max"].into_iter().zip([
        "$editor.density.axis_value_field.min_width",
        "$editor.density.axis_value_field.preferred_width",
        "$editor.density.axis_value_field.max_width",
    ]) {
        assert_eq!(
            width.get(field).and_then(toml::Value::as_str),
            Some(expected_token),
            "WorkbenchAxisValueField must resolve its {field} width through the density cascade"
        );
    }
}

#[test]
fn inspector_axis_fields_inherit_runtime_density_width_tokens() {
    let source: toml::Value = toml::from_str(WORKBENCH_INSPECTOR_PANEL_V2_TOML)
        .expect("inspector panel source should stay valid TOML");
    let nodes = source
        .get("nodes")
        .and_then(toml::Value::as_table)
        .expect("inspector panel must retain node definitions");

    for node_id in [
        "position_value_x",
        "position_value_y",
        "position_value_z",
        "rotation_value_x",
        "rotation_value_y",
        "rotation_value_z",
        "scale_value_x",
        "scale_value_y",
        "scale_value_z",
    ] {
        let node = nodes
            .get(node_id)
            .and_then(toml::Value::as_table)
            .unwrap_or_else(|| panic!("missing inspector axis field `{node_id}`"));
        assert_eq!(
            node.get("component").and_then(toml::Value::as_str),
            Some("WorkbenchAxisValueField"),
            "{node_id} must reuse the standard axis input primitive"
        );
        let width = node
            .get("layout")
            .and_then(toml::Value::as_table)
            .and_then(|layout| layout.get("width"))
            .and_then(toml::Value::as_table)
            .unwrap_or_else(|| panic!("{node_id} must declare a responsive width"));

        for (field, expected_token) in ["min", "preferred", "max"].into_iter().zip([
            "$editor.density.axis_value_field.min_width",
            "$editor.density.axis_value_field.preferred_width",
            "$editor.density.axis_value_field.max_width",
        ]) {
            assert_eq!(
                width.get(field).and_then(toml::Value::as_str),
                Some(expected_token),
                "{node_id} must resolve its {field} width through the shared axis-field token"
            );
        }
    }
}

#[test]
fn ui_asset_editor_v2_projection_keeps_header_text_as_renderable_primitives() {
    let layout = UiV2AssetLoader::load_toml_str(UI_ASSET_EDITOR_PROJECTION_V2_TOML)
        .expect("ui asset editor v2 projection asset");
    let source: toml::Value = toml::from_str(UI_ASSET_EDITOR_PROJECTION_V2_TOML)
        .expect("ui asset editor projection source should stay valid TOML");

    for (node_id, component, expected_text) in [
        ("header_asset_title", "WorkbenchLabel", "UI Asset"),
        ("header_asset_kind", "WorkbenchCaption", "DESIGNER"),
        (
            "header_status_caption",
            "WorkbenchCaption",
            "Prototype cache valid",
        ),
    ] {
        let node = layout
            .nodes
            .get(node_id)
            .unwrap_or_else(|| panic!("missing header text node `{node_id}`"));
        assert_eq!(node.component, component);
        assert_eq!(
            node.props.get("text").and_then(toml::Value::as_str),
            Some(expected_text),
            "{node_id} must own its visible header text"
        );
        assert_eq!(
            node.props
                .get("text_overflow")
                .and_then(toml::Value::as_str),
            Some("ellipsis"),
            "{node_id} must ellipsize rather than overflow a narrow header"
        );
    }

    for container_id in ["header_asset_row", "header_status_row", "header_action_row"] {
        let node = layout
            .nodes
            .get(container_id)
            .unwrap_or_else(|| panic!("missing header container `{container_id}`"));
        assert!(
            !node.props.contains_key("text"),
            "{container_id} must compose renderable child primitives instead of storing text"
        );
    }

    let header_layout = source
        .get("nodes")
        .and_then(toml::Value::as_table)
        .and_then(|nodes| nodes.get("header_panel"))
        .and_then(toml::Value::as_table)
        .and_then(|header| header.get("layout"))
        .and_then(toml::Value::as_table)
        .expect("header panel should retain a layout table");
    assert!(
        !header_layout.contains_key("height"),
        "the header must derive its height from tokenized child rows instead of a duplicated aggregate metric"
    );
}

#[test]
fn ui_asset_editor_v2_action_bar_composes_standard_actions_from_primitives() {
    let layout = UiV2AssetLoader::load_toml_str(UI_ASSET_EDITOR_ACTION_BAR_V2_TOML)
        .expect("ui asset editor action bar component");

    assert_eq!(
        layout.asset.id,
        "res://ui/editor/components/workbench/composites/chrome/workbench_ui_asset_action_bar.zui"
    );
    assert!(layout.components.contains_key("WorkbenchUiAssetActionBar"));
    for (node_id, component) in [
        ("insert_caption", "WorkbenchCaption"),
        ("reparent_caption", "WorkbenchCaption"),
        ("structure_caption", "WorkbenchCaption"),
        ("insert_child_button", "WorkbenchButton"),
        ("insert_after_button", "WorkbenchButton"),
        ("reparent_previous_button", "WorkbenchButton"),
        ("reparent_next_button", "WorkbenchButton"),
        ("reparent_outdent_button", "WorkbenchButton"),
        ("structure_up_button", "WorkbenchButton"),
        ("structure_down_button", "WorkbenchButton"),
        ("structure_wrap_button", "WorkbenchButton"),
    ] {
        assert_eq!(
            layout
                .nodes
                .get(node_id)
                .unwrap_or_else(|| panic!("missing action-bar node `{node_id}`"))
                .component,
            component
        );
    }
}

#[test]
fn ui_asset_editor_v2_action_bar_uses_shared_responsive_action_width_tokens() {
    let source: toml::Value = toml::from_str(UI_ASSET_EDITOR_ACTION_BAR_V2_TOML)
        .expect("ui asset editor action bar source should stay valid TOML");
    let nodes = source
        .get("nodes")
        .and_then(toml::Value::as_table)
        .expect("ui asset action bar should retain node definitions");

    for node_id in [
        "insert_child_button",
        "insert_after_button",
        "reparent_previous_button",
        "reparent_next_button",
        "reparent_outdent_button",
        "structure_up_button",
        "structure_down_button",
        "structure_wrap_button",
    ] {
        let width = nodes
            .get(node_id)
            .and_then(toml::Value::as_table)
            .and_then(|node| node.get("layout"))
            .and_then(toml::Value::as_table)
            .and_then(|layout| layout.get("width"))
            .and_then(toml::Value::as_table)
            .unwrap_or_else(|| panic!("{node_id} must declare its responsive width"));
        assert_eq!(
            width.get("min").and_then(toml::Value::as_str),
            Some("$editor.density.ui_asset_action.min_width"),
            "{node_id} must share the compact action minimum width"
        );
        assert_eq!(
            width.get("preferred").and_then(toml::Value::as_str),
            Some("$editor.density.ui_asset_action.preferred_width"),
            "{node_id} must share the compact action preferred width"
        );
        assert_eq!(
            width.get("max").and_then(toml::Value::as_str),
            Some("$editor.density.ui_asset_action.max_width"),
            "{node_id} must share the compact action maximum width"
        );
    }
}

#[test]
fn ui_asset_editor_v2_palette_samples_do_not_advertise_unbound_actions() {
    let layout = UiV2AssetLoader::load_toml_str(UI_ASSET_EDITOR_PROJECTION_V2_TOML)
        .expect("ui asset editor v2 projection asset");

    for node_id in [
        "palette_button",
        "palette_label",
        "palette_image",
        "palette_container",
    ] {
        let node = layout
            .nodes
            .get(node_id)
            .unwrap_or_else(|| panic!("missing palette sample node `{node_id}`"));
        assert_eq!(node.component, "WorkbenchButton");
        assert_eq!(
            node.props.get("disabled").and_then(toml::Value::as_bool),
            Some(true),
            "{node_id} must stay disabled until the dynamic palette projection owns its insert route"
        );
        assert!(
            node.events.is_empty(),
            "{node_id} must not publish an event without a real session command binding"
        );
    }
}

#[test]
fn ui_asset_editor_v2_designer_canvas_clips_real_preview_projection() {
    let source: toml::Value = toml::from_str(UI_ASSET_EDITOR_PROJECTION_V2_TOML)
        .expect("ui asset editor projection source should stay valid TOML");
    let canvas = source
        .get("nodes")
        .and_then(toml::Value::as_table)
        .and_then(|nodes| nodes.get("designer_canvas_panel"))
        .and_then(toml::Value::as_table)
        .expect("designer canvas node should remain declared");
    let layout = canvas
        .get("layout")
        .and_then(toml::Value::as_table)
        .expect("designer canvas should retain a layout table");

    assert_eq!(
        canvas.get("component").and_then(toml::Value::as_str),
        Some("CanvasBox")
    );
    assert_eq!(
        layout.get("clip").and_then(toml::Value::as_bool),
        Some(true),
        "the real preview projection must be clipped to the canvas frame"
    );
    assert!(
        !canvas.contains_key("children"),
        "the designer canvas must remain a projection host, not a static preview fixture"
    );
}
