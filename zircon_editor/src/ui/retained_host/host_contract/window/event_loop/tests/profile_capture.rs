use super::*;

#[test]
fn successful_warmup_presents_request_a_quiescent_restart_exactly_once() {
    let mut warmup = UiProfileWarmupState::new(2);

    assert!(!warmup.measurement_active());
    assert!(!warmup.complete_present());
    assert!(!warmup.measurement_active());
    assert!(warmup.complete_present());
    assert!(warmup.restart_pending());
    assert!(!warmup.measurement_active());
    assert!(!warmup.complete_present());

    warmup.complete_restart(true);
    assert!(warmup.measurement_active());
    assert!(!warmup.complete_present());
    assert!(!warmup.restart_pending());
}

#[test]
fn zero_warmup_presents_measure_from_process_start() {
    let mut warmup = UiProfileWarmupState::new(0);

    assert!(warmup.measurement_active());
    assert!(!warmup.complete_present());
}

#[test]
fn failed_restart_never_opens_measurement_or_retries_the_transition() {
    let mut warmup = UiProfileWarmupState::new(1);

    assert!(warmup.complete_present());
    warmup.complete_restart(false);

    assert!(!warmup.measurement_active());
    assert!(!warmup.restart_pending());
    assert!(!warmup.complete_present());
}

#[test]
fn recorder_restart_is_owned_by_about_to_wait_not_present() {
    let present = include_str!("../redraw/present.rs");
    let present_production = present
        .split("#[cfg(test)]")
        .next()
        .expect("present production implementation");
    let lifecycle = include_str!("../lifecycle.rs");

    assert!(!present_production.contains("reset_capture"));
    assert!(!present_production.contains("start_capture_from_env"));
    assert!(lifecycle.contains("self.restart_profile_measurement_if_ready();"));
}

#[test]
fn measurement_readiness_is_published_only_after_capture_restart() {
    let source = include_str!("../profile_capture.rs");
    let restart = source
        .split("pub(super) fn restart_profile_measurement_if_ready")
        .nth(1)
        .and_then(|body| body.split("#[cfg(test)]").next())
        .expect("profile restart implementation");
    let capture = restart
        .find("start_capture_from_env")
        .expect("capture restart");
    let ready = restart
        .find("publish_profile_measurement_ready")
        .expect("measurement readiness publication");
    let present = include_str!("../redraw/present.rs")
        .split("#[cfg(test)]")
        .next()
        .expect("present production implementation");

    assert!(capture < ready);
    assert!(!present.contains("ui_profile_measurement_ready"));
}
