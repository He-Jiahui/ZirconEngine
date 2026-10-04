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
