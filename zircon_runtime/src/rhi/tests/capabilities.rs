use crate::rhi::{AccelerationStructureCaps, RenderBackendCaps, RenderQueueClass};

#[test]
fn backend_caps_report_queue_classes_and_rt_support_independently() {
    let caps = RenderBackendCaps::new("test-backend")
        .with_queue(RenderQueueClass::Graphics)
        .with_queue(RenderQueueClass::Compute)
        .with_surface_support(true)
        .with_pipeline_cache(true)
        .with_acceleration_structures(AccelerationStructureCaps::disabled());

    assert!(caps.supports_queue(RenderQueueClass::Graphics));
    assert!(caps.supports_queue(RenderQueueClass::Compute));
    assert!(!caps.supports_queue(RenderQueueClass::Copy));
    assert!(caps.supports_surface);
    assert!(caps.supports_pipeline_cache);
    assert!(!caps.acceleration_structures.supported);
}
