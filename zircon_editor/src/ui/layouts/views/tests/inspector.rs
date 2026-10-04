use super::*;
use zircon_runtime_interface::ui::design_tokens::EditorDensityTokens;

fn projected_nodes(inspector: Option<&InspectorSnapshot>) -> Vec<ViewTemplateNodeData> {
    let pane = inspector_pane_nodes(inspector, UiSize::new(360.0, 520.0));
    (0..pane.row_count())
        .filter_map(|row| pane.row_data(row))
        .collect()
}

fn node_by_control_id<'a>(
    nodes: &'a [ViewTemplateNodeData],
    control_id: &str,
) -> Option<&'a ViewTemplateNodeData> {
    nodes.iter().find(|node| node.control_id == control_id)
}

#[test]
fn no_selection_projects_a_muted_centered_empty_state() {
    let nodes = projected_nodes(None);

    assert!(nodes
        .iter()
        .any(|node| node.control_id == "InspectorEmptyState"));
    assert!(nodes
        .iter()
        .any(|node| node.control_id == "InspectorEmptyStateMessage"));

    let Some(header) = node_by_control_id(&nodes, "InspectorHeaderPanel") else {
        return;
    };
    let Some(empty_state) = node_by_control_id(&nodes, "InspectorEmptyState") else {
        return;
    };
    let Some(name) = node_by_control_id(&nodes, INSPECTOR_NAME_VALUE_CONTROL_ID) else {
        return;
    };
    let Some(message) = node_by_control_id(&nodes, "InspectorEmptyStateMessage") else {
        return;
    };

    assert_eq!(header.text.to_string(), "Inspector");
    assert_eq!(header.text_tone.to_string(), "default");
    assert_eq!(header.surface_variant.to_string(), "transparent");
    assert!(!header.selected);
    assert!(!header.focused);
    assert_eq!(
        header.frame.height,
        EditorDensityTokens::WORKBENCH_ROW_HEIGHT
    );
    assert_eq!(name.frame.height, EditorDensityTokens::WORKBENCH_ROW_HEIGHT);
    assert!(!name.selected);
    assert_eq!(name.value_text.to_string(), "-");
    assert_eq!(empty_state.surface_variant.to_string(), "inset");
    assert_eq!(message.text.to_string(), "No object selected");
    assert_eq!(message.text_align.to_string(), "center");
    assert!(empty_state.frame.height > 120.0);
}

#[test]
fn selection_hides_empty_state_without_synthesizing_keyboard_focus() {
    let inspector = InspectorSnapshot {
        rotation_degrees: None,
        id: zircon_runtime::scene::NodeId::default(),
        name: "Camera".to_string(),
        parent: "Root".to_string(),
        translation: ["1.0".to_string(), "2.0".to_string(), "3.0".to_string()],
        scale: ["1.0".to_string(), "1.0".to_string(), "1.0".to_string()],
        render_layer_mask: 1,
        native_fields: Vec::new(),
        plugin_components: Vec::new(),
    };
    let nodes = projected_nodes(Some(&inspector));

    let Some(header) = node_by_control_id(&nodes, "InspectorHeaderPanel") else {
        return;
    };
    let Some(empty_state) = node_by_control_id(&nodes, "InspectorEmptyState") else {
        return;
    };
    let Some(message) = node_by_control_id(&nodes, "InspectorEmptyStateMessage") else {
        return;
    };

    assert!(!header.selected);
    assert!(!header.focused);
    assert_eq!(empty_state.surface_variant.to_string(), "transparent");
    assert!(message.text.is_empty());

    for (control_id, expected_value) in [
        (INSPECTOR_NAME_VALUE_CONTROL_ID, "Camera"),
        (INSPECTOR_PARENT_VALUE_CONTROL_ID, "Root"),
        (INSPECTOR_POSITION_VALUE_CONTROL_ID, "1.0, 2.0, 3.0"),
        (INSPECTOR_COMPONENTS_VALUE_CONTROL_ID, "0"),
    ] {
        let Some(readout) = node_by_control_id(&nodes, control_id) else {
            return;
        };
        assert_eq!(readout.value_text.to_string(), expected_value);
        assert!(
            !readout.selected,
            "{control_id} must not impersonate selection"
        );
    }
}
