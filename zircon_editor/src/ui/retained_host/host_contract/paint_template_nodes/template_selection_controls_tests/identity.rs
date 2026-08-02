use super::super::{SelectionControlKind, selection_control_kind};
use super::support::node_with_role;

#[test]
fn selection_control_kind_matches_roles_and_workbench_ids() {
    assert_eq!(
        selection_control_kind(&node_with_role("Checkbox", "checkbox", "Custom")),
        Some(SelectionControlKind::Checkbox)
    );
    assert_eq!(
        selection_control_kind(&node_with_role("Radio", "radio", "Custom")),
        Some(SelectionControlKind::Radio)
    );
    assert_eq!(
        selection_control_kind(&node_with_role("Toggle", "toggle", "Custom")),
        Some(SelectionControlKind::Toggle)
    );
    assert_eq!(
        selection_control_kind(&node_with_role("Mount", "", "WorkbenchToggleOn")),
        Some(SelectionControlKind::Toggle)
    );
}
