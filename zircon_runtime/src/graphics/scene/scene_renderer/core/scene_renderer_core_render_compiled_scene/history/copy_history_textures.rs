use crate::core::framework::render::RenderHistoryCopyReport;
use crate::core::math::UVec2;
use crate::graphics::backend::OffscreenTarget;
use crate::graphics::pipeline::{CompiledHistoryEpiloguePlan, CompiledHistoryTextureSource};
use crate::graphics::scene::scene_renderer::graph_execution::RenderGraphExecutionResources;
use crate::graphics::scene::scene_renderer::history::{
    SceneFrameHistoryTextures, SceneHistoryDomain, SceneHistoryWriteIntent,
};
use crate::graphics::types::ViewportRenderRegion;
use crate::rhi::TextureDesc;

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
        history_epilogue_plan: &CompiledHistoryEpiloguePlan,
        graph_history_writes: SceneHistoryWriteIntent,
        history_textures: Option<&SceneFrameHistoryTextures>,
        runtime_features: SceneRuntimeFeatureFlags,
        taa_history_enabled: bool,
        screen_space_reflection_history_enabled: bool,
        hzb_history_enabled: bool,
        exposure_history_enabled: bool,
        volumetric_history_enabled: bool,
    ) -> Result<(RenderHistoryCopyReport, SceneHistoryWriteIntent), String> {
        let requested_copy_count = taa_history_enabled as usize
            + runtime_features.hybrid_global_illumination_enabled as usize
            + screen_space_reflection_history_enabled as usize
            + hzb_history_enabled as usize
            + exposure_history_enabled as usize
            + volumetric_history_enabled as usize;
        let history_target_present = history_textures.is_some();
        let mut scene_color_copied = false;
        let mut global_illumination_copied = false;
        let mut screen_space_reflection_copied = false;
        let mut hzb_furthest_copied = false;
        let mut exposure_copied = false;
        let mut volumetric_scattering_copied = false;
        let mut write_intent = SceneHistoryWriteIntent::default();

        if let Some(history) = history_textures {
            if taa_history_enabled {
                scene_color_copied = history.taa_scene_color_current_texture().is_some()
                    && graph_history_writes.was_written(SceneHistoryDomain::TaaSceneColor);
            }
            if runtime_features.hybrid_global_illumination_enabled
                && graph_history_writes.was_written(SceneHistoryDomain::HybridGlobalIllumination)
            {
                global_illumination_copied = copy_global_illumination_history(
                    encoder,
                    target,
                    render_region,
                    graph_resources,
                    history_epilogue_plan,
                    history,
                )?;
            }
            if screen_space_reflection_history_enabled
                && graph_history_writes.was_written(SceneHistoryDomain::ScreenSpaceReflection)
            {
                if let Some((screen_space_reflection_history, extent)) =
                    screen_space_reflection_history_copy_extent(
                        graph_resources,
                        history_epilogue_plan.screen_space_reflection(),
                    )?
                {
                    if let Some(destination) = history.screen_space_reflection_texture() {
                        encoder.copy_texture_to_texture(
                            screen_space_reflection_history.as_image_copy(),
                            destination.as_image_copy(),
                            extent,
                        );
                        screen_space_reflection_copied = true;
                    }
                }
            }
            if hzb_history_enabled
                && graph_history_writes.was_written(SceneHistoryDomain::HzbFurthest)
            {
                hzb_furthest_copied = copy_hzb_furthest_mip_chain(
                    encoder,
                    graph_resources,
                    history_epilogue_plan.hzb_furthest(),
                    history,
                )?;
            }
            if exposure_history_enabled {
                exposure_copied = history.exposure_current_buffer().is_some()
                    && graph_history_writes.was_written(SceneHistoryDomain::Exposure);
            }
            if volumetric_history_enabled
                && graph_history_writes.was_written(SceneHistoryDomain::VolumetricScattering)
            {
                volumetric_scattering_copied = copy_volumetric_scattering_history(
                    encoder,
                    graph_resources,
                    history_epilogue_plan.volumetric_scattering(),
                    history,
                )?;
            }
        }
        if taa_history_enabled {
            write_intent.record(SceneHistoryDomain::TaaSceneColor, scene_color_copied);
        }
        if runtime_features.hybrid_global_illumination_enabled {
            write_intent.record(
                SceneHistoryDomain::HybridGlobalIllumination,
                global_illumination_copied,
            );
        }
        if screen_space_reflection_history_enabled {
            write_intent.record(
                SceneHistoryDomain::ScreenSpaceReflection,
                screen_space_reflection_copied,
            );
        }
        if hzb_history_enabled {
            write_intent.record(SceneHistoryDomain::HzbFurthest, hzb_furthest_copied);
        }
        if exposure_history_enabled {
            write_intent.record(SceneHistoryDomain::Exposure, exposure_copied);
        }
        if volumetric_history_enabled {
            write_intent.record(
                SceneHistoryDomain::VolumetricScattering,
                volumetric_scattering_copied,
            );
        }
        let report = RenderHistoryCopyReport::new(
            history_target_present,
            target.size,
            requested_copy_count,
            scene_color_copied,
            global_illumination_copied,
            false,
            screen_space_reflection_copied,
            hzb_furthest_copied,
            exposure_copied,
            volumetric_scattering_copied,
        );
        Ok((report, write_intent))
    }
}

fn copy_volumetric_scattering_history(
    encoder: &mut wgpu::CommandEncoder,
    graph_resources: &RenderGraphExecutionResources,
    source: Option<&CompiledHistoryTextureSource>,
    history: &SceneFrameHistoryTextures,
) -> Result<bool, String> {
    let (Some(source), Some(destination), Some(quality)) = (
        source,
        history.volumetric_history_texture(),
        history.volumetric_history_quality(),
    ) else {
        return Ok(false);
    };
    let texture = graph_resources.graph_owned_texture_for_access(source.access_id())?;
    let desc = source.desc();
    if desc.sample_count != 1
        || desc.format != crate::rhi::TextureFormat::Rgba16Float
        || desc.dimension != crate::rhi::TextureDimension::D3
        || [desc.width, desc.height, desc.depth] != quality.dimensions()
    {
        return Ok(false);
    }
    encoder.copy_texture_to_texture(
        texture.as_image_copy(),
        destination.as_image_copy(),
        wgpu::Extent3d {
            width: desc.width,
            height: desc.height,
            depth_or_array_layers: desc.depth_or_array_layers(),
        },
    );
    Ok(true)
}

fn copy_global_illumination_history(
    encoder: &mut wgpu::CommandEncoder,
    target: &OffscreenTarget,
    render_region: ViewportRenderRegion,
    graph_resources: &RenderGraphExecutionResources,
    history_epilogue_plan: &CompiledHistoryEpiloguePlan,
    history: &SceneFrameHistoryTextures,
) -> Result<bool, String> {
    let Some(fallback_destination) = history.global_illumination_texture() else {
        return Ok(false);
    };
    let mut graph_lighting_copied = false;
    for source in history_epilogue_plan.global_illumination_sources() {
        if copy_graph_global_illumination_history_source(
            encoder,
            target,
            render_region,
            graph_resources,
            history,
            source,
        )? {
            graph_lighting_copied = true;
            break;
        }
    }

    if !graph_lighting_copied {
        encoder.copy_texture_to_texture(
            target.global_illumination.as_image_copy(),
            fallback_destination.as_image_copy(),
            texture_extent(target.render_size),
        );
    }

    copy_graph_global_illumination_temporal_metadata(
        encoder,
        target,
        render_region,
        graph_resources,
        history_epilogue_plan.global_illumination_temporal_metadata(),
        history,
    )
}

fn copy_graph_global_illumination_history_source(
    encoder: &mut wgpu::CommandEncoder,
    target: &OffscreenTarget,
    render_region: ViewportRenderRegion,
    graph_resources: &RenderGraphExecutionResources,
    history: &SceneFrameHistoryTextures,
    source: &CompiledHistoryTextureSource,
) -> Result<bool, String> {
    let global_illumination = graph_resources.graph_owned_texture_for_access(source.access_id())?;
    let desc = source.desc();
    if !owned_global_illumination_history_source_is_copyable(desc) {
        return Ok(false);
    }
    let Some(extent) = history_region_copy_extent(
        UVec2::new(desc.width, desc.height),
        target.size,
        render_region,
    ) else {
        return Ok(false);
    };
    let Some(destination) = history.global_illumination_texture() else {
        return Ok(false);
    };

    let mut destination = destination.as_image_copy();
    destination.origin = history_region_copy_origin(render_region);
    encoder.copy_texture_to_texture(global_illumination.as_image_copy(), destination, extent);
    Ok(true)
}

fn owned_global_illumination_history_source_is_copyable(desc: &TextureDesc) -> bool {
    desc.sample_count == 1 && desc.format == crate::rhi::TextureFormat::Rgba16Float
}

fn copy_graph_global_illumination_temporal_metadata(
    encoder: &mut wgpu::CommandEncoder,
    target: &OffscreenTarget,
    render_region: ViewportRenderRegion,
    graph_resources: &RenderGraphExecutionResources,
    source: Option<&CompiledHistoryTextureSource>,
    history: &SceneFrameHistoryTextures,
) -> Result<bool, String> {
    let Some(source) = source else {
        return Ok(false);
    };
    let metadata = graph_resources.graph_owned_texture_for_access(source.access_id())?;
    let desc = source.desc();
    if !owned_global_illumination_temporal_metadata_is_copyable(desc) {
        return Ok(false);
    }
    let Some(extent) = history_region_copy_extent(
        UVec2::new(desc.width, desc.height),
        target.size,
        render_region,
    ) else {
        return Ok(false);
    };
    let Some(destination) = history.global_illumination_temporal_metadata_texture() else {
        return Ok(false);
    };

    let mut destination = destination.as_image_copy();
    destination.origin = history_region_copy_origin(render_region);
    encoder.copy_texture_to_texture(metadata.as_image_copy(), destination, extent);
    Ok(true)
}

fn owned_global_illumination_temporal_metadata_is_copyable(desc: &TextureDesc) -> bool {
    desc.sample_count == 1 && desc.format == crate::rhi::TextureFormat::Rgba16Float
}

fn screen_space_reflection_history_copy_extent<'a>(
    graph_resources: &'a RenderGraphExecutionResources,
    source: Option<&CompiledHistoryTextureSource>,
) -> Result<Option<(&'a wgpu::Texture, wgpu::Extent3d)>, String> {
    let Some(source) = source else {
        return Ok(None);
    };
    let texture = graph_resources.graph_owned_texture_for_access(source.access_id())?;
    let desc = source.desc();
    Ok(Some((
        texture,
        texture_extent(UVec2::new(desc.width, desc.height)),
    )))
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
    source: Option<&CompiledHistoryTextureSource>,
    history: &SceneFrameHistoryTextures,
) -> Result<bool, String> {
    let Some(source) = source else {
        return Ok(false);
    };
    let (Some(destination), Some(destination_size), Some(destination_mip_count)) = (
        history.hzb_furthest_texture(),
        history.hzb_furthest_size(),
        history.hzb_furthest_mip_count(),
    ) else {
        return Ok(false);
    };
    let hzb_furthest = graph_resources.graph_owned_texture_for_access(source.access_id())?;
    if source.desc().mip_levels != destination_mip_count {
        return Ok(false);
    }
    for mip_level in 0..destination_mip_count {
        encoder.copy_texture_to_texture(
            texture_mip_copy(hzb_furthest, mip_level),
            texture_mip_copy(destination, mip_level),
            mip_extent(destination_size, mip_level),
        );
    }
    Ok(true)
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
#[path = "tests/copy_history_textures.rs"]
mod tests;
