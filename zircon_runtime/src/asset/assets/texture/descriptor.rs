use serde::{Deserialize, Serialize};

use crate::core::framework::render::{
    RenderImageAssetUsage, RenderImageColorSpace, RenderImageDescriptor, RenderImageDimension,
    RenderImageFallbackKind, RenderImageUsage, RenderSamplerAddressMode, RenderSamplerDescriptor,
    RenderSamplerFilter,
};

use super::TexturePayload;

pub const RGBA8_UNORM_SRGB_FORMAT: &str = "rgba8unorm_srgb";
pub const RGBA8_UNORM_FORMAT: &str = "rgba8unorm";

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum TextureArrayLayout {
    RowCount { rows: u32 },
    RowHeight { pixels: u32 },
}

impl TextureArrayLayout {
    pub fn from_import_settings(settings: &toml::Table) -> Result<Option<Self>, String> {
        settings
            .get("array_layout")
            .map(parse_array_layout)
            .transpose()
    }
}

/// Render-facing texture metadata kept beside CPU/container payload bytes.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct TextureAssetDescriptor {
    pub format: String,
    pub color_space: RenderImageColorSpace,
    #[serde(default)]
    pub dimension: RenderImageDimension,
    /// Bevy-style extent depth, or array-layer count for 1D/2D array textures.
    #[serde(default = "default_depth_or_array_layers")]
    pub depth_or_array_layers: u32,
    pub sampler: RenderSamplerDescriptor,
    pub usage: Vec<RenderImageUsage>,
    #[serde(default)]
    pub asset_usage: Vec<RenderImageAssetUsage>,
    pub mip_count: u32,
    pub array_layer_count: u32,
    pub fallback: RenderImageFallbackKind,
}

impl TextureAssetDescriptor {
    pub fn rgba8_srgb() -> Self {
        Self {
            format: RGBA8_UNORM_SRGB_FORMAT.to_string(),
            color_space: RenderImageColorSpace::Srgb,
            dimension: RenderImageDimension::D2,
            depth_or_array_layers: 1,
            sampler: RenderSamplerDescriptor::default(),
            usage: default_render_image_usage(),
            asset_usage: default_render_image_asset_usage(),
            mip_count: 1,
            array_layer_count: 1,
            fallback: RenderImageFallbackKind::MissingImage,
        }
    }

    pub fn container(format: impl Into<String>, mip_count: u32, array_layer_count: u32) -> Self {
        Self {
            format: format.into(),
            mip_count: mip_count.max(1),
            array_layer_count: array_layer_count.max(1),
            depth_or_array_layers: array_layer_count.max(1),
            ..Self::rgba8_srgb()
        }
    }

    pub fn from_payload(payload: &TexturePayload) -> Self {
        match payload {
            TexturePayload::Rgba8 => Self::rgba8_srgb(),
            TexturePayload::Container {
                format,
                mip_count,
                array_layers,
                ..
            } => Self::container(format.clone(), *mip_count, *array_layers),
        }
    }

    pub fn normalized(mut self) -> Self {
        if self.format.trim().is_empty() {
            self.format = RGBA8_UNORM_SRGB_FORMAT.to_string();
        }
        if self.usage.is_empty() {
            self.usage = default_render_image_usage();
        }
        if self.asset_usage.is_empty() {
            self.asset_usage = default_render_image_asset_usage();
        }
        self.mip_count = self.mip_count.max(1);
        self.array_layer_count = self.array_layer_count.max(1);
        self.depth_or_array_layers = self.depth_or_array_layers.max(1);
        self.normalize_extent_fields();
        self.normalize_rgba8_color_space_format();
        self
    }

    pub fn with_import_settings(mut self, settings: &toml::Table) -> Result<Self, String> {
        let mut extent_keys = ExtentSettingKeys::default();
        if let Some(value) = settings.get("format") {
            self.format = string_setting("format", value)?.to_string();
        } else if let Some(value) = settings.get("texture_format") {
            self.format = string_setting("texture_format", value)?.to_string();
        }
        if let Some(value) = settings.get("color_space") {
            self.color_space = parse_color_space(string_setting("color_space", value)?)?;
        } else if let Some(value) = settings.get("is_srgb") {
            self.color_space = if bool_setting("is_srgb", value)? {
                RenderImageColorSpace::Srgb
            } else {
                RenderImageColorSpace::Linear
            };
        }
        if let Some(value) = settings.get("dimension") {
            self.dimension = parse_dimension(string_setting("dimension", value)?)?;
        }
        if let Some(value) = settings.get("usage") {
            self.usage = parse_usage_list("usage", value)?;
        }
        if let Some(value) = settings.get("asset_usage") {
            self.asset_usage = parse_asset_usage_list("asset_usage", value)?;
        } else if let Some(value) = settings.get("render_asset_usage") {
            self.asset_usage = parse_asset_usage_list("render_asset_usage", value)?;
        }
        if let Some(value) = settings.get("mip_count") {
            self.mip_count = u32_setting("mip_count", value)?;
        }
        if let Some(value) = settings.get("array_layer_count") {
            self.array_layer_count = u32_setting("array_layer_count", value)?;
            extent_keys.array_layer_count = Some("array_layer_count");
        } else if let Some(value) = settings.get("array_layers") {
            self.array_layer_count = u32_setting("array_layers", value)?;
            extent_keys.array_layer_count = Some("array_layers");
        }
        if let Some(value) = settings.get("depth_or_array_layers") {
            self.depth_or_array_layers = u32_setting("depth_or_array_layers", value)?;
            extent_keys.depth_or_array_layers = Some("depth_or_array_layers");
        } else if let Some(value) = settings.get("depth") {
            self.depth_or_array_layers = u32_setting("depth", value)?;
            extent_keys.depth_or_array_layers = Some("depth");
        }
        if let Some(value) = settings.get("sampler") {
            self.sampler = parse_sampler(value, self.sampler)?;
        }
        if self.format.trim().is_empty() {
            self.format = RGBA8_UNORM_SRGB_FORMAT.to_string();
        }
        if self.usage.is_empty() {
            self.usage = default_render_image_usage();
        }
        if self.asset_usage.is_empty() {
            self.asset_usage = default_render_image_asset_usage();
        }
        self.mip_count = self.mip_count.max(1);
        self.array_layer_count = self.array_layer_count.max(1);
        self.depth_or_array_layers = self.depth_or_array_layers.max(1);
        self.normalize_import_extent_fields(extent_keys)?;
        self.normalize_rgba8_color_space_format();
        Ok(self)
    }

    pub fn to_render_image_descriptor(&self, width: u32, height: u32) -> RenderImageDescriptor {
        let descriptor = self.clone().normalized();
        RenderImageDescriptor {
            width,
            height,
            depth_or_array_layers: descriptor.depth_or_array_layers,
            dimension: descriptor.dimension,
            format: descriptor.format,
            color_space: descriptor.color_space,
            sampler: descriptor.sampler,
            usage: descriptor.usage,
            asset_usage: descriptor.asset_usage,
            mip_count: descriptor.mip_count,
            array_layer_count: descriptor.array_layer_count,
            fallback: descriptor.fallback,
        }
    }

    fn normalize_extent_fields(&mut self) {
        if self.dimension == RenderImageDimension::D3 {
            self.array_layer_count = 1;
        } else {
            let layers = self
                .depth_or_array_layers
                .max(self.array_layer_count)
                .max(1);
            self.depth_or_array_layers = layers;
            self.array_layer_count = layers;
        }
    }

    fn normalize_import_extent_fields(&mut self, keys: ExtentSettingKeys) -> Result<(), String> {
        if self.dimension == RenderImageDimension::D3 {
            if let Some(key) = keys.array_layer_count {
                if self.array_layer_count != 1 {
                    return Err(format!(
                        "texture import setting `{key}` must be 1 for 3d textures"
                    ));
                }
            }
            self.array_layer_count = 1;
            return Ok(());
        }

        match (keys.array_layer_count, keys.depth_or_array_layers) {
            (Some(array_key), Some(depth_key)) => {
                if self.array_layer_count != self.depth_or_array_layers {
                    return Err(format!(
                        "texture import settings `{array_key}` and `{depth_key}` must match for 1d/2d array textures"
                    ));
                }
            }
            (Some(_), None) => {
                self.depth_or_array_layers = self.array_layer_count;
            }
            (None, Some(_)) => {
                self.array_layer_count = self.depth_or_array_layers;
            }
            (None, None) => {
                self.normalize_extent_fields();
            }
        }
        if self.array_layer_count != self.depth_or_array_layers {
            return Err(format!(
                "texture extent metadata must match for 1d/2d array textures: array_layer_count = {}, depth_or_array_layers = {}",
                self.array_layer_count, self.depth_or_array_layers
            ));
        }
        Ok(())
    }

    fn normalize_rgba8_color_space_format(&mut self) {
        if self.color_space == RenderImageColorSpace::Linear
            && self
                .format
                .trim()
                .eq_ignore_ascii_case(RGBA8_UNORM_SRGB_FORMAT)
        {
            self.format = RGBA8_UNORM_FORMAT.to_string();
        }
    }
}

impl Default for TextureAssetDescriptor {
    fn default() -> Self {
        Self::rgba8_srgb()
    }
}

fn default_render_image_usage() -> Vec<RenderImageUsage> {
    vec![RenderImageUsage::Sampled, RenderImageUsage::CopyDst]
}

fn default_render_image_asset_usage() -> Vec<RenderImageAssetUsage> {
    vec![
        RenderImageAssetUsage::MainWorld,
        RenderImageAssetUsage::RenderWorld,
    ]
}

fn default_depth_or_array_layers() -> u32 {
    1
}

#[derive(Clone, Copy, Debug, Default)]
struct ExtentSettingKeys {
    array_layer_count: Option<&'static str>,
    depth_or_array_layers: Option<&'static str>,
}

fn string_setting<'a>(name: &str, value: &'a toml::Value) -> Result<&'a str, String> {
    value
        .as_str()
        .ok_or_else(|| format!("texture import setting `{name}` must be a string"))
}

fn u32_setting(name: &str, value: &toml::Value) -> Result<u32, String> {
    let Some(value) = value.as_integer() else {
        return Err(format!(
            "texture import setting `{name}` must be an integer"
        ));
    };
    u32::try_from(value).map_err(|_| format!("texture import setting `{name}` must fit in u32"))
}

fn bool_setting(name: &str, value: &toml::Value) -> Result<bool, String> {
    value
        .as_bool()
        .ok_or_else(|| format!("texture import setting `{name}` must be a boolean"))
}

fn parse_usage_list(name: &str, value: &toml::Value) -> Result<Vec<RenderImageUsage>, String> {
    if let Some(value) = value.as_str() {
        return Ok(vec![parse_usage(value)?]);
    }
    let Some(values) = value.as_array() else {
        return Err(format!(
            "texture import setting `{name}` must be a string or array of strings"
        ));
    };
    values
        .iter()
        .map(|value| parse_usage(string_setting(name, value)?))
        .collect()
}

fn parse_asset_usage_list(
    name: &str,
    value: &toml::Value,
) -> Result<Vec<RenderImageAssetUsage>, String> {
    if let Some(value) = value.as_str() {
        return Ok(vec![parse_asset_usage(name, value)?]);
    }
    let Some(values) = value.as_array() else {
        return Err(format!(
            "texture import setting `{name}` must be a string or array of strings"
        ));
    };
    values
        .iter()
        .map(|value| parse_asset_usage(name, string_setting(name, value)?))
        .collect()
}

fn parse_sampler(
    value: &toml::Value,
    mut sampler: RenderSamplerDescriptor,
) -> Result<RenderSamplerDescriptor, String> {
    if let Some(value) = value.as_str() {
        return parse_sampler_shorthand(value, sampler);
    }
    let Some(table) = value.as_table() else {
        return Err("texture import setting `sampler` must be a table or string".to_string());
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

fn parse_sampler_shorthand(
    value: &str,
    sampler: RenderSamplerDescriptor,
) -> Result<RenderSamplerDescriptor, String> {
    match normalized_token(value).as_str() {
        "default" => Ok(sampler),
        "linear" => Ok(sampler_with_filter(sampler, RenderSamplerFilter::Linear)),
        "nearest" => Ok(sampler_with_filter(sampler, RenderSamplerFilter::Nearest)),
        _ => Err(format!("unsupported texture sampler `{value}`")),
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

fn parse_array_layout(value: &toml::Value) -> Result<TextureArrayLayout, String> {
    let Some(table) = value.as_table() else {
        return Err("texture import setting `array_layout` must be a table".to_string());
    };
    match (table.get("row_count"), table.get("row_height")) {
        (Some(rows), None) => Ok(TextureArrayLayout::RowCount {
            rows: u32_setting("array_layout.row_count", rows)?,
        }),
        (None, Some(pixels)) => Ok(TextureArrayLayout::RowHeight {
            pixels: u32_setting("array_layout.row_height", pixels)?,
        }),
        (Some(_), Some(_)) => Err(
            "texture import setting `array_layout` must set only one of row_count or row_height"
                .to_string(),
        ),
        (None, None) => Err(
            "texture import setting `array_layout` must set row_count or row_height".to_string(),
        ),
    }
}

fn parse_color_space(value: &str) -> Result<RenderImageColorSpace, String> {
    match normalized_token(value).as_str() {
        "srgb" => Ok(RenderImageColorSpace::Srgb),
        "linear" => Ok(RenderImageColorSpace::Linear),
        "hdr" => Ok(RenderImageColorSpace::Hdr),
        "unknown" => Ok(RenderImageColorSpace::Unknown),
        _ => Err(format!("unsupported texture color_space `{value}`")),
    }
}

fn parse_dimension(value: &str) -> Result<RenderImageDimension, String> {
    match normalized_token(value).as_str() {
        "1d" | "d1" => Ok(RenderImageDimension::D1),
        "2d" | "d2" => Ok(RenderImageDimension::D2),
        "3d" | "d3" => Ok(RenderImageDimension::D3),
        _ => Err(format!("unsupported texture dimension `{value}`")),
    }
}

fn parse_usage(value: &str) -> Result<RenderImageUsage, String> {
    match normalized_token(value).as_str() {
        "sampled" => Ok(RenderImageUsage::Sampled),
        "storage" => Ok(RenderImageUsage::Storage),
        "render_target" => Ok(RenderImageUsage::RenderTarget),
        "copy_src" => Ok(RenderImageUsage::CopySrc),
        "copy_dst" => Ok(RenderImageUsage::CopyDst),
        _ => Err(format!("unsupported texture usage `{value}`")),
    }
}

fn parse_asset_usage(name: &str, value: &str) -> Result<RenderImageAssetUsage, String> {
    match normalized_token(value).as_str() {
        "main_world" | "main" | "cpu" => Ok(RenderImageAssetUsage::MainWorld),
        "render_world" | "render" | "gpu" => Ok(RenderImageAssetUsage::RenderWorld),
        _ => Err(format!("unsupported texture {name} `{value}`")),
    }
}

fn parse_address_mode(value: &str) -> Result<RenderSamplerAddressMode, String> {
    match normalized_token(value).as_str() {
        "clamp_to_edge" => Ok(RenderSamplerAddressMode::ClampToEdge),
        "repeat" => Ok(RenderSamplerAddressMode::Repeat),
        "mirror_repeat" => Ok(RenderSamplerAddressMode::MirrorRepeat),
        _ => Err(format!(
            "unsupported texture sampler address mode `{value}`"
        )),
    }
}

fn parse_filter(value: &str) -> Result<RenderSamplerFilter, String> {
    match normalized_token(value).as_str() {
        "nearest" => Ok(RenderSamplerFilter::Nearest),
        "linear" => Ok(RenderSamplerFilter::Linear),
        _ => Err(format!("unsupported texture sampler filter `{value}`")),
    }
}

fn normalized_token(value: &str) -> String {
    value.trim().to_ascii_lowercase().replace('-', "_")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn render_asset_usage_alias_accepts_single_token() {
        let settings = r#"render_asset_usage = "gpu""#.parse::<toml::Table>().expect("valid toml");

        let descriptor = TextureAssetDescriptor::default()
            .with_import_settings(&settings)
            .expect("valid render asset usage alias");

        assert_eq!(
            descriptor.asset_usage,
            vec![RenderImageAssetUsage::RenderWorld]
        );
    }

    #[test]
    fn depth_or_array_layers_updates_array_layer_count_for_2d_arrays() {
        let settings = r#"depth_or_array_layers = 4"#.parse::<toml::Table>().expect("valid toml");

        let descriptor = TextureAssetDescriptor::default()
            .with_import_settings(&settings)
            .expect("valid depth override");

        assert_eq!(descriptor.depth_or_array_layers, 4);
        assert_eq!(descriptor.array_layer_count, 4);
    }

    #[test]
    fn array_layer_count_updates_depth_or_array_layers_for_2d_arrays() {
        let settings = r#"array_layer_count = 3"#.parse::<toml::Table>().expect("valid toml");

        let descriptor = TextureAssetDescriptor::default()
            .with_import_settings(&settings)
            .expect("valid array layer override");

        assert_eq!(descriptor.depth_or_array_layers, 3);
        assert_eq!(descriptor.array_layer_count, 3);
    }

    #[test]
    fn mismatched_2d_extent_settings_report_error() {
        let settings = r#"
array_layer_count = 2
depth_or_array_layers = 4
"#
        .parse::<toml::Table>()
        .expect("valid toml");

        let error = TextureAssetDescriptor::default()
            .with_import_settings(&settings)
            .expect_err("mismatched extent settings");

        assert!(
            error.contains(
                "texture import settings `array_layer_count` and `depth_or_array_layers` must match for 1d/2d array textures"
            ),
            "unexpected error: {error}"
        );
    }

    #[test]
    fn dimension_3d_rejects_multiple_array_layers() {
        let settings = r#"
dimension = "3d"
array_layers = 2
"#
        .parse::<toml::Table>()
        .expect("valid toml");

        let error = TextureAssetDescriptor::default()
            .with_import_settings(&settings)
            .expect_err("3d array layer override");

        assert!(
            error.contains("texture import setting `array_layers` must be 1 for 3d textures"),
            "unexpected error: {error}"
        );
    }

    #[test]
    fn dimension_3d_keeps_depth_and_single_array_layer() {
        let settings = r#"
dimension = "3d"
depth = 4
"#
        .parse::<toml::Table>()
        .expect("valid toml");

        let descriptor = TextureAssetDescriptor::default()
            .with_import_settings(&settings)
            .expect("valid 3d depth override");

        assert_eq!(descriptor.dimension, RenderImageDimension::D3);
        assert_eq!(descriptor.depth_or_array_layers, 4);
        assert_eq!(descriptor.array_layer_count, 1);
    }

    #[test]
    fn import_extent_override_replaces_existing_2d_container_layers() {
        let settings = r#"depth_or_array_layers = 4"#.parse::<toml::Table>().expect("valid toml");

        let descriptor = TextureAssetDescriptor::container("dds/DXT1", 1, 12)
            .with_import_settings(&settings)
            .expect("valid depth override");

        assert_eq!(descriptor.depth_or_array_layers, 4);
        assert_eq!(descriptor.array_layer_count, 4);
    }

    #[test]
    fn bevy_alias_diagnostics_report_actual_setting_keys() {
        let cases = [
            (
                r#"texture_format = 1"#,
                "texture import setting `texture_format` must be a string",
            ),
            (
                r#"is_srgb = "false""#,
                "texture import setting `is_srgb` must be a boolean",
            ),
            (
                r#"sampler = 1"#,
                "texture import setting `sampler` must be a table or string",
            ),
            (
                r#"render_asset_usage = 1"#,
                "texture import setting `render_asset_usage` must be a string or array of strings",
            ),
            (
                r#"render_asset_usage = "video_memory""#,
                "unsupported texture render_asset_usage `video_memory`",
            ),
        ];

        for (settings, expected) in cases {
            let settings = settings.parse::<toml::Table>().expect("valid toml");
            let error = TextureAssetDescriptor::default()
                .with_import_settings(&settings)
                .expect_err("invalid alias setting");

            assert!(
                error.contains(expected),
                "expected `{expected}` in `{error}`"
            );
        }
    }

    #[test]
    fn linear_color_space_normalizes_default_rgba8_format_to_linear() {
        let settings = r#"color_space = "linear""#.parse::<toml::Table>().expect("valid toml");

        let descriptor = TextureAssetDescriptor::default()
            .with_import_settings(&settings)
            .expect("valid linear color space");

        assert_eq!(descriptor.format, RGBA8_UNORM_FORMAT);
        assert_eq!(
            descriptor.to_render_image_descriptor(2, 2).format,
            RGBA8_UNORM_FORMAT
        );
    }
}
