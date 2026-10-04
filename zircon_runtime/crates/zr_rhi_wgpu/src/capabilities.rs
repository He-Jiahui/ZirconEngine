//! 将已启用的 WGPU feature 与实际 device limits 转成中立能力回执。
//! 队列类别是逻辑通道，同一设备共享一条原生队列；调度器不能据此假设异步并行。

use zr_rhi::{AccelerationStructureCaps, RenderBackendCaps, RenderQueueClass};

/// 传入设备已启用的 feature，并用适配器事实补充无法从 feature 单独推导的操作支持。
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
#[path = "tests/capabilities_unit.rs"]
mod tests;
