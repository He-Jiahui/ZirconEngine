use std::borrow::Cow;

use crate::core::framework::render::{
    RenderImageAssetUsage, RenderImageColorSpace, RenderImageDimension, RenderImageUsage,
    RenderSamplerAddressMode, RenderSamplerDescriptor, RenderSamplerFilter,
    TextureCompressionTarget, TextureMipFilter, TextureMipPolicy, TextureNormalConvention,
    TextureUsageHint,
};

use super::{TextureArrayLayout, TextureDescriptorError, TextureDescriptorResult};

#[derive(Clone, Copy, Debug, Default)]
pub(super) struct ExtentSettingKeys {
    pub(super) array_layer_count: Option<&'static str>,
    pub(super) depth_or_array_layers: Option<&'static str>,
}

pub(super) fn string_setting<'a>(
    name: &str,
    value: &'a toml::Value,
) -> TextureDescriptorResult<&'a str> {
    value
        .as_str()
        .ok_or_else(|| TextureDescriptorError::setting_type(name, "a string"))
}

pub(super) fn u32_setting(name: &str, value: &toml::Value) -> TextureDescriptorResult<u32> {
    let Some(value) = value.as_integer() else {
        return Err(TextureDescriptorError::setting_type(name, "an integer"));
    };
    u32::try_from(value).map_err(|_| TextureDescriptorError::setting_u32_overflow(name))
}

pub(super) fn u8_setting(name: &str, value: &toml::Value) -> TextureDescriptorResult<u8> {
    let value = u32_setting(name, value)?;
    u8::try_from(value).map_err(|_| TextureDescriptorError::setting_u8_overflow(name))
}

pub(super) fn f32_setting(name: &str, value: &toml::Value) -> TextureDescriptorResult<f32> {
    let value = value
        .as_float()
        .or_else(|| value.as_integer().map(|value| value as f64))
        .ok_or_else(|| TextureDescriptorError::setting_type(name, "a number"))?;
    if !value.is_finite() || value < f32::MIN as f64 || value > f32::MAX as f64 {
        return Err(TextureDescriptorError::setting_f32_range(name));
    }
    Ok(value as f32)
}

pub(super) fn bool_setting(name: &str, value: &toml::Value) -> TextureDescriptorResult<bool> {
    value
        .as_bool()
        .ok_or_else(|| TextureDescriptorError::setting_type(name, "a boolean"))
}

pub(super) fn parse_usage_list(
    name: &str,
    value: &toml::Value,
) -> TextureDescriptorResult<Vec<RenderImageUsage>> {
    if let Some(value) = value.as_str() {
        return Ok(vec![parse_usage(value)?]);
    }
    let Some(values) = value.as_array() else {
        return Err(TextureDescriptorError::setting_type(
            name,
            "a string or array of strings",
        ));
    };
    values
        .iter()
        .map(|value| parse_usage(string_setting(name, value)?))
        .collect()
}

pub(super) fn parse_asset_usage_list(
    name: &str,
    value: &toml::Value,
) -> TextureDescriptorResult<Vec<RenderImageAssetUsage>> {
    if let Some(value) = value.as_str() {
        return Ok(vec![parse_asset_usage(name, value)?]);
    }
    let Some(values) = value.as_array() else {
        return Err(TextureDescriptorError::setting_type(
            name,
            "a string or array of strings",
        ));
    };
    values
        .iter()
        .map(|value| parse_asset_usage(name, string_setting(name, value)?))
        .collect()
}

pub(super) fn parse_sampler(
    value: &toml::Value,
    mut sampler: RenderSamplerDescriptor,
) -> TextureDescriptorResult<RenderSamplerDescriptor> {
    if let Some(value) = value.as_str() {
        return parse_sampler_shorthand(value, sampler);
    }
    let Some(table) = value.as_table() else {
        return Err(TextureDescriptorError::setting_type(
            "sampler",
            "a table or string",
        ));
    };
    if let Some(value) = table.get("address_mode_u") {
        sampler.address_mode_u =
            parse_address_mode(string_setting("sampler.address_mode_u", value)?)?;
    }
    if let Some(value) = table.get("address_mode_v") {
        sampler.address_mode_v =
            parse_address_mode(string_setting("sampler.address_mode_v", value)?)?;
    }
    if let Some(value) = table.get("address_mode_w") {
        sampler.address_mode_w =
            parse_address_mode(string_setting("sampler.address_mode_w", value)?)?;
    }
    if let Some(value) = table.get("mag_filter") {
        sampler.mag_filter = parse_filter(string_setting("sampler.mag_filter", value)?)?;
    }
    if let Some(value) = table.get("min_filter") {
        sampler.min_filter = parse_filter(string_setting("sampler.min_filter", value)?)?;
    }
    if let Some(value) = table.get("mipmap_filter") {
        sampler.mipmap_filter = parse_filter(string_setting("sampler.mipmap_filter", value)?)?;
    }
    Ok(sampler)
}

pub(super) fn parse_array_layout(
    value: &toml::Value,
) -> TextureDescriptorResult<TextureArrayLayout> {
    let Some(table) = value.as_table() else {
        return Err(TextureDescriptorError::setting_type(
            "array_layout",
            "a table",
        ));
    };
    match (table.get("row_count"), table.get("row_height")) {
        (Some(rows), None) => Ok(TextureArrayLayout::RowCount {
            rows: u32_setting("array_layout.row_count", rows)?,
        }),
        (None, Some(pixels)) => Ok(TextureArrayLayout::RowHeight {
            pixels: u32_setting("array_layout.row_height", pixels)?,
        }),
        (Some(_), Some(_)) => Err(TextureDescriptorError::ArrayLayoutExclusiveMode),
        (None, None) => Err(TextureDescriptorError::ArrayLayoutMissingMode),
    }
}

pub(super) fn parse_color_space(value: &str) -> TextureDescriptorResult<RenderImageColorSpace> {
    match normalized_token(value).as_ref() {
        "srgb" => Ok(RenderImageColorSpace::Srgb),
        "linear" => Ok(RenderImageColorSpace::Linear),
        "hdr" => Ok(RenderImageColorSpace::Hdr),
        _ => Err(TextureDescriptorError::unsupported("color_space", value)),
    }
}

pub(super) fn parse_usage_hint(value: &str) -> TextureDescriptorResult<TextureUsageHint> {
    match normalized_token(value).as_ref() {
        "albedo" => Ok(TextureUsageHint::Albedo),
        "normal" => Ok(TextureUsageHint::Normal),
        "mask" => Ok(TextureUsageHint::Mask),
        "data" => Ok(TextureUsageHint::Data),
        "hdr" => Ok(TextureUsageHint::Hdr),
        "ui" => Ok(TextureUsageHint::Ui),
        _ => Err(TextureDescriptorError::unsupported("usage_hint", value)),
    }
}

pub(super) fn parse_mip_policy(value: &str) -> TextureDescriptorResult<TextureMipPolicy> {
    match normalized_token(value).as_ref() {
        "from_source" => Ok(TextureMipPolicy::FromSource),
        "generate_offline" => Ok(TextureMipPolicy::GenerateOffline),
        "generate_runtime" => Ok(TextureMipPolicy::GenerateRuntime),
        "none" => Ok(TextureMipPolicy::None),
        _ => Err(TextureDescriptorError::unsupported("mip_policy", value)),
    }
}

pub(super) fn parse_mip_filter(value: &str) -> TextureDescriptorResult<TextureMipFilter> {
    match normalized_token(value).as_ref() {
        "kaiser" => Ok(TextureMipFilter::Kaiser),
        "box" => Ok(TextureMipFilter::Box),
        _ => Err(TextureDescriptorError::unsupported("mip_filter", value)),
    }
}

pub(super) fn parse_normal_convention(
    value: &str,
) -> TextureDescriptorResult<TextureNormalConvention> {
    match normalized_token(value).as_ref() {
        "none" => Ok(TextureNormalConvention::None),
        "tangent_space_dx" | "dx" => Ok(TextureNormalConvention::TangentSpaceDx),
        "tangent_space_gl" | "gl" => Ok(TextureNormalConvention::TangentSpaceGl),
        _ => Err(TextureDescriptorError::unsupported(
            "normal_convention",
            value,
        )),
    }
}

pub(super) fn parse_compression(value: &str) -> TextureDescriptorResult<TextureCompressionTarget> {
    match normalized_token(value).as_ref() {
        "auto" => Ok(TextureCompressionTarget::Auto),
        "uncompressed" => Ok(TextureCompressionTarget::Uncompressed),
        "bc1" => Ok(TextureCompressionTarget::Bc1),
        "bc4" => Ok(TextureCompressionTarget::Bc4),
        "bc5" => Ok(TextureCompressionTarget::Bc5),
        "bc6h" => Ok(TextureCompressionTarget::Bc6h),
        "bc7" => Ok(TextureCompressionTarget::Bc7),
        _ => Err(TextureDescriptorError::unsupported("compression", value)),
    }
}

pub(super) fn parse_dimension(value: &str) -> TextureDescriptorResult<RenderImageDimension> {
    match normalized_token(value).as_ref() {
        "1d" | "d1" => Ok(RenderImageDimension::D1),
        "2d" | "d2" => Ok(RenderImageDimension::D2),
        "3d" | "d3" => Ok(RenderImageDimension::D3),
        "cube" | "cubemap" => Ok(RenderImageDimension::Cube),
        _ => Err(TextureDescriptorError::unsupported("dimension", value)),
    }
}

fn parse_sampler_shorthand(
    value: &str,
    sampler: RenderSamplerDescriptor,
) -> TextureDescriptorResult<RenderSamplerDescriptor> {
    match normalized_token(value).as_ref() {
        "default" => Ok(sampler),
        "linear" => Ok(sampler_with_filter(sampler, RenderSamplerFilter::Linear)),
        "nearest" => Ok(sampler_with_filter(sampler, RenderSamplerFilter::Nearest)),
        _ => Err(TextureDescriptorError::unsupported("sampler", value)),
    }
}

fn sampler_with_filter(
    mut sampler: RenderSamplerDescriptor,
    filter: RenderSamplerFilter,
) -> RenderSamplerDescriptor {
    sampler.mag_filter = filter;
    sampler.min_filter = filter;
    sampler.mipmap_filter = filter;
    sampler
}

fn parse_usage(value: &str) -> TextureDescriptorResult<RenderImageUsage> {
    match normalized_token(value).as_ref() {
        "sampled" => Ok(RenderImageUsage::Sampled),
        "storage" => Ok(RenderImageUsage::Storage),
        "render_target" => Ok(RenderImageUsage::RenderTarget),
        "copy_src" => Ok(RenderImageUsage::CopySrc),
        "copy_dst" => Ok(RenderImageUsage::CopyDst),
        _ => Err(TextureDescriptorError::unsupported("usage", value)),
    }
}

fn parse_asset_usage(name: &str, value: &str) -> TextureDescriptorResult<RenderImageAssetUsage> {
    match normalized_token(value).as_ref() {
        "main_world" | "main" | "cpu" => Ok(RenderImageAssetUsage::MainWorld),
        "render_world" | "render" | "gpu" => Ok(RenderImageAssetUsage::RenderWorld),
        _ => Err(TextureDescriptorError::unsupported(name, value)),
    }
}

fn parse_address_mode(value: &str) -> TextureDescriptorResult<RenderSamplerAddressMode> {
    match normalized_token(value).as_ref() {
        "clamp_to_edge" => Ok(RenderSamplerAddressMode::ClampToEdge),
        "repeat" => Ok(RenderSamplerAddressMode::Repeat),
        "mirror_repeat" => Ok(RenderSamplerAddressMode::MirrorRepeat),
        _ => Err(TextureDescriptorError::unsupported(
            "sampler address mode",
            value,
        )),
    }
}

fn parse_filter(value: &str) -> TextureDescriptorResult<RenderSamplerFilter> {
    match normalized_token(value).as_ref() {
        "nearest" => Ok(RenderSamplerFilter::Nearest),
        "linear" => Ok(RenderSamplerFilter::Linear),
        _ => Err(TextureDescriptorError::unsupported("sampler filter", value)),
    }
}

fn normalized_token(value: &str) -> Cow<'_, str> {
    let value = value.trim();
    if value
        .bytes()
        .any(|byte| byte.is_ascii_uppercase() || byte == b'-')
    {
        Cow::Owned(value.to_ascii_lowercase().replace('-', "_"))
    } else {
        Cow::Borrowed(value)
    }
}

#[cfg(test)]
mod plugins07_setting_token_hotpath_tests {
    use std::hint::black_box;
    use std::time::Instant;

    use super::*;

    const SAMPLE_PAIRS: usize = 21;
    const LOOKUPS_PER_SAMPLE: usize = 80_000;
    const TOKENS: [&str; 3] = ["render_target", "copy_src", "copy_dst"];

    #[test]
    fn borrowed_texture_metadata_contract_setting_token() {
        assert!(matches!(
            normalized_token(" render_target "),
            std::borrow::Cow::Borrowed("render_target")
        ));
        assert!(matches!(
            normalized_token("Render-Target"),
            std::borrow::Cow::Owned(ref value) if value == "render_target"
        ));
        assert_eq!(
            parse_usage("Render-Target"),
            Ok(RenderImageUsage::RenderTarget)
        );
    }

    #[test]
    #[ignore = "release performance gate"]
    fn borrowed_texture_metadata_performance_release_setting_token() {
        let mut legacy_samples = Vec::with_capacity(SAMPLE_PAIRS);
        let mut optimized_samples = Vec::with_capacity(SAMPLE_PAIRS);
        for pair_index in 0..SAMPLE_PAIRS {
            let (legacy_ns, optimized_ns) = if pair_index % 2 == 0 {
                (measure_legacy(), measure_borrowed())
            } else {
                let optimized_ns = measure_borrowed();
                (measure_legacy(), optimized_ns)
            };
            legacy_samples.push(legacy_ns);
            optimized_samples.push(optimized_ns);
        }

        let legacy_p95 = nearest_rank_p95(&legacy_samples);
        let optimized_p95 = nearest_rank_p95(&optimized_samples);
        let improvement_percent =
            legacy_p95.saturating_sub(optimized_p95).saturating_mul(100) / legacy_p95.max(1);
        println!(
            "PERF_RESULT plugins07_texture_setting_token_borrow sample_pairs={SAMPLE_PAIRS} legacy_ns={} optimized_ns={} legacy_p95_ns={legacy_p95} optimized_p95_ns={optimized_p95} improvement_percent={improvement_percent} threshold_percent=50 legacy_allocations_per_sample={} optimized_allocations_per_sample=0 order=alternating_legacy_first_even legacy_first_pairs=11 optimized_first_pairs=10",
            csv(&legacy_samples),
            csv(&optimized_samples),
            LOOKUPS_PER_SAMPLE * TOKENS.len() * 2,
        );
        assert!(
            improvement_percent >= 50,
            "borrowed canonical texture setting tokens must improve P95 by at least 50%"
        );
    }

    fn measure_legacy() -> u128 {
        let started = Instant::now();
        let mut parsed = 0_u64;
        for _ in 0..LOOKUPS_PER_SAMPLE {
            for token in TOKENS {
                let normalized = black_box(token)
                    .trim()
                    .to_ascii_lowercase()
                    .replace('-', "_");
                parsed += u64::from(matches!(
                    normalized.as_str(),
                    "render_target" | "copy_src" | "copy_dst"
                ));
                black_box(normalized);
            }
        }
        black_box(parsed);
        started.elapsed().as_nanos()
    }

    fn measure_borrowed() -> u128 {
        let started = Instant::now();
        let mut parsed = 0_u64;
        for _ in 0..LOOKUPS_PER_SAMPLE {
            for token in TOKENS {
                parsed += u64::from(parse_usage(black_box(token)).is_ok());
            }
        }
        black_box(parsed);
        started.elapsed().as_nanos()
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
}
