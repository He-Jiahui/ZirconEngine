use crate::core::math::UVec2;

use crate::graphics::scene::scene_renderer::HybridGiScenePrepareResourcesSnapshot;
use crate::graphics::types::ViewportRenderFrame;

use super::super::super::super::super::scene_post_process_resources::ScenePostProcessResources;
use super::super::super::encode_hybrid_gi_probes::encode_hybrid_gi_probes;
use super::super::super::encode_hybrid_gi_trace_regions::encode_hybrid_gi_trace_regions;
use super::write_probe_buffer::write_probe_buffer;
use super::write_trace_region_buffer::write_trace_region_buffer;

pub(in super::super) fn write_hybrid_gi_buffers(
    resources: &ScenePostProcessResources,
    queue: &wgpu::Queue,
    frame: &ViewportRenderFrame,
    viewport_size: UVec2,
    hybrid_global_illumination_enabled: bool,
    hybrid_gi_scene_prepare_resources: Option<&HybridGiScenePrepareResourcesSnapshot>,
) -> (u32, u32) {
    let (hybrid_gi_probes, hybrid_gi_probe_count, scheduled_trace_region_count) =
        encode_hybrid_gi_probes(
            frame,
            viewport_size,
            hybrid_global_illumination_enabled,
            hybrid_gi_scene_prepare_resources,
        );
    write_probe_buffer(resources, queue, &hybrid_gi_probes);

    let (hybrid_gi_trace_regions, hybrid_gi_trace_region_count) =
        encode_hybrid_gi_trace_regions(frame, viewport_size, hybrid_global_illumination_enabled);
    debug_assert_eq!(
        scheduled_trace_region_count, hybrid_gi_trace_region_count,
        "scheduled trace region count should match encoded trace region resources"
    );
    write_trace_region_buffer(resources, queue, &hybrid_gi_trace_regions);

    (hybrid_gi_probe_count, scheduled_trace_region_count)
}
