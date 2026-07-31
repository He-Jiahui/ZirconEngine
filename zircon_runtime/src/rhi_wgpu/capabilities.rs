use crate::rhi::{AccelerationStructureCaps, RenderBackendCaps, RenderQueueClass};

pub fn wgpu_backend_caps(
    backend_name: impl Into<String>,
    features: wgpu::Features,
    limits: wgpu::Limits,
    supports_surface: bool,
    supports_fragment_writable_storage: bool,
) -> RenderBackendCaps {
    RenderBackendCaps::new(backend_name)
        .with_queue(RenderQueueClass::Graphics)
        .with_queue(RenderQueueClass::Compute)
        .with_queue(RenderQueueClass::Copy)
        .with_surface_support(supports_surface)
        .with_offscreen_support(true)
        .with_async_copy(true)
        .with_pipeline_cache(false)
        .with_gpu_timestamp(features.contains(
            wgpu::Features::TIMESTAMP_QUERY | wgpu::Features::TIMESTAMP_QUERY_INSIDE_ENCODERS,
        ))
        .with_storage_buffers(true)
        .with_fragment_writable_storage(supports_fragment_writable_storage)
        .with_max_storage_buffers_per_shader_stage(limits.max_storage_buffers_per_shader_stage)
        .with_max_storage_buffer_binding_size(u64::from(limits.max_storage_buffer_binding_size))
        .with_indirect_draw(true)
        .with_multi_draw_indirect(features.contains(wgpu::Features::MULTI_DRAW_INDIRECT_COUNT))
        .with_indirect_first_instance(features.contains(wgpu::Features::INDIRECT_FIRST_INSTANCE))
        .with_buffer_readback(true)
        .with_buffer_binding_array(features.contains(wgpu::Features::BUFFER_BINDING_ARRAY))
        .with_texture_binding_array(features.contains(wgpu::Features::TEXTURE_BINDING_ARRAY))
        .with_non_uniform_resource_indexing(features.contains(
            wgpu::Features::SAMPLED_TEXTURE_AND_STORAGE_BUFFER_ARRAY_NON_UNIFORM_INDEXING,
        ))
        .with_partially_bound_binding_array(
            features.contains(wgpu::Features::PARTIALLY_BOUND_BINDING_ARRAY),
        )
        .with_sparse_texture(false)
        .with_debug_markers(true)
        .with_debug_groups(true)
        .with_graphics_debugger_capture(true)
        .with_acceleration_structures(AccelerationStructureCaps::disabled())
}

#[cfg(test)]
mod tests {
    use super::wgpu_backend_caps;

    #[test]
    fn timestamp_capability_requires_query_and_encoder_writes() {
        let full = wgpu_backend_caps(
            "full",
            wgpu::Features::TIMESTAMP_QUERY | wgpu::Features::TIMESTAMP_QUERY_INSIDE_ENCODERS,
            wgpu::Limits::default(),
            false,
            false,
        );
        let query_only = wgpu_backend_caps(
            "query-only",
            wgpu::Features::TIMESTAMP_QUERY,
            wgpu::Limits::default(),
            false,
            false,
        );

        assert!(full.supports_gpu_timestamp);
        assert!(!query_only.supports_gpu_timestamp);
    }
}
