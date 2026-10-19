use super::{
    builtin_texture_desc_for, post_process_intermediate_format, PostProcessGraphResourceNames,
    TextureFormat,
};
use crate::core::framework::render::{
    RenderFrameExtract, RenderPipelinePhase, RenderResolutionPolicy, RenderUpscalerKind,
    RenderViewFamilyPipeline, RenderWorldSnapshotHandle, ShaderQualityTier,
};
use crate::core::math::UVec2;
use crate::graphics::RenderPipelineCompileOptions;
use crate::scene::world::World;

#[test]
fn reflection_and_gi_products_preserve_hdr_before_output_transfer() {
    assert_eq!(
        post_process_intermediate_format(PostProcessGraphResourceNames::DEPTH_OF_FIELD_BOKEH),
        Some(TextureFormat::Rgba16Float)
    );
    assert_eq!(
        post_process_intermediate_format(
            PostProcessGraphResourceNames::SCREEN_SPACE_REFLECTION_HISTORY
        ),
        Some(TextureFormat::Rgba16Float)
    );
    assert_eq!(
        post_process_intermediate_format(PostProcessGraphResourceNames::GLOBAL_ILLUMINATION),
        Some(TextureFormat::Rgba16Float)
    );
    assert_eq!(
        post_process_intermediate_format(PostProcessGraphResourceNames::HYBRID_GI_LIGHTING),
        Some(TextureFormat::Rgba16Float)
    );
}

#[test]
fn terminal_srgb_texture_omits_unsupported_storage_usage() {
    let extract = RenderFrameExtract::from_snapshot(
        RenderWorldSnapshotHandle::new(1),
        World::new().to_render_snapshot_cloning(),
    );
    let descriptor = builtin_texture_desc_for(
        PostProcessGraphResourceNames::FINAL_COMPOSITED,
        &extract,
        &RenderPipelineCompileOptions::default(),
    )
    .expect("terminal composited descriptor");

    assert_eq!(descriptor.format, TextureFormat::Rgba8UnormSrgb);
    assert!(!descriptor.usage.contains(crate::rhi::TextureUsage::STORAGE));
    assert!(descriptor
        .usage
        .contains(crate::rhi::TextureUsage::RENDER_ATTACHMENT));
}

#[test]
fn previous_hzb_uses_the_current_hzb_geometry_with_read_only_graph_usage() {
    let mut extract = RenderFrameExtract::from_snapshot(
        RenderWorldSnapshotHandle::new(1),
        World::new().to_render_snapshot_cloning(),
    );
    extract.apply_viewport_size(UVec2::new(1923, 1081));
    extract
        .view
        .apply_view_family_pipeline(RenderViewFamilyPipeline::resolve(
            UVec2::new(1923, 1081),
            RenderResolutionPolicy::with_temporal_fractions(1.0, 1.0),
            RenderUpscalerKind::Temporal,
        ));
    let options = RenderPipelineCompileOptions::default();
    let current = builtin_texture_desc_for(
        PostProcessGraphResourceNames::HZB_FURTHEST,
        &extract,
        &options,
    )
    .expect("current HZB descriptor");
    let previous = super::builtin_external_texture_desc_for(
        PostProcessGraphResourceNames::HISTORY_PREVIOUS_HZB_FURTHEST,
        &extract,
        &options,
    )
    .expect("previous HZB external descriptor");

    assert_eq!((current.width, current.height), (1024, 1024));
    assert_eq!(current.mip_levels, 11);
    assert_eq!(
        (previous.width, previous.height, previous.mip_levels),
        (current.width, current.height, current.mip_levels)
    );
    assert_eq!(previous.format, TextureFormat::Rgba16Float);
    assert_eq!(previous.sample_count, 1);
    assert_eq!(previous.usage, crate::rhi::TextureUsage::SAMPLED);
}

#[test]
fn previous_volumetric_history_uses_current_froxel_geometry_with_read_only_graph_usage() {
    let extract = RenderFrameExtract::from_snapshot(
        RenderWorldSnapshotHandle::new(1),
        World::new().to_render_snapshot_cloning(),
    );
    let options =
        RenderPipelineCompileOptions::default().with_shader_quality(ShaderQualityTier::High);
    let current = builtin_texture_desc_for(
        PostProcessGraphResourceNames::VOLUMETRIC_SCATTERING,
        &extract,
        &options,
    )
    .expect("current volumetric scattering descriptor");
    let previous = super::builtin_external_texture_desc_for(
        PostProcessGraphResourceNames::HISTORY_PREVIOUS_VOLUMETRIC_SCATTERING,
        &extract,
        &options,
    )
    .expect("previous volumetric external descriptor");

    assert_eq!(
        (current.width, current.height, current.depth),
        (160, 90, 96)
    );
    assert_eq!(current.dimension, crate::rhi::TextureDimension::D3);
    assert_eq!(
        (
            previous.width,
            previous.height,
            previous.depth,
            previous.dimension,
            previous.mip_levels,
            previous.sample_count,
        ),
        (
            current.width,
            current.height,
            current.depth,
            current.dimension,
            current.mip_levels,
            current.sample_count,
        )
    );
    assert_eq!(previous.format, TextureFormat::Rgba16Float);
    assert_eq!(previous.usage, crate::rhi::TextureUsage::SAMPLED);
}

#[test]
fn builtin_textures_follow_view_family_phase_allocations() {
    let mut extract = RenderFrameExtract::from_snapshot(
        RenderWorldSnapshotHandle::new(1),
        World::new().to_render_snapshot_cloning(),
    );
    extract.apply_viewport_size(UVec2::new(1920, 1080));
    extract
        .view
        .apply_view_family_pipeline(RenderViewFamilyPipeline::resolve(
            UVec2::new(1920, 1080),
            RenderResolutionPolicy::with_temporal_fractions(0.5, 0.75),
            RenderUpscalerKind::Temporal,
        ));
    let options = RenderPipelineCompileOptions::default();

    let extent = |name| {
        let desc = builtin_texture_desc_for(name, &extract, &options)
            .expect("built-in texture descriptor");
        UVec2::new(desc.width, desc.height)
    };

    assert_eq!(
        extent(PostProcessGraphResourceNames::SCENE_COLOR),
        UVec2::new(960, 544)
    );
    assert_eq!(
        extent(PostProcessGraphResourceNames::DEPTH_OF_FIELDED),
        UVec2::new(960, 544)
    );
    assert_eq!(
        extent(PostProcessGraphResourceNames::TAA_OUTPUT),
        UVec2::new(1440, 816)
    );
    assert_eq!(
        extent(PostProcessGraphResourceNames::MOTION_BLURRED),
        UVec2::new(1440, 816)
    );
    assert_eq!(
        extent(PostProcessGraphResourceNames::SCREEN_SPACE_REFLECTION_HISTORY),
        UVec2::new(1440, 816)
    );
    assert_eq!(
        extent(PostProcessGraphResourceNames::SCREEN_SPACE_REFLECTION_REFLECTION_PYRAMID),
        UVec2::new(720, 408)
    );
    assert_eq!(
        extent(PostProcessGraphResourceNames::SCREEN_SPACE_REFLECTION_REFLECTION_PYRAMID_COARSE),
        UVec2::new(360, 204)
    );
    assert_eq!(
        extent(PostProcessGraphResourceNames::FINAL_COMPOSITED),
        UVec2::new(1440, 816)
    );
    assert_eq!(
        extent(PostProcessGraphResourceNames::SECONDARY_UPSCALED),
        UVec2::new(1920, 1080)
    );
    assert_eq!(
        extent(PostProcessGraphResourceNames::FINAL_COLOR),
        UVec2::new(1920, 1080)
    );
}

#[test]
fn dual_spatial_upscale_textures_follow_distinct_phase_allocations() {
    let mut extract = RenderFrameExtract::from_snapshot(
        RenderWorldSnapshotHandle::new(1),
        World::new().to_render_snapshot_cloning(),
    );
    extract.apply_viewport_size(UVec2::new(1920, 1080));
    extract
        .view
        .apply_view_family_pipeline(RenderViewFamilyPipeline::resolve(
            UVec2::new(1920, 1080),
            RenderResolutionPolicy::with_scales(0.5, 0.75),
            RenderUpscalerKind::Spatial,
        ));
    let options = RenderPipelineCompileOptions::default();
    let extent = |name| {
        let desc = builtin_texture_desc_for(name, &extract, &options)
            .expect("built-in texture descriptor");
        UVec2::new(desc.width, desc.height)
    };

    assert_eq!(
        extent(PostProcessGraphResourceNames::PRIMARY_UPSCALED),
        UVec2::new(1440, 816)
    );
    assert_eq!(
        extent(PostProcessGraphResourceNames::SECONDARY_UPSCALED),
        UVec2::new(1920, 1080)
    );
}

#[test]
fn builtin_view_family_textures_have_explicit_pipeline_phase_owners() {
    let cases = [
        (
            PostProcessGraphResourceNames::SCENE_COLOR,
            RenderPipelinePhase::SceneLinear,
        ),
        (
            PostProcessGraphResourceNames::DEPTH_OF_FIELDED,
            RenderPipelinePhase::PreReconstructionScenePostProcess,
        ),
        (
            PostProcessGraphResourceNames::TAA_OUTPUT,
            RenderPipelinePhase::TemporalReconstruction,
        ),
        (
            PostProcessGraphResourceNames::SCREEN_SPACE_REFLECTION_HISTORY,
            RenderPipelinePhase::PostReconstructionScenePostProcess,
        ),
        (
            PostProcessGraphResourceNames::TONEMAPPED,
            RenderPipelinePhase::DisplayMapping,
        ),
        (
            PostProcessGraphResourceNames::FINAL_COMPOSITED,
            RenderPipelinePhase::DisplayPostProcess,
        ),
        (
            PostProcessGraphResourceNames::PRIMARY_UPSCALED,
            RenderPipelinePhase::PrimarySpatialUpscale,
        ),
        (
            PostProcessGraphResourceNames::SECONDARY_UPSCALED,
            RenderPipelinePhase::SecondarySpatialUpscale,
        ),
        (
            PostProcessGraphResourceNames::FINAL_COLOR,
            RenderPipelinePhase::OutputTransform,
        ),
        (
            PostProcessGraphResourceNames::VIEWPORT_OUTPUT,
            RenderPipelinePhase::Present,
        ),
    ];

    for (name, expected_phase) in cases {
        assert_eq!(
            PostProcessGraphResourceNames::view_family_pipeline_phase(name),
            Some(expected_phase),
            "{name}"
        );
    }

    for fixed_extent_name in [
        PostProcessGraphResourceNames::COLOR_LUT,
        PostProcessGraphResourceNames::VOLUMETRIC_MEDIA,
        PostProcessGraphResourceNames::VOLUMETRIC_SCATTERING,
        PostProcessGraphResourceNames::VOLUMETRIC_INTEGRATED,
    ] {
        assert_eq!(
            PostProcessGraphResourceNames::view_family_pipeline_phase(fixed_extent_name),
            None,
            "{fixed_extent_name}"
        );
    }
}

#[test]
fn builtin_texture_allocation_extent_fails_closed_without_production_panic() {
    let source = include_str!("../resource_descriptors.rs");
    let production = source
        .split_once("#[cfg(test)]")
        .map(|(production, _)| production)
        .expect("resource descriptor source must retain its test-module boundary");
    assert!(production.contains("builtin_texture_allocation_extent(name"));
    assert!(production.contains("?;"));
    assert!(!production.contains("view-family texture resource `{name}` has no pipeline-phase"));
    assert!(!production.contains("view-family scene phase must always be enabled"));
}

#[test]
fn scene_light_external_contract_tracks_packed_record_count() {
    use crate::core::framework::render::{
        GpuLightData, RenderLayerSet, RenderPointLightSnapshot, DEFAULT_RENDER_LAYER_MASK,
    };
    use crate::core::math::Vec3;
    use crate::graphics::scene::lighting::light_buffer::pack_lighting_extract;
    use crate::rhi::BufferUsage;

    for (point_count, lighting_enabled) in [(0, false), (12, false), (0, true), (12, true)] {
        let mut extract = RenderFrameExtract::from_snapshot(
            RenderWorldSnapshotHandle::new(1),
            World::new().to_render_snapshot_cloning(),
        );
        extract.post_process.preview.lighting_enabled = lighting_enabled;
        extract.lighting.point_lights = (0..point_count)
            .map(|slot| RenderPointLightSnapshot {
                node_id: slot,
                light_id: slot + 100,
                layer_mask: RenderLayerSet::from_scene_schema_v1_mask(DEFAULT_RENDER_LAYER_MASK),
                position: Vec3::new(slot as f32, 1.0, -2.0),
                color: Vec3::new(1.0, 0.5, 0.25),
                intensity: 2.0,
                range: 3.0,
                mobility: crate::core::framework::scene::Mobility::Dynamic,
                shadow: None,
            })
            .collect();
        let packed = pack_lighting_extract(&extract.lighting, lighting_enabled);
        let producer_bytes =
            std::mem::size_of_val(packed.lights.as_slice()).max(GpuLightData::STRIDE);
        let desc = super::builtin_external_buffer_desc_for(
            PostProcessGraphResourceNames::SCENE_LIGHT_DATA,
            &extract,
            None,
        )
        .expect("scene-light layout must resolve")
        .expect("scene-light is a catalog-owned external buffer");

        assert_eq!(
            desc.size_bytes,
            u64::try_from(producer_bytes).expect("test light data fits the RHI buffer size"),
            "point lights={point_count}, lighting={lighting_enabled}"
        );
        assert_eq!(
            desc.usage,
            BufferUsage::STORAGE | BufferUsage::COPY_SRC | BufferUsage::COPY_DST
        );
    }
}

#[test]
fn scene_light_external_contract_preserves_explicit_schema_validation() {
    use super::super::resource_schema_catalog::RenderResourceSchemaCatalog;
    use crate::graphics::{RenderBufferSchema, RenderResourceSchema, RenderTextureSchema};
    use crate::rhi::{BufferUsage, TextureUsage};

    let extract = RenderFrameExtract::from_snapshot(
        RenderWorldSnapshotHandle::new(1),
        World::new().to_render_snapshot_cloning(),
    );
    let options = RenderPipelineCompileOptions::default();
    let catalog = RenderResourceSchemaCatalog::new(&extract, &options);
    let name = PostProcessGraphResourceNames::SCENE_LIGHT_DATA;
    let schema = RenderResourceSchema::buffer(RenderBufferSchema::new(256, BufferUsage::STORAGE));
    let explicit = catalog
        .external_buffer_desc(name, Some(schema), Some(512))
        .expect("valid explicit buffer schema")
        .expect("explicit external buffer descriptor");
    assert_eq!(explicit.size_bytes, 512);
    assert_eq!(explicit.usage, BufferUsage::STORAGE);
    assert!(
        catalog
            .external_buffer_desc(
                name,
                Some(RenderResourceSchema::buffer(RenderBufferSchema::new(
                    0,
                    BufferUsage::STORAGE
                ))),
                None,
            )
            .is_err(),
        "an invalid explicit schema must not fall back to the builtin descriptor"
    );
    assert!(
        catalog
            .external_buffer_desc(
                name,
                Some(RenderResourceSchema::texture(RenderTextureSchema::new(
                    TextureFormat::Rgba8Unorm,
                    TextureUsage::SAMPLED
                ))),
                None,
            )
            .is_err(),
        "a texture schema cannot satisfy an external buffer contract"
    );
    assert!(catalog
        .external_buffer_desc("plugin-owned-unknown-buffer", None, None)
        .expect("unknown external buffers remain unresolved")
        .is_none());
}
