use serde::{Deserialize, Serialize};
use thiserror::Error;

use zircon_runtime_interface::ui::surface::UiTextRenderMode;

pub type FontAssetResult<T> = std::result::Result<T, FontAssetError>;

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct FontAsset {
    pub source: String,
    #[serde(default)]
    pub family: Option<String>,
    #[serde(default)]
    pub render_mode: Option<UiTextRenderMode>,
    #[serde(default, skip_serializing_if = "is_default_face_index")]
    pub face_index: u32,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub family_members: Vec<FontAssetFamilyMember>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub variable_instances: Vec<FontAssetVariableInstance>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub fallback_families: Vec<String>,
    #[serde(default, skip_serializing_if = "FontAssetRenderStrategy::is_default")]
    pub render_strategy: FontAssetRenderStrategy,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub metadata: Option<FontAssetMetadata>,
}

#[derive(Clone, Debug, Default, PartialEq, Serialize, Deserialize)]
pub struct FontAssetRenderStrategy {
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub default_mode: Option<UiTextRenderMode>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub allow_native: Option<bool>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub allow_sdf: Option<bool>,
}

impl FontAssetRenderStrategy {
    fn is_default(&self) -> bool {
        self == &Self::default()
    }

    pub fn effective_render_mode(
        &self,
        legacy_render_mode: Option<UiTextRenderMode>,
    ) -> Option<UiTextRenderMode> {
        let mode = legacy_render_mode.or(self.default_mode);
        let allow_native = self.allow_native.unwrap_or(true);
        let allow_sdf = self.allow_sdf.unwrap_or(true);
        match mode {
            Some(UiTextRenderMode::Native) if !allow_native => {
                allow_sdf.then_some(UiTextRenderMode::Sdf)
            }
            Some(UiTextRenderMode::Sdf) if !allow_sdf => {
                allow_native.then_some(UiTextRenderMode::Native)
            }
            Some(UiTextRenderMode::Auto) => match (allow_native, allow_sdf) {
                (true, true) => Some(UiTextRenderMode::Auto),
                (true, false) => Some(UiTextRenderMode::Native),
                (false, true) => Some(UiTextRenderMode::Sdf),
                (false, false) => None,
            },
            mode => mode,
        }
    }
}

#[derive(Clone, Debug, Default, PartialEq, Serialize, Deserialize)]
pub struct FontAssetFamilyMember {
    pub family: String,
    #[serde(default, skip_serializing_if = "is_default_face_index")]
    pub face_index: u32,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub weight: Option<u16>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub width_class: Option<u16>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub style: Option<FontAssetFaceStyle>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub variations: Vec<FontAssetVariationCoord>,
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum FontAssetFaceStyle {
    #[default]
    Normal,
    Italic,
    Oblique,
}

#[derive(Clone, Debug, Default, PartialEq, Serialize, Deserialize)]
pub struct FontAssetVariationAxis {
    pub tag: String,
    pub min: f32,
    pub default: f32,
    pub max: f32,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub name: Option<String>,
    #[serde(default, skip_serializing_if = "is_false")]
    pub hidden: bool,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct FontAssetVariationCoord {
    pub tag: String,
    pub value: f32,
}

#[derive(Clone, Debug, Default, PartialEq, Serialize, Deserialize)]
pub struct FontAssetVariableInstance {
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub name: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub post_script_name: Option<String>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub coordinates: Vec<FontAssetVariationCoord>,
}

#[derive(Clone, Debug, Default, PartialEq, Serialize, Deserialize)]
pub struct FontAssetMetadata {
    pub source_format: FontAssetSourceFormat,
    pub face_count: u32,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub faces: Vec<FontAssetParsedFace>,
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum FontAssetSourceFormat {
    #[default]
    Sfnt,
    TrueTypeCollection,
    Woff2,
}

#[derive(Clone, Debug, Default, PartialEq, Serialize, Deserialize)]
pub struct FontAssetParsedFace {
    pub face_index: u32,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub family: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub subfamily: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub full_name: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub post_script_name: Option<String>,
    pub weight: u16,
    pub width_class: u16,
    pub style: FontAssetFaceStyle,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub variation_axes: Vec<FontAssetVariationAxis>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub named_instances: Vec<FontAssetVariableInstance>,
    pub cmap: FontAssetCmapCoverage,
}

impl FontAssetParsedFace {
    pub fn family_member(&self) -> Option<FontAssetFamilyMember> {
        Some(FontAssetFamilyMember {
            family: self.family.clone()?,
            face_index: self.face_index,
            weight: Some(self.weight),
            width_class: Some(self.width_class),
            style: Some(self.style),
            variations: Vec::new(),
        })
    }
}

#[derive(Clone, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct FontAssetCmapCoverage {
    pub codepoint_count: u32,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub ranges: Vec<FontAssetCodepointRange>,
}

impl FontAssetCmapCoverage {
    pub fn contains_codepoint(&self, codepoint: u32) -> bool {
        self.ranges
            .iter()
            .any(|range| range.start <= codepoint && codepoint <= range.end)
    }
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct FontAssetCodepointRange {
    pub start: u32,
    pub end: u32,
}

#[derive(Debug, Error)]
pub enum FontAssetError {
    #[error("failed to parse font asset document: {0}")]
    Parse(#[source] toml::de::Error),
}

impl FontAsset {
    pub fn from_toml_str(document: &str) -> FontAssetResult<Self> {
        toml::from_str(document).map_err(FontAssetError::Parse)
    }

    pub fn to_toml_string(&self) -> Result<String, toml::ser::Error> {
        toml::to_string_pretty(self)
    }

    pub fn effective_render_mode(&self) -> Option<UiTextRenderMode> {
        self.render_strategy.effective_render_mode(self.render_mode)
    }
}

fn is_default_face_index(face_index: &u32) -> bool {
    *face_index == 0
}

fn is_false(value: &bool) -> bool {
    !*value
}
