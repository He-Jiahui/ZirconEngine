use serde::{Deserialize, Serialize};
use thiserror::Error;

use crate::core::framework::render::{
    default_color_space_for_texture_usage, default_compression_for_texture_usage,
    default_mip_filter_for_texture_usage, validate_texture_metadata, RenderImageAssetUsage,
    RenderImageColorSpace, RenderImageDescriptor, RenderImageDimension, RenderImageFallbackKind,
    RenderImageUsage, RenderSamplerDescriptor, TextureCompressionTarget, TextureMetadata,
    TextureMetadataDiagnostic, TextureMetadataDiagnosticSeverity, TextureMipFilter,
    TextureMipPolicy, TextureNormalConvention, TextureUsageHint,
};

use super::TexturePayload;

mod settings;

use self::settings::{
    bool_setting, f32_setting, parse_array_layout, parse_asset_usage_list, parse_color_space,
    parse_compression, parse_dimension, parse_mip_filter, parse_mip_policy,
    parse_normal_convention, parse_sampler, parse_usage_hint, parse_usage_list, string_setting,
    u32_setting, u8_setting,
};

pub const RGBA8_UNORM_SRGB_FORMAT: &str = "rgba8unorm_srgb";
pub const RGBA8_UNORM_FORMAT: &str = "rgba8unorm";

pub type TextureDescriptorResult<T> = std::result::Result<T, TextureDescriptorError>;

#[derive(Clone, Debug, Error, PartialEq, Eq)]
pub enum TextureDescriptorError {
    #[error("texture import setting `{name}` must be {expected}")]
    SettingType {
        name: String,
        expected: &'static str,
    },
    #[error("texture import setting `{name}` must fit in u32")]
    SettingU32Overflow { name: String },
    #[error("texture import setting `{name}` must fit in u8")]
    SettingU8Overflow { name: String },
    #[error("texture import setting `{name}` must be a finite f32")]
    SettingF32Range { name: String },
    #[error("unsupported texture {kind} `{value}`")]
    UnsupportedToken { kind: String, value: String },
    #[error("texture import setting `array_layout` must set only one of row_count or row_height")]
    ArrayLayoutExclusiveMode,
    #[error("texture import setting `array_layout` must set row_count or row_height")]
    ArrayLayoutMissingMode,
    #[error("texture import setting `{key}` is not valid for {dimension} textures")]
    ExtentSettingForDimension {
        key: &'static str,
        dimension: &'static str,
    },
    #[error("texture import setting `{key}` was removed; use `{replacement}`")]
    RetiredExtentSetting {
        key: &'static str,
        replacement: &'static str,
    },
    #[error("texture import setting `{key}` must be greater than zero")]
    TextureExtentZero { key: &'static str },
    #[error("cube texture layer count must be a non-zero multiple of six faces, found {layers}")]
    CubeLayerCount { layers: u32 },
    #[error("texture import setting `array_layout` requires a decoded rgba8 image")]
    ArrayLayoutRequiresRgba8,
    #[error("texture import setting `array_layout` requires a 2d image")]
    ArrayLayoutRequires2d,
    #[error("texture import setting `array_layout` requires a single-layer image")]
    ArrayLayoutRequiresSingleLayer,
    #[error("texture import setting `{name}` must be greater than zero")]
    ArrayLayoutZero { name: String },
    #[error(
        "texture import setting `array_layout` can not evenly divide height = {height} by row_height = {row_height}"
    )]
    ArrayLayoutRowHeightDivisibility { height: u32, row_height: u32 },
    #[error(
        "texture import setting `array_layout` can not evenly divide height = {height} by layers = {layers}"
    )]
    ArrayLayoutLayerDivisibility { height: u32, layers: u32 },
    #[error(
        "texture import setting `array_layout` expected rgba byte length {expected_len} but found {actual_len}"
    )]
    ArrayLayoutRgbaLength {
        expected_len: usize,
        actual_len: usize,
    },
    #[error("texture rgba8 extent {width}x{height} is too large to validate")]
    Rgba8ExtentTooLarge { width: u32, height: u32 },
}

impl TextureDescriptorError {
    pub(super) fn setting_type(name: &str, expected: &'static str) -> Self {
        Self::SettingType {
            name: name.to_string(),
            expected,
        }
    }

    pub(super) fn setting_u32_overflow(name: &str) -> Self {
        Self::SettingU32Overflow {
            name: name.to_string(),
        }
    }

    pub(super) fn setting_u8_overflow(name: &str) -> Self {
        Self::SettingU8Overflow {
            name: name.to_string(),
        }
    }

    pub(super) fn setting_f32_range(name: &str) -> Self {
        Self::SettingF32Range {
            name: name.to_string(),
        }
    }

    pub(super) fn unsupported(kind: impl Into<String>, value: &str) -> Self {
        Self::UnsupportedToken {
            kind: kind.into(),
            value: value.to_string(),
        }
    }

    pub(super) fn array_layout_zero(name: &str) -> Self {
        Self::ArrayLayoutZero {
            name: name.to_string(),
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum TextureArrayLayout {
    RowCount { rows: u32 },
    RowHeight { pixels: u32 },
}

impl TextureArrayLayout {
    pub fn from_import_settings(settings: &toml::Table) -> TextureDescriptorResult<Option<Self>> {
        settings
            .get("array_layout")
            .map(parse_array_layout)
            .transpose()
    }
}

/// Render-facing texture metadata kept beside CPU/container payload bytes.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct TextureAssetDescriptor {
    pub format: String,
    pub color_space: RenderImageColorSpace,
    #[serde(default)]
    pub metadata: TextureMetadata,
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
    pub fallback: RenderImageFallbackKind,
}

impl TextureAssetDescriptor {
    pub fn rgba8_srgb() -> Self {
        Self {
            format: RGBA8_UNORM_SRGB_FORMAT.to_string(),
            color_space: RenderImageColorSpace::Srgb,
            metadata: TextureMetadata::default(),
            dimension: RenderImageDimension::D2,
            depth_or_array_layers: 1,
            sampler: RenderSamplerDescriptor::default(),
            usage: default_render_image_usage(),
            asset_usage: default_render_image_asset_usage(),
            mip_count: 1,
            fallback: RenderImageFallbackKind::MissingImage,
        }
    }

    /// Defaults for a decoded one-mip RGBA8 source without claiming absent cooked artifacts.
    pub fn decoded_rgba8_for_import_usage(usage_hint: TextureUsageHint) -> Self {
        let mut descriptor = Self::rgba8_srgb();
        descriptor.metadata.usage_hint = usage_hint;
        descriptor.color_space = default_color_space_for_texture_usage(usage_hint);
        descriptor.metadata.color_space = descriptor.color_space;
        descriptor.metadata.mip_policy = TextureMipPolicy::FromSource;
        descriptor.metadata.mip_filter = default_mip_filter_for_texture_usage(usage_hint);
        descriptor.metadata.compression = TextureCompressionTarget::Uncompressed;
        descriptor.metadata.normal_convention = if usage_hint == TextureUsageHint::Normal {
            TextureNormalConvention::TangentSpaceDx
        } else {
            TextureNormalConvention::None
        };
        descriptor.normalize_rgba8_color_space_format();
        descriptor
    }

    pub fn container(format: impl Into<String>, mip_count: u32, array_layer_count: u32) -> Self {
        Self {
            format: format.into(),
            mip_count: mip_count.max(1),
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
        self.depth_or_array_layers = self.depth_or_array_layers.max(1);
        self.normalize_extent_fields();
        self.normalize_rgba8_color_space_format();
        self.metadata.color_space = self.color_space;
        self
    }

    pub fn apply_import_settings(
        mut self,
        settings: &toml::Table,
    ) -> TextureDescriptorResult<Self> {
        reject_retired_extent_settings(settings)?;
        let has_explicit_color_space =
            settings.contains_key("color_space") || settings.contains_key("is_srgb");
        let has_explicit_mip_policy = settings.contains_key("mip_policy");
        let has_explicit_mip_filter = settings.contains_key("mip_filter");
        let has_explicit_compression = settings.contains_key("compression");
        if let Some(value) = settings.get("format") {
            self.format = string_setting("format", value)?.to_string();
        } else if let Some(value) = settings.get("texture_format") {
            self.format = string_setting("texture_format", value)?.to_string();
        }
        if let Some(value) = settings.get("color_space") {
            self.color_space = parse_color_space(string_setting("color_space", value)?)?;
            self.metadata.color_space = self.color_space;
        } else if let Some(value) = settings.get("is_srgb") {
            self.color_space = if bool_setting("is_srgb", value)? {
                RenderImageColorSpace::Srgb
            } else {
                RenderImageColorSpace::Linear
            };
            self.metadata.color_space = self.color_space;
        }
        if let Some(value) = settings.get("usage_hint") {
            self.metadata.usage_hint = parse_usage_hint(string_setting("usage_hint", value)?)?;
        }
        if !has_explicit_color_space {
            self.color_space = default_color_space_for_texture_usage(self.metadata.usage_hint);
            self.metadata.color_space = self.color_space;
        }
        if let Some(value) = settings.get("mip_policy") {
            self.metadata.mip_policy = parse_mip_policy(string_setting("mip_policy", value)?)?;
        } else if !has_explicit_mip_policy
            && self.mip_count == 1
            && is_decoded_rgba8_format(&self.format)
        {
            // Decoded source images contain only their base level; the import pipeline owns the
            // complete offline chain unless an author explicitly requests another policy.
            self.metadata.mip_policy = TextureMipPolicy::GenerateOffline;
        }
        if !has_explicit_mip_filter {
            self.metadata.mip_filter =
                if self.metadata.mip_policy == TextureMipPolicy::GenerateRuntime {
                    // The four-level runtime workgroup reduction is the matching box reference.
                    TextureMipFilter::Box
                } else {
                    default_mip_filter_for_texture_usage(self.metadata.usage_hint)
                };
        }
        if !has_explicit_compression {
            self.metadata.compression =
                if self.metadata.mip_policy == TextureMipPolicy::GenerateRuntime {
                    TextureCompressionTarget::Uncompressed
                } else {
                    default_compression_for_texture_usage(self.metadata.usage_hint)
                };
        }
        if let Some(value) = settings.get("mip_filter") {
            self.metadata.mip_filter = parse_mip_filter(string_setting("mip_filter", value)?)?;
        }
        if let Some(value) = settings.get("normal_convention") {
            self.metadata.normal_convention =
                parse_normal_convention(string_setting("normal_convention", value)?)?;
        } else if self.metadata.usage_hint == TextureUsageHint::Normal {
            self.metadata.normal_convention = TextureNormalConvention::TangentSpaceDx;
        }
        if let Some(value) = settings.get("compression") {
            self.metadata.compression = parse_compression(string_setting("compression", value)?)?;
        }
        if let Some(value) = settings.get("mip_bias") {
            self.metadata.mip_bias = f32_setting("mip_bias", value)?;
        }
        if let Some(value) = settings.get("max_anisotropy") {
            self.metadata.max_anisotropy = u8_setting("max_anisotropy", value)?;
        }
        if let Some(value) = settings.get("streaming_enabled") {
            self.metadata.streaming_enabled = bool_setting("streaming_enabled", value)?;
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
        let array_layers = settings
            .get("array_layers")
            .map(|value| non_zero_extent_setting("array_layers", value))
            .transpose()?;
        let depth = settings
            .get("depth")
            .map(|value| non_zero_extent_setting("depth", value))
            .transpose()?;
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
        self.depth_or_array_layers = self.depth_or_array_layers.max(1);
        self.apply_import_extent_settings(depth, array_layers)?;
        self.normalize_rgba8_color_space_format();
        self.metadata.color_space = self.color_space;
        Ok(self)
    }

    pub fn to_render_image_descriptor(&self, width: u32, height: u32) -> RenderImageDescriptor {
        self.clone().into_render_image_descriptor(width, height)
    }

    pub fn validate_metadata(&self, uri: &str) -> Vec<TextureMetadataDiagnostic> {
        let mut diagnostics =
            validate_texture_metadata(uri, &self.format, &self.metadata, &self.sampler);
        if self.metadata.mip_policy == TextureMipPolicy::GenerateOffline && self.mip_count > 1 {
            diagnostics.push(TextureMetadataDiagnostic {
                severity: TextureMetadataDiagnosticSeverity::Warning,
                message: format!(
                    "'{uri}' already contains {} mips; falling back to from_source",
                    self.mip_count
                ),
            });
        }
        if self.metadata.mip_policy == TextureMipPolicy::GenerateRuntime
            && self.mip_count > 1
            && !matches!(
                self.dimension,
                RenderImageDimension::D2 | RenderImageDimension::Cube
            )
        {
            diagnostics.push(TextureMetadataDiagnostic {
                severity: TextureMetadataDiagnosticSeverity::Error,
                message: format!(
                    "runtime mip generation supports only 2d or cube textures: '{uri}'"
                ),
            });
        }
        diagnostics
    }

    pub fn into_render_image_descriptor(self, width: u32, height: u32) -> RenderImageDescriptor {
        let descriptor = self.normalized();
        RenderImageDescriptor {
            width,
            height,
            depth_or_array_layers: descriptor.depth_or_array_layers,
            dimension: descriptor.dimension,
            format: descriptor.format,
            color_space: descriptor.metadata.color_space,
            metadata: descriptor.metadata,
            sampler: descriptor.sampler,
            usage: descriptor.usage,
            asset_usage: descriptor.asset_usage,
            mip_count: descriptor.mip_count,
            fallback: descriptor.fallback,
        }
    }

    /// Returns independently addressable array layers without storing a second shape authority.
    pub const fn array_layer_count(&self) -> u32 {
        match self.dimension {
            RenderImageDimension::D2 | RenderImageDimension::Cube => self.depth_or_array_layers,
            RenderImageDimension::D1 | RenderImageDimension::D3 => 1,
        }
    }

    /// Returns the volume depth. Array and cube layers are never interpreted as Z slices.
    pub const fn depth(&self) -> u32 {
        match self.dimension {
            RenderImageDimension::D3 => self.depth_or_array_layers,
            RenderImageDimension::D1 | RenderImageDimension::D2 | RenderImageDimension::Cube => 1,
        }
    }

    fn normalize_extent_fields(&mut self) {
        match self.dimension {
            RenderImageDimension::D1 => {
                self.depth_or_array_layers = 1;
            }
            RenderImageDimension::D3 => {
                self.depth_or_array_layers = self.depth_or_array_layers.max(1);
            }
            RenderImageDimension::Cube => {
                self.depth_or_array_layers = self.depth_or_array_layers.max(6);
            }
            RenderImageDimension::D2 => {
                self.depth_or_array_layers = self.depth_or_array_layers.max(1);
            }
        }
    }

    fn apply_import_extent_settings(
        &mut self,
        depth: Option<u32>,
        array_layers: Option<u32>,
    ) -> TextureDescriptorResult<()> {
        match self.dimension {
            RenderImageDimension::D1 => {
                if depth.is_some() {
                    return Err(TextureDescriptorError::ExtentSettingForDimension {
                        key: "depth",
                        dimension: "1d",
                    });
                }
                if array_layers.is_some() {
                    return Err(TextureDescriptorError::ExtentSettingForDimension {
                        key: "array_layers",
                        dimension: "1d",
                    });
                }
                self.depth_or_array_layers = 1;
            }
            RenderImageDimension::D2 => {
                if depth.is_some() {
                    return Err(TextureDescriptorError::ExtentSettingForDimension {
                        key: "depth",
                        dimension: "2d",
                    });
                }
                if let Some(array_layers) = array_layers {
                    self.depth_or_array_layers = array_layers;
                }
            }
            RenderImageDimension::D3 => {
                if array_layers.is_some() {
                    return Err(TextureDescriptorError::ExtentSettingForDimension {
                        key: "array_layers",
                        dimension: "3d",
                    });
                }
                if let Some(depth) = depth {
                    self.depth_or_array_layers = depth;
                }
            }
            RenderImageDimension::Cube => {
                if depth.is_some() {
                    return Err(TextureDescriptorError::ExtentSettingForDimension {
                        key: "depth",
                        dimension: "cube",
                    });
                }
                let layers = array_layers.unwrap_or(self.depth_or_array_layers.max(6));
                if !valid_cube_layer_count(layers) {
                    return Err(TextureDescriptorError::CubeLayerCount { layers });
                }
                self.depth_or_array_layers = layers;
            }
        }
        Ok(())
    }

    fn normalize_rgba8_color_space_format(&mut self) {
        let format = self.format.trim();
        match self.color_space {
            RenderImageColorSpace::Linear
                if format.eq_ignore_ascii_case(RGBA8_UNORM_SRGB_FORMAT) =>
            {
                self.format = RGBA8_UNORM_FORMAT.to_string();
            }
            RenderImageColorSpace::Srgb if format.eq_ignore_ascii_case(RGBA8_UNORM_FORMAT) => {
                self.format = RGBA8_UNORM_SRGB_FORMAT.to_string();
            }
            _ => {}
        }
    }
}

fn valid_cube_layer_count(layers: u32) -> bool {
    layers != 0 && layers % 6 == 0
}

fn reject_retired_extent_settings(settings: &toml::Table) -> TextureDescriptorResult<()> {
    for (key, replacement) in [
        ("array_layer_count", "array_layers"),
        ("depth_or_array_layers", "depth or array_layers"),
    ] {
        if settings.contains_key(key) {
            return Err(TextureDescriptorError::RetiredExtentSetting { key, replacement });
        }
    }
    Ok(())
}

fn non_zero_extent_setting(key: &'static str, value: &toml::Value) -> TextureDescriptorResult<u32> {
    let value = u32_setting(key, value)?;
    if value == 0 {
        return Err(TextureDescriptorError::TextureExtentZero { key });
    }
    Ok(value)
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

fn is_decoded_rgba8_format(format: &str) -> bool {
    let format = format.trim();
    format.eq_ignore_ascii_case(RGBA8_UNORM_FORMAT)
        || format.eq_ignore_ascii_case(RGBA8_UNORM_SRGB_FORMAT)
}

fn default_depth_or_array_layers() -> u32 {
    1
}

#[cfg(test)]
#[path = "tests/descriptor_plugins07_decoded_format_hotpath_tests.rs"]
mod plugins07_decoded_format_hotpath_tests;

#[cfg(test)]
#[path = "descriptor/tests/cases.rs"]
mod tests;
