use std::path::PathBuf;

use super::ZuiVisualEvidenceSummary;

fn summary(captured: usize, failed: usize, pending: usize) -> ZuiVisualEvidenceSummary {
    ZuiVisualEvidenceSummary {
        captured,
        failed,
        pending,
        report_path: PathBuf::from("native-evidence.json"),
    }
}

#[test]
fn zui_visual_acceptance_empty_or_incomplete_capture_is_not_ready() {
    assert!(!summary(0, 0, 0).is_ready());
    assert!(!summary(1, 1, 0).is_ready());
    assert!(!summary(1, 0, 1).is_ready());
}

#[test]
fn zui_visual_acceptance_ready_capture_does_not_accept_design_parity() {
    let ready = summary(1, 0, 0);
    assert!(ready.is_ready());
    assert!(!ready.is_accepted());
}
