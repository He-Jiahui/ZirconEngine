use super::{
    inspector_component_key, inspector_dynamic_component_control_id,
    inspector_dynamic_component_edit_action_id, inspector_field_nodes_with_metrics,
    InspectorVisualFields,
};
use crate::ui::layouts::windows::workbench_host_window::PaneContentSize;
use crate::ui::retained_host::METRICS;

#[test]
fn plugin_field_identity_distinguishes_punctuation_that_was_previously_collapsed() {
    let hyphenated = "camera-offset";
    let underscored = "camera_offset";

    assert_ne!(
        inspector_component_key(hyphenated),
        inspector_component_key(underscored)
    );
    assert_ne!(
        inspector_dynamic_component_control_id(hyphenated),
        inspector_dynamic_component_control_id(underscored)
    );
    assert_ne!(
        inspector_dynamic_component_edit_action_id(hyphenated),
        inspector_dynamic_component_edit_action_id(underscored)
    );
}

#[test]
fn inspector_fields_follow_shared_metrics_for_default_and_scaled_layouts() {
    let fields = InspectorVisualFields {
        info: String::new(),
        name: "Camera".to_string(),
        parent: "Root".to_string(),
        x: "1.25".to_string(),
        y: "2.50".to_string(),
        z: "3.75".to_string(),
        delete_enabled: true,
        plugin_components: Vec::new(),
    };
    let default_nodes = inspector_field_nodes_with_metrics(
        &fields,
        &[],
        PaneContentSize::new(360.0, 240.0),
        METRICS,
    );
    let default_name = find_node(&default_nodes, "NameField");
    let default_apply = find_node(&default_nodes, "ApplyBatchButton");

    assert_eq!(default_name.frame.x, 8.0);
    assert_eq!(default_name.frame.y, 8.0);
    assert_eq!(default_name.frame.height, 28.0);
    assert_eq!(default_name.corner_radius, 4.0);
    assert_eq!(default_apply.frame.width, 84.0);
    assert_eq!(default_apply.frame.height, 24.0);

    let mut scaled = METRICS;
    scaled.row_height = 36.0;
    scaled.gap_s = 6.0;
    scaled.gap_m = 12.0;
    scaled.button_pad_x = 14.0;
    scaled.radius_control = 5.0;
    scaled.border_width = 2.0;
    let scaled_nodes = inspector_field_nodes_with_metrics(
        &fields,
        &[],
        PaneContentSize::new(360.0, 240.0),
        scaled,
    );
    let scaled_name = find_node(&scaled_nodes, "NameField");
    let scaled_parent = find_node(&scaled_nodes, "ParentField");
    let scaled_apply = find_node(&scaled_nodes, "ApplyBatchButton");

    assert_eq!(scaled_name.frame.x, 12.0);
    assert_eq!(scaled_name.frame.height, 36.0);
    assert_eq!(scaled_name.corner_radius, 5.0);
    assert_eq!(scaled_name.border_width, 2.0);
    assert_eq!(scaled_parent.frame.y, 57.0);
    assert_eq!(scaled_apply.frame.width, 98.0);
    assert_eq!(scaled_apply.frame.height, 30.0);

    let mut compact = METRICS;
    compact.gap_s = 8.0;
    compact.gap_m = 2.0;
    let compact_nodes = inspector_field_nodes_with_metrics(
        &fields,
        &[],
        PaneContentSize::new(360.0, 240.0),
        compact,
    );
    let compact_name = find_node(&compact_nodes, "NameField");
    let compact_parent = find_node(&compact_nodes, "ParentField");

    assert!(
        compact_parent.frame.y >= compact_name.frame.y + compact_name.frame.height,
        "invalid themed gap combinations must not make editable rows overlap"
    );
}

fn find_node<'a>(
    nodes: &'a [crate::ui::retained_host::TemplatePaneNodeData],
    control_id: &str,
) -> &'a crate::ui::retained_host::TemplatePaneNodeData {
    nodes
        .iter()
        .find(|node| node.control_id.as_str() == control_id)
        .unwrap_or_else(|| panic!("{control_id} node should be projected"))
}
