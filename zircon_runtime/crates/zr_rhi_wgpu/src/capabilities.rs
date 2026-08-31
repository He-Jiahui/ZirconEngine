use zr_rhi::{AccelerationStructureCaps, RenderBackendCaps, RenderQueueClass};

pub fn wgpu_backend_caps(
    backend_name: impl Into<String>,
    features: wgpu::Features,
    limits: wgpu::Limits,
    supports_surface: bool,
    supports_fragment_writable_storage: bool,
    supports_indirect_execution: bool,
) -> RenderBackendCaps {
    // WGPU serializes these command classes through one physical queue. They remain
    // admissible logical lanes, while the async flags below stay fail-closed.
    RenderBackendCaps::new(backend_name)
        .with_queue(RenderQueueClass::Graphics)
        .with_queue(RenderQueueClass::Compute)
        .with_queue(RenderQueueClass::Copy)
        .with_surface_support(supports_surface)
        .with_offscreen_support(true)
        .with_pipeline_cache(false)
        .with_gpu_timestamp(features.contains(
            wgpu::Features::TIMESTAMP_QUERY | wgpu::Features::TIMESTAMP_QUERY_INSIDE_ENCODERS,
        ))
        .with_subgroup(features.contains(wgpu::Features::SUBGROUP))
        .with_pipeline_statistics_query(
            features.contains(wgpu::Features::PIPELINE_STATISTICS_QUERY),
        )
        .with_storage_buffers(true)
        .with_fragment_writable_storage(supports_fragment_writable_storage)
        .with_max_storage_buffers_per_shader_stage(limits.max_storage_buffers_per_shader_stage)
        .with_max_storage_buffer_binding_size(u64::from(limits.max_storage_buffer_binding_size))
        .with_indirect_draw(supports_indirect_execution)
        // Both fixed-count forms require adapter indirect-execution support; the optional
        // feature is only required for the GPU-written count-buffer overload.
        .with_multi_draw_indirect(supports_indirect_execution)
        .with_multi_draw_indirect_count(
            supports_indirect_execution
                && features.contains(wgpu::Features::MULTI_DRAW_INDIRECT_COUNT),
        )
        .with_indirect_first_instance(
            supports_indirect_execution
                && features.contains(wgpu::Features::INDIRECT_FIRST_INSTANCE),
        )
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
    use zr_rhi::{RenderOperation, RenderOperationSupport, RenderQueueClass};

    #[test]
    fn timestamp_capability_requires_query_and_encoder_writes() {
        let full = wgpu_backend_caps(
            "full",
            wgpu::Features::TIMESTAMP_QUERY | wgpu::Features::TIMESTAMP_QUERY_INSIDE_ENCODERS,
            wgpu::Limits::default(),
            false,
            false,
            false,
        );
        let query_only = wgpu_backend_caps(
            "query-only",
            wgpu::Features::TIMESTAMP_QUERY,
            wgpu::Limits::default(),
            false,
            false,
            false,
        );

        assert!(full.supports_gpu_timestamp);
        assert!(!query_only.supports_gpu_timestamp);
    }

    #[test]
    fn optional_wgpu_feature_gates_report_negotiated_device_capabilities() {
        let caps = wgpu_backend_caps(
            "optional-features",
            wgpu::Features::SUBGROUP | wgpu::Features::PIPELINE_STATISTICS_QUERY,
            wgpu::Limits::default(),
            false,
            false,
            false,
        );

        assert!(caps.supports_subgroup);
        assert!(caps.supports_pipeline_statistics_query);
    }

    #[test]
    fn multi_draw_count_capability_is_distinct_from_the_fixed_count_fallback() {
        let without_count = wgpu_backend_caps(
            "fixed-count-only",
            wgpu::Features::empty(),
            wgpu::Limits::default(),
            false,
            false,
            true,
        );
        let with_count = wgpu_backend_caps(
            "count-enabled",
            wgpu::Features::MULTI_DRAW_INDIRECT_COUNT,
            wgpu::Limits::default(),
            false,
            false,
            true,
        );

        assert!(without_count.supports_multi_draw_indirect);
        assert!(!without_count.supports_multi_draw_indirect_count);
        assert!(with_count.supports_multi_draw_indirect_count);
    }

    #[test]
    fn indirect_capabilities_require_adapter_indirect_execution() {
        let caps = wgpu_backend_caps(
            "indirect-downlevel-missing",
            wgpu::Features::MULTI_DRAW_INDIRECT_COUNT | wgpu::Features::INDIRECT_FIRST_INSTANCE,
            wgpu::Limits::default(),
            false,
            false,
            false,
        );

        assert!(!caps.supports_indirect_draw);
        assert!(!caps.supports_multi_draw_indirect);
        assert!(!caps.supports_multi_draw_indirect_count);
        assert!(!caps.supports_indirect_first_instance);
    }

    #[test]
    fn neutral_operation_matrix_rejects_wgpu_features_without_a_neutral_command() {
        let caps = wgpu_backend_caps(
            "operation-contract",
            wgpu::Features::MULTI_DRAW_INDIRECT_COUNT,
            wgpu::Limits::default(),
            false,
            false,
            true,
        );

        assert!(caps.supports_queue(RenderQueueClass::Graphics));
        assert!(caps.supports_queue(RenderQueueClass::Compute));
        assert!(caps.supports_queue(RenderQueueClass::Copy));
        assert!(!caps.supports_async_compute);
        assert!(!caps.supports_async_copy);
        assert!(caps.supports_multi_draw_indirect_count);
        assert!(caps.supports_graphics_debugger_capture);

        for operation in RenderOperation::ALL {
            assert_eq!(
                caps.operation_support(operation),
                RenderOperationSupport::Unsupported,
                "{operation:?} requires the production neutral device introduced in M2"
            );
        }
    }
}
