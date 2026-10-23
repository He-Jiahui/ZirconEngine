use super::*;
use std::time::{Duration, Instant};

#[test]
fn welcome_probe_request_after_tick_arms_idle_wake_and_preserves_earlier_deadlines() {
    let _guard = lock_env();
    let harness = ChildWindowHostHarness::new("zircon_welcome_probe_idle_wake");
    harness.host.borrow_mut().clear_welcome_project_probe();
    harness.root_ui.set_lifecycle_frame_update(None);
    let before = Instant::now();
    harness.host.borrow_mut().schedule_welcome_project_probe();
    let wake = harness
        .root_ui
        .lifecycle_frame_wake_deadline_for_test()
        .expect("a field callback after tick must wake the idle host");
    assert!(wake >= before);
    assert!(wake <= Instant::now() + Duration::from_millis(50));

    let earlier = before;
    harness.root_ui.set_lifecycle_frame_update(Some(earlier));
    harness.host.borrow_mut().schedule_welcome_project_probe();
    assert_eq!(
        harness.root_ui.lifecycle_frame_wake_deadline_for_test(),
        Some(earlier)
    );
    harness.host.borrow_mut().clear_welcome_project_probe();
}

#[test]
fn welcome_snapshot_refresh_requests_projection_and_present_damage() {
    let _guard = lock_env();
    let harness = ChildWindowHostHarness::new("zircon_welcome_probe_present_damage");
    let _ = harness.root_ui.take_external_redraw_for_test();
    harness.host.borrow_mut().refresh_welcome_snapshot();
    let request = harness.root_ui.take_external_redraw_for_test();
    assert!(
        request.requires_frame_update(),
        "new welcome data needs projection"
    );
    assert!(
        request.requires_present(),
        "idle probe completion must repaint its result"
    );
}
