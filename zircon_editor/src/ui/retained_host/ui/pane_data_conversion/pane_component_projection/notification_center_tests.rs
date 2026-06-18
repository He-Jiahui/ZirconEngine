use std::collections::BTreeMap;

use super::host_template_node;
use crate::ui::template_runtime::RetainedUiHostNodeProjection;
use toml::Value;
use zircon_runtime_interface::ui::layout::UiFrame;

#[test]
fn runtime_component_projection_projects_notification_center_rows_for_native_painter() {
    let node = host_template_node(projected_node(
        "NotificationCenter",
        [
            ("open", Value::Boolean(true)),
            ("title", Value::String("Notifications".into())),
            ("empty_text", Value::String("No notifications".into())),
            ("selected_notification_id", Value::String("build".into())),
            ("focused_index", Value::Integer(1)),
            ("visible_limit", Value::Integer(2)),
            ("placement", Value::String("bottom-end".into())),
            ("popup_anchor_x", Value::Float(400.0)),
            ("popup_anchor_y", Value::Float(40.0)),
            ("popup_anchor_width", Value::Float(64.0)),
            ("popup_anchor_height", Value::Float(16.0)),
            ("popup_offset_y", Value::Float(8.0)),
            (
                "notifications",
                Value::Array(vec![
                    notification_entry([
                        ("id", Value::String("build".into())),
                        ("title", Value::String("Build failed".into())),
                        ("message", Value::String("Shader compile error".into())),
                        ("severity", Value::String("error".into())),
                        ("unread", Value::Boolean(true)),
                    ]),
                    notification_entry([
                        ("id", Value::String("asset".into())),
                        ("title", Value::String("Asset import complete".into())),
                        ("description", Value::String("StoneWall.mesh ready".into())),
                        ("severity", Value::String("success".into())),
                        ("new", Value::Boolean(true)),
                    ]),
                    notification_entry([
                        ("id", Value::String("source".into())),
                        ("title", Value::String("Source control synced".into())),
                    ]),
                ]),
            ),
        ],
    ))
    .expect("NotificationCenter should project into the host contract");

    assert_eq!(node.component_role.as_str(), "notification-center");
    assert!(node.popup_open);
    assert_eq!(node.text.as_str(), "Notifications");
    assert_eq!(node.value_text.as_str(), "No notifications");
    assert_eq!(node.options.row_count(), 2);
    assert_eq!(node.options.row_data(0).as_deref(), Some("Build failed"));
    assert_eq!(
        node.options_text.as_str(),
        "Build failed, Asset import complete"
    );
    assert_eq!(node.frame.x, 144.0);
    assert_eq!(node.frame.y, 64.0);

    let first = node
        .structured_options
        .row_data(0)
        .expect("first notification row should be projected");
    assert_eq!(first.id.as_str(), "build");
    assert_eq!(first.label.as_str(), "Build failed");
    assert_eq!(first.description.as_str(), "Shader compile error");
    assert_eq!(first.tone.as_str(), "error");
    assert!(first.selected);
    assert!(first.unread);
    assert!(first.special);

    let second = node
        .structured_options
        .row_data(1)
        .expect("second notification row should be projected");
    assert_eq!(second.id.as_str(), "asset");
    assert_eq!(second.description.as_str(), "StoneWall.mesh ready");
    assert_eq!(second.tone.as_str(), "success");
    assert!(second.focused);
    assert!(second.unread);
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
        frame: UiFrame::new(0.0, 0.0, 320.0, 240.0),
        clip_frame: None,
        z_index: 0,
        attributes: attributes
            .into_iter()
            .map(|(name, value)| (name.to_owned(), value))
            .collect::<BTreeMap<_, _>>(),
        style_overrides: BTreeMap::new(),
        style_tokens: BTreeMap::new(),
        bindings: Vec::new(),
    }
}

fn notification_entry(values: impl IntoIterator<Item = (&'static str, Value)>) -> Value {
    let mut table = toml::map::Map::new();
    for (name, value) in values {
        table.insert(name.to_owned(), value);
    }
    Value::Table(table)
}
