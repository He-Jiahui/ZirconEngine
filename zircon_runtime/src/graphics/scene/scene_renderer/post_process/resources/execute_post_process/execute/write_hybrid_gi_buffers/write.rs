use crate::core::math::UVec2;

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
) -> (u32, u32) {
    let (hybrid_gi_probes, hybrid_gi_probe_count) =
        encode_hybrid_gi_probes(frame, viewport_size, hybrid_global_illumination_enabled);
    if hybrid_gi_probe_count > 0 {
        write_probe_buffer(
            resources,
            queue,
            &hybrid_gi_probes[..hybrid_gi_probe_count as usize],
        );
    }

    let (hybrid_gi_trace_regions, hybrid_gi_trace_region_count) =
        encode_hybrid_gi_trace_regions(frame, viewport_size, hybrid_global_illumination_enabled);
    if hybrid_gi_trace_region_count > 0 {
        write_trace_region_buffer(
            resources,
            queue,
            &hybrid_gi_trace_regions[..hybrid_gi_trace_region_count as usize],
        );
    }

    (hybrid_gi_probe_count, hybrid_gi_trace_region_count)
}

#[cfg(test)]
mod tests {
    #[test]
    fn hybrid_gi_buffer_uploads_are_count_bounded() {
        let source = include_str!("write.rs");
        let probe_guard = ["if hybrid_gi_probe_count", " > 0"].concat();
        let probe_slice = ["&hybrid_gi_probes[..", "hybrid_gi_probe_count as usize]"].concat();
        let trace_guard = ["if hybrid_gi_trace_region_count", " > 0"].concat();
        let trace_slice = [
            "&hybrid_gi_trace_regions[..",
            "hybrid_gi_trace_region_count as usize]",
        ]
        .concat();

        assert!(source.contains(&probe_guard));
        assert!(source.contains(&probe_slice));
        assert!(source.contains(&trace_guard));
        assert!(source.contains(&trace_slice));
    }
}
