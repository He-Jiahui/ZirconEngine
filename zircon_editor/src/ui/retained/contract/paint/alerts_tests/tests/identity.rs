use super::super::{workbench_alert_kind, AlertTone, WorkbenchAlertKind};
use super::support::alert_node;

#[test]
fn workbench_alert_kind_matches_drawer_ids_and_toast_root() {
    assert_eq!(
        workbench_alert_kind(&alert_node("WorkbenchInfoAlert", "Info Alert", "info")),
        Some(WorkbenchAlertKind::Inline(AlertTone::Info))
    );
    assert_eq!(
        workbench_alert_kind(&alert_node("WorkbenchErrorAlert", "Error Alert", "error")),
        Some(WorkbenchAlertKind::Inline(AlertTone::Error))
    );
    assert_eq!(
        workbench_alert_kind(&alert_node(
            "WorkbenchToastRoot",
            "Operation completed successfully",
            "info"
        )),
        Some(WorkbenchAlertKind::Toast)
    );
    assert_eq!(
        workbench_alert_kind(&alert_node(
            "WorkbenchToastRoot",
            "Imported 24 assets into the project",
            "success"
        )),
        Some(WorkbenchAlertKind::Toast)
    );
    assert_eq!(
        workbench_alert_kind(&alert_node("PlainAlert", "Info Alert", "info")),
        None
    );
}

#[test]
fn generic_alert_tone_uses_declared_severity_not_message_words() {
    assert_eq!(
        workbench_alert_kind(&alert_node(
            "WorkbenchImportAlert",
            "Error text from an imported asset is informational here",
            "info"
        )),
        Some(WorkbenchAlertKind::Inline(AlertTone::Info))
    );
}

#[test]
fn mixed_case_alert_severity_preserves_tone() {
    assert_eq!(
        workbench_alert_kind(&alert_node(
            "WorkbenchImportAlert",
            "Import requires attention",
            "WaRnInG"
        )),
        Some(WorkbenchAlertKind::Inline(AlertTone::Warning))
    );
}
