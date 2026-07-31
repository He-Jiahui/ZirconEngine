use crate::core::framework::render::RenderFrameExtract;
use crate::core::math::UVec2;

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
    if reflection_probe_count > 0 {
        queue.write_buffer(
            &resources.reflection_probe_buffer,
            0,
            bytemuck::cast_slice(&reflection_probes[..reflection_probe_count as usize]),
        );
    }
    reflection_probe_count
}

#[cfg(test)]
mod tests {
    #[test]
    fn reflection_probe_upload_is_count_bounded() {
        let source = include_str!("write.rs");
        let upload_guard = ["if reflection_probe_count", " > 0"].concat();
        let active_slice = ["&reflection_probes[..", "reflection_probe_count as usize]"].concat();

        assert!(source.contains(&upload_guard));
        assert!(source.contains(&active_slice));
    }
}
