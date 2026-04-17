use zircon_render_server::RenderCapabilitySummary;

use super::super::queue_capability::queue_capability;
use super::flagship_baseline_supported::flagship_baseline_supported;

pub(in crate::runtime::server) fn capability_summary(
    caps: &zircon_rhi::RenderBackendCaps,
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
        acceleration_structures_supported: caps.acceleration_structures.supported,
        inline_ray_query: caps.acceleration_structures.inline_ray_query,
        ray_tracing_pipeline: caps.acceleration_structures.ray_tracing_pipeline,
        virtual_geometry_supported: flagship_baseline_supported,
        hybrid_global_illumination_supported: flagship_baseline_supported,
    }
}
