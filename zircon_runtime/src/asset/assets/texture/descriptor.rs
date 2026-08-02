use serde::{Deserialize, Serialize};
use thiserror::Error;

use crate::core::framework::render::{
    RenderImageAssetUsage, RenderImageColorSpace, RenderImageDescriptor, RenderImageDimension,
    RenderImageFallbackKind, RenderImageUsage, RenderSamplerDescriptor, TextureCompressionTarget,
    TextureMetadata, TextureMetadataDiagnostic, TextureMetadataDiagnosticSeverity,
    TextureMipPolicy, TextureNormalConvention, TextureUsageHint,
    default_color_space_for_texture_usage, validate_texture_metadata,
};

use super::TexturePayload;

mod settings;

use self::settings::{
    ExtentSettingKeys, bool_setting, parse_array_layout, parse_asset_usage_list, parse_color_space,
    parse_compression, parse_dimension, parse_mip_policy, parse_normal_convention, parse_sampler,
    parse_usage_hint, parse_usage_list, string_setting, u32_setting,
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
    #[error("unsupported texture {kind} `{value}`")]
    UnsupportedToken { kind: String, value: String },
    #[error("texture import setting `array_layout` must set only one of row_count or row_height")]
    ArrayLayoutExclusiveMode,
    #[error("texture import setting `array_layout` must set row_count or row_height")]
    ArrayLayoutMissingMode,
    #[error("texture import setting `{key}` must be 1 for 3d textures")]
    ArrayLayerCountFor3d { key: &'static str },
    #[error("cube texture layer count must be a non-zero multiple of six faces, found {layers}")]
    CubeLayerCount { layers: u32 },
    #[error(
        "texture import settings `{array_key}` and `{depth_key}` must match for 1d/2d array textures"
    )]
    MismatchedExtentSettings {
        array_key: &'static str,
        depth_key: &'static str,
    },
    #[error(
        "texture extent metadata must match for 1d/2d array textures: array_layer_count = {array_layer_count}, depth_or_array_layers = {depth_or_array_layers}"
    )]
    MismatchedExtentMetadata {
        array_layer_count: u32,
        depth_or_array_layers: u32,
    },
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
    pub array_layer_count: u32,
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
        self.metadata.color_space = self.color_space;
        self
    }

    pub fn apply_import_settings(
        mut self,
        settings: &toml::Table,
    ) -> TextureDescriptorResult<Self> {
        let mut extent_keys = ExtentSettingKeys::default();
        let has_explicit_color_space =
            settings.contains_key("color_space") || settings.contains_key("is_srgb");
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
        }
        if let Some(value) = settings.get("normal_convention") {
            self.metadata.normal_convention =
                parse_normal_convention(string_setting("normal_convention", value)?)?;
        }
        if let Some(value) = settings.get("compression") {
            self.metadata.compression = parse_compression(string_setting("compression", value)?)?;
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
        self.metadata.color_space = self.color_space;
        Ok(self)
    }

    pub fn to_render_image_descriptor(&self, width: u32, height: u32) -> RenderImageDescriptor {
        self.clone().into_render_image_descriptor(width, height)
    }

    pub fn validate_metadata(&self, uri: &str) -> Vec<TextureMetadataDiagnostic> {
        validate_texture_metadata(uri, &self.format, &self.metadata, &self.sampler)
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
            array_layer_count: descriptor.array_layer_count,
            fallback: descriptor.fallback,
        }
    }

    fn normalize_extent_fields(&mut self) {
        match self.dimension {
            RenderImageDimension::D3 => {
                self.array_layer_count = 1;
            }
            RenderImageDimension::Cube => {
                let layers = self
                    .depth_or_array_layers
                    .max(self.array_layer_count)
                    .max(6);
                self.depth_or_array_layers = layers;
                self.array_layer_count = layers;
            }
            RenderImageDimension::D1 | RenderImageDimension::D2 => {
                let layers = self
                    .depth_or_array_layers
                    .max(self.array_layer_count)
                    .max(1);
                self.depth_or_array_layers = layers;
                self.array_layer_count = layers;
            }
        }
    }

    fn normalize_import_extent_fields(
        &mut self,
        keys: ExtentSettingKeys,
    ) -> TextureDescriptorResult<()> {
        if self.dimension == RenderImageDimension::D3 {
            if let Some(key) = keys.array_layer_count {
                if self.array_layer_count != 1 {
                    return Err(TextureDescriptorError::ArrayLayerCountFor3d { key });
                }
            }
            self.array_layer_count = 1;
            return Ok(());
        }

        if self.dimension == RenderImageDimension::Cube {
            match (keys.array_layer_count, keys.depth_or_array_layers) {
                (Some(array_key), Some(depth_key)) => {
                    if self.array_layer_count != self.depth_or_array_layers {
                        return Err(TextureDescriptorError::MismatchedExtentSettings {
                            array_key,
                            depth_key,
                        });
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
            if !valid_cube_layer_count(self.array_layer_count)
                || self.array_layer_count != self.depth_or_array_layers
            {
                return Err(TextureDescriptorError::CubeLayerCount {
                    layers: self.array_layer_count.max(self.depth_or_array_layers),
                });
            }
            return Ok(());
        }

        match (keys.array_layer_count, keys.depth_or_array_layers) {
            (Some(array_key), Some(depth_key)) => {
                if self.array_layer_count != self.depth_or_array_layers {
                    return Err(TextureDescriptorError::MismatchedExtentSettings {
                        array_key,
                        depth_key,
                    });
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
            return Err(TextureDescriptorError::MismatchedExtentMetadata {
                array_layer_count: self.array_layer_count,
                depth_or_array_layers: self.depth_or_array_layers,
            });
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

fn valid_cube_layer_count(layers: u32) -> bool {
    layers != 0 && layers % 6 == 0
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn render_asset_usage_alias_accepts_single_token() {
        let settings = r#"render_asset_usage = "gpu""#.parse::<toml::Table>().expect("valid toml");

        let descriptor = TextureAssetDescriptor::default()
            .apply_import_settings(&settings)
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
            .apply_import_settings(&settings)
            .expect("valid depth override");

        assert_eq!(descriptor.depth_or_array_layers, 4);
        assert_eq!(descriptor.array_layer_count, 4);
    }

    #[test]
    fn array_layer_count_updates_depth_or_array_layers_for_2d_arrays() {
        let settings = r#"array_layer_count = 3"#.parse::<toml::Table>().expect("valid toml");

        let descriptor = TextureAssetDescriptor::default()
            .apply_import_settings(&settings)
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
            .apply_import_settings(&settings)
            .expect_err("mismatched extent settings");

        assert!(matches!(
            error,
            TextureDescriptorError::MismatchedExtentSettings {
                array_key: "array_layer_count",
                depth_key: "depth_or_array_layers",
            }
        ));
        assert!(
            error.to_string().contains(
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
            .apply_import_settings(&settings)
            .expect_err("3d array layer override");

        assert!(
            error
                .to_string()
                .contains("texture import setting `array_layers` must be 1 for 3d textures"),
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
            .apply_import_settings(&settings)
            .expect("valid 3d depth override");

        assert_eq!(descriptor.dimension, RenderImageDimension::D3);
        assert_eq!(descriptor.depth_or_array_layers, 4);
        assert_eq!(descriptor.array_layer_count, 1);
    }

    #[test]
    fn dimension_cube_defaults_to_six_faces() {
        let settings = r#"dimension = "cube""#.parse::<toml::Table>().expect("valid toml");

        let descriptor = TextureAssetDescriptor::default()
            .apply_import_settings(&settings)
            .expect("valid cube dimension");

        assert_eq!(descriptor.dimension, RenderImageDimension::Cube);
        assert_eq!(descriptor.depth_or_array_layers, 6);
        assert_eq!(descriptor.array_layer_count, 6);
    }

    #[test]
    fn dimension_cubemap_alias_requires_face_multiple_layers() {
        let settings = r#"
dimension = "cubemap"
array_layers = 5
"#
        .parse::<toml::Table>()
        .expect("valid toml");

        let error = TextureAssetDescriptor::default()
            .apply_import_settings(&settings)
            .expect_err("invalid cube face count");

        assert!(
            error.to_string().contains(
                "cube texture layer count must be a non-zero multiple of six faces, found 5"
            ),
            "unexpected error: {error}"
        );
    }

    #[test]
    fn import_extent_override_replaces_existing_2d_container_layers() {
        let settings = r#"depth_or_array_layers = 4"#.parse::<toml::Table>().expect("valid toml");

        let descriptor = TextureAssetDescriptor::container("dds/DXT1", 1, 12)
            .apply_import_settings(&settings)
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
                .apply_import_settings(&settings)
                .expect_err("invalid alias setting");

            assert!(
                error.to_string().contains(expected),
                "expected `{expected}` in `{error}`"
            );
        }
    }

    #[test]
    fn invalid_import_settings_report_typed_error_variants() {
        let settings = r#"sampler = 1"#.parse::<toml::Table>().expect("valid toml");
        let error = TextureAssetDescriptor::default()
            .apply_import_settings(&settings)
            .expect_err("invalid sampler setting");

        assert!(matches!(
            error,
            TextureDescriptorError::SettingType {
                ref name,
                expected: "a table or string",
            } if name == "sampler"
        ));

        let settings = r#"render_asset_usage = "video_memory""#
            .parse::<toml::Table>()
            .expect("valid toml");
        let error = TextureAssetDescriptor::default()
            .apply_import_settings(&settings)
            .expect_err("unsupported render asset usage");

        assert!(matches!(
            error,
            TextureDescriptorError::UnsupportedToken {
                ref kind,
                ref value,
            } if kind == "render_asset_usage" && value == "video_memory"
        ));
    }

    #[test]
    fn linear_color_space_normalizes_default_rgba8_format_to_linear() {
        let settings = r#"color_space = "linear""#.parse::<toml::Table>().expect("valid toml");

        let descriptor = TextureAssetDescriptor::default()
            .apply_import_settings(&settings)
            .expect("valid linear color space");

        assert_eq!(descriptor.format, RGBA8_UNORM_FORMAT);
        assert_eq!(
            descriptor.to_render_image_descriptor(2, 2).format,
            RGBA8_UNORM_FORMAT
        );
    }

    #[test]
    fn unknown_color_space_is_rejected_by_the_import_contract() {
        let settings = r#"color_space = "unknown""#.parse::<toml::Table>().expect("valid toml");

        let error = TextureAssetDescriptor::default()
            .apply_import_settings(&settings)
            .expect_err("unknown color spaces are not valid texture metadata");

        assert!(matches!(
            error,
            TextureDescriptorError::UnsupportedToken {
                ref kind,
                ref value,
            } if kind == "color_space" && value == "unknown"
        ));
    }

    #[test]
    fn import_color_space_is_written_to_texture_metadata() {
        let settings = r#"color_space = "linear""#.parse::<toml::Table>().expect("valid toml");

        let descriptor = TextureAssetDescriptor::default()
            .apply_import_settings(&settings)
            .expect("valid linear color space");

        assert_eq!(
            descriptor.metadata.color_space,
            RenderImageColorSpace::Linear
        );
        assert_eq!(
            descriptor.to_render_image_descriptor(2, 2).color_space,
            RenderImageColorSpace::Linear
        );
    }

    #[test]
    fn render_image_descriptor_preserves_texture_metadata() {
        let settings = r#"
usage_hint = "normal"
mip_policy = "generate_offline"
normal_convention = "dx"
compression = "bc5"
"#
        .parse::<toml::Table>()
        .expect("valid texture metadata settings");
        let descriptor = TextureAssetDescriptor::default()
            .apply_import_settings(&settings)
            .expect("valid descriptor metadata");

        let render_descriptor = descriptor.to_render_image_descriptor(4, 4);

        assert_eq!(render_descriptor.metadata, descriptor.metadata);
    }

    #[test]
    fn import_settings_parse_texture_metadata_tokens() {
        let settings = r#"
usage_hint = "normal"
mip_policy = "generate_offline"
normal_convention = "dx"
compression = "bc5"
"#
        .parse::<toml::Table>()
        .expect("valid toml");

        let descriptor = TextureAssetDescriptor::default()
            .apply_import_settings(&settings)
            .expect("valid texture metadata");

        assert_eq!(descriptor.metadata.usage_hint, TextureUsageHint::Normal);
        assert_eq!(
            descriptor.metadata.mip_policy,
            TextureMipPolicy::GenerateOffline
        );
        assert_eq!(
            descriptor.metadata.normal_convention,
            TextureNormalConvention::TangentSpaceDx
        );
        assert_eq!(
            descriptor.metadata.compression,
            TextureCompressionTarget::Bc5
        );
        assert_eq!(
            descriptor.metadata.color_space,
            RenderImageColorSpace::Linear
        );
    }

    #[test]
    fn usage_hint_selects_color_space_when_not_explicitly_overridden() {
        let normal_settings =
            r#"usage_hint = "normal""#.parse::<toml::Table>().expect("valid normal settings");
        let ui_settings = r#"usage_hint = "ui""#.parse::<toml::Table>().expect("valid ui settings");

        let normal = TextureAssetDescriptor::default()
            .apply_import_settings(&normal_settings)
            .expect("normal defaults should be valid");
        let ui = TextureAssetDescriptor::default()
            .apply_import_settings(&ui_settings)
            .expect("ui defaults should be valid");

        assert_eq!(normal.color_space, RenderImageColorSpace::Linear);
        assert_eq!(normal.metadata.color_space, RenderImageColorSpace::Linear);
        assert_eq!(normal.format, RGBA8_UNORM_FORMAT);
        assert_eq!(ui.color_space, RenderImageColorSpace::Srgb);
        assert_eq!(ui.metadata.color_space, RenderImageColorSpace::Srgb);
    }

    #[test]
    fn descriptor_metadata_validation_uses_the_canonical_metadata_field() {
        let mut descriptor = TextureAssetDescriptor::default();
        descriptor.metadata.usage_hint = TextureUsageHint::Normal;

        assert!(
            descriptor
                .validate_metadata("textures/normal.png")
                .iter()
                .any(|diagnostic| diagnostic.severity == TextureMetadataDiagnosticSeverity::Error)
        );
    }
}
