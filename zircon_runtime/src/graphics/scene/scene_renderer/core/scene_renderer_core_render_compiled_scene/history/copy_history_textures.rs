use crate::core::framework::render::PostProcessGraphResourceNames;
use crate::core::framework::render::RenderHistoryCopyReport;
use crate::core::math::UVec2;
use crate::graphics::backend::OffscreenTarget;
use crate::graphics::scene::scene_renderer::graph_execution::RenderGraphExecutionResources;
use crate::graphics::scene::scene_renderer::history::SceneFrameHistoryTextures;
use crate::graphics::types::ViewportRenderRegion;

use super::super::super::super::post_process::SceneRuntimeFeatureFlags;
use super::super::super::scene_renderer_core::SceneRendererCore;
use super::super::super::target_extent::texture_extent;

impl SceneRendererCore {
    pub(in crate::graphics::scene::scene_renderer::core::scene_renderer_core_render_compiled_scene) fn copy_history_textures(
        &self,
        encoder: &mut wgpu::CommandEncoder,
        target: &OffscreenTarget,
        render_region: ViewportRenderRegion,
        graph_resources: &RenderGraphExecutionResources,
        history_textures: Option<&mut SceneFrameHistoryTextures>,
        runtime_features: SceneRuntimeFeatureFlags,
        taa_history_enabled: bool,
        screen_space_reflection_history_enabled: bool,
        hzb_history_enabled: bool,
        exposure_history_enabled: bool,
    ) -> RenderHistoryCopyReport {
        let requested_copy_count = taa_history_enabled as usize
            + runtime_features.hybrid_global_illumination_enabled as usize
            + runtime_features.ssao_enabled as usize
            + screen_space_reflection_history_enabled as usize
            + hzb_history_enabled as usize
            + exposure_history_enabled as usize;
        let history_target_present = history_textures.is_some();
        let mut scene_color_copied = false;
        let mut global_illumination_copied = false;
        let mut ambient_occlusion_copied = false;
        let mut screen_space_reflection_copied = false;
        let mut hzb_furthest_copied = false;
        let mut exposure_copied = false;

        if let Some(history) = history_textures {
            if taa_history_enabled {
                history.flip_taa_scene_color_history();
                scene_color_copied = true;
            }
            if runtime_features.hybrid_global_illumination_enabled {
                global_illumination_copied = copy_global_illumination_history(
                    encoder,
                    target,
                    render_region,
                    graph_resources,
                    history,
                );
            }
            if runtime_features.ssao_enabled {
                encoder.copy_texture_to_texture(
                    target.ambient_occlusion.as_image_copy(),
                    history.ambient_occlusion.as_image_copy(),
                    texture_extent(target.render_size),
                );
                ambient_occlusion_copied = true;
            }
            if screen_space_reflection_history_enabled {
                if let Some((screen_space_reflection_history, extent)) =
                    screen_space_reflection_history_copy_extent(graph_resources, target.render_size)
                {
                    encoder.copy_texture_to_texture(
                        screen_space_reflection_history.as_image_copy(),
                        history.screen_space_reflection.as_image_copy(),
                        extent,
                    );
                    screen_space_reflection_copied = true;
                }
            }
            if hzb_history_enabled {
                hzb_furthest_copied =
                    copy_hzb_furthest_mip_chain(encoder, graph_resources, history);
            }
            if exposure_history_enabled {
                history.flip_exposure_history();
                exposure_copied = true;
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
            exposure_copied,
        )
    }
}

fn copy_global_illumination_history(
    encoder: &mut wgpu::CommandEncoder,
    target: &OffscreenTarget,
    render_region: ViewportRenderRegion,
    graph_resources: &RenderGraphExecutionResources,
    history: &SceneFrameHistoryTextures,
) -> bool {
    if let (Some(global_illumination), Some(desc)) = (
        graph_resources.owned_texture(PostProcessGraphResourceNames::GLOBAL_ILLUMINATION),
        graph_resources.owned_texture_desc(PostProcessGraphResourceNames::GLOBAL_ILLUMINATION),
    ) {
        if let Some(extent) = history_region_copy_extent(
            UVec2::new(desc.width, desc.height),
            target.size,
            render_region,
        ) {
            let mut destination = history.global_illumination.as_image_copy();
            destination.origin = history_region_copy_origin(render_region);
            encoder.copy_texture_to_texture(
                global_illumination.as_image_copy(),
                destination,
                extent,
            );
            return true;
        }
    }

    encoder.copy_texture_to_texture(
        target.global_illumination.as_image_copy(),
        history.global_illumination.as_image_copy(),
        texture_extent(target.render_size),
    );
    true
}

fn screen_space_reflection_history_copy_extent(
    graph_resources: &RenderGraphExecutionResources,
    fallback_size: UVec2,
) -> Option<(&wgpu::Texture, wgpu::Extent3d)> {
    let texture = graph_resources
        .owned_texture(PostProcessGraphResourceNames::SCREEN_SPACE_REFLECTION_HISTORY)?;
    let size = graph_resources
        .owned_texture_desc(PostProcessGraphResourceNames::SCREEN_SPACE_REFLECTION_HISTORY)
        .map(|desc| UVec2::new(desc.width, desc.height))
        .unwrap_or(fallback_size);
    Some((texture, texture_extent(size)))
}

fn history_region_copy_extent(
    source_size: UVec2,
    target_size: UVec2,
    render_region: ViewportRenderRegion,
) -> Option<wgpu::Extent3d> {
    let origin = render_region.physical_position();
    let region_size = render_region.physical_size();
    let local_size = render_region.local_size();
    let available_size = UVec2::new(
        target_size.x.saturating_sub(origin.x).min(region_size.x),
        target_size.y.saturating_sub(origin.y).min(region_size.y),
    );
    let copy_size = UVec2::new(
        source_size.x.min(local_size.x).min(available_size.x),
        source_size.y.min(local_size.y).min(available_size.y),
    );
    if copy_size.x == 0 || copy_size.y == 0 {
        return None;
    }
    Some(wgpu::Extent3d {
        width: copy_size.x,
        height: copy_size.y,
        depth_or_array_layers: 1,
    })
}

fn history_region_copy_origin(render_region: ViewportRenderRegion) -> wgpu::Origin3d {
    let origin = render_region.physical_position();
    wgpu::Origin3d {
        x: origin.x,
        y: origin.y,
        z: 0,
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

#[cfg(test)]
mod tests {
    use crate::core::framework::render::{
        CameraRenderDescriptor, RenderViewportRect, ViewportCameraSnapshot,
    };
    use crate::core::math::UVec2;
    use crate::graphics::types::ViewportRenderRegion;

    use super::{history_region_copy_extent, history_region_copy_origin};

    #[test]
    fn history_region_copy_targets_selected_camera_region() {
        let region = selected_camera_region(
            UVec2::new(1280, 720),
            UVec2::new(640, 0),
            UVec2::new(640, 720),
        );

        let extent =
            history_region_copy_extent(UVec2::new(640, 720), UVec2::new(1280, 720), region)
                .expect("selected camera copy should fit target");
        let origin = history_region_copy_origin(region);

        assert_eq!(extent.width, 640);
        assert_eq!(extent.height, 720);
        assert_eq!(origin.x, 640);
        assert_eq!(origin.y, 0);
    }

    #[test]
    fn history_region_copy_clamps_dynamic_resolution_to_viewport_region() {
        let region = selected_camera_region(
            UVec2::new(1280, 720),
            UVec2::new(960, 540),
            UVec2::new(512, 512),
        );

        let extent =
            history_region_copy_extent(UVec2::new(512, 512), UVec2::new(1280, 720), region)
                .expect("partially clipped selected camera copy should retain visible area");

        assert_eq!(extent.width, 320);
        assert_eq!(extent.height, 180);
    }

    #[test]
    fn history_region_copy_uses_local_extent_and_physical_destination() {
        let viewport_region = selected_camera_region(
            UVec2::new(1280, 720),
            UVec2::new(960, 540),
            UVec2::new(512, 512),
        );
        let render_region = viewport_region.with_local_size(UVec2::new(160, 90));

        let extent =
            history_region_copy_extent(UVec2::new(256, 256), UVec2::new(1280, 720), render_region)
                .expect("selected camera copy should retain local internal area");
        let origin = history_region_copy_origin(render_region);

        assert_eq!(extent.width, 160);
        assert_eq!(extent.height, 90);
        assert_eq!(origin.x, 960);
        assert_eq!(origin.y, 540);
    }

    fn selected_camera_region(
        target_size: UVec2,
        position: UVec2,
        size: UVec2,
    ) -> ViewportRenderRegion {
        let mut camera =
            CameraRenderDescriptor::from_camera_payload(None, ViewportCameraSnapshot::default());
        camera.viewport_rect = Some(RenderViewportRect::new(position, size));
        ViewportRenderRegion::from_camera(Some(&camera), target_size)
    }
}
