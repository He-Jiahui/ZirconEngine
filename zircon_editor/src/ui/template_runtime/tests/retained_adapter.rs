use super::*;
use crate::ui::template_runtime::RetainedUiHostNodeProjection;
use zircon_runtime_interface::ui::layout::UiFrame;

#[test]
fn unknown_source_component_can_declare_a_native_painter_role() {
    let node = RetainedUiHostNodeProjection {
        node_id: "agent-plan".to_string(),
        surface_node_id: None,
        has_workbench_icon_tooltip: false,
        parent_id: None,
        component: "AgentPlan".to_string(),
        control_id: Some("AgentPlan".to_string()),
        source_path: None,
        source_node_id: None,
        instance_path: None,
        parent_source_path: None,
        parent_source_node_id: None,
        parent_instance_path: None,
        frame: UiFrame::default(),
        clip_frame: None,
        z_index: 0,
        attributes: BTreeMap::from([(
            "component_role".to_string(),
            Value::String("mui-x-agent-plan".to_string()),
        )]),
        style_overrides: BTreeMap::new(),
        style_tokens: BTreeMap::new(),
        bindings: Vec::new(),
    };

    let projected = RetainedUiHostAdapter::build_node(&node);

    assert_eq!(
        projected.component_role.as_deref(),
        Some("mui-x-agent-plan")
    );
}
