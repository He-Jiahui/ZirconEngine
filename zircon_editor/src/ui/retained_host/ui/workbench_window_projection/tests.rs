use super::*;
use crate::ui::template_runtime::RetainedUiHostComponentKind;

#[test]
fn workbench_button_text_prefers_authored_label_over_value_render_text() {
    let node = test_host_node(
        "Button",
        "button",
        Some("thumbnail"),
        [("text", "Thumb"), ("value", "thumbnail")],
    );

    assert_eq!(projected_workbench_text(&node, "button"), "Thumb");
}

#[test]
fn workbench_input_text_keeps_rendered_value_display_semantics() {
    let node = test_host_node(
        "TextField",
        "input-field",
        Some("albedo"),
        [("text", "Search"), ("value", "albedo")],
    );

    assert_eq!(projected_workbench_text(&node, "input-field"), "albedo");
}

#[test]
fn workbench_segmented_control_projects_selected_value_text() {
    let node = test_host_node(
        "SegmentedControl",
        "segmented-control",
        None,
        [("value", "grid")],
    );

    assert_eq!(
        projected_workbench_value_text(&node, "segmented-control", &BTreeMap::new()),
        "grid"
    );
}

fn test_host_node<const N: usize>(
    component: &str,
    component_role: &str,
    text: Option<&str>,
    properties: [(&str, &str); N],
) -> RetainedUiHostNodeModel {
    RetainedUiHostNodeModel {
        node_id: "test-node".to_string(),
        parent_id: None,
        kind: RetainedUiHostComponentKind::from_component(component),
        component: component.to_string(),
        control_id: Some("test-control".to_string()),
        frame: UiFrame::new(0.0, 0.0, 100.0, 24.0),
        clip_frame: None,
        z_index: 0,
        text: text.map(str::to_string),
        icon: None,
        component_role: Some(component_role.to_string()),
        value_text: None,
        validation_level: None,
        validation_message: None,
        popup_open: false,
        has_popup_anchor: false,
        popup_anchor_x: 0.0,
        popup_anchor_y: 0.0,
        selection_state: None,
        options_text: None,
        options: Vec::new(),
        collection_items: Vec::new(),
        menu_items: Vec::new(),
        accepted_drag_payloads: Vec::new(),
        drop_source_summary: None,
        checked: false,
        expanded: false,
        focused: false,
        hovered: false,
        pressed: false,
        dragging: false,
        drop_hovered: false,
        active_drag_target: false,
        disabled: false,
        properties: properties
            .into_iter()
            .map(|(key, value)| {
                (
                    key.to_string(),
                    RetainedUiHostValue::String(value.to_string()),
                )
            })
            .collect(),
        style_tokens: BTreeMap::new(),
        routes: Vec::new(),
    }
}
