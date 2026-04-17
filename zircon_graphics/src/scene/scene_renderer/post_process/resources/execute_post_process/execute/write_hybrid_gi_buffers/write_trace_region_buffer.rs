use super::super::super::super::super::hybrid_gi_trace_region_gpu::GpuHybridGiTraceRegion;
use super::super::super::super::super::scene_post_process_resources::ScenePostProcessResources;

pub(super) fn write_trace_region_buffer(
    resources: &ScenePostProcessResources,
    queue: &wgpu::Queue,
    trace_regions: &[GpuHybridGiTraceRegion],
) {
    queue.write_buffer(
        &resources.hybrid_gi_trace_region_buffer,
        0,
        bytemuck::cast_slice(trace_regions),
    );
}
