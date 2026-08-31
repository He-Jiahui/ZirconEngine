use std::fmt;

use crate::asset::AssetUri;
use crate::core::framework::render::{
    RenderImageColorSpace, RenderImageDimension, TextureCompressionTarget, TextureMipFilter,
    TextureMipPolicy, TextureUsageHint,
};

use super::normal_convention::normalize_texture_normal_map_convention;
use super::{TextureAsset, TextureAssetDescriptor};

pub const DECODED_RGBA8_TEXTURE_BUILD_VERSION: u32 = 2;

const RGBA8_TEXEL_SIZE: usize = 4;
const KAISER_RADIUS: f32 = 2.0;
const KAISER_BETA: f32 = 4.0;
const MAX_KAISER_AXIS_SAMPLES: usize = 5;

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct DecodedRgba8TextureBuildError {
    message: String,
}

impl DecodedRgba8TextureBuildError {
    fn new(message: impl Into<String>) -> Self {
        Self {
            message: message.into(),
        }
    }
}

impl fmt::Display for DecodedRgba8TextureBuildError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str(&self.message)
    }
}

impl std::error::Error for DecodedRgba8TextureBuildError {}

/// Builds an upload-ready decoded RGBA8 texture without importer-global state.
pub fn build_decoded_rgba8_texture(
    uri: AssetUri,
    width: u32,
    height: u32,
    rgba: Vec<u8>,
    descriptor: TextureAssetDescriptor,
) -> Result<TextureAsset, DecodedRgba8TextureBuildError> {
    if width == 0 || height == 0 {
        return Err(DecodedRgba8TextureBuildError::new(format!(
            "decoded rgba8 texture build requires non-zero dimensions for {uri}"
        )));
    }
    if descriptor.dimension != RenderImageDimension::D2
        || descriptor.depth_or_array_layers != 1
        || descriptor.array_layer_count != 1
    {
        return Err(DecodedRgba8TextureBuildError::new(format!(
            "decoded rgba8 texture build supports only a single-layer 2d texture for {uri}; arrays and cubes require a dedicated mip owner"
        )));
    }
    let descriptor = descriptor.normalized();
    if descriptor.metadata.compression != TextureCompressionTarget::Uncompressed {
        return Err(DecodedRgba8TextureBuildError::new(format!(
            "decoded rgba8 texture build requires uncompressed output for {uri}; {:?} needs a platform texture encoder",
            descriptor.metadata.compression
        )));
    }

    let layer_count = descriptor.depth_or_array_layers.max(1);
    let base_layer_len = rgba8_level_len(width, height).ok_or_else(|| {
        DecodedRgba8TextureBuildError::new(format!(
            "decoded rgba8 texture dimensions overflow for {uri}"
        ))
    })?;
    let base_len = base_layer_len
        .checked_mul(layer_count as usize)
        .ok_or_else(|| {
            DecodedRgba8TextureBuildError::new(format!(
                "decoded rgba8 texture layer size overflows for {uri}"
            ))
        })?;
    if rgba.len() != base_len {
        return Err(DecodedRgba8TextureBuildError::new(format!(
            "decoded rgba8 texture build expected {base_len} base bytes for {uri}, found {}",
            rgba.len()
        )));
    }

    let texture = normalize_texture_normal_map_convention(
        TextureAsset::new_rgba8(uri.clone(), width, height, rgba).with_descriptor(descriptor),
    )
    .map_err(|error| DecodedRgba8TextureBuildError::new(error.to_string()))?;
    match texture.texture_descriptor().metadata.mip_policy {
        TextureMipPolicy::GenerateOffline => generate_offline_mips(texture),
        TextureMipPolicy::FromSource | TextureMipPolicy::None => {
            if texture.texture_descriptor().mip_count != 1 {
                return Err(DecodedRgba8TextureBuildError::new(format!(
                    "decoded rgba8 base-only input for {uri} cannot claim {} source mips",
                    texture.texture_descriptor().mip_count
                )));
            }
            Ok(texture)
        }
        TextureMipPolicy::GenerateRuntime => Err(DecodedRgba8TextureBuildError::new(format!(
            "decoded rgba8 runtime mip generation for {uri} requires the GPU build path"
        ))),
    }
}

fn generate_offline_mips(
    mut texture: TextureAsset,
) -> Result<TextureAsset, DecodedRgba8TextureBuildError> {
    let mut descriptor = texture.texture_descriptor();
    if descriptor.mip_count != 1 {
        return Err(DecodedRgba8TextureBuildError::new(format!(
            "decoded rgba8 offline build for {} requires one source mip, found {}",
            texture.uri, descriptor.mip_count
        )));
    }

    let layer_count = descriptor.depth_or_array_layers.max(1) as usize;
    let mip_count = full_mip_count(texture.width, texture.height);
    let total_len = rgba8_mip_chain_len(texture.width, texture.height, mip_count, layer_count)
        .ok_or_else(|| {
            DecodedRgba8TextureBuildError::new(format!(
                "decoded rgba8 mip chain size overflows for {}",
                texture.uri
            ))
        })?;
    let mut packed_mips = std::mem::take(&mut texture.rgba);
    packed_mips.reserve_exact(total_len.saturating_sub(packed_mips.len()));
    packed_mips.resize(total_len, 0);

    let srgb_decode_lut = (descriptor.metadata.usage_hint != TextureUsageHint::Normal
        && descriptor.metadata.color_space == RenderImageColorSpace::Srgb)
        .then(build_srgb_decode_lut);
    let mut current_width = texture.width;
    let mut current_height = texture.height;
    let mut current_level_offset = 0_usize;
    let mut current_layer_len = rgba8_level_len(current_width, current_height)
        .expect("validated decoded rgba8 base extent");
    let mut next_level_offset = current_layer_len * layer_count;

    while current_width > 1 || current_height > 1 {
        let next_width = (current_width / 2).max(1);
        let next_height = (current_height / 2).max(1);
        let next_layer_len =
            rgba8_level_len(next_width, next_height).expect("validated decoded rgba8 mip extent");
        let next_level_len = next_layer_len * layer_count;
        let kaiser_axis_weights = (descriptor.metadata.usage_hint != TextureUsageHint::Normal
            && descriptor.metadata.mip_filter == TextureMipFilter::Kaiser)
            .then(|| {
                let normalizer = bessel_i0(KAISER_BETA);
                (
                    build_kaiser_axis_weights(next_width, current_width, normalizer),
                    build_kaiser_axis_weights(next_height, current_height, normalizer),
                )
            });

        let (source_levels, target_levels) = packed_mips.split_at_mut(next_level_offset);
        let current_level_len = current_layer_len * layer_count;
        let current_level =
            &source_levels[current_level_offset..current_level_offset + current_level_len];
        let next_level = &mut target_levels[..next_level_len];
        for (source, target) in current_level
            .chunks_exact(current_layer_len)
            .zip(next_level.chunks_exact_mut(next_layer_len))
        {
            downsample_rgba8_into(
                source,
                target,
                current_width,
                current_height,
                descriptor.metadata.color_space,
                descriptor.metadata.usage_hint,
                descriptor.metadata.mip_filter,
                srgb_decode_lut.as_ref(),
                kaiser_axis_weights.as_ref(),
            );
        }

        current_level_offset = next_level_offset;
        next_level_offset += next_level_len;
        current_layer_len = next_layer_len;
        current_width = next_width;
        current_height = next_height;
    }

    debug_assert_eq!(next_level_offset, total_len);
    descriptor.mip_count = mip_count;
    texture.rgba = packed_mips;
    texture.descriptor = Some(descriptor);
    Ok(texture)
}

#[derive(Clone, Copy, Debug)]
struct KaiserAxisWeights {
    samples: [(u32, f32); MAX_KAISER_AXIS_SAMPLES],
    len: usize,
}

impl KaiserAxisWeights {
    fn iter(&self) -> impl Iterator<Item = (u32, f32)> + '_ {
        self.samples[..self.len].iter().copied()
    }
}

#[allow(clippy::too_many_arguments)]
fn downsample_rgba8_into(
    source: &[u8],
    target: &mut [u8],
    source_width: u32,
    source_height: u32,
    color_space: RenderImageColorSpace,
    usage_hint: TextureUsageHint,
    mip_filter: TextureMipFilter,
    srgb_decode_lut: Option<&[f32; 256]>,
    kaiser_axis_weights: Option<&(Vec<KaiserAxisWeights>, Vec<KaiserAxisWeights>)>,
) {
    let target_width = (source_width / 2).max(1);
    let target_height = (source_height / 2).max(1);
    debug_assert_eq!(
        target.len(),
        rgba8_level_len(target_width, target_height).unwrap()
    );
    for target_y in 0..target_height {
        for target_x in 0..target_width {
            let pixel = if usage_hint == TextureUsageHint::Normal {
                downsample_normal_pixel(source, source_width, source_height, target_x, target_y)
            } else {
                match mip_filter {
                    TextureMipFilter::Box => downsample_box_color_pixel(
                        source,
                        source_width,
                        source_height,
                        target_x,
                        target_y,
                        color_space,
                        srgb_decode_lut,
                    ),
                    TextureMipFilter::Kaiser => {
                        let (x_weights, y_weights) = kaiser_axis_weights
                            .expect("Kaiser weights are prepared once per mip level");
                        downsample_kaiser_color_pixel(
                            source,
                            source_width,
                            source_height,
                            target_x,
                            target_y,
                            color_space,
                            &x_weights[target_x as usize],
                            &y_weights[target_y as usize],
                            srgb_decode_lut,
                        )
                    }
                }
            };
            let offset = ((target_y * target_width + target_x) as usize) * RGBA8_TEXEL_SIZE;
            target[offset..offset + RGBA8_TEXEL_SIZE].copy_from_slice(&pixel);
        }
    }
}

fn build_srgb_decode_lut() -> [f32; 256] {
    std::array::from_fn(|value| srgb_to_linear(value as f32 / 255.0))
}

fn decode_color_byte(value: u8, srgb_decode_lut: Option<&[f32; 256]>) -> f32 {
    srgb_decode_lut.map_or_else(|| f32::from(value) / 255.0, |lut| lut[value as usize])
}

fn build_kaiser_axis_weights(
    target_extent: u32,
    source_extent: u32,
    normalizer: f32,
) -> Vec<KaiserAxisWeights> {
    (0..target_extent)
        .map(|target| {
            let center = target as f32 * 2.0 + 1.0;
            let min = (center - KAISER_RADIUS).ceil().max(0.0) as u32;
            let max = (center + KAISER_RADIUS)
                .floor()
                .min((source_extent - 1) as f32) as u32;
            let mut weights = KaiserAxisWeights {
                samples: [(0, 0.0); MAX_KAISER_AXIS_SAMPLES],
                len: 0,
            };
            for source in min..=max {
                debug_assert!(weights.len < MAX_KAISER_AXIS_SAMPLES);
                weights.samples[weights.len] = (
                    source,
                    kaiser_weight(source as f32 + 0.5 - center, normalizer),
                );
                weights.len += 1;
            }
            weights
        })
        .collect()
}

fn downsample_box_color_pixel(
    source: &[u8],
    source_width: u32,
    source_height: u32,
    target_x: u32,
    target_y: u32,
    color_space: RenderImageColorSpace,
    srgb_decode_lut: Option<&[f32; 256]>,
) -> [u8; RGBA8_TEXEL_SIZE] {
    let mut sums = [0.0; RGBA8_TEXEL_SIZE];
    let source_x = target_x * 2;
    let source_y = target_y * 2;
    if source_x + 1 < source_width && source_y + 1 < source_height {
        let row_stride = source_width as usize * RGBA8_TEXEL_SIZE;
        let top_left =
            (source_y as usize * source_width as usize + source_x as usize) * RGBA8_TEXEL_SIZE;
        for offset in [
            top_left,
            top_left + RGBA8_TEXEL_SIZE,
            top_left + row_stride,
            top_left + row_stride + RGBA8_TEXEL_SIZE,
        ] {
            for channel in 0..3 {
                sums[channel] += decode_color_byte(source[offset + channel], srgb_decode_lut);
            }
            sums[3] += f32::from(source[offset + 3]) / 255.0;
        }
        return encode_weighted_pixel(sums, 4.0, color_space);
    }

    let mut sample_count = 0.0;
    for source_y in source_y..((source_y + 2).min(source_height)) {
        for source_x in source_x..((source_x + 2).min(source_width)) {
            let offset = ((source_y * source_width + source_x) as usize) * RGBA8_TEXEL_SIZE;
            for channel in 0..3 {
                sums[channel] += decode_color_byte(source[offset + channel], srgb_decode_lut);
            }
            sums[3] += f32::from(source[offset + 3]) / 255.0;
            sample_count += 1.0;
        }
    }
    encode_weighted_pixel(sums, sample_count, color_space)
}

#[allow(clippy::too_many_arguments)]
fn downsample_kaiser_color_pixel(
    source: &[u8],
    source_width: u32,
    source_height: u32,
    target_x: u32,
    target_y: u32,
    color_space: RenderImageColorSpace,
    x_weights: &KaiserAxisWeights,
    y_weights: &KaiserAxisWeights,
    srgb_decode_lut: Option<&[f32; 256]>,
) -> [u8; RGBA8_TEXEL_SIZE] {
    let mut sums = [0.0; RGBA8_TEXEL_SIZE];
    let mut weight_sum = 0.0;
    for (source_y, weight_y) in y_weights.iter() {
        for (source_x, weight_x) in x_weights.iter() {
            let weight = weight_y * weight_x;
            let offset = ((source_y * source_width + source_x) as usize) * RGBA8_TEXEL_SIZE;
            for channel in 0..3 {
                sums[channel] +=
                    weight * decode_color_byte(source[offset + channel], srgb_decode_lut);
            }
            sums[3] += weight * f32::from(source[offset + 3]) / 255.0;
            weight_sum += weight;
        }
    }
    if weight_sum <= f32::EPSILON {
        return downsample_box_color_pixel(
            source,
            source_width,
            source_height,
            target_x,
            target_y,
            color_space,
            srgb_decode_lut,
        );
    }
    encode_weighted_pixel(sums, weight_sum, color_space)
}

fn downsample_normal_pixel(
    source: &[u8],
    source_width: u32,
    source_height: u32,
    target_x: u32,
    target_y: u32,
) -> [u8; RGBA8_TEXEL_SIZE] {
    let mut normal = [0.0; 3];
    let mut alpha = 0.0;
    let mut sample_count = 0.0;
    for source_y in target_y * 2..((target_y * 2 + 2).min(source_height)) {
        for source_x in target_x * 2..((target_x * 2 + 2).min(source_width)) {
            let offset = ((source_y * source_width + source_x) as usize) * RGBA8_TEXEL_SIZE;
            for channel in 0..3 {
                normal[channel] += f32::from(source[offset + channel]) / 127.5 - 1.0;
            }
            alpha += f32::from(source[offset + 3]) / 255.0;
            sample_count += 1.0;
        }
    }
    let length = normal.iter().map(|value| value * value).sum::<f32>().sqrt();
    let normal = if length > f32::EPSILON {
        normal.map(|value| value / length)
    } else {
        [0.0, 0.0, 1.0]
    };
    [
        encode_unorm8(normal[0] * 0.5 + 0.5),
        encode_unorm8(normal[1] * 0.5 + 0.5),
        encode_unorm8(normal[2] * 0.5 + 0.5),
        encode_unorm8(alpha / sample_count),
    ]
}

fn encode_weighted_pixel(
    sums: [f32; RGBA8_TEXEL_SIZE],
    weight_sum: f32,
    color_space: RenderImageColorSpace,
) -> [u8; RGBA8_TEXEL_SIZE] {
    let mut pixel = [0; RGBA8_TEXEL_SIZE];
    for channel in 0..3 {
        let average = sums[channel] / weight_sum;
        pixel[channel] = encode_unorm8(if color_space == RenderImageColorSpace::Srgb {
            linear_to_srgb(average)
        } else {
            average
        });
    }
    pixel[3] = encode_unorm8(sums[3] / weight_sum);
    pixel
}

fn kaiser_weight(distance: f32, normalizer: f32) -> f32 {
    let normalized = distance.abs() / KAISER_RADIUS;
    if normalized >= 1.0 {
        return 0.0;
    }
    let window = bessel_i0(KAISER_BETA * (1.0 - normalized * normalized).sqrt()) / normalizer;
    let phase = distance * 0.5;
    let sinc = if phase.abs() <= f32::EPSILON {
        1.0
    } else {
        (std::f32::consts::PI * phase).sin() / (std::f32::consts::PI * phase)
    };
    sinc * window
}

fn bessel_i0(value: f32) -> f32 {
    let mut term = 1.0;
    let mut sum = 1.0;
    for index in 1..=10 {
        let index = index as f32;
        term *= value * value / (4.0 * index * index);
        sum += term;
    }
    sum
}

fn full_mip_count(mut width: u32, mut height: u32) -> u32 {
    let mut count = 1;
    while width > 1 || height > 1 {
        width = (width / 2).max(1);
        height = (height / 2).max(1);
        count += 1;
    }
    count
}

fn rgba8_mip_chain_len(
    width: u32,
    height: u32,
    mip_count: u32,
    layer_count: usize,
) -> Option<usize> {
    (0..mip_count).try_fold(0_usize, |total, level| {
        let level_len = rgba8_level_len(mip_extent(width, level), mip_extent(height, level))?;
        total.checked_add(level_len.checked_mul(layer_count)?)
    })
}

const fn mip_extent(value: u32, level: u32) -> u32 {
    if level >= u32::BITS {
        1
    } else {
        let shifted = value >> level;
        if shifted == 0 {
            1
        } else {
            shifted
        }
    }
}

fn rgba8_level_len(width: u32, height: u32) -> Option<usize> {
    (width as usize)
        .checked_mul(height as usize)?
        .checked_mul(RGBA8_TEXEL_SIZE)
}

fn srgb_to_linear(value: f32) -> f32 {
    if value <= 0.04045 {
        value / 12.92
    } else {
        ((value + 0.055) / 1.055).powf(2.4)
    }
}

fn linear_to_srgb(value: f32) -> f32 {
    if value <= 0.003_130_8 {
        value * 12.92
    } else {
        1.055 * value.powf(1.0 / 2.4) - 0.055
    }
}

fn encode_unorm8(value: f32) -> u8 {
    (value.clamp(0.0, 1.0) * 255.0).round() as u8
}

#[cfg(test)]
mod tests {
    use crate::core::framework::render::{
        RenderImageColorSpace, TextureMipPolicy, TextureNormalConvention, TextureUsageHint,
    };

    use super::*;

    fn descriptor(usage: TextureUsageHint) -> TextureAssetDescriptor {
        let mut descriptor = TextureAssetDescriptor::decoded_rgba8_for_import_usage(usage);
        descriptor.metadata.mip_policy = TextureMipPolicy::GenerateOffline;
        descriptor
    }

    #[test]
    fn offline_build_packs_complete_chain_and_preserves_base_payload() {
        let mut rgba = Vec::with_capacity(84);
        rgba.extend((0_u8..64).map(|value| value.saturating_mul(3)));
        let base = rgba.clone();
        let pointer = rgba.as_ptr();

        let texture = build_decoded_rgba8_texture(
            AssetUri::parse("res://textures/packed.png").unwrap(),
            4,
            4,
            rgba,
            descriptor(TextureUsageHint::Data),
        )
        .unwrap();

        assert_eq!(texture.texture_descriptor().mip_count, 3);
        assert_eq!(texture.rgba.len(), 84);
        assert_eq!(&texture.rgba[..64], base.as_slice());
        assert_eq!(texture.rgba.as_ptr(), pointer);
        assert!(texture
            .upload_readiness(super::super::TextureUploadSupport::uncompressed_only())
            .is_ready());
    }

    #[test]
    fn srgb_box_filter_averages_in_linear_space() {
        let mut descriptor = descriptor(TextureUsageHint::Albedo);
        descriptor.metadata.mip_filter = TextureMipFilter::Box;
        descriptor.color_space = RenderImageColorSpace::Srgb;
        descriptor.metadata.color_space = RenderImageColorSpace::Srgb;
        let texture = build_decoded_rgba8_texture(
            AssetUri::parse("res://textures/srgb.png").unwrap(),
            2,
            2,
            vec![
                0, 0, 0, 255, 255, 255, 255, 255, 0, 0, 0, 255, 255, 255, 255, 255,
            ],
            descriptor,
        )
        .unwrap();

        assert_eq!(&texture.rgba[16..20], &[188, 188, 188, 255]);
    }

    #[test]
    fn normal_mip_is_renormalized_after_dx_to_gl_projection() {
        let mut descriptor = descriptor(TextureUsageHint::Normal);
        descriptor.metadata.normal_convention = TextureNormalConvention::TangentSpaceDx;
        let texture = build_decoded_rgba8_texture(
            AssetUri::parse("res://textures/normal.png").unwrap(),
            2,
            2,
            vec![
                255, 128, 128, 255, 128, 255, 128, 255, 255, 128, 128, 255, 128, 255, 128, 255,
            ],
            descriptor,
        )
        .unwrap();

        let mip = &texture.rgba[16..20];
        assert!(mip[0] >= 217 && mip[1] <= 38);
        assert!((126..=130).contains(&mip[2]));
        assert_eq!(mip[3], 255);
        assert_eq!(
            texture.texture_descriptor().metadata.normal_convention,
            TextureNormalConvention::TangentSpaceGl
        );
    }

    #[test]
    fn decoded_build_rejects_truncated_base_payload() {
        let error = build_decoded_rgba8_texture(
            AssetUri::parse("res://textures/truncated.png").unwrap(),
            2,
            2,
            vec![0; 15],
            descriptor(TextureUsageHint::Data),
        )
        .unwrap_err();

        assert!(error.to_string().contains("expected 16 base bytes"));
    }

    #[test]
    fn decoded_build_rejects_mismatched_array_shape() {
        let mut descriptor = descriptor(TextureUsageHint::Data);
        descriptor.depth_or_array_layers = 2;
        descriptor.array_layer_count = 1;
        let error = build_decoded_rgba8_texture(
            AssetUri::parse("res://textures/array.png").unwrap(),
            1,
            1,
            vec![0; 8],
            descriptor,
        )
        .unwrap_err();

        assert!(error.to_string().contains("single-layer 2d"));
    }

    #[test]
    fn decoded_build_rejects_matching_array_layers_without_an_array_mip_owner() {
        let mut descriptor = descriptor(TextureUsageHint::Data);
        descriptor.depth_or_array_layers = 2;
        descriptor.array_layer_count = 2;

        let error = build_decoded_rgba8_texture(
            AssetUri::parse("res://textures/array.png").unwrap(),
            1,
            1,
            vec![0; 8],
            descriptor,
        )
        .unwrap_err();

        assert!(error.to_string().contains("single-layer 2d"));
    }

    #[test]
    fn decoded_build_rejects_cube_without_a_seam_aware_mip_owner() {
        let mut descriptor = descriptor(TextureUsageHint::Data);
        descriptor.dimension = RenderImageDimension::Cube;
        descriptor.depth_or_array_layers = 6;
        descriptor.array_layer_count = 6;

        let error = build_decoded_rgba8_texture(
            AssetUri::parse("res://textures/cube.png").unwrap(),
            1,
            1,
            vec![0; 24],
            descriptor,
        )
        .unwrap_err();

        assert!(error.to_string().contains("single-layer 2d"));
    }
}
