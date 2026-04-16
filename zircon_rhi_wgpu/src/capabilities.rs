use zircon_rhi::{AccelerationStructureCaps, RenderBackendCaps, RenderQueueClass};

pub fn wgpu_backend_caps(
    backend_name: impl Into<String>,
    _features: wgpu::Features,
    supports_surface: bool,
) -> RenderBackendCaps {
    RenderBackendCaps::new(backend_name)
        .with_queue(RenderQueueClass::Graphics)
        .with_queue(RenderQueueClass::Copy)
        .with_surface_support(supports_surface)
        .with_offscreen_support(true)
        .with_async_copy(true)
        .with_pipeline_cache(false)
        .with_acceleration_structures(AccelerationStructureCaps::disabled())
}
