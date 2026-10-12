use super::*;

#[test]
fn axis_label_roles_include_text_and_icon_shapes_only() {
    assert!(is_axis_label_role("Label"));
    assert!(is_axis_label_role("Icon"));
    assert!(is_axis_label_role("SvgIcon"));
    assert!(!is_axis_label_role("Button"));
}
