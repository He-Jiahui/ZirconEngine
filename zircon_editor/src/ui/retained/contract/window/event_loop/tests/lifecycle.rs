use std::time::{Duration, Instant};

use super::earliest_wake_deadline;

#[test]
fn no_runtime_deadline_resets_the_native_wait_policy() {
    let source = include_str!("../lifecycle.rs");
    assert!(source.contains("None => event_loop.set_control_flow(ControlFlow::Wait)"));
}

#[test]
fn about_to_wait_does_not_poll_native_window_metrics_per_event_batch() {
    let source = include_str!("../lifecycle.rs");
    let function = source
        .split("pub(in crate::ui::retained_host::host_contract) fn about_to_wait_impl")
        .nth(1)
        .and_then(|body| body.split("fn schedule_due_surface_present_retry").next())
        .expect("about-to-wait implementation");

    assert!(!function.contains("sync_host_window_state"));
    assert!(!function.contains("window.surface_size()"));
    assert!(!function.contains("window.outer_position()"));
    assert!(!function.contains("window.is_maximized()"));
}

#[test]
fn about_to_wait_restarts_profile_measurement_after_callback_scopes_drop() {
    let source = include_str!("../lifecycle.rs");
    let function = source
        .split("pub(in crate::ui::retained_host::host_contract) fn about_to_wait_impl")
        .nth(1)
        .and_then(|body| body.split("fn schedule_due_surface_present_retry").next())
        .expect("about-to-wait implementation");

    assert!(function.contains("self.restart_profile_measurement_if_ready();"));
}

#[test]
fn surface_retry_deadline_participates_in_the_native_wait_policy() {
    let now = Instant::now();
    let surface_retry = now + Duration::from_millis(8);
    let runtime = now + Duration::from_secs(1);

    assert_eq!(
        earliest_wake_deadline(Some(runtime), Some(surface_retry)),
        Some(surface_retry)
    );
}

#[test]
fn pending_resize_is_consumed_before_testing_presenter_availability() {
    let source = include_str!("../lifecycle.rs");
    let function = source
        .split("pub(super) fn apply_pending_presenter_resize")
        .nth(1)
        .and_then(|body| body.split("fn poll_presenter_startup").next())
        .expect("pending presenter resize implementation");
    let consume_size = function
        .find("self.pending_presenter_resize.take()")
        .expect("pending size consumption");
    let presenter_gate = function
        .find("self.presenter.as_mut()")
        .expect("presenter availability gate");

    assert!(consume_size < presenter_gate);
}

#[test]
fn capture_demand_presents_without_mutating_the_retained_business_projection() {
    let absent = super::super::lifecycle::normal_capture_redraw(false);
    assert!(!absent.request_redraw());
    let capture = super::super::lifecycle::normal_capture_redraw(true);
    assert!(capture.request_redraw());
    assert!(capture.requires_present());
    assert!(!capture.requires_frame_update());
    assert!(!capture.prefers_interactive_frame_update());
}
