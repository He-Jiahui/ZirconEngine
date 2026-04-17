use crate::backend::OffscreenTarget;
use crate::scene::scene_renderer::history::SceneFrameHistoryTextures;

use super::super::super::post_process::SceneRuntimeFeatureFlags;
use super::super::scene_renderer_core::SceneRendererCore;
use super::super::target_extent::texture_extent;

impl SceneRendererCore {
    pub(super) fn copy_history_textures(
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
