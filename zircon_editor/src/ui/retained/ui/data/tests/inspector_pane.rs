use super::*;
use crate::ui::layouts::common::model_rc;
use crate::ui::layouts::views::blank_viewport_chrome;
use crate::ui::layouts::windows::workbench_host_window::{
    InspectorPaneViewData, InspectorPluginComponentPropertyViewData,
    InspectorPluginComponentViewData, PaneContentSize, PaneData, PaneNativeBodyData,
};
use crate::ui::retained_host as host_contract;
use crate::ui::retained_host::primitives::ModelRc;

#[test]
fn inspector_pane_projects_editable_field_nodes_and_actions() {
    let pane = inspector_pane_fixture("scene entity selected");
    let data =
        to_host_contract_inspector_pane_from_host_pane(&pane, PaneContentSize::new(360.0, 240.0));

    let panel = find_node(&data.nodes, "InspectorEditableFieldsPanel");
    assert!(
        panel.text.is_empty(),
        "the inspector fields surface is a container and must not paint a centered title"
    );

    let name = find_node(&data.nodes, "NameField");
    assert_eq!(name.role.as_str(), "InputField");
    assert_eq!(name.value_text.as_str(), "Camera");
    assert_eq!(name.edit_action_id.as_str(), "inspector.field.name.edit");
    assert_eq!(
        name.commit_action_id.as_str(),
        "inspector.apply_batch.commit"
    );
    assert_eq!(name.surface_variant.as_str(), "inspector-field");
    assert_eq!(name.text_tone.as_str(), "default");
    assert!(!name.disabled);

    let position_x = find_node(&data.nodes, "PositionXField");
    assert_eq!(position_x.role.as_str(), "NumberField");
    assert_eq!(position_x.value_text.as_str(), "1.25");
    assert_eq!(
        position_x.edit_action_id.as_str(),
        "inspector.transform.position_x.edit"
    );
    assert_eq!(position_x.surface_variant.as_str(), "inspector-field");
    assert_eq!(position_x.text_tone.as_str(), "default");

    let transform = find_node(&data.nodes, "InspectorTransformLabel");
    assert!(
        transform.frame.y + transform.frame.height <= position_x.frame.y,
        "the transform label must not overlap the vector fields"
    );

    let apply = find_node(&data.nodes, "ApplyBatchButton");
    assert_eq!(apply.role.as_str(), "Button");
    assert_eq!(apply.action_id.as_str(), "inspector.apply_batch.invoke");
    assert_eq!(apply.surface_variant.as_str(), "panel");
    assert_eq!(apply.text_tone.as_str(), "default");
    assert!(apply.selected);
    assert!(!apply.disabled);

    let delete = find_node(&data.nodes, "DeleteSelected");
    assert_eq!(
        delete.action_id.as_str(),
        "workbench.selection.delete_selected"
    );
    assert_eq!(delete.surface_variant.as_str(), "panel");
    assert_eq!(delete.text_tone.as_str(), "default");
    assert!(delete.selected);
    assert!(!delete.disabled);
}

#[test]
fn inspector_pane_marks_plugin_inspector_customization_fallback() {
    let pane = inspector_pane_fixture(
        "plugin inspector customization unavailable: particles plugin unloaded",
    );
    let data =
        to_host_contract_inspector_pane_from_host_pane(&pane, PaneContentSize::new(360.0, 240.0));

    let fallback = find_node(&data.nodes, "InspectorPluginComponentFallback");
    assert_eq!(fallback.role.as_str(), "Diagnostic");
    assert_eq!(fallback.surface_variant.as_str(), "inset");
    assert_eq!(fallback.text_tone.as_str(), "warning");
    assert_eq!(fallback.validation_level.as_str(), "warning");
    assert!(fallback
        .validation_message
        .as_str()
        .contains("serialized component data stays protected"));
    assert!(fallback.disabled);
}

#[test]
fn inspector_pane_disables_fields_and_actions_when_selection_is_empty() {
    let mut pane = inspector_pane_fixture("no selection");
    pane.native_body.inspector.delete_enabled = false;
    pane.native_body.inspector.inspector_name = "".into();
    pane.native_body.inspector.inspector_parent = "".into();
    pane.native_body.inspector.inspector_x = "".into();
    pane.native_body.inspector.inspector_y = "".into();
    pane.native_body.inspector.inspector_z = "".into();

    let data =
        to_host_contract_inspector_pane_from_host_pane(&pane, PaneContentSize::new(360.0, 240.0));

    let panel = find_node(&data.nodes, "InspectorEditableFieldsPanel");
    assert_eq!(panel.surface_variant.as_str(), "inset");
    assert_eq!(panel.text_tone.as_str(), "muted");
    assert!(!panel.selected);

    let name = find_node(&data.nodes, "NameField");
    assert!(name.disabled);
    assert_eq!(name.surface_variant.as_str(), "inset");
    assert_eq!(name.text_tone.as_str(), "muted");

    let position_x = find_node(&data.nodes, "PositionXField");
    assert!(position_x.disabled);
    assert_eq!(position_x.surface_variant.as_str(), "inset");
    assert_eq!(position_x.text_tone.as_str(), "muted");

    let apply = find_node(&data.nodes, "ApplyBatchButton");
    assert!(apply.disabled);
    assert_eq!(apply.surface_variant.as_str(), "inset");
    assert_eq!(apply.text_tone.as_str(), "muted");
    assert!(!apply.selected);

    let delete = find_node(&data.nodes, "DeleteSelected");
    assert!(delete.disabled);
    assert_eq!(delete.surface_variant.as_str(), "inset");
    assert_eq!(delete.text_tone.as_str(), "muted");
    assert!(!delete.selected);

    let empty = find_node(&data.nodes, "InspectorEmptySelectionHint");
    assert_eq!(empty.text.as_str(), "No scene entity selected");
    assert_eq!(empty.text_tone.as_str(), "muted");
}

#[test]
fn inspector_pane_projects_plugin_inspector_customization_fields_and_unload_degradation() {
    let mut pane = inspector_pane_fixture("scene entity selected");
    pane.native_body.inspector.plugin_components = vec![
        InspectorPluginComponentViewData {
            component_id: "weather.Component.CloudLayer".to_string(),
            display_name: "Cloud Layer".to_string(),
            customization_available: true,
            customization_ui_document: None,
            customization_template_id: None,
            diagnostic: None,
            properties: vec![InspectorPluginComponentPropertyViewData {
                field_id: "weather.Component.CloudLayer.coverage".to_string(),
                label: "Coverage".to_string(),
                value: "0.75".to_string(),
                value_kind: "scalar".to_string(),
                editable: true,
            }],
        },
        InspectorPluginComponentViewData {
            component_id: "particles.Component.Emitter".to_string(),
            display_name: "Emitter".to_string(),
            customization_available: false,
            customization_ui_document: None,
            customization_template_id: None,
            diagnostic: Some(
                "Plugin inspector customization unavailable for `particles.Component.Emitter`; serialized data stays protected until the plugin reloads."
                    .to_string(),
            ),
            properties: vec![InspectorPluginComponentPropertyViewData {
                field_id: "particles.Component.Emitter.rate".to_string(),
                label: "Rate".to_string(),
                value: "12".to_string(),
                value_kind: "integer".to_string(),
                editable: false,
            }],
        },
    ];

    let data =
        to_host_contract_inspector_pane_from_host_pane(&pane, PaneContentSize::new(360.0, 320.0));

    let available_header = find_node(
        &data.nodes,
        "PluginComponentHeader:weather.Component.CloudLayer",
    );
    assert_eq!(available_header.surface_variant.as_str(), "panel");
    assert_eq!(available_header.text_tone.as_str(), "default");
    assert!(available_header.selected);

    let coverage = find_node(
        &data.nodes,
        "DynamicComponentField:weather.Component.CloudLayer.coverage",
    );
    assert_eq!(coverage.role.as_str(), "NumberField");
    assert_eq!(coverage.value_text.as_str(), "0.75");
    assert_eq!(
        coverage.edit_action_id.as_str(),
        "inspector.dynamic_component.weather_component_cloudlayer_coverage.edit"
    );
    assert_eq!(coverage.surface_variant.as_str(), "inspector-field");
    assert_eq!(coverage.text_tone.as_str(), "default");
    assert!(!coverage.disabled);

    let degraded_header = find_node(
        &data.nodes,
        "PluginComponentHeader:particles.Component.Emitter",
    );
    assert_eq!(degraded_header.surface_variant.as_str(), "inset");
    assert_eq!(degraded_header.text_tone.as_str(), "warning");
    assert!(!degraded_header.selected);

    let degraded_diagnostic = find_node(
        &data.nodes,
        "PluginComponentDiagnostic:particles.Component.Emitter",
    );
    assert_eq!(degraded_diagnostic.surface_variant.as_str(), "inset");
    assert_eq!(degraded_diagnostic.text_tone.as_str(), "warning");
    assert!(degraded_diagnostic.disabled);

    let degraded = find_node(
        &data.nodes,
        "DynamicComponentField:particles.Component.Emitter.rate",
    );
    assert!(degraded.disabled);
    assert_eq!(degraded.surface_variant.as_str(), "inset");
    assert_eq!(degraded.text_tone.as_str(), "muted");
    assert_eq!(degraded.validation_level.as_str(), "warning");
    assert!(degraded
        .validation_message
        .as_str()
        .contains("serialized data stays protected"));
}

fn find_node(
    nodes: &ModelRc<host_contract::TemplatePaneNodeData>,
    control_id: &str,
) -> host_contract::TemplatePaneNodeData {
    (0..nodes.row_count())
        .filter_map(|row| nodes.row_data(row))
        .find(|node| node.control_id.as_str() == control_id)
        .unwrap_or_else(|| panic!("{control_id} node should be projected"))
}

fn inspector_pane_fixture(info: &str) -> PaneData {
    PaneData {
        id: "editor.inspector#1".into(),
        slot: "right".into(),
        kind: "Inspector".into(),
        title: "Inspector".into(),
        icon_key: "inspector".into(),
        subtitle: "Selection".into(),
        info: info.into(),
        show_empty: false,
        empty_title: "".into(),
        empty_body: "".into(),
        primary_action_label: "".into(),
        primary_action_id: "".into(),
        secondary_action_label: "".into(),
        secondary_action_id: "".into(),
        secondary_hint: "".into(),
        show_toolbar: false,
        viewport: blank_viewport_chrome(),
        native_body: PaneNativeBodyData {
            inspector: InspectorPaneViewData {
                nodes: model_rc(Vec::new()),
                info: info.into(),
                inspector_name: "Camera".into(),
                inspector_parent: "Root".into(),
                inspector_x: "1.25".into(),
                inspector_y: "2.50".into(),
                inspector_z: "3.75".into(),
                delete_enabled: true,
                plugin_components: Vec::new(),
            },
            ..PaneNativeBodyData::default()
        },
        pane_presentation: None,
    }
}
