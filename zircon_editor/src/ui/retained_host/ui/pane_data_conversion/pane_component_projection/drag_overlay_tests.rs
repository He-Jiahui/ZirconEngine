use std::collections::BTreeMap;

use super::host_template_node;
use crate::ui::template_runtime::RetainedUiHostNodeProjection;
use toml::Value;
use zircon_runtime_interface::ui::layout::UiFrame;

#[test]
fn runtime_component_projection_projects_drag_overlay_for_native_painter() {
    let node = host_template_node(projected_node(
        "DragOverlay",
        [
            ("open", Value::Boolean(false)),
            ("dragging", Value::Boolean(true)),
            ("drop_hovered", Value::Boolean(true)),
            ("active_drag_target", Value::Boolean(true)),
            ("payload_kind", Value::String("asset".into())),
            ("payload_label", Value::String("StoneWall.mesh".into())),
            (
                "payload_reference",
                Value::String("assets/stone_wall.mesh".into()),
            ),
            ("cursor_x", Value::Float(72.0)),
            ("cursor_y", Value::Float(48.0)),
            ("offset_x", Value::Float(16.0)),
            ("offset_y", Value::Float(18.0)),
            ("preview_width", Value::Float(184.0)),
            ("preview_height", Value::Float(36.0)),
            ("drop_allowed", Value::Boolean(false)),
            ("drop_target_x", Value::Float(24.0)),
            ("drop_target_y", Value::Float(148.0)),
            ("drop_target_width", Value::Float(280.0)),
            ("drop_target_height", Value::Float(30.0)),
            ("drop_indicator_edge", Value::String("bottom".into())),
            (
                "drop_indicator_text",
                Value::String("Cannot drop on locked folder".into()),
            ),
        ],
    ))
    .expect("DragOverlay should project into the host contract");

    assert_eq!(node.component_role.as_str(), "drag-overlay");
    assert!(
        node.popup_open,
        "dragging=true should make the native overlay paint even when open=false"
    );
    assert!(node.dragging);
    assert!(node.drop_hovered);
    assert!(node.active_drag_target);
    assert_eq!(node.text.as_str(), "StoneWall.mesh");
    assert_eq!(node.value_text.as_str(), "assets/stone_wall.mesh");
    assert_eq!(node.drag_payload_kind.as_str(), "asset");
    assert_eq!(node.drag_payload_label.as_str(), "StoneWall.mesh");
    assert_eq!(
        node.drag_payload_reference.as_str(),
        "assets/stone_wall.mesh"
    );
    assert!(node.has_drag_cursor);
    assert_eq!(node.drag_cursor_x, 72.0);
    assert_eq!(node.drag_cursor_y, 48.0);
    assert_eq!(node.drag_offset_x, 16.0);
    assert_eq!(node.drag_offset_y, 18.0);
    assert_eq!(node.drag_preview_width, 184.0);
    assert_eq!(node.drag_preview_height, 36.0);
    assert!(!node.drop_allowed);
    assert!(node.has_drop_target);
    assert_eq!(node.drop_target_x, 24.0);
    assert_eq!(node.drop_target_y, 148.0);
    assert_eq!(node.drop_target_width, 280.0);
    assert_eq!(node.drop_target_height, 30.0);
    assert_eq!(node.drop_indicator_edge.as_str(), "bottom");
    assert_eq!(
        node.drop_indicator_text.as_str(),
        "Cannot drop on locked folder"
    );
}

fn projected_node(
    component: &str,
    attributes: impl IntoIterator<Item = (&'static str, Value)>,
) -> RetainedUiHostNodeProjection {
    RetainedUiHostNodeProjection {
        node_id: format!("{component}Node"),
        parent_id: None,
        component: component.to_owned(),
        control_id: Some(format!("{component}Control")),
        frame: UiFrame::new(0.0, 0.0, 360.0, 220.0),
        clip_frame: None,
        z_index: 0,
        attributes: attributes
            .into_iter()
            .map(|(name, value)| (name.to_owned(), value))
            .collect::<BTreeMap<_, _>>(),
        style_tokens: BTreeMap::new(),
        bindings: Vec::new(),
    }
}
