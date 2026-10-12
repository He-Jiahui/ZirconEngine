use super::super::super::super::data::TemplatePaneNodeData;
use super::super::identity::is_workbench_dropdown;
use super::support::dropdown_node;

#[test]
fn workbench_dropdown_matches_dropdown_nodes_only() {
    assert!(is_workbench_dropdown(&dropdown_node(false)));
    assert!(is_workbench_dropdown(&TemplatePaneNodeData {
        control_id: "WorkbenchDropdownRoot".into(),
        role: "ComboBox".into(),
        component_role: "combo-box".into(),
        ..TemplatePaneNodeData::default()
    }));
    assert!(!is_workbench_dropdown(&TemplatePaneNodeData {
        control_id: "WorkbenchInputDropdownRow".into(),
        role: "HorizontalGroup".into(),
        ..TemplatePaneNodeData::default()
    }));
}
