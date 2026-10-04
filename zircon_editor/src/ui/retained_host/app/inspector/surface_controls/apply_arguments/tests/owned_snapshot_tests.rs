use super::*;

#[test]
fn apply_button_does_not_resend_unedited_primary_snapshot_fields() {
    let arguments = inspector_apply_arguments_for_active_selection(Some(2)).unwrap();
    assert_eq!(arguments[0].as_str(), Some("entity://selected"));
    assert!(binding_array(&arguments[1]).is_empty());
}

#[test]
fn apply_button_without_selection_keeps_nothing_selected_feedback() {
    assert_eq!(
        inspector_apply_arguments_for_active_selection(None).unwrap_err(),
        "Nothing selected"
    );
}

fn binding_array(value: &UiBindingValue) -> &[UiBindingValue] {
    let UiBindingValue::Array(values) = value else {
        panic!("expected binding array");
    };
    values
}
