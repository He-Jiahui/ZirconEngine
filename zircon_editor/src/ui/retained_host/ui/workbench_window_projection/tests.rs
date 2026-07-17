use super::*;
use crate::ui::template_runtime::RetainedUiHostComponentKind;

#[test]
fn workbench_projection_uses_a_memoized_node_index() {
    let source = include_str!("../workbench_window_projection.rs");

    assert!(source.contains("mod node_index;"));
    assert!(source.contains("ProjectionNodeIndex::new"));
    assert!(!source.contains("fn host_projection_node_render_visible("));
    assert!(!source.contains("fn projected_parent_node_id("));
}

#[test]
fn workbench_projection_memoizes_collapsed_ancestor_visibility() {
    let mut parent = test_host_node("Panel", "panel", None, []);
    parent.node_id = "collapsed-parent".to_string();
    parent.properties.insert(
        "visibility".to_string(),
        RetainedUiHostValue::String("Collapsed".to_string()),
    );
    let mut child = test_host_node("Label", "label", Some("hidden"), []);
    child.node_id = "hidden-child".to_string();
    child.parent_id = Some(parent.node_id.clone());

    let node_index = ProjectionNodeIndex::new([&parent, &child]);

    assert!(!node_index.render_visible(&parent));
    assert!(!node_index.render_visible(&child));
}

#[test]
fn workbench_projection_treats_parent_cycles_as_not_render_visible() {
    let mut first = test_host_node("Panel", "panel", None, []);
    first.node_id = "cycle-first".to_string();
    first.parent_id = Some("cycle-second".to_string());
    let mut second = test_host_node("Panel", "panel", None, []);
    second.node_id = "cycle-second".to_string();
    second.parent_id = Some("cycle-first".to_string());

    let node_index = ProjectionNodeIndex::new([&first, &second]);

    assert!(!node_index.render_visible(&first));
    assert!(!node_index.render_visible(&second));
}

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
fn search_field_projection_preserves_placeholder_and_text_input_identity() {
    let node = test_host_node(
        "SearchField",
        "search-field",
        None,
        [("placeholder", "Search samples"), ("query", "")],
    );
    let node_index = ProjectionNodeIndex::new([&node]);

    let projected = to_host_contract_workbench_window_node(&node, &node_index)
        .expect("SearchField should project into the native host contract");

    assert_eq!(projected.role.as_str(), "SearchField");
    assert_eq!(projected.component_role.as_str(), "search-field");
    assert_eq!(projected.text.as_str(), "Search samples");
    assert_eq!(projected.surface_variant.as_str(), "inset");
    assert_eq!(projected.border_width, 1.0);
    assert_eq!(projected.corner_radius, 5.0);
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

#[test]
fn workbench_property_row_projects_authored_value_into_runtime_text_contract() {
    let node = test_host_node(
        "PropertyRow",
        "property-row",
        Some("Horizontal"),
        [("value", "Speed 0 - 620")],
    );
    let node_index = ProjectionNodeIndex::new([&node]);

    assert_eq!(
        projected_workbench_value_text(&node, "property-row", &BTreeMap::new()),
        "Speed 0 - 620"
    );
    let projected = to_host_contract_workbench_window_node(&node, &node_index)
        .expect("PropertyRow should project into the native host contract");
    assert_eq!(projected.text.as_str(), "Horizontal");
    assert_eq!(projected.value_text.as_str(), "Speed 0 - 620");
}

#[test]
fn workbench_node_projection_preserves_retained_parent_identity() {
    let mut parent = test_host_node("Panel", "panel", None, []);
    parent.node_id = "generated-parent-17".to_string();
    let mut node = test_host_node("Panel", "panel", None, []);
    node.parent_id = Some(parent.node_id.clone());
    let node_index = ProjectionNodeIndex::new([&node, &parent]);

    let projected = to_host_contract_workbench_window_node(&node, &node_index)
        .expect("a controlled retained node should project into the host contract");

    assert_eq!(projected.parent_node_id.as_str(), "generated-parent-17");
}

#[test]
fn workbench_node_projection_skips_control_less_component_expansion_parents() {
    let mut host = test_host_node("Panel", "panel", None, []);
    host.node_id = "extension-workspaces-host".to_string();
    let mut component_expansion = test_host_node("VerticalGroup", "layout", None, []);
    component_expansion.node_id = "generated-layout-wrapper".to_string();
    component_expansion.control_id = None;
    component_expansion.parent_id = Some(host.node_id.clone());
    let mut leaf = test_host_node("Label", "label", None, []);
    leaf.node_id = "generated-caption".to_string();
    leaf.parent_id = Some(component_expansion.node_id.clone());
    let node_index = ProjectionNodeIndex::new([&host, &component_expansion, &leaf]);

    let projected = to_host_contract_workbench_window_node(&leaf, &node_index)
        .expect("a controlled leaf should project into the host contract");

    assert_eq!(
        projected.parent_node_id.as_str(),
        "extension-workspaces-host"
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
