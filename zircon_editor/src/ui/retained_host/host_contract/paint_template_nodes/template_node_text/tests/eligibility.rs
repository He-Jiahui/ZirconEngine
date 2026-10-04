use super::*;

#[test]
fn icon_only_node_skips_before_label_materialization() {
    let node = TemplatePaneNodeData {
        role: "IconButton".into(),
        text: "unused fallback text".into(),
        ..TemplatePaneNodeData::default()
    };

    assert!(should_skip_template_text_before_label(&node, false, false));
    assert!(should_skip_template_text(&node, &node.text, false, false));
}

#[test]
fn native_painter_owner_does_not_receive_a_duplicate_fallback_label() {
    let node = TemplatePaneNodeData {
        component_role: "mui-x-agent-plan".into(),
        role: "Mount".into(),
        text: "Agent plan".into(),
        ..TemplatePaneNodeData::default()
    };

    assert!(should_skip_template_text_before_label(&node, false, false));
    assert!(should_skip_template_text(&node, &node.text, false, false));
}

#[test]
fn data_surface_native_painters_do_not_receive_duplicate_fallback_labels() {
    for component_role in ["mui-x-data-grid", "mui-x-tree-view"] {
        let node = TemplatePaneNodeData {
            component_role: component_role.into(),
            role: "Mount".into(),
            text: "Source-owned title".into(),
            ..TemplatePaneNodeData::default()
        };

        assert!(should_skip_template_text_before_label(&node, false, false));
        assert!(should_skip_template_text(&node, &node.text, false, false));
    }
}
