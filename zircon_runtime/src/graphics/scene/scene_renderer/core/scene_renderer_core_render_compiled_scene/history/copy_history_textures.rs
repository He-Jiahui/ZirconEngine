use crate::graphics::backend::OffscreenTarget;
use crate::graphics::scene::scene_renderer::history::SceneFrameHistoryTextures;

use super::super::super::super::post_process::SceneRuntimeFeatureFlags;
use super::super::super::scene_renderer_core::SceneRendererCore;
use super::super::super::target_extent::texture_extent;

impl SceneRendererCore {
    pub(in crate::graphics::scene::scene_renderer::core::scene_renderer_core_render_compiled_scene) fn copy_history_textures(
        &self,
        encoder: &mut wgpu::CommandEncoder,
        target: &OffscreenTarget,
        history_textures: Option<&mut SceneFrameHistoryTextures>,
        runtime_features: SceneRuntimeFeatureFlags,
    ) {
        if let Some(history) = history_textures {
            if runtime_features.history_resolve_enabled {
                encoder.copy_texture_to_texture(
                    target.final_color.as_image_copy(),
                    history.scene_color.as_image_copy(),
                    texture_extent(target.size),
                );
            }
            if runtime_features.hybrid_global_illumination_enabled {
                encoder.copy_texture_to_texture(
                    target.global_illumination.as_image_copy(),
                    history.global_illumination.as_image_copy(),
                    texture_extent(target.size),
                );
            }
            if runtime_features.ssao_enabled {
                encoder.copy_texture_to_texture(
                    target.ambient_occlusion.as_image_copy(),
                    history.ambient_occlusion.as_image_copy(),
                    texture_extent(target.size),
                );
            }
        }
    }
}
