use crate::core::editor_event::ViewInstanceId;
use crate::core::editor_message::EditorViewInvalidationMask;

use crate::tests::editor_event::support::{env_lock, EventRuntimeHarness};

#[test]
fn refresh_view_marks_view_dirty_and_materializes_current_snapshot_backend() {
    let _lock = env_lock().lock().unwrap();
    let harness = EventRuntimeHarness::new("editor_message_refresh_view");
    let view = ViewInstanceId::new("scene.workspace");
    let mask =
        EditorViewInvalidationMask::PRESENTATION_DATA.union(EditorViewInvalidationMask::HIT_TEST);

    let report = harness.runtime.refresh_view(view.clone(), mask);

    assert_eq!(report.dirty().len(), 1);
    assert_eq!(report.dirty().mask_for(&view), Some(mask));
    assert!(report.used_full_snapshot_fallback());

    let empty_report = harness.runtime.drain_pending_view_refreshes();
    assert!(empty_report.dirty().is_empty());
    assert!(!empty_report.used_full_snapshot_fallback());
}
