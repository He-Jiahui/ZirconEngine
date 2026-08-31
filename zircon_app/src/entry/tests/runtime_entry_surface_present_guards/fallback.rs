use super::sources::{runtime_app_source, runtime_session_source, runtime_surface_present_source};

#[test]
fn runtime_preview_admits_reference_cpu_presenter_only_after_explicit_opt_in() {
    let runtime_app_source = runtime_app_source();
    let runtime_surface_present_source = runtime_surface_present_source();
    let runtime_session_source = runtime_session_source();
    let reference_cpu_presenter_source = include_str!("../../../reference_cpu_presenter.rs");

    assert!(
        runtime_app_source.contains("surface_present_enabled: bool"),
        "runtime entry app should track whether optional surface present is active"
    );
    assert!(
        runtime_app_source.contains("presenter: Option<ReferenceCpuPresenter>"),
        "reference CPU presenter should remain a distinct app-owned degraded path"
    );
    assert!(
        runtime_session_source.contains("return Ok(false);")
            && runtime_session_source.contains("bind_viewport_surface")
            && runtime_session_source.contains("present_viewport"),
        "runtime session dynamic wrappers should retain ABI-level unavailable reports"
    );
    assert!(
        runtime_surface_present_source.contains("ReferenceCpuPresenter::new"),
        "runtime surface-present helper should construct the explicitly selected reference CPU presenter"
    );
    assert!(
        runtime_surface_present_source.contains("capture_frame"),
        "reference CPU presenter should continue using runtime-owned capture_frame() output"
    );
    assert!(
        runtime_surface_present_source.contains("reference_cpu_presenter_enabled")
            && runtime_surface_present_source.contains("runtime_reference_cpu_presenter_enabled")
            && !runtime_surface_present_source.contains("ZR_RUNTIME_FORCE_CAPTURE_PRESENT"),
        "reference CPU presentation must remain a named opt-in instead of an ambient capture override"
    );
    for metric in [
        "reference_cpu_presenter.copy_bytes",
        "reference_cpu_presenter.latency_micros",
        "reference_cpu_presenter.dropped_frames",
    ] {
        assert!(
            reference_cpu_presenter_source.contains(metric),
            "reference CPU presentation must publish `{metric}`"
        );
    }
}
