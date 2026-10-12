use super::{node_is_detached, node_is_visible};
use crate::ui::template_runtime::RetainedUiHostNodeProjection;
use std::collections::BTreeMap;

fn node(attributes: BTreeMap<String, toml::Value>) -> RetainedUiHostNodeProjection {
    RetainedUiHostNodeProjection {
        node_id: "host/path".into(),
        surface_node_id: None,
        has_workbench_icon_tooltip: false,
        parent_id: None,
        component: "Label".into(),
        control_id: Some("label".into()),
        source_path: Some("test.zui".into()),
        source_node_id: Some("label".into()),
        instance_path: Some(Vec::new()),
        parent_source_path: None,
        parent_source_node_id: None,
        parent_instance_path: None,
        frame: zircon_runtime_interface::ui::layout::UiFrame::new(0.0, 0.0, 10.0, 10.0),
        clip_frame: None,
        z_index: 0,
        attributes,
        style_overrides: Default::default(),
        style_tokens: Default::default(),
        bindings: Vec::new(),
    }
}

#[test]
fn visibility_and_detachment_follow_explicit_retained_attributes() {
    assert!(node_is_visible(&node(BTreeMap::new())));
    assert!(!node_is_visible(&node(BTreeMap::from([(
        "visibility".into(),
        toml::Value::String("collapsed".into()),
    )]))));
    assert!(node_is_detached(&node(BTreeMap::from([(
        "detached".into(),
        toml::Value::Boolean(true),
    )]))));
}
