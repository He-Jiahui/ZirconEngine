use super::super::{StatusControlKind, StatusIconKind, StatusSignalKind, status_control_kind};
use super::support::status_node;

#[test]
fn status_control_kind_matches_workbench_status_ids() {
    assert_eq!(
        status_control_kind(&status_node("WorkbenchStatusReady", "Ready", 96.0, 46.0)),
        Some(StatusControlKind::Signal(StatusSignalKind::Ready))
    );
    assert_eq!(
        status_control_kind(&status_node(
            "WorkbenchStatusGrid",
            "Grid: 10 cm",
            112.0,
            30.0
        )),
        Some(StatusControlKind::Chip)
    );
    assert_eq!(
        status_control_kind(&status_node("WorkbenchStatusTarget", "", 34.0, 30.0)),
        Some(StatusControlKind::Icon(StatusIconKind::Target))
    );
    assert_eq!(
        status_control_kind(&status_node("WorkbenchStatusFill", "", 80.0, 46.0)),
        None
    );
}
