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
fn runtime_presenter_upgrade_releases_previous_native_surface_first() {
    let source = include_str!("../lifecycle.rs");
    let release = source
        .find("drop(self.presenter.take());")
        .expect("upgrade must release the startup surface");
    let create = source
        .find("match create_runtime_host_chrome_presenter")
        .expect("upgrade must create the runtime presenter");
    assert!(release < create);
}

#[test]
fn runtime_presenter_upgrade_waits_for_readiness_before_releasing_fallback() {
    let source = include_str!("../lifecycle.rs");
    let function = source
        .split("fn try_upgrade_to_runtime_presenter")
        .nth(1)
        .and_then(|body| {
            body.split("pub(in crate::ui::retained_host::host_contract) fn sync_host_window_state")
                .next()
        })
        .expect("runtime presenter upgrade implementation");
    let readiness = function
        .find("factory.poll_ready()")
        .expect("upgrade must poll the runtime presenter factory without releasing fallback");
    let release = function
        .find("drop(self.presenter.take());")
        .expect("ready handoff must release the standalone native surface first");

    assert!(readiness < release);
    assert!(function.contains("runtime_presenter_upgrade_attempted"));
    assert!(function.contains("runtime_presenter_upgrade_poll_deadline"));
}

#[test]
fn pending_runtime_presenter_upgrade_participates_in_native_wait_policy() {
    let source = include_str!("../lifecycle.rs");
    let about_to_wait = source
        .split("pub(in crate::ui::retained_host::host_contract) fn about_to_wait_impl")
        .nth(1)
        .and_then(|body| body.split("fn schedule_due_surface_present_retry").next())
        .expect("about-to-wait implementation");

    assert!(about_to_wait.contains("runtime_presenter_upgrade_poll_deadline"));
    assert!(source.contains("RUNTIME_PRESENTER_UPGRADE_POLL_INTERVAL"));
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
        .and_then(|body| body.split("fn try_upgrade_to_runtime_presenter").next())
        .expect("pending presenter resize implementation");
    let consume_size = function
        .find("self.pending_presenter_resize.take()")
        .expect("pending size consumption");
    let presenter_gate = function
        .find("self.presenter.as_mut()")
        .expect("presenter availability gate");

    assert!(consume_size < presenter_gate);
}
