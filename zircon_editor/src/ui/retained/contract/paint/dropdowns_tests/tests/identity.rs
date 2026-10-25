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

#[test]
fn semantic_dropdown_roles_do_not_require_workbench_names_or_variants() {
    for role in ["dropdown", "combo-box", "select", "search-select"] {
        assert!(is_workbench_dropdown(&TemplatePaneNodeData {
            control_id: "ProjectionChooser".into(),
            component_role: role.into(),
            ..TemplatePaneNodeData::default()
        }));
    }
    for role in ["Dropdown", "ComboBox", "Select"] {
        assert!(is_workbench_dropdown(&TemplatePaneNodeData {
            control_id: "ProjectionChooser".into(),
            role: role.into(),
            ..TemplatePaneNodeData::default()
        }));
    }
}

#[test]
fn semantic_dropdown_admission_preserves_component_family_precedence() {
    assert!(!is_workbench_dropdown(&TemplatePaneNodeData {
        control_id: "WorkbenchInputDropdown".into(),
        role: "Dropdown".into(),
        component_role: "button".into(),
        ..TemplatePaneNodeData::default()
    }));
    assert!(!is_workbench_dropdown(&TemplatePaneNodeData {
        control_id: "ProjectionChooser".into(),
        role: "HorizontalGroup".into(),
        ..TemplatePaneNodeData::default()
    }));
}
