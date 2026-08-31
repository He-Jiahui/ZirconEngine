use crate::asset::AssetUri;
use crate::core::framework::render::{
    RenderImageColorSpace, RenderImageDimension, RenderSamplerAddressMode, RenderSamplerDescriptor,
    RenderSamplerFilter, TextureMetadata, MAX_COLOR_LOOKUP_TEXTURE_SIZE,
    MIN_COLOR_LOOKUP_TEXTURE_SIZE,
};

use super::{TextureAsset, TextureAssetDescriptor, RGBA8_UNORM_FORMAT};

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
        array_layer_count: 1,
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
    use std::hint::black_box;
    use std::time::Instant;

    use super::*;
    use crate::asset::AssetUri;

    const SAMPLE_PAIRS: usize = 21;
    const KEYWORD_CHECKS_PER_SAMPLE: usize = 180_000;
    const KEYWORDS: [&str; 10] = [
        "TITLE",
        "domain_min",
        "DOMAIN_MAX",
        "lut_3d_input_range",
        "LUT_IN_VIDEO_RANGE",
        "lut_out_video_range",
        "LUT_1D_SIZE",
        "lut_1d_input_range",
        "LuT_3D_SiZe",
        "0.125",
    ];

    #[test]
    fn borrowed_import_keyword_contract_cube_lut_directives() {
        assert_eq!(
            cube_lut_line_kind("title"),
            CubeLutLineKind::IgnoredMetadata
        );
        assert_eq!(
            cube_lut_line_kind("LuT_1D_SiZe"),
            CubeLutLineKind::UnsupportedOneDimensional
        );
        assert_eq!(cube_lut_line_kind("lut_3d_size"), CubeLutLineKind::Size3d);
        assert_eq!(cube_lut_line_kind("0.25"), CubeLutLineKind::Sample);
    }

    #[test]
    #[ignore = "release performance gate"]
    fn borrowed_import_keyword_performance_release_cube_lut_directives() {
        let mut legacy_samples = Vec::with_capacity(SAMPLE_PAIRS);
        let mut optimized_samples = Vec::with_capacity(SAMPLE_PAIRS);
        for pair_index in 0..SAMPLE_PAIRS {
            let (legacy_ns, optimized_ns) = if pair_index % 2 == 0 {
                (measure_legacy_keywords(), measure_borrowed_keywords())
            } else {
                let optimized_ns = measure_borrowed_keywords();
                (measure_legacy_keywords(), optimized_ns)
            };
            legacy_samples.push(legacy_ns);
            optimized_samples.push(optimized_ns);
        }

        report_keyword_performance(
            "plugins07_cube_lut_borrowed_keyword",
            KEYWORD_CHECKS_PER_SAMPLE,
            KEYWORD_CHECKS_PER_SAMPLE,
            &legacy_samples,
            &optimized_samples,
        );
    }

    fn measure_legacy_keywords() -> u128 {
        let started = Instant::now();
        let mut classified = 0_u64;
        for check in 0..KEYWORD_CHECKS_PER_SAMPLE {
            let keyword = black_box(KEYWORDS[check % KEYWORDS.len()]).to_ascii_uppercase();
            classified += u64::from(matches!(
                keyword.as_str(),
                "TITLE"
                    | "DOMAIN_MIN"
                    | "DOMAIN_MAX"
                    | "LUT_3D_INPUT_RANGE"
                    | "LUT_IN_VIDEO_RANGE"
                    | "LUT_OUT_VIDEO_RANGE"
                    | "LUT_1D_SIZE"
                    | "LUT_1D_INPUT_RANGE"
                    | "LUT_3D_SIZE"
            ));
            black_box(keyword);
        }
        black_box(classified);
        started.elapsed().as_nanos().max(1)
    }

    fn measure_borrowed_keywords() -> u128 {
        let started = Instant::now();
        let mut classified = 0_u64;
        for check in 0..KEYWORD_CHECKS_PER_SAMPLE {
            classified += u64::from(!matches!(
                cube_lut_line_kind(black_box(KEYWORDS[check % KEYWORDS.len()])),
                CubeLutLineKind::Sample
            ));
        }
        black_box(classified);
        started.elapsed().as_nanos().max(1)
    }

    fn report_keyword_performance(
        name: &str,
        checks_per_sample: usize,
        legacy_allocations_per_sample: usize,
        legacy_samples: &[u128],
        optimized_samples: &[u128],
    ) {
        let legacy_p95 = nearest_rank_p95(legacy_samples);
        let optimized_p95 = nearest_rank_p95(optimized_samples);
        let improvement_percent =
            legacy_p95.saturating_sub(optimized_p95).saturating_mul(100) / legacy_p95.max(1);
        println!(
            "PERF_RESULT {name} sample_pairs={SAMPLE_PAIRS} checks_per_sample={checks_per_sample} legacy_ns={} optimized_ns={} legacy_p95_ns={legacy_p95} optimized_p95_ns={optimized_p95} improvement_percent={improvement_percent} threshold_percent=50 legacy_allocations_per_sample={legacy_allocations_per_sample} optimized_allocations_per_sample=0 order=alternating_legacy_first_even legacy_first_pairs=11 optimized_first_pairs=10",
            csv(legacy_samples),
            csv(optimized_samples),
        );
        assert!(
            improvement_percent >= 50,
            "borrowed cube LUT keyword classification must improve P95 by at least 50%"
        );
    }

    fn nearest_rank_p95(samples: &[u128]) -> u128 {
        let mut sorted = samples.to_vec();
        sorted.sort_unstable();
        let rank = (sorted.len() * 95).div_ceil(100);
        sorted[rank.saturating_sub(1)]
    }

    fn csv(samples: &[u128]) -> String {
        samples
            .iter()
            .map(u128::to_string)
            .collect::<Vec<_>>()
            .join(",")
    }

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
