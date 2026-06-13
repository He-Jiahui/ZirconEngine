use crate::core::framework::render::RenderStats;

use super::{record_bool, record_count, DiagnosticStore};

pub(super) fn record(store: &mut DiagnosticStore, stats: &RenderStats) {
    let frame_index = stats.submitted_frames;
    let capabilities = &stats.capabilities;
    record_count(
        store,
        "render.capability.queue_class_count",
        frame_index,
        capabilities.queue_classes.len(),
        &["render", "capability", "queue"],
    );
    record_bool(
        store,
        "render.capability.surface_supported",
        frame_index,
        capabilities.supports_surface,
        &["render", "capability", "surface"],
    );
    record_bool(
        store,
        "render.capability.offscreen_supported",
        frame_index,
        capabilities.supports_offscreen,
        &["render", "capability", "offscreen"],
    );
    record_bool(
        store,
        "render.capability.async_compute_supported",
        frame_index,
        capabilities.supports_async_compute,
        &["render", "capability", "async_compute"],
    );
    record_bool(
        store,
        "render.capability.async_copy_supported",
        frame_index,
        capabilities.supports_async_copy,
        &["render", "capability", "async_copy"],
    );
    record_bool(
        store,
        "render.capability.pipeline_cache_supported",
        frame_index,
        capabilities.supports_pipeline_cache,
        &["render", "capability", "pipeline_cache"],
    );
    record_bool(
        store,
        "render.capability.storage_buffer_supported",
        frame_index,
        capabilities.supports_storage_buffers,
        &["render", "capability", "storage_buffer"],
    );
    record_count(
        store,
        "render.capability.max_storage_buffers_per_shader_stage",
        frame_index,
        capabilities.max_storage_buffers_per_shader_stage as usize,
        &["render", "capability", "storage_buffer"],
    );
    record_bool(
        store,
        "render.capability.indirect_draw_supported",
        frame_index,
        capabilities.supports_indirect_draw,
        &["render", "capability", "indirect_draw"],
    );
    record_bool(
        store,
        "render.capability.buffer_readback_supported",
        frame_index,
        capabilities.supports_buffer_readback,
        &["render", "capability", "readback"],
    );
    record_bool(
        store,
        "render.capability.acceleration_structure_supported",
        frame_index,
        capabilities.acceleration_structures_supported,
        &["render", "capability", "raytracing"],
    );
    record_bool(
        store,
        "render.capability.inline_ray_query_supported",
        frame_index,
        capabilities.inline_ray_query,
        &["render", "capability", "raytracing"],
    );
    record_bool(
        store,
        "render.capability.ray_tracing_pipeline_supported",
        frame_index,
        capabilities.ray_tracing_pipeline,
        &["render", "capability", "raytracing"],
    );
    record_bool(
        store,
        "render.capability.buffer_binding_array_supported",
        frame_index,
        capabilities.supports_buffer_binding_array,
        &["render", "capability", "binding_array"],
    );
    record_bool(
        store,
        "render.capability.texture_binding_array_supported",
        frame_index,
        capabilities.supports_texture_binding_array,
        &["render", "capability", "binding_array"],
    );
    record_bool(
        store,
        "render.capability.non_uniform_resource_indexing_supported",
        frame_index,
        capabilities.supports_non_uniform_resource_indexing,
        &["render", "capability", "resource_indexing"],
    );
    record_bool(
        store,
        "render.capability.partially_bound_binding_array_supported",
        frame_index,
        capabilities.supports_partially_bound_binding_array,
        &["render", "capability", "binding_array"],
    );
    record_bool(
        store,
        "render.capability.fxaa_supported",
        frame_index,
        capabilities.supports_fxaa,
        &["render", "capability", "anti_alias"],
    );
    record_bool(
        store,
        "render.capability.smaa_supported",
        frame_index,
        capabilities.supports_smaa,
        &["render", "capability", "anti_alias"],
    );
    record_bool(
        store,
        "render.capability.taa_supported",
        frame_index,
        capabilities.supports_taa,
        &["render", "capability", "anti_alias"],
    );
    record_bool(
        store,
        "render.capability.cas_supported",
        frame_index,
        capabilities.supports_cas,
        &["render", "capability", "anti_alias"],
    );
    record_bool(
        store,
        "render.capability.dlss_supported",
        frame_index,
        capabilities.supports_dlss,
        &["render", "capability", "anti_alias"],
    );
    record_bool(
        store,
        "render.capability.neural_compute_supported",
        frame_index,
        capabilities.supports_neural_compute,
        &["render", "capability", "neural_compute"],
    );
    record_bool(
        store,
        "render.capability.sparse_texture_supported",
        frame_index,
        capabilities.supports_sparse_texture,
        &["render", "capability", "sparse_texture"],
    );
    record_count(
        store,
        "render.capability.max_msaa_samples",
        frame_index,
        capabilities.max_supported_msaa_samples as usize,
        &["render", "capability", "anti_alias"],
    );
    record_bool(
        store,
        "render.capability.virtual_geometry_supported",
        frame_index,
        capabilities.virtual_geometry_supported,
        &["render", "capability", "virtual_geometry"],
    );
    record_bool(
        store,
        "render.capability.hybrid_gi_supported",
        frame_index,
        capabilities.hybrid_global_illumination_supported,
        &["render", "capability", "hybrid_gi"],
    );
}
