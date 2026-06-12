use crate::core::framework::render::PostProcessGraphResourceNames;
use crate::core::framework::render::RenderHistoryCopyReport;
use crate::core::math::UVec2;
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
        hzb_history_enabled: bool,
    ) -> RenderHistoryCopyReport {
        let requested_copy_count = runtime_features.history_resolve_enabled as usize
            + runtime_features.hybrid_global_illumination_enabled as usize
            + runtime_features.ssao_enabled as usize
            + screen_space_reflection_history_enabled as usize
            + hzb_history_enabled as usize;
        let history_target_present = history_textures.is_some();
        let mut scene_color_copied = false;
        let mut global_illumination_copied = false;
        let mut ambient_occlusion_copied = false;
        let mut screen_space_reflection_copied = false;
        let mut hzb_furthest_copied = false;

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
            if hzb_history_enabled {
                hzb_furthest_copied =
                    copy_hzb_furthest_mip_chain(encoder, graph_resources, history);
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
            hzb_furthest_copied,
        )
    }
}

fn copy_hzb_furthest_mip_chain(
    encoder: &mut wgpu::CommandEncoder,
    graph_resources: &RenderGraphExecutionResources,
    history: &SceneFrameHistoryTextures,
) -> bool {
    let Some(hzb_furthest) =
        graph_resources.owned_texture(PostProcessGraphResourceNames::HZB_FURTHEST)
    else {
        return false;
    };
    if graph_resources.owned_texture_mip_level_count(PostProcessGraphResourceNames::HZB_FURTHEST)
        != Some(history.hzb_furthest_mip_count)
    {
        return false;
    }
    for mip_level in 0..history.hzb_furthest_mip_count {
        encoder.copy_texture_to_texture(
            texture_mip_copy(hzb_furthest, mip_level),
            texture_mip_copy(&history.hzb_furthest, mip_level),
            mip_extent(history.hzb_furthest_size, mip_level),
        );
    }
    true
}

fn texture_mip_copy(texture: &wgpu::Texture, mip_level: u32) -> wgpu::TexelCopyTextureInfo<'_> {
    wgpu::TexelCopyTextureInfo {
        texture,
        mip_level,
        origin: wgpu::Origin3d::ZERO,
        aspect: wgpu::TextureAspect::All,
    }
}

fn mip_extent(size: UVec2, mip_level: u32) -> wgpu::Extent3d {
    wgpu::Extent3d {
        width: (size.x >> mip_level).max(1),
        height: (size.y >> mip_level).max(1),
        depth_or_array_layers: 1,
    }
}
