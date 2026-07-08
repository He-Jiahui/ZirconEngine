use crate::core::framework::render::{
    RenderImageAssetUsage, RenderImageColorSpace, RenderImageDimension, RenderImageUsage,
    RenderSamplerAddressMode, RenderSamplerDescriptor, RenderSamplerFilter,
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
    match normalized_token(value).as_str() {
        "srgb" => Ok(RenderImageColorSpace::Srgb),
        "linear" => Ok(RenderImageColorSpace::Linear),
        "hdr" => Ok(RenderImageColorSpace::Hdr),
        "unknown" => Ok(RenderImageColorSpace::Unknown),
        _ => Err(TextureDescriptorError::unsupported("color_space", value)),
    }
}

pub(super) fn parse_dimension(value: &str) -> TextureDescriptorResult<RenderImageDimension> {
    match normalized_token(value).as_str() {
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
    match normalized_token(value).as_str() {
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
    match normalized_token(value).as_str() {
        "sampled" => Ok(RenderImageUsage::Sampled),
        "storage" => Ok(RenderImageUsage::Storage),
        "render_target" => Ok(RenderImageUsage::RenderTarget),
        "copy_src" => Ok(RenderImageUsage::CopySrc),
        "copy_dst" => Ok(RenderImageUsage::CopyDst),
        _ => Err(TextureDescriptorError::unsupported("usage", value)),
    }
}

fn parse_asset_usage(name: &str, value: &str) -> TextureDescriptorResult<RenderImageAssetUsage> {
    match normalized_token(value).as_str() {
        "main_world" | "main" | "cpu" => Ok(RenderImageAssetUsage::MainWorld),
        "render_world" | "render" | "gpu" => Ok(RenderImageAssetUsage::RenderWorld),
        _ => Err(TextureDescriptorError::unsupported(name, value)),
    }
}

fn parse_address_mode(value: &str) -> TextureDescriptorResult<RenderSamplerAddressMode> {
    match normalized_token(value).as_str() {
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
    match normalized_token(value).as_str() {
        "nearest" => Ok(RenderSamplerFilter::Nearest),
        "linear" => Ok(RenderSamplerFilter::Linear),
        _ => Err(TextureDescriptorError::unsupported("sampler filter", value)),
    }
}

fn normalized_token(value: &str) -> String {
    value.trim().to_ascii_lowercase().replace('-', "_")
}
