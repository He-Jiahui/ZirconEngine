use crate::core::framework::render::{
    PostProcessGraphResourceNames, RenderFrameExtract, RenderPostProcessTextureFormat,
    COLOR_LUT_FORMAT, COLOR_LUT_SIZE_DEFAULT, EXPOSURE_BUFFER_WORD_COUNT,
    EXPOSURE_HISTOGRAM_BIN_COUNT, INTERMEDIATE_HDR_FORMAT_DEFAULT,
    INTERMEDIATE_HDR_FORMAT_HIGH_QUALITY, TONEMAPPED_SDR_FORMAT,
};
use crate::core::math::UVec2;
use crate::graphics::pipeline::RenderPipelineCompileOptions;
use crate::graphics::visibility::HzbBuilder;
use crate::rhi::{
    BufferDesc, BufferUsage, TextureDesc, TextureDimension, TextureFormat, TextureUsage,
};

pub(super) fn texture_desc_for(
    name: &str,
    extract: &RenderFrameExtract,
    options: &RenderPipelineCompileOptions,
) -> TextureDesc {
    if name == PostProcessGraphResourceNames::COLOR_LUT {
        return TextureDesc::new(
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
        .with_depth(COLOR_LUT_SIZE_DEFAULT);
    }

    let view_size = if name == PostProcessGraphResourceNames::UPSCALED
        || name == PostProcessGraphResourceNames::FINAL_COMPOSITED
    {
        extract.view.effective_view_size()
    } else {
        extract.view.effective_render_size()
    };
    let base_width = view_size.x.max(1);
    let base_height = view_size.y.max(1);
    let (width, height) = post_process_intermediate_size(name, base_width, base_height);
    let post_process_format = post_process_intermediate_format(name);
    let format = match post_process_format {
        Some(format) => format,
        None if name.contains("depth") || name.contains("shadow") => TextureFormat::Depth32Float,
        None if extract.view.camera.hdr && is_scene_color_resource(name) => {
            post_process_intermediate_hdr_format()
        }
        None => TextureFormat::Rgba8UnormSrgb,
    };
    let mut usage =
        TextureUsage::RENDER_ATTACHMENT | TextureUsage::SAMPLED | TextureUsage::COPY_SRC;
    if !format.is_depth() {
        usage |= TextureUsage::STORAGE | TextureUsage::COPY_DST;
    }
    let sample_count = if post_process_format.is_some() || name.contains("shadow") {
        1
    } else {
        options.graph_msaa_sample_count(extract.view.camera.msaa_samples)
    };
    TextureDesc::new(name, width, height, format, usage)
        .with_sample_count(sample_count)
        .with_mip_levels(post_process_intermediate_mip_levels(name, width, height))
}

pub(super) fn buffer_desc_for(name: &str, extract: &RenderFrameExtract) -> BufferDesc {
    use crate::graphics::scene::lighting::light_grid_builder::{
        LIGHT_GRID_MAX_TILE_WORDS, LIGHT_GRID_MAX_ZBIN_WORDS, LIGHT_GRID_PARAMS_UNIFORM_SIZE_BYTES,
    };

    let view_size = extract.view.effective_render_size();
    let pixel_count = u64::from(view_size.x.max(1)) * u64::from(view_size.y.max(1));
    let size_bytes = match name {
        PostProcessGraphResourceNames::LIGHT_GRID_PARAMS => {
            LIGHT_GRID_PARAMS_UNIFORM_SIZE_BYTES as u64
        }
        PostProcessGraphResourceNames::LIGHT_ZBINS => u64::from(LIGHT_GRID_MAX_ZBIN_WORDS) * 4,
        PostProcessGraphResourceNames::LIGHT_TILE_MASKS => u64::from(LIGHT_GRID_MAX_TILE_WORDS) * 4,
        PostProcessGraphResourceNames::EXPOSURE_HISTOGRAM => {
            u64::from(EXPOSURE_HISTOGRAM_BIN_COUNT) * 4
        }
        PostProcessGraphResourceNames::EXPOSURE_PREVIOUS
        | PostProcessGraphResourceNames::EXPOSURE_CURRENT => {
            u64::from(EXPOSURE_BUFFER_WORD_COUNT) * 4
        }
        _ => pixel_count.max(1) * 4,
    };
    let usage = match name {
        PostProcessGraphResourceNames::LIGHT_GRID_PARAMS => {
            BufferUsage::UNIFORM | BufferUsage::COPY_DST
        }
        _ => BufferUsage::STORAGE | BufferUsage::COPY_SRC | BufferUsage::COPY_DST,
    };
    BufferDesc::new(name, size_bytes, usage)
}

fn post_process_intermediate_format(name: &str) -> Option<TextureFormat> {
    match name {
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
        PostProcessGraphResourceNames::TONEMAPPED | PostProcessGraphResourceNames::UPSCALED => {
            Some(post_process_texture_format(TONEMAPPED_SDR_FORMAT))
        }
        PostProcessGraphResourceNames::FINAL_COMPOSITED => Some(TextureFormat::Rgba8UnormSrgb),
        PostProcessGraphResourceNames::COLOR_LUT => {
            Some(post_process_texture_format(COLOR_LUT_FORMAT))
        }
        PostProcessGraphResourceNames::TAA_OUTPUT
        | PostProcessGraphResourceNames::SCREEN_SPACE_REFLECTION_REFLECTION_PYRAMID
        | PostProcessGraphResourceNames::SCREEN_SPACE_REFLECTION_REFLECTION_PYRAMID_COARSE => {
            Some(post_process_intermediate_hdr_format())
        }
        PostProcessGraphResourceNames::MOTION_VECTOR_TILE_MAX
        | PostProcessGraphResourceNames::MOTION_VECTOR_TILE_MAX_COARSE
        | PostProcessGraphResourceNames::MOTION_VECTOR_NEIGHBOR_MAX
        | PostProcessGraphResourceNames::HZB_FURTHEST
        | PostProcessGraphResourceNames::TAA_HISTORY_CURRENT => {
            Some(post_process_high_quality_hdr_format())
        }
        PostProcessGraphResourceNames::DEPTH_OF_FIELD_COC
        | PostProcessGraphResourceNames::CONTACT_SHADOW_OCCLUSION
        | PostProcessGraphResourceNames::SCREEN_SPACE_REFLECTION_SPECULAR_OCCLUSION => Some(
            post_process_texture_format(RenderPostProcessTextureFormat::Rgba8Unorm),
        ),
        PostProcessGraphResourceNames::DEPTH_OF_FIELD_BOKEH
        | PostProcessGraphResourceNames::SCREEN_SPACE_REFLECTION_HISTORY => Some(
            post_process_texture_format(RenderPostProcessTextureFormat::Rgba8UnormSrgb),
        ),
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
        PostProcessGraphResourceNames::HZB_FURTHEST => {
            let plan = HzbBuilder::new(UVec2::new(width, height)).build_plan();
            (plan.hzb_size.x, plan.hzb_size.y)
        }
        PostProcessGraphResourceNames::MOTION_VECTOR_TILE_MAX
        | PostProcessGraphResourceNames::SCREEN_SPACE_REFLECTION_REFLECTION_PYRAMID => {
            (half_extent(width), half_extent(height))
        }
        PostProcessGraphResourceNames::MOTION_VECTOR_TILE_MAX_COARSE
        | PostProcessGraphResourceNames::SCREEN_SPACE_REFLECTION_REFLECTION_PYRAMID_COARSE => {
            let half_width = half_extent(width);
            let half_height = half_extent(height);
            (half_extent(half_width), half_extent(half_height))
        }
        _ => (width, height),
    }
}

fn post_process_intermediate_mip_levels(name: &str, width: u32, height: u32) -> u32 {
    match name {
        PostProcessGraphResourceNames::HZB_FURTHEST => full_mip_chain_level_count(width, height),
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

fn is_scene_color_resource(name: &str) -> bool {
    matches!(
        name,
        "scene-color" | "final-color" | "postprocess.terminal-aa-input" | "ambient-occlusion"
    ) || name.starts_with("gbuffer-")
}
