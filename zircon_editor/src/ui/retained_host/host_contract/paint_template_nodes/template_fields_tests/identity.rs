use super::super::identity::is_workbench_field;
use super::support::field_node;

#[test]
fn workbench_field_matches_component_fields_but_not_axis_fields() {
    assert!(is_workbench_field(&field_node(
        "WorkbenchInputText",
        "Text field"
    )));
    assert!(is_workbench_field(&field_node("WorkbenchFieldRoot", "")));
    assert!(!is_workbench_field(&field_node(
        "WorkbenchTransformPositionX",
        "128.4"
    )));
}
