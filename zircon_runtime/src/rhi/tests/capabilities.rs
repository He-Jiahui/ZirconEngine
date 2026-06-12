use crate::rhi::{
    AccelerationStructureCaps, RenderBackendCaps, RenderDebugInstrumentationStatus,
    RenderQueueClass,
};

#[test]
fn backend_caps_report_queue_classes_and_rt_support_independently() {
    let caps = RenderBackendCaps::new("test-backend")
        .with_queue(RenderQueueClass::Graphics)
        .with_queue(RenderQueueClass::Compute)
        .with_surface_support(true)
        .with_pipeline_cache(true)
        .with_storage_buffers(true)
        .with_indirect_draw(true)
        .with_multi_draw_indirect(true)
        .with_indirect_first_instance(true)
        .with_buffer_readback(true)
        .with_neural_compute(true)
        .with_sparse_texture(true)
        .with_debug_markers(true)
        .with_debug_groups(true)
        .with_graphics_debugger_capture(true)
        .with_acceleration_structures(AccelerationStructureCaps::disabled());

    assert!(caps.supports_queue(RenderQueueClass::Graphics));
    assert!(caps.supports_queue(RenderQueueClass::Compute));
    assert!(!caps.supports_queue(RenderQueueClass::Copy));
    assert!(caps.supports_surface);
    assert!(caps.supports_pipeline_cache);
    assert!(caps.supports_storage_buffers);
    assert!(caps.supports_indirect_draw);
    assert!(caps.supports_multi_draw_indirect);
    assert!(caps.supports_indirect_first_instance);
    assert!(caps.supports_buffer_readback);
    assert!(caps.supports_neural_compute);
    assert!(caps.supports_sparse_texture);
    assert!(caps.supports_debug_markers);
    assert!(caps.supports_debug_groups);
    assert!(caps.supports_graphics_debugger_capture);
    assert!(!caps.acceleration_structures.supported);
}

#[test]
fn backend_debug_instrumentation_status_is_derived_from_caps() {
    let caps = RenderBackendCaps::new("instrumented-test")
        .with_debug_markers(true)
        .with_debug_groups(true)
        .with_graphics_debugger_capture(true);

    let status = RenderDebugInstrumentationStatus::from_caps(&caps);

    assert_eq!(status.backend_name, "instrumented-test");
    assert!(status.debug_markers_supported);
    assert!(status.debug_groups_supported);
    assert!(status.graphics_debugger_capture_supported);
    assert!(!status.active_graphics_debugger_capture);
    assert_eq!(status.last_error, None);

    assert_eq!(
        RenderDebugInstrumentationStatus::unavailable("offline"),
        RenderDebugInstrumentationStatus {
            backend_name: "offline".to_string(),
            debug_markers_supported: false,
            debug_groups_supported: false,
            graphics_debugger_capture_supported: false,
            active_graphics_debugger_capture: false,
            last_error: None,
        }
    );
}
