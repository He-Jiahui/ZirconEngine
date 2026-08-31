use crate::core::framework::render::{
    COLOR_LUT_FORMAT, COLOR_LUT_SIZE_DEFAULT, EXPOSURE_BUFFER_WORD_COUNT,
    EXPOSURE_HISTOGRAM_BIN_COUNT, FroxelGridQuality, INTERMEDIATE_HDR_FORMAT_DEFAULT,
    INTERMEDIATE_HDR_FORMAT_HIGH_QUALITY, OitBufferPlan, PostProcessGraphResourceNames,
    RenderFrameExtract, RenderPipelinePhase, RenderPostProcessTextureFormat,
    RenderViewFamilyPipeline, TONEMAPPED_SDR_FORMAT,
};
use crate::core::math::UVec2;
use crate::graphics::pipeline::RenderPipelineCompileOptions;
use crate::graphics::visibility::HzbBuilder;
use crate::rhi::{
    BufferDesc, BufferUsage, TextureDesc, TextureDimension, TextureFormat, TextureUsage,
};

mod schema_allocation;

#[cfg(test)]
use schema_allocation::resolve_relative_extent_axis;
pub(super) use schema_allocation::{buffer_desc_from_schema, texture_desc_from_schema};

pub(super) fn builtin_texture_desc_for(
    name: &str,
    extract: &RenderFrameExtract,
    options: &RenderPipelineCompileOptions,
) -> Option<TextureDesc> {
    if !is_builtin_texture_resource(name) {
        return None;
    }
    if name == PostProcessGraphResourceNames::COLOR_LUT {
        return Some(
            TextureDesc::new(
                name,
                COLOR_LUT_SIZE_DEFAULT,
                COLOR_LUT_SIZE_DEFAULT,
                post_process_texture_format(COLOR_LUT_FORMAT),
                TextureUsage::SAMPLED
                    | TextureUsage::STORAGE
                    | TextureUsage::COPY_SRC
                    | TextureUsage::COPY_DST,
            )
            .with_dimension(TextureDimension::D3)
            .with_depth(COLOR_LUT_SIZE_DEFAULT),
        );
    }

    if is_volumetric_froxel_resource(name) {
        let usage = match name {
            PostProcessGraphResourceNames::VOLUMETRIC_SCATTERING => {
                TextureUsage::SAMPLED | TextureUsage::STORAGE | TextureUsage::COPY_SRC
            }
            _ => TextureUsage::SAMPLED | TextureUsage::STORAGE,
        };
        return Some(builtin_volumetric_texture_desc_for(
            name, extract, options, usage,
        ));
    }

    if name == PostProcessGraphResourceNames::HZB_FURTHEST {
        return builtin_hzb_texture_desc_for(
            name,
            extract,
            TextureUsage::SAMPLED | TextureUsage::STORAGE | TextureUsage::COPY_SRC,
        );
    }

    let allocation_extent =
        builtin_texture_allocation_extent(name, extract.view.view_family_pipeline())?;
    let base_width = allocation_extent.x.max(1);
    let base_height = allocation_extent.y.max(1);
    let (width, height) = post_process_intermediate_size(name, base_width, base_height);
    let post_process_format = post_process_intermediate_format(name);
    let format = post_process_format.unwrap_or_else(|| {
        if name == PostProcessGraphResourceNames::GBUFFER_NORMAL {
            TextureFormat::Rgba8Unorm
        } else if is_builtin_depth_texture(name) {
            TextureFormat::Depth32Float
        } else if extract.view.camera.hdr && is_scene_color_resource(name) {
            post_process_intermediate_hdr_format()
        } else {
            TextureFormat::Rgba8UnormSrgb
        }
    });
    let mut usage =
        TextureUsage::RENDER_ATTACHMENT | TextureUsage::SAMPLED | TextureUsage::COPY_SRC;
    if !format.is_depth() {
        usage |= TextureUsage::STORAGE | TextureUsage::COPY_DST;
    }
    let sample_count = if post_process_format.is_some()
        || is_single_sample_graph_product(name)
        || name == PostProcessGraphResourceNames::SHADOW_ATLAS
    {
        1
    } else {
        options.graph_msaa_sample_count(extract.view.camera.msaa_samples)
    };
    Some(
        TextureDesc::new(name, width, height, format, usage)
            .with_sample_count(sample_count)
            .with_mip_levels(post_process_intermediate_mip_levels(name, width, height)),
    )
}

pub(super) fn builtin_external_texture_desc_for(
    name: &str,
    extract: &RenderFrameExtract,
    options: &RenderPipelineCompileOptions,
) -> Option<TextureDesc> {
    match name {
        PostProcessGraphResourceNames::HISTORY_PREVIOUS_HZB_FURTHEST => {
            builtin_hzb_texture_desc_for(name, extract, TextureUsage::SAMPLED)
        }
        PostProcessGraphResourceNames::HISTORY_PREVIOUS_VOLUMETRIC_SCATTERING => Some(
            builtin_volumetric_texture_desc_for(name, extract, options, TextureUsage::SAMPLED),
        ),
        _ => None,
    }
}

fn builtin_hzb_texture_desc_for(
    name: &str,
    extract: &RenderFrameExtract,
    usage: TextureUsage,
) -> Option<TextureDesc> {
    let view_size = builtin_texture_allocation_extent(name, extract.view.view_family_pipeline())?;
    let plan = HzbBuilder::new(view_size).build_plan();
    Some(
        TextureDesc::new(
            name,
            plan.hzb_size.x,
            plan.hzb_size.y,
            TextureFormat::Rgba16Float,
            usage,
        )
        .with_mip_levels(plan.mip_count),
    )
}

fn builtin_volumetric_texture_desc_for(
    name: &str,
    extract: &RenderFrameExtract,
    options: &RenderPipelineCompileOptions,
    usage: TextureUsage,
) -> TextureDesc {
    let [width, height, depth] = extract.lighting.advanced_lighting.froxel_dimensions(
        FroxelGridQuality::from_shader_quality(options.shader_quality),
    );
    TextureDesc::new(name, width, height, TextureFormat::Rgba16Float, usage)
        .with_dimension(TextureDimension::D3)
        .with_depth(depth)
}

pub(super) fn builtin_buffer_desc_for(
    name: &str,
    extract: &RenderFrameExtract,
    minimum_size_bytes: Option<u64>,
) -> Result<Option<BufferDesc>, String> {
    if !is_builtin_buffer_resource(name) {
        return Ok(None);
    }
    use crate::graphics::scene::lighting::light_grid_builder::{
        LIGHT_GRID_MAX_TILE_WORDS, LIGHT_GRID_MAX_ZBIN_WORDS, LIGHT_GRID_PARAMS_UNIFORM_SIZE_BYTES,
    };
    use crate::graphics::scene::{
        SSS_PARAMS_BUFFER_SIZE_BYTES, SSS_PROFILE_TABLE_BUFFER_SIZE_BYTES,
    };

    let view_size = extract.view.effective_render_size();
    let catalog_size_bytes = match name {
        PostProcessGraphResourceNames::OIT_LAYERS => {
            let plan = OitBufferPlan::for_view(
                [view_size.x, view_size.y],
                extract.lighting.advanced_lighting.oit.unwrap_or_default(),
            );
            plan.layer_buffer_size_bytes
        }
        PostProcessGraphResourceNames::OIT_COUNTS => {
            let plan = OitBufferPlan::for_view(
                [view_size.x, view_size.y],
                extract.lighting.advanced_lighting.oit.unwrap_or_default(),
            );
            plan.count_buffer_size_bytes
        }
        PostProcessGraphResourceNames::SSS_TILE_LIST => {
            let tile_width = view_size.x.max(1).div_ceil(8);
            let tile_height = view_size.y.max(1).div_ceil(8);
            u64::from(tile_width) * u64::from(tile_height) * 8
        }
        PostProcessGraphResourceNames::SSS_INDIRECT_ARGS => 16,
        PostProcessGraphResourceNames::SSS_PARAMS => SSS_PARAMS_BUFFER_SIZE_BYTES,
        PostProcessGraphResourceNames::SSS_PROFILES => SSS_PROFILE_TABLE_BUFFER_SIZE_BYTES,
        PostProcessGraphResourceNames::LIGHT_GRID_PARAMS => {
            LIGHT_GRID_PARAMS_UNIFORM_SIZE_BYTES as u64
        }
        PostProcessGraphResourceNames::LIGHT_ZBINS => u64::from(LIGHT_GRID_MAX_ZBIN_WORDS) * 4,
        PostProcessGraphResourceNames::LIGHT_TILE_MASKS => u64::from(LIGHT_GRID_MAX_TILE_WORDS) * 4,
        PostProcessGraphResourceNames::LIGHT_LIST => u64::try_from(
            crate::graphics::scene::cluster_buffer_bytes_for_size(view_size),
        )
        .map_err(|_| format!("builtin light-list resource `{name}` exceeds u64 capacity"))?,
        PostProcessGraphResourceNames::EXPOSURE_HISTOGRAM => {
            u64::from(EXPOSURE_HISTOGRAM_BIN_COUNT) * 4
        }
        PostProcessGraphResourceNames::EXPOSURE_PREVIOUS
        | PostProcessGraphResourceNames::EXPOSURE_CURRENT => {
            u64::from(EXPOSURE_BUFFER_WORD_COUNT) * 4
        }
        PostProcessGraphResourceNames::HYBRID_GI_SCENE
        | PostProcessGraphResourceNames::HYBRID_GI_TRACE => {
            minimum_size_bytes.ok_or_else(|| {
                format!(
                    "builtin packet buffer `{name}` requires a producer-declared minimum_size_bytes"
                )
            })?
        }
        _ => {
            return Err(format!(
                "builtin buffer resource `{name}` has no catalog capacity policy"
            ));
        }
    };
    let size_bytes = catalog_size_bytes.max(minimum_size_bytes.unwrap_or(0));
    let usage = match name {
        PostProcessGraphResourceNames::LIGHT_GRID_PARAMS
        | PostProcessGraphResourceNames::SSS_PARAMS
        | PostProcessGraphResourceNames::SSS_PROFILES => {
            BufferUsage::UNIFORM | BufferUsage::COPY_DST
        }
        PostProcessGraphResourceNames::SSS_INDIRECT_ARGS => {
            BufferUsage::STORAGE | BufferUsage::INDIRECT | BufferUsage::COPY_DST
        }
        _ => BufferUsage::STORAGE | BufferUsage::COPY_SRC | BufferUsage::COPY_DST,
    };
    Ok(Some(BufferDesc::new(name, size_bytes, usage)))
}

fn post_process_intermediate_format(name: &str) -> Option<TextureFormat> {
    match name {
        PostProcessGraphResourceNames::HALF_RES_TRANSPARENCY_COLOR => {
            Some(TextureFormat::Rgba16Float)
        }
        PostProcessGraphResourceNames::SCENE_VELOCITY => Some(post_process_texture_format(
            RenderPostProcessTextureFormat::Rg16Float,
        )),
        PostProcessGraphResourceNames::TAA_REACTIVE_MASK => Some(post_process_texture_format(
            RenderPostProcessTextureFormat::R8Unorm,
        )),
        PostProcessGraphResourceNames::DEPTH_OF_FIELDED
        | PostProcessGraphResourceNames::MOTION_BLURRED
        | PostProcessGraphResourceNames::BLURRED
        | PostProcessGraphResourceNames::BLOOM
        | PostProcessGraphResourceNames::SCENE_COMPOSITED => {
            Some(post_process_intermediate_hdr_format())
        }
        PostProcessGraphResourceNames::TRANSMISSION_SCENE_COLOR => Some(TextureFormat::Rgba16Float),
        PostProcessGraphResourceNames::TONEMAPPED
        | PostProcessGraphResourceNames::PRIMARY_UPSCALED
        | PostProcessGraphResourceNames::SECONDARY_UPSCALED => {
            Some(post_process_texture_format(TONEMAPPED_SDR_FORMAT))
        }
        PostProcessGraphResourceNames::FINAL_COMPOSITED => Some(TextureFormat::Rgba8UnormSrgb),
        PostProcessGraphResourceNames::COLOR_LUT => {
            Some(post_process_texture_format(COLOR_LUT_FORMAT))
        }
        PostProcessGraphResourceNames::TAA_OUTPUT
        | PostProcessGraphResourceNames::SCREEN_SPACE_REFLECTION_REFLECTION_PYRAMID => {
            Some(post_process_intermediate_hdr_format())
        }
        PostProcessGraphResourceNames::MOTION_VECTOR_TILE_MAX
        | PostProcessGraphResourceNames::MOTION_VECTOR_TILE_MAX_COARSE
        | PostProcessGraphResourceNames::MOTION_VECTOR_NEIGHBOR_MAX
        | PostProcessGraphResourceNames::HZB_FURTHEST
        | PostProcessGraphResourceNames::HYBRID_GI_LIGHTING
        | PostProcessGraphResourceNames::HYBRID_GI_TEMPORAL_METADATA
        | PostProcessGraphResourceNames::TAA_HISTORY_CURRENT => {
            Some(post_process_high_quality_hdr_format())
        }
        PostProcessGraphResourceNames::DEPTH_OF_FIELD_COC
        | PostProcessGraphResourceNames::CONTACT_SHADOW_OCCLUSION
        | PostProcessGraphResourceNames::SCREEN_SPACE_REFLECTION_SPECULAR_OCCLUSION => Some(
            post_process_texture_format(RenderPostProcessTextureFormat::Rgba8Unorm),
        ),
        PostProcessGraphResourceNames::DEPTH_OF_FIELD_BOKEH => {
            Some(post_process_intermediate_hdr_format())
        }
        PostProcessGraphResourceNames::SCREEN_SPACE_REFLECTION_HISTORY
        | PostProcessGraphResourceNames::GLOBAL_ILLUMINATION
        | PostProcessGraphResourceNames::SSS_DIFFUSE
        | PostProcessGraphResourceNames::SSS_SPECULAR
        | PostProcessGraphResourceNames::SSS_SCATTERED => {
            Some(post_process_high_quality_hdr_format())
        }
        _ => None,
    }
}

const fn post_process_intermediate_hdr_format() -> TextureFormat {
    post_process_texture_format(INTERMEDIATE_HDR_FORMAT_DEFAULT)
}

const fn post_process_high_quality_hdr_format() -> TextureFormat {
    post_process_texture_format(INTERMEDIATE_HDR_FORMAT_HIGH_QUALITY)
}

const fn post_process_texture_format(format: RenderPostProcessTextureFormat) -> TextureFormat {
    match format {
        RenderPostProcessTextureFormat::R8Unorm => TextureFormat::R8Unorm,
        RenderPostProcessTextureFormat::Rg16Float => TextureFormat::Rg16Float,
        RenderPostProcessTextureFormat::Rgba8Unorm => TextureFormat::Rgba8Unorm,
        RenderPostProcessTextureFormat::Rgba8UnormSrgb => TextureFormat::Rgba8UnormSrgb,
        RenderPostProcessTextureFormat::Rg11b10Ufloat => TextureFormat::Rg11b10Ufloat,
        RenderPostProcessTextureFormat::Rgba16Float => TextureFormat::Rgba16Float,
        RenderPostProcessTextureFormat::Rgba32Float => TextureFormat::Rgba32Float,
    }
}

fn post_process_intermediate_size(name: &str, width: u32, height: u32) -> (u32, u32) {
    match name {
        PostProcessGraphResourceNames::HALF_RES_TRANSPARENCY_COLOR
        | PostProcessGraphResourceNames::HALF_RES_TRANSPARENCY_DEPTH => {
            (half_extent(width), half_extent(height))
        }
        PostProcessGraphResourceNames::MOTION_VECTOR_TILE_MAX
        | PostProcessGraphResourceNames::SCREEN_SPACE_REFLECTION_REFLECTION_PYRAMID => {
            (half_extent(width), half_extent(height))
        }
        PostProcessGraphResourceNames::MOTION_VECTOR_TILE_MAX_COARSE => {
            let half_width = half_extent(width);
            let half_height = half_extent(height);
            (half_extent(half_width), half_extent(half_height))
        }
        _ => (width, height),
    }
}

fn post_process_intermediate_mip_levels(name: &str, width: u32, height: u32) -> u32 {
    match name {
        PostProcessGraphResourceNames::SCREEN_SPACE_REFLECTION_REFLECTION_PYRAMID => {
            full_mip_chain_level_count(width, height)
        }
        _ => 1,
    }
}

fn full_mip_chain_level_count(width: u32, height: u32) -> u32 {
    u32::BITS - width.max(height).max(1).leading_zeros()
}

fn half_extent(value: u32) -> u32 {
    (value.saturating_add(1) / 2).max(1)
}

fn builtin_texture_allocation_extent(
    name: &str,
    view_family: &RenderViewFamilyPipeline,
) -> Option<UVec2> {
    let phase = PostProcessGraphResourceNames::view_family_pipeline_phase(name)?;
    view_family
        .output_target_for_phase(phase)
        .unwrap_or_else(|| view_family.output_target_for_phase(RenderPipelinePhase::SceneLinear))
        .map(|target| target.allocation_extent())
}

fn is_scene_color_resource(name: &str) -> bool {
    matches!(
        name,
        PostProcessGraphResourceNames::SCENE_COLOR
            | PostProcessGraphResourceNames::TRANSMISSION_SCENE_COLOR
            | PostProcessGraphResourceNames::FINAL_COLOR
            | PostProcessGraphResourceNames::FINAL_COMPOSITED
            | PostProcessGraphResourceNames::AMBIENT_OCCLUSION
            | PostProcessGraphResourceNames::GBUFFER_ALBEDO
            | PostProcessGraphResourceNames::GBUFFER_NORMAL
            | PostProcessGraphResourceNames::GBUFFER_MATERIAL
            | PostProcessGraphResourceNames::GBUFFER_EMISSIVE
    )
}

fn is_builtin_depth_texture(name: &str) -> bool {
    matches!(
        name,
        PostProcessGraphResourceNames::SCENE_DEPTH
            | PostProcessGraphResourceNames::HALF_RES_TRANSPARENCY_DEPTH
            | PostProcessGraphResourceNames::SHADOW_ATLAS
    )
}

fn is_builtin_texture_resource(name: &str) -> bool {
    PostProcessGraphResourceNames::view_family_pipeline_phase(name).is_some()
        || name == PostProcessGraphResourceNames::COLOR_LUT
        || is_volumetric_froxel_resource(name)
}

fn is_builtin_buffer_resource(name: &str) -> bool {
    matches!(
        name,
        PostProcessGraphResourceNames::HYBRID_GI_SCENE
            | PostProcessGraphResourceNames::HYBRID_GI_TRACE
            | PostProcessGraphResourceNames::LIGHT_LIST
            | PostProcessGraphResourceNames::LIGHT_GRID_PARAMS
            | PostProcessGraphResourceNames::LIGHT_ZBINS
            | PostProcessGraphResourceNames::LIGHT_TILE_MASKS
            | PostProcessGraphResourceNames::OIT_LAYERS
            | PostProcessGraphResourceNames::OIT_COUNTS
            | PostProcessGraphResourceNames::SSS_TILE_LIST
            | PostProcessGraphResourceNames::SSS_INDIRECT_ARGS
            | PostProcessGraphResourceNames::SSS_PARAMS
            | PostProcessGraphResourceNames::SSS_PROFILES
            | PostProcessGraphResourceNames::EXPOSURE_HISTOGRAM
            | PostProcessGraphResourceNames::EXPOSURE_PREVIOUS
            | PostProcessGraphResourceNames::EXPOSURE_CURRENT
    )
}

fn is_single_sample_graph_product(name: &str) -> bool {
    matches!(
        name,
        PostProcessGraphResourceNames::HYBRID_GI_LIGHTING
            | PostProcessGraphResourceNames::HALF_RES_TRANSPARENCY_COLOR
            | PostProcessGraphResourceNames::HALF_RES_TRANSPARENCY_DEPTH
            | PostProcessGraphResourceNames::TRANSMISSION_SCENE_COLOR
            | PostProcessGraphResourceNames::HYBRID_GI_TEMPORAL_METADATA
            | PostProcessGraphResourceNames::SSS_DIFFUSE
            | PostProcessGraphResourceNames::SSS_SPECULAR
            | PostProcessGraphResourceNames::SSS_SCATTERED
    )
}

fn is_volumetric_froxel_resource(name: &str) -> bool {
    matches!(
        name,
        PostProcessGraphResourceNames::VOLUMETRIC_MEDIA
            | PostProcessGraphResourceNames::VOLUMETRIC_SCATTERING
            | PostProcessGraphResourceNames::VOLUMETRIC_INTEGRATED
    )
}

#[cfg(test)]
mod buffer_schema_tests {
    use super::{
        buffer_desc_from_schema, builtin_buffer_desc_for, resolve_relative_extent_axis,
        texture_desc_from_schema,
    };
    use crate::core::framework::render::{
        PostProcessGraphResourceNames, RenderFrameExtract, RenderWorldSnapshotHandle,
    };
    use crate::graphics::{
        RenderBufferSchema, RenderResourceSchema, RenderTextureExtentPolicy,
        RenderTextureExtentReference, RenderTextureExtentRounding, RenderTextureSchema,
    };
    use crate::rhi::{BufferUsage, TextureFormat, TextureUsage};
    use crate::scene::world::World;

    fn test_extract() -> RenderFrameExtract {
        RenderFrameExtract::from_snapshot(
            RenderWorldSnapshotHandle::new(1),
            World::new().to_render_snapshot(),
        )
    }

    #[test]
    fn explicit_buffer_schema_rejects_zero_size_and_empty_usage() {
        let zero_size = buffer_desc_from_schema(
            "zero-size",
            RenderResourceSchema::buffer(RenderBufferSchema::new(0, BufferUsage::STORAGE)),
            None,
        )
        .expect_err("buffer schemas require a non-zero byte size");
        assert!(zero_size.contains("non-zero byte size"), "{zero_size}");

        let empty_usage = buffer_desc_from_schema(
            "empty-usage",
            RenderResourceSchema::buffer(RenderBufferSchema::new(16, BufferUsage::NONE)),
            None,
        )
        .expect_err("buffer schemas require usage");
        assert!(
            empty_usage.contains("usage must not be empty"),
            "{empty_usage}"
        );
    }

    #[test]
    fn explicit_texture_schema_rejects_empty_usage() {
        let error = texture_desc_from_schema(
            "empty-texture-usage",
            RenderResourceSchema::texture(RenderTextureSchema::new(
                TextureFormat::Rgba8Unorm,
                TextureUsage::NONE,
            )),
            &test_extract(),
        )
        .expect_err("texture schemas require usage");

        assert!(error.contains("usage must not be empty"), "{error}");
    }

    #[test]
    fn relative_texture_extent_ceil_divides_the_selected_reference_extent() {
        let extract = test_extract();
        let schema = RenderResourceSchema::texture(
            RenderTextureSchema::new(TextureFormat::R8Unorm, TextureUsage::STORAGE).with_extent(
                RenderTextureExtentPolicy::Relative {
                    reference: RenderTextureExtentReference::Render,
                    numerator: 1,
                    denominator: 2,
                    rounding: RenderTextureExtentRounding::Ceil,
                },
            ),
        );

        let desc = texture_desc_from_schema("half-render", schema, &extract)
            .expect("a valid relative render extent should resolve");
        let render_extent = extract
            .view
            .view_family_pipeline()
            .resolution()
            .primary_allocation_extent();

        assert_eq!(desc.width, render_extent.x.div_ceil(2).max(1));
        assert_eq!(desc.height, render_extent.y.div_ceil(2).max(1));
        assert_eq!(
            resolve_relative_extent_axis(5, 1, 2, RenderTextureExtentRounding::Ceil)
                .expect("ceil-scaled odd extent"),
            3
        );
        assert_eq!(
            resolve_relative_extent_axis(5, 1, 2, RenderTextureExtentRounding::Floor)
                .expect("floor-scaled odd extent"),
            2
        );
    }

    #[test]
    fn relative_texture_extent_rejects_zero_scale_terms_and_u32_overflow() {
        let extract = test_extract();
        for (numerator, denominator, expected) in [
            (0, 2, "non-zero numerator and denominator"),
            (1, 0, "non-zero numerator and denominator"),
        ] {
            let schema = RenderResourceSchema::texture(
                RenderTextureSchema::new(TextureFormat::R8Unorm, TextureUsage::STORAGE)
                    .with_extent(RenderTextureExtentPolicy::Relative {
                        reference: RenderTextureExtentReference::View,
                        numerator,
                        denominator,
                        rounding: RenderTextureExtentRounding::Floor,
                    }),
            );

            let error = texture_desc_from_schema("invalid-relative", schema, &extract)
                .expect_err("invalid relative extents must be rejected");
            assert!(error.contains(expected), "unexpected error: {error}");
        }

        let overflow =
            resolve_relative_extent_axis(2, u32::MAX, 1, RenderTextureExtentRounding::Floor)
                .expect_err("relative extents larger than u32 must be rejected");
        assert!(overflow.contains("exceeds the supported u32 texture extent"));
    }

    #[test]
    fn builtin_packet_buffer_requires_the_producer_owned_capacity() {
        let extract = test_extract();
        let error = builtin_buffer_desc_for(
            PostProcessGraphResourceNames::HYBRID_GI_SCENE,
            &extract,
            None,
        )
        .expect_err("packet buffers cannot infer their capacity from a name");
        assert!(
            error.contains("requires a producer-declared minimum_size_bytes"),
            "{error}"
        );

        let desc = builtin_buffer_desc_for(
            PostProcessGraphResourceNames::HYBRID_GI_SCENE,
            &extract,
            Some(128),
        )
        .expect("catalog resolves a producer-owned packet capacity")
        .expect("hybrid GI scene is a built-in packet buffer");
        assert_eq!(desc.size_bytes, 128);
    }

    #[test]
    fn builtin_light_list_uses_the_cluster_buffer_capacity_policy() {
        let extract = test_extract();
        let desc =
            builtin_buffer_desc_for(PostProcessGraphResourceNames::LIGHT_LIST, &extract, None)
                .expect("catalog resolves the light-list capacity")
                .expect("light-list is a built-in graph buffer");
        let expected = u64::try_from(crate::graphics::scene::cluster_buffer_bytes_for_size(
            extract.view.effective_render_size(),
        ))
        .expect("usize must fit the RHI buffer capacity");

        assert_eq!(desc.size_bytes, expected);
    }

    #[test]
    fn builtin_sss_uniforms_use_exact_shader_layout_capacities() {
        use crate::graphics::scene::{
            SSS_PARAMS_BUFFER_SIZE_BYTES, SSS_PROFILE_TABLE_BUFFER_SIZE_BYTES,
        };

        let extract = test_extract();
        for (name, expected_size) in [
            (
                PostProcessGraphResourceNames::SSS_PARAMS,
                SSS_PARAMS_BUFFER_SIZE_BYTES,
            ),
            (
                PostProcessGraphResourceNames::SSS_PROFILES,
                SSS_PROFILE_TABLE_BUFFER_SIZE_BYTES,
            ),
        ] {
            let desc = builtin_buffer_desc_for(name, &extract, None)
                .expect("SSS uniform catalog lookup succeeds")
                .expect("SSS uniform is a built-in graph buffer");
            assert_eq!(desc.size_bytes, expected_size);
            assert_eq!(desc.usage, BufferUsage::UNIFORM | BufferUsage::COPY_DST);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::{
        PostProcessGraphResourceNames, TextureFormat, builtin_texture_desc_for,
        post_process_intermediate_format,
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
            Some(TextureFormat::Rg11b10Ufloat)
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
    fn previous_hzb_uses_the_current_hzb_geometry_with_read_only_graph_usage() {
        let mut extract = RenderFrameExtract::from_snapshot(
            RenderWorldSnapshotHandle::new(1),
            World::new().to_render_snapshot(),
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
            World::new().to_render_snapshot(),
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
            World::new().to_render_snapshot(),
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
            extent(
                PostProcessGraphResourceNames::SCREEN_SPACE_REFLECTION_REFLECTION_PYRAMID_COARSE
            ),
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
            World::new().to_render_snapshot(),
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
        let source = include_str!("resource_descriptors.rs");
        let production = source
            .split_once("#[cfg(test)]")
            .map(|(production, _)| production)
            .expect("resource descriptor source must retain its test-module boundary");
        assert!(production.contains("builtin_texture_allocation_extent(name"));
        assert!(production.contains("?;"));
        assert!(
            !production.contains("view-family texture resource `{name}` has no pipeline-phase")
        );
        assert!(!production.contains("view-family scene phase must always be enabled"));
    }
}
