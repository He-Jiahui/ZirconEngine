use crate::asset::AssetUri;
use crate::core::framework::render::{
    RenderImageColorSpace, RenderImageDimension, RenderSamplerAddressMode, RenderSamplerDescriptor,
    RenderSamplerFilter, TextureMetadata, MAX_COLOR_LOOKUP_TEXTURE_SIZE,
    MIN_COLOR_LOOKUP_TEXTURE_SIZE,
};

use super::{TextureAsset, TextureAssetDescriptor, RGBA8_UNORM_FORMAT};

const CHANNELS_PER_SAMPLE: usize = 4;

pub fn texture_asset_from_cube_lut(
    uri: AssetUri,
    source: &str,
) -> Result<TextureAsset, CubeLutParseError> {
    let parsed = parse_cube_lut(source)?;
    let size = parsed.size;
    let rgba = parsed.rgba;
    let defaults = TextureAssetDescriptor::rgba8_srgb();
    let descriptor = TextureAssetDescriptor {
        format: RGBA8_UNORM_FORMAT.to_string(),
        color_space: RenderImageColorSpace::Linear,
        metadata: TextureMetadata {
            color_space: RenderImageColorSpace::Linear,
            ..TextureMetadata::default()
        },
        dimension: RenderImageDimension::D3,
        depth_or_array_layers: size,
        sampler: RenderSamplerDescriptor {
            address_mode_u: RenderSamplerAddressMode::ClampToEdge,
            address_mode_v: RenderSamplerAddressMode::ClampToEdge,
            address_mode_w: RenderSamplerAddressMode::ClampToEdge,
            mag_filter: RenderSamplerFilter::Linear,
            min_filter: RenderSamplerFilter::Linear,
            mipmap_filter: RenderSamplerFilter::Nearest,
        },
        usage: defaults.usage,
        asset_usage: defaults.asset_usage,
        mip_count: 1,
        fallback: defaults.fallback,
    };
    Ok(TextureAsset::new_rgba8(uri, size, size, rgba).with_descriptor(descriptor))
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct CubeLutParseError {
    message: String,
}

impl CubeLutParseError {
    fn new(message: impl Into<String>) -> Self {
        Self {
            message: message.into(),
        }
    }
}

impl std::fmt::Display for CubeLutParseError {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        formatter.write_str(&self.message)
    }
}

impl std::error::Error for CubeLutParseError {}

struct ParsedCubeLut {
    size: u32,
    rgba: Vec<u8>,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum CubeLutLineKind {
    IgnoredMetadata,
    UnsupportedOneDimensional,
    Size3d,
    Sample,
}

fn cube_lut_line_kind(token: &str) -> CubeLutLineKind {
    let first = token
        .as_bytes()
        .first()
        .copied()
        .map(|byte| byte.to_ascii_uppercase());
    match first {
        Some(b'T') if token.eq_ignore_ascii_case("TITLE") => CubeLutLineKind::IgnoredMetadata,
        Some(b'D')
            if token.eq_ignore_ascii_case("DOMAIN_MIN")
                || token.eq_ignore_ascii_case("DOMAIN_MAX") =>
        {
            CubeLutLineKind::IgnoredMetadata
        }
        Some(b'L')
            if token.eq_ignore_ascii_case("LUT_3D_INPUT_RANGE")
                || token.eq_ignore_ascii_case("LUT_IN_VIDEO_RANGE")
                || token.eq_ignore_ascii_case("LUT_OUT_VIDEO_RANGE") =>
        {
            CubeLutLineKind::IgnoredMetadata
        }
        Some(b'L')
            if token.eq_ignore_ascii_case("LUT_1D_SIZE")
                || token.eq_ignore_ascii_case("LUT_1D_INPUT_RANGE") =>
        {
            CubeLutLineKind::UnsupportedOneDimensional
        }
        Some(b'L') if token.eq_ignore_ascii_case("LUT_3D_SIZE") => CubeLutLineKind::Size3d,
        _ => CubeLutLineKind::Sample,
    }
}

fn parse_cube_lut(source: &str) -> Result<ParsedCubeLut, CubeLutParseError> {
    let mut size = None;
    let mut samples = Vec::new();
    for (line_index, raw_line) in source.lines().enumerate() {
        let line_number = line_index + 1;
        let line = raw_line.split('#').next().unwrap_or_default().trim();
        if line.is_empty() {
            continue;
        }
        let mut parts = line.split_whitespace();
        let Some(first) = parts.next() else {
            continue;
        };
        match cube_lut_line_kind(first) {
            CubeLutLineKind::IgnoredMetadata => continue,
            CubeLutLineKind::UnsupportedOneDimensional => {
                return Err(CubeLutParseError::new(format!(
                    "cube LUT 1D shaper sections are not supported at line {line_number}"
                )));
            }
            CubeLutLineKind::Size3d => {
                if size.is_some() {
                    return Err(CubeLutParseError::new(format!(
                        "cube LUT declares LUT_3D_SIZE more than once at line {line_number}"
                    )));
                }
                let token = parts.next().ok_or_else(|| {
                    CubeLutParseError::new(format!(
                        "cube LUT missing LUT_3D_SIZE value at line {line_number}"
                    ))
                })?;
                if parts.next().is_some() {
                    return Err(CubeLutParseError::new(format!(
                        "cube LUT LUT_3D_SIZE must contain one value at line {line_number}"
                    )));
                }
                let parsed_size = parse_size(token, line_number)?;
                if !(MIN_COLOR_LOOKUP_TEXTURE_SIZE..=MAX_COLOR_LOOKUP_TEXTURE_SIZE)
                    .contains(&parsed_size)
                {
                    return Err(CubeLutParseError::new(format!(
                        "cube LUT size {parsed_size} is outside supported range {MIN_COLOR_LOOKUP_TEXTURE_SIZE}..={MAX_COLOR_LOOKUP_TEXTURE_SIZE}"
                    )));
                }
                let expected_bytes = expected_sample_count(parsed_size)?
                    .checked_mul(CHANNELS_PER_SAMPLE)
                    .ok_or_else(|| CubeLutParseError::new("cube LUT byte count overflows usize"))?;
                let reservation_target = expected_bytes.min(source.len());
                samples.reserve_exact(reservation_target.saturating_sub(samples.len()));
                size = Some(parsed_size);
            }
            CubeLutLineKind::Sample => {
                let mut channels = [0.0_f32; 3];
                channels[0] = parse_channel(first, line_number)?;
                for channel in channels.iter_mut().skip(1) {
                    let token = parts.next().ok_or_else(|| {
                        CubeLutParseError::new(format!(
                            "cube LUT RGB sample at line {line_number} must contain 3 values"
                        ))
                    })?;
                    *channel = parse_channel(token, line_number)?;
                }
                if parts.next().is_some() {
                    return Err(CubeLutParseError::new(format!(
                        "cube LUT RGB sample at line {line_number} must contain 3 values"
                    )));
                }
                samples.extend(channels.into_iter().map(float_to_unorm8));
                samples.push(u8::MAX);
            }
        }
    }
    let size = size.ok_or_else(|| CubeLutParseError::new("cube LUT missing LUT_3D_SIZE"))?;
    let expected_samples = expected_sample_count(size)?;
    let actual_samples = samples.len() / CHANNELS_PER_SAMPLE;
    if actual_samples != expected_samples {
        return Err(CubeLutParseError::new(format!(
            "cube LUT expected {expected_samples} RGB samples but found {actual_samples}"
        )));
    }
    Ok(ParsedCubeLut {
        size,
        rgba: samples,
    })
}

fn parse_size(token: &str, line_number: usize) -> Result<u32, CubeLutParseError> {
    token.parse::<u32>().map_err(|_| {
        CubeLutParseError::new(format!(
            "cube LUT size at line {line_number} must be an unsigned integer"
        ))
    })
}

fn parse_channel(token: &str, line_number: usize) -> Result<f32, CubeLutParseError> {
    let value = token.parse::<f32>().map_err(|_| {
        CubeLutParseError::new(format!(
            "cube LUT channel at line {line_number} must be a finite float"
        ))
    })?;
    if !value.is_finite() {
        return Err(CubeLutParseError::new(format!(
            "cube LUT channel at line {line_number} must be finite"
        )));
    }
    Ok(value)
}

fn expected_sample_count(size: u32) -> Result<usize, CubeLutParseError> {
    size.checked_mul(size)
        .and_then(|value| value.checked_mul(size))
        .and_then(|value| usize::try_from(value).ok())
        .ok_or_else(|| CubeLutParseError::new("cube LUT sample count overflows usize"))
}

fn float_to_unorm8(value: f32) -> u8 {
    (value.clamp(0.0, 1.0) * f32::from(u8::MAX)).round() as u8
}

#[cfg(test)]
#[path = "tests/cube_lut.rs"]
mod tests;

#[cfg(test)]
#[path = "cube_lut/tests/optimization_batch_jb_runtime641_tests.rs"]
mod optimization_batch_jb_runtime641_tests;
