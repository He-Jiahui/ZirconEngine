use super::super::identity::{is_stepper_field, is_workbench_field};
use super::super::search::is_search_field;
use super::support::field_node;

#[test]
fn workbench_field_matches_component_fields_but_not_axis_fields() {
    assert!(is_workbench_field(&field_node(
        "WorkbenchInputText",
        "Text field"
    )));
    assert!(is_workbench_field(&field_node("WorkbenchFieldRoot", "")));
    assert!(is_workbench_field(&field_node("SearchEdited", "")));
    assert!(!is_workbench_field(&field_node(
        "WorkbenchTransformPositionX",
        "128.4"
    )));
}

#[test]
fn number_field_stepper_is_driven_by_the_component_layout_property() {
    let mut number = field_node("NumberFieldDemo", "42");
    number.component_role = "number-field".into();
    number.layout_stepper = true;

    assert!(is_stepper_field(&number));
}

#[test]
fn mixed_case_search_identity_remains_search_field() {
    assert!(is_search_field(&field_node(
        "WorkbenchSeArChQuery",
        "material"
    )));
}
