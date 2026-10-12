use super::super::super::super::data::TemplatePaneNodeData;
use super::super::identity::is_workbench_tree_row;
use super::support::tree_node;

#[test]
fn tree_row_kind_matches_roles_and_scene_ids() {
    assert!(is_workbench_tree_row(&tree_node(
        "Custom", "TreeRow", "", "Root", 0, false
    )));
    assert!(is_workbench_tree_row(&tree_node(
        "WorkbenchScenePropsItem",
        "",
        "",
        "Props",
        2,
        true
    )));
    assert!(is_workbench_tree_row(&tree_node(
        "Custom", "", "tree-row", "Node", 0, false
    )));
    assert!(!is_workbench_tree_row(&TemplatePaneNodeData {
        control_id: "WorkbenchListSelected".into(),
        role: "ListRow".into(),
        component_role: "list-row".into(),
        ..TemplatePaneNodeData::default()
    }));
}
