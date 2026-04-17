use super::super::super::super::super::hybrid_gi_probe_gpu::GpuHybridGiProbe;
use super::super::super::super::super::scene_post_process_resources::ScenePostProcessResources;

pub(super) fn write_probe_buffer(
    resources: &ScenePostProcessResources,
    queue: &wgpu::Queue,
    probes: &[GpuHybridGiProbe],
) {
    queue.write_buffer(
        &resources.hybrid_gi_probe_buffer,
        0,
        bytemuck::cast_slice(probes),
    );
}
