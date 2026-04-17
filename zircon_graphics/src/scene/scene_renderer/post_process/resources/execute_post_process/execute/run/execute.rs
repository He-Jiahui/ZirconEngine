use zircon_math::UVec2;

use crate::types::EditorOrRuntimeFrame;

use super::super::super::super::super::scene_post_process_resources::ScenePostProcessResources;
use super::super::super::super::super::scene_runtime_feature_flags::SceneRuntimeFeatureFlags;
use super::super::build_post_process_params::build_post_process_params;
use super::super::create_bind_group::create_bind_group;
use super::super::write_hybrid_gi_buffers::write_hybrid_gi_buffers;
use super::super::write_reflection_probes::write_reflection_probes;
use super::queue_post_process_params::queue_post_process_params;
use super::record_pass::record_pass;

impl ScenePostProcessResources {
    #[allow(clippy::too_many_arguments)]
    pub(crate) fn execute_post_process(
        &self,
        device: &wgpu::Device,
        queue: &wgpu::Queue,
        encoder: &mut wgpu::CommandEncoder,
        viewport_size: UVec2,
        cluster_dimensions: UVec2,
        scene_color_view: &wgpu::TextureView,
        ao_view: &wgpu::TextureView,
        previous_scene_color_view: Option<&wgpu::TextureView>,
        bloom_view: &wgpu::TextureView,
        final_color_view: &wgpu::TextureView,
        cluster_buffer: &wgpu::Buffer,
        frame: &EditorOrRuntimeFrame,
        features: SceneRuntimeFeatureFlags,
        history_available: bool,
    ) {
        let extract = &frame.extract;
        let reflection_probe_count = write_reflection_probes(
            self,
            queue,
            extract,
            viewport_size,
            features.reflection_probes_enabled,
        );
        let (hybrid_gi_probe_count, scheduled_trace_region_count) = write_hybrid_gi_buffers(
            self,
            queue,
            frame,
            viewport_size,
            features.hybrid_global_illumination_enabled,
        );
        let params = build_post_process_params(
            viewport_size,
            cluster_dimensions,
            extract,
            features,
            history_available,
            reflection_probe_count,
            hybrid_gi_probe_count,
            scheduled_trace_region_count,
        );
        queue_post_process_params(self, queue, &params);

        let bind_group = create_bind_group(
            self,
            device,
            scene_color_view,
            ao_view,
            previous_scene_color_view,
            bloom_view,
            cluster_buffer,
        );
        record_pass(self, encoder, final_color_view, &bind_group);
    }
}
