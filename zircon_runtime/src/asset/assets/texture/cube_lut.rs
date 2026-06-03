use crate::asset::AssetUri;
use crate::core::framework::render::{
    RenderImageColorSpace, RenderImageDimension, RenderSamplerAddressMode, RenderSamplerDescriptor,
    RenderSamplerFilter, MAX_COLOR_LOOKUP_TEXTURE_SIZE, MIN_COLOR_LOOKUP_TEXTURE_SIZE,
};

use super::{TextureAsset, TextureAssetDescriptor, RGBA8_UNORM_FORMAT};

pub fn texture_asset_from_cube_lut(
    uri: AssetUri,
    source: &str,
) -> Result<TextureAsset, CubeLutParseError> {
    let parsed = parse_cube_lut(source)?;
    let size = parsed.size;
    let rgba = parsed.rgba;
    let descriptor = TextureAssetDescriptor {
        format: RGBA8_UNORM_FORMAT.to_string(),
        color_space: RenderImageColorSpace::Linear,
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
        usage: TextureAssetDescriptor::rgba8_srgb().usage,
        asset_usage: TextureAssetDescriptor::rgba8_srgb().asset_usage,
        mip_count: 1,
        array_layer_count: 1,
        fallback: TextureAssetDescriptor::rgba8_srgb().fallback,
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
        let keyword = first.to_ascii_uppercase();
        match keyword.as_str() {
            "TITLE"
            | "DOMAIN_MIN"
            | "DOMAIN_MAX"
            | "LUT_3D_INPUT_RANGE"
            | "LUT_IN_VIDEO_RANGE"
            | "LUT_OUT_VIDEO_RANGE" => continue,
            "LUT_1D_SIZE" | "LUT_1D_INPUT_RANGE" => {
                return Err(CubeLutParseError::new(format!(
                    "cube LUT 1D shaper sections are not supported at line {line_number}"
                )));
            }
            "LUT_3D_SIZE" => {
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
                size = Some(parsed_size);
            }
            _ => {
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
    let actual_samples = samples.len() / 4;
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
mod tests {
    use super::*;
    use crate::asset::AssetUri;

    #[test]
    fn cube_lut_parser_clamps_channels_and_builds_3d_texture() {
        let texture = texture_asset_from_cube_lut(
            AssetUri::parse("res://textures/clamped.cube").unwrap(),
            "\
LUT_3D_SIZE 2
-1.0 0.0 0.5
1.5 0.25 0.0
0.0 1.0 0.0
1.0 1.0 0.0
0.0 0.0 1.0
1.0 0.0 1.0
0.0 1.0 1.0
1.0 1.0 1.0
",
        )
        .expect("valid cube lut");

        assert_eq!(texture.rgba[0..8], [0, 0, 128, 255, 255, 64, 0, 255]);
        assert_eq!(
            texture.render_image_descriptor().dimension,
            RenderImageDimension::D3
        );
    }

    #[test]
    fn cube_lut_parser_rejects_out_of_range_sizes() {
        let error = texture_asset_from_cube_lut(
            AssetUri::parse("res://textures/tiny.cube").unwrap(),
            "\
LUT_3D_SIZE 1
0.0 0.0 0.0
",
        )
        .expect_err("invalid size");

        assert!(
            error
                .to_string()
                .contains("outside supported range 2..=256"),
            "unexpected error: {error}"
        );
    }

    #[test]
    fn cube_lut_parser_ignores_common_metadata_rows() {
        let texture = texture_asset_from_cube_lut(
            AssetUri::parse("res://textures/metadata.cube").unwrap(),
            "\
TITLE \"metadata\"
LUT_3D_INPUT_RANGE 0.0 1.0
LUT_IN_VIDEO_RANGE 0
LUT_OUT_VIDEO_RANGE 0
LUT_3D_SIZE 2
0.0 0.0 0.0
1.0 0.0 0.0
0.0 1.0 0.0
1.0 1.0 0.0
0.0 0.0 1.0
1.0 0.0 1.0
0.0 1.0 1.0
1.0 1.0 1.0
",
        )
        .expect("cube metadata rows should not be treated as samples");

        assert_eq!(texture.rgba.len(), 2 * 2 * 2 * 4);
    }

    #[test]
    fn cube_lut_parser_rejects_1d_shaper_sections() {
        let error = texture_asset_from_cube_lut(
            AssetUri::parse("res://textures/shaper.cube").unwrap(),
            "\
LUT_1D_SIZE 2
0.0 0.0 0.0
1.0 1.0 1.0
LUT_3D_SIZE 2
",
        )
        .expect_err("1D shaper sections are unsupported");

        assert!(
            error
                .to_string()
                .contains("1D shaper sections are not supported"),
            "unexpected error: {error}"
        );
    }
}
