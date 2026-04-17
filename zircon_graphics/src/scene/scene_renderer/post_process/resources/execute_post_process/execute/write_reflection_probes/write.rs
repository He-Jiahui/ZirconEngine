use zircon_math::UVec2;
use zircon_scene::RenderFrameExtract;

use super::super::super::super::super::scene_post_process_resources::ScenePostProcessResources;
use super::super::super::encode_reflection_probes::encode_reflection_probes;

pub(in super::super) fn write_reflection_probes(
    resources: &ScenePostProcessResources,
    queue: &wgpu::Queue,
    extract: &RenderFrameExtract,
    viewport_size: UVec2,
    reflection_probes_enabled: bool,
) -> u32 {
    let (reflection_probes, reflection_probe_count) =
        encode_reflection_probes(extract, viewport_size, reflection_probes_enabled);
    queue.write_buffer(
        &resources.reflection_probe_buffer,
        0,
        bytemuck::cast_slice(&reflection_probes),
    );
    reflection_probe_count
}
