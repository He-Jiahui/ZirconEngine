use crate::core::framework::render::PostProcessGraphResourceNames;
use crate::core::framework::render::RenderHistoryCopyReport;
use crate::graphics::backend::OffscreenTarget;
use crate::graphics::scene::scene_renderer::graph_execution::RenderGraphExecutionResources;
use crate::graphics::scene::scene_renderer::history::SceneFrameHistoryTextures;

use super::super::super::super::post_process::SceneRuntimeFeatureFlags;
use super::super::super::scene_renderer_core::SceneRendererCore;
use super::super::super::target_extent::texture_extent;

impl SceneRendererCore {
    pub(in crate::graphics::scene::scene_renderer::core::scene_renderer_core_render_compiled_scene) fn copy_history_textures(
        &self,
        encoder: &mut wgpu::CommandEncoder,
        target: &OffscreenTarget,
        graph_resources: &RenderGraphExecutionResources,
        history_textures: Option<&mut SceneFrameHistoryTextures>,
        runtime_features: SceneRuntimeFeatureFlags,
        screen_space_reflection_history_enabled: bool,
    ) -> RenderHistoryCopyReport {
        let requested_copy_count = runtime_features.history_resolve_enabled as usize
            + runtime_features.hybrid_global_illumination_enabled as usize
            + runtime_features.ssao_enabled as usize
            + screen_space_reflection_history_enabled as usize;
        let history_target_present = history_textures.is_some();
        let mut scene_color_copied = false;
        let mut global_illumination_copied = false;
        let mut ambient_occlusion_copied = false;
        let mut screen_space_reflection_copied = false;

        if let Some(history) = history_textures {
            if runtime_features.history_resolve_enabled {
                encoder.copy_texture_to_texture(
                    target.scene_color.as_image_copy(),
                    history.scene_color.as_image_copy(),
                    texture_extent(target.size),
                );
                scene_color_copied = true;
            }
            if runtime_features.hybrid_global_illumination_enabled {
                encoder.copy_texture_to_texture(
                    target.global_illumination.as_image_copy(),
                    history.global_illumination.as_image_copy(),
                    texture_extent(target.size),
                );
                global_illumination_copied = true;
            }
            if runtime_features.ssao_enabled {
                encoder.copy_texture_to_texture(
                    target.ambient_occlusion.as_image_copy(),
                    history.ambient_occlusion.as_image_copy(),
                    texture_extent(target.size),
                );
                ambient_occlusion_copied = true;
            }
            if screen_space_reflection_history_enabled {
                if let Some(screen_space_reflection_history) = graph_resources
                    .owned_texture(PostProcessGraphResourceNames::SCREEN_SPACE_REFLECTION_HISTORY)
                {
                    encoder.copy_texture_to_texture(
                        screen_space_reflection_history.as_image_copy(),
                        history.screen_space_reflection.as_image_copy(),
                        texture_extent(target.size),
                    );
                    screen_space_reflection_copied = true;
                }
            }
        }
        RenderHistoryCopyReport::new(
            history_target_present,
            target.size,
            requested_copy_count,
            scene_color_copied,
            global_illumination_copied,
            ambient_occlusion_copied,
            screen_space_reflection_copied,
        )
    }
}
