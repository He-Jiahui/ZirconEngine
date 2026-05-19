use crate::rhi::{AccelerationStructureCaps, RenderBackendCaps, RenderQueueClass};

pub fn wgpu_backend_caps(
    backend_name: impl Into<String>,
    features: wgpu::Features,
    supports_surface: bool,
) -> RenderBackendCaps {
    RenderBackendCaps::new(backend_name)
        .with_queue(RenderQueueClass::Graphics)
        .with_queue(RenderQueueClass::Copy)
        .with_surface_support(supports_surface)
        .with_offscreen_support(true)
        .with_async_copy(true)
        .with_pipeline_cache(false)
        .with_storage_buffers(true)
        .with_indirect_draw(true)
        .with_buffer_readback(true)
        .with_buffer_binding_array(features.contains(wgpu::Features::BUFFER_BINDING_ARRAY))
        .with_texture_binding_array(features.contains(wgpu::Features::TEXTURE_BINDING_ARRAY))
        .with_non_uniform_resource_indexing(features.contains(
            wgpu::Features::SAMPLED_TEXTURE_AND_STORAGE_BUFFER_ARRAY_NON_UNIFORM_INDEXING,
        ))
        .with_partially_bound_binding_array(
            features.contains(wgpu::Features::PARTIALLY_BOUND_BINDING_ARRAY),
        )
        .with_acceleration_structures(AccelerationStructureCaps::disabled())
}
