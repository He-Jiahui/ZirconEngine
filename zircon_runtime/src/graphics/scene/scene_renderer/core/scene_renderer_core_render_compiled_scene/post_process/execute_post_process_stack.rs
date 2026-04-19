use crate::graphics::backend::OffscreenTarget;
use crate::graphics::scene::scene_renderer::history::SceneFrameHistoryTextures;
use crate::graphics::types::EditorOrRuntimeFrame;

use super::super::super::super::post_process::SceneRuntimeFeatureFlags;
use super::super::super::scene_renderer_core::SceneRendererCore;

impl SceneRendererCore {
    pub(in crate::graphics::scene::scene_renderer::core::scene_renderer_core_render_compiled_scene) fn execute_post_process_stack(
        &self,
        device: &wgpu::Device,
        queue: &wgpu::Queue,
        encoder: &mut wgpu::CommandEncoder,
        target: &mut OffscreenTarget,
        frame: &EditorOrRuntimeFrame,
        runtime_features: SceneRuntimeFeatureFlags,
        history_textures: Option<&SceneFrameHistoryTextures>,
        history_available: bool,
    ) {
        self.post_process.execute_ssao(
            device,
            queue,
            encoder,
            target.size,
            &target.depth_view,
            &target.normal_view,
            history_textures.map(|history| &history.ambient_occlusion_view),
            &target.ambient_occlusion_view,
            runtime_features.ssao_enabled,
            history_available,
        );
        self.post_process.execute_clustered_lighting(
            device,
            queue,
            encoder,
            target.size,
            target.cluster_dimensions,
            &target.cluster_buffer,
            target.cluster_buffer_bytes,
            &frame.extract.lighting.directional_lights,
            runtime_features.clustered_lighting_enabled,
        );
        self.post_process.execute_bloom(
            device,
            queue,
            encoder,
            target.size,
            &target.scene_color_view,
            &target.bloom_view,
            frame.extract.post_process.bloom,
            runtime_features.bloom_enabled,
        );
        self.post_process.execute_post_process(
            device,
            queue,
            encoder,
            target.size,
            target.cluster_dimensions,
            &target.scene_color_view,
            &target.ambient_occlusion_view,
            history_textures.map(|history| &history.scene_color_view),
            &target.bloom_view,
            &target.final_color_view,
            &target.cluster_buffer,
            frame,
            runtime_features,
            history_available,
        );
    }
}
