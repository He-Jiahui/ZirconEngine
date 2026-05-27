use super::sources::{runtime_app_source, runtime_session_source, runtime_surface_present_source};

#[test]
fn runtime_preview_keeps_softbuffer_fallback_when_surface_api_is_optional() {
    let runtime_app_source = runtime_app_source();
    let runtime_surface_present_source = runtime_surface_present_source();
    let runtime_session_source = runtime_session_source();

    assert!(
        runtime_app_source.contains("surface_present_enabled: bool"),
        "runtime entry app should track whether optional surface present is active"
    );
    assert!(
        runtime_app_source.contains("presenter: Option<SoftbufferRuntimePresenter>"),
        "softbuffer presenter should remain available as the fallback path"
    );
    assert!(
        runtime_session_source.contains("return Ok(false);")
            && runtime_session_source.contains("bind_viewport_surface")
            && runtime_session_source.contains("present_viewport"),
        "runtime session optional surface wrappers should report unavailable APIs without error"
    );
    assert!(
        runtime_surface_present_source.contains("SoftbufferRuntimePresenter::new"),
        "runtime surface-present helper should still be able to create the softbuffer fallback presenter"
    );
    assert!(
        runtime_surface_present_source.contains("capture_frame"),
        "runtime redraw fallback should continue using capture_frame()"
    );
}
