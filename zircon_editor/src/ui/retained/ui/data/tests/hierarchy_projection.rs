use super::*;

#[test]
fn hierarchy_template_state_projects_the_transient_search_query() {
    let mut nodes = vec![host_contract::TemplatePaneNodeData {
        control_id: "HierarchySearchQuery".into(),
        ..host_contract::TemplatePaneNodeData::default()
    }];

    apply_hierarchy_template_state(&mut nodes, true, false, "sun");

    assert_eq!(nodes[0].value_text.as_str(), "sun");
    assert_eq!(nodes[0].text.as_str(), "sun");
}

#[test]
fn hierarchy_selection_never_fabricates_template_keyboard_focus() {
    let mut nodes = vec![
        host_contract::TemplatePaneNodeData {
            control_id: "HierarchyListPanel".into(),
            ..Default::default()
        },
        host_contract::TemplatePaneNodeData {
            control_id: "SelectRoot".into(),
            ..Default::default()
        },
    ];
    apply_hierarchy_template_state(&mut nodes, true, true, "");
    assert!(nodes.iter().all(|node| node.selected && !node.focused));
    nodes[0].focused = true;
    apply_hierarchy_template_state(&mut nodes, true, false, "");
    assert!(nodes[0].focused);
    assert!(!nodes[0].selected);
}
