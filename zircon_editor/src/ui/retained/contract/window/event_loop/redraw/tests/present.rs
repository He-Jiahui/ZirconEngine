use super::{first_presented_frame_diagnostic, retry_present_request};
use crate::ui::retained_host::host_contract::data::FrameRect;
use crate::ui::retained_host::host_contract::redraw::HostRedrawRequest;
use crate::ui::retained_host::ui_perf::UiPerfScenario;

#[test]
fn first_frame_exit_emits_a_presented_frame_diagnostic() {
    assert_eq!(
        first_presented_frame_diagnostic(true),
        Some("editor_first_frame_presented")
    );
}

#[test]
fn continuous_editor_does_not_emit_a_one_shot_presented_frame_diagnostic() {
    assert_eq!(first_presented_frame_diagnostic(false), None);
}

#[test]
fn retryable_surface_present_requeues_the_same_present_without_a_frame_update() {
    let damage = FrameRect {
        x: 3.0,
        y: 4.0,
        width: 20.0,
        height: 12.0,
    };
    let region = retry_present_request(
        UiPerfScenario::IdleHover,
        HostRedrawRequest::region_for_scenario_with_frame_update(
            UiPerfScenario::Click,
            damage.clone(),
            true,
        ),
    );
    assert!(region.request_redraw());
    assert!(region.requires_present());
    assert!(!region.requires_frame_update());
    assert_eq!(region.damage_region(), Some(&damage));
    assert_eq!(region.scenario(), UiPerfScenario::IdleHover);

    let full = retry_present_request(
        UiPerfScenario::WindowResize,
        HostRedrawRequest::full_frame_for_scenario(UiPerfScenario::Click, true),
    );
    assert!(full.request_redraw());
    assert!(full.requires_present());
    assert!(!full.requires_frame_update());
    assert_eq!(full.damage_region(), None);
    assert_eq!(full.scenario(), UiPerfScenario::WindowResize);
}

#[test]
fn retryable_surface_present_preserves_bounded_damage_pressure() {
    let redraw = HostRedrawRequest::region(FrameRect {
        x: 0.0,
        y: 0.0,
        width: 10.0,
        height: 10.0,
    })
    .merge(HostRedrawRequest::region(FrameRect {
        x: 20.0,
        y: 0.0,
        width: 10.0,
        height: 10.0,
    }))
    .merge(HostRedrawRequest::region(FrameRect {
        x: 100.0,
        y: 0.0,
        width: 10.0,
        height: 10.0,
    }))
    .merge(HostRedrawRequest::region_with_frame_update(FrameRect {
        x: 32.0,
        y: 0.0,
        width: 8.0,
        height: 10.0,
    }));
    let expected_damage = redraw.damage_region().cloned();
    let expected_metrics = redraw.damage_region_metrics();

    let retry = retry_present_request(UiPerfScenario::IdleHover, redraw);

    assert!(!retry.requires_frame_update());
    assert_eq!(retry.damage_region(), expected_damage.as_ref());
    assert_eq!(retry.damage_region_metrics(), expected_metrics);
    assert_eq!(retry.scenario(), UiPerfScenario::IdleHover);
}

#[test]
fn successful_present_consumes_input_batch_but_retry_retains_it() {
    let source = include_str!("../present.rs");
    let success = source
        .split("Ok(diagnostics) =>")
        .nth(1)
        .and_then(|source| {
            source
                .split("Err(HostPresenterError::RetryableSurfacePresent)")
                .next()
        })
        .expect("successful present branch");
    let retry = source
        .split("Err(HostPresenterError::RetryableSurfacePresent) =>")
        .nth(1)
        .and_then(|source| source.split("Err(error) =>").next())
        .expect("retryable present branch");

    assert!(success.contains("record_presented_input_batch(scenario)"));
    assert!(!retry.contains("record_presented_input_batch"));
}

#[test]
fn first_present_notification_is_emitted_only_from_the_successful_present_branch() {
    let source = include_str!("../present.rs");
    let success = source
        .split("Ok(diagnostics) =>")
        .nth(1)
        .and_then(|source| {
            source
                .split("Err(HostPresenterError::RetryableSurfacePresent)")
                .next()
        })
        .expect("successful present branch");
    let retry = source
        .split("Err(HostPresenterError::RetryableSurfacePresent) =>")
        .nth(1)
        .and_then(|source| source.split("Err(error) =>").next())
        .expect("retryable present branch");

    assert!(success.contains("notify_first_presented()"));
    assert!(!retry.contains("notify_first_presented"));
}

#[test]
fn warmup_exports_source_bound_geometry_before_requesting_measurement_restart() {
    let source = include_str!("../present.rs");
    let success = source
        .split("Ok(diagnostics) =>")
        .nth(1)
        .and_then(|source| {
            source
                .split("Err(HostPresenterError::RetryableSurfacePresent)")
                .next()
        })
        .expect("successful present branch");
    let artifacts = success
        .find("submit_present_artifacts(")
        .expect("warmup must publish source-bound geometry");
    let measurement = success
        .find("if profile_measurement_active")
        .expect("successful present must gate measured counters");
    let warmup_complete = success
        .find("complete_profile_warmup_present()")
        .expect("warmup completion must request a quiescent recorder restart");

    assert!(artifacts < measurement);
    assert!(measurement < warmup_complete);
    assert!(!success.contains("reset_capture"));
    assert!(!success.contains("start_capture_from_env"));
}
