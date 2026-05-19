use crate::core::framework::render::RenderCapabilitySummary;

use super::super::queue_capability::queue_capability;
use super::flagship_baseline_supported::flagship_baseline_supported;

pub(in crate::graphics::runtime::render_framework) fn capability_summary(
    caps: &crate::rhi::RenderBackendCaps,
) -> RenderCapabilitySummary {
    let flagship_baseline_supported = flagship_baseline_supported(caps);

    RenderCapabilitySummary {
        backend_name: caps.backend_name.clone(),
        queue_classes: caps
            .queue_classes
            .iter()
            .copied()
            .map(queue_capability)
            .collect(),
        supports_surface: caps.supports_surface,
        supports_offscreen: caps.supports_offscreen,
        supports_async_compute: caps.supports_async_compute,
        supports_async_copy: caps.supports_async_copy,
        supports_pipeline_cache: caps.supports_pipeline_cache,
        supports_storage_buffers: caps.supports_storage_buffers,
        supports_indirect_draw: caps.supports_indirect_draw,
        supports_buffer_readback: caps.supports_buffer_readback,
        acceleration_structures_supported: caps.acceleration_structures.supported,
        inline_ray_query: caps.acceleration_structures.inline_ray_query,
        ray_tracing_pipeline: caps.acceleration_structures.ray_tracing_pipeline,
        supports_buffer_binding_array: caps.supports_buffer_binding_array,
        supports_texture_binding_array: caps.supports_texture_binding_array,
        supports_non_uniform_resource_indexing: caps.supports_non_uniform_resource_indexing,
        supports_partially_bound_binding_array: caps.supports_partially_bound_binding_array,
        supports_fxaa: caps.supports_offscreen,
        supports_smaa: false,
        supports_taa: false,
        supports_cas: false,
        supports_dlss: false,
        max_supported_msaa_samples: 1,
        virtual_geometry_supported: flagship_baseline_supported,
        hybrid_global_illumination_supported: flagship_baseline_supported,
    }
}
