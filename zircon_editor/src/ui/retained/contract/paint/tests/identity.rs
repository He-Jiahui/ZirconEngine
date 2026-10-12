use super::super::identity::is_workbench_axis_value_field;
use super::support::{axis_node, label_node};

#[test]
fn axis_value_field_kind_matches_transform_axis_inputs_only() {
    assert!(is_workbench_axis_value_field(&axis_node(
        "WorkbenchTransformPositionX",
        "128.4",
    )));
    assert!(is_workbench_axis_value_field(&axis_node(
        "WorkbenchTransformRotationZ",
        "0 deg",
    )));
    assert!(is_workbench_axis_value_field(&axis_node(
        "WorkbenchTransformScaleY",
        "1.00",
    )));
    assert!(!is_workbench_axis_value_field(&label_node(
        "WorkbenchTransformPositionAxisX",
        "X",
    )));
    assert!(!is_workbench_axis_value_field(&axis_node(
        "WorkbenchInputText",
        "Text field",
    )));
}
