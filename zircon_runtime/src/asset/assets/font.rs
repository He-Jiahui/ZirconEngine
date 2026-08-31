use std::sync::Arc;

use serde::{Deserialize, Serialize};
use thiserror::Error;

use zircon_runtime_interface::ui::surface::UiTextRenderMode;

pub type FontAssetResult<T> = std::result::Result<T, FontAssetError>;

#[derive(Clone, Debug, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct FontFamilyName(pub String);

impl FontFamilyName {
    pub fn new(name: impl Into<String>) -> Self {
        Self(name.into().trim().to_string())
    }

    pub fn as_str(&self) -> &str {
        &self.0
    }

    pub fn is_empty(&self) -> bool {
        self.0.trim().is_empty()
    }
}

impl From<&str> for FontFamilyName {
    fn from(value: &str) -> Self {
        Self::new(value)
    }
}

impl From<String> for FontFamilyName {
    fn from(value: String) -> Self {
        Self::new(value)
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum FontScript {
    Latin,
    Cyrillic,
    Greek,
    Han,
    Hiragana,
    Katakana,
    Hangul,
    Arabic,
    Hebrew,
    Devanagari,
    Unknown,
    /// Packed big-endian ISO 15924 tag for scripts without a dedicated variant.
    Other(FontScriptTag),
}

/// Validated packed representation of a canonical four-letter ISO 15924 tag.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub struct FontScriptTag(u32);

impl FontScriptTag {
    pub const EMOJI: Self = Self(u32::from_be_bytes(*b"Zsye"));

    pub const fn from_bytes(bytes: [u8; 4]) -> Option<Self> {
        let canonical = bytes[0] >= b'A'
            && bytes[0] <= b'Z'
            && bytes[1] >= b'a'
            && bytes[1] <= b'z'
            && bytes[2] >= b'a'
            && bytes[2] <= b'z'
            && bytes[3] >= b'a'
            && bytes[3] <= b'z';
        if canonical {
            Some(Self(u32::from_be_bytes(bytes)))
        } else {
            None
        }
    }

    pub const fn from_packed(packed: u32) -> Option<Self> {
        Self::from_bytes(packed.to_be_bytes())
    }

    pub fn parse(tag: &str) -> Option<Self> {
        let bytes = tag.as_bytes().try_into().ok()?;
        Self::from_bytes(bytes)
    }

    pub const fn packed(self) -> u32 {
        self.0
    }
}

impl Serialize for FontScriptTag {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        self.0.serialize(serializer)
    }
}

impl<'de> Deserialize<'de> for FontScriptTag {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        let packed = u32::deserialize(deserializer)?;
        Self::from_packed(packed).ok_or_else(|| {
            serde::de::Error::custom(
                "font script tag must be a packed canonical four-letter ISO 15924 code",
            )
        })
    }
}

impl FontScript {
    pub(crate) fn from_iso15924_tag(tag: &str) -> Self {
        match tag {
            "Latn" => Self::Latin,
            "Cyrl" => Self::Cyrillic,
            "Grek" => Self::Greek,
            "Hani" => Self::Han,
            "Hira" => Self::Hiragana,
            "Kana" => Self::Katakana,
            "Hang" => Self::Hangul,
            "Arab" => Self::Arabic,
            "Hebr" => Self::Hebrew,
            "Deva" => Self::Devanagari,
            "Zzzz" => Self::Unknown,
            other => FontScriptTag::parse(other)
                .map(Self::Other)
                .unwrap_or(Self::Unknown),
        }
    }
}

/// Authored BCP-47 culture selector for script-equivalent composite sub-fonts.
/// Runtime Text compiles and matches this opaque asset value.
#[derive(Clone, Debug, Default, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(transparent)]
pub struct FontCultureTag(String);

impl FontCultureTag {
    pub fn new(tag: impl Into<String>) -> Self {
        Self(tag.into().trim().to_string())
    }

    pub fn as_str(&self) -> &str {
        &self.0
    }
}

impl From<&str> for FontCultureTag {
    fn from(value: &str) -> Self {
        Self::new(value)
    }
}

impl From<String> for FontCultureTag {
    fn from(value: String) -> Self {
        Self::new(value)
    }
}

/// Asset-authored composite font configuration consumed by the text runtime.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct CompositeFontDescriptor {
    pub default_family: FontFamilyName,
    #[serde(default)]
    pub sub_fonts: Vec<SubFontRange>,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct SubFontRange {
    pub family: FontFamilyName,
    #[serde(default)]
    pub scripts: Vec<FontScript>,
    #[serde(default)]
    pub ranges: Vec<(u32, u32)>,
    #[serde(default)]
    pub cultures: Vec<FontCultureTag>,
}

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
    #[cfg(feature = "text")]
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub composite_font: Option<CompositeFontDescriptor>,
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
        schema_v1_render_mode: Option<UiTextRenderMode>,
    ) -> Option<UiTextRenderMode> {
        let mode = schema_v1_render_mode.or(self.default_mode);
        let allow_native = self.allow_native.unwrap_or(true);
        let allow_sdf = self.allow_sdf.unwrap_or(true);
        match mode {
            Some(UiTextRenderMode::Native) if !allow_native => {
                allow_sdf.then_some(UiTextRenderMode::Sdf)
            }
            Some(UiTextRenderMode::Sdf | UiTextRenderMode::Msdf | UiTextRenderMode::Mtsdf)
                if !allow_sdf =>
            {
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
    /// Decoded font bytes retained by the cooked artifact cache. This is not
    /// authoring data: the cache payload mirrors it explicitly so packaged
    /// runtime code can resolve faces without reopening the source file.
    #[serde(skip)]
    pub cooked_blob: Option<FontBlobArtifact>,
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum FontAssetSourceFormat {
    #[default]
    Sfnt,
    TrueTypeCollection,
    Woff2,
}

const FONT_BLOB_ARTIFACT_SCHEMA_VERSION: u32 = 1;

/// Immutable decoded font payload produced during import.
///
/// `source_format` records the authored container while `bytes` always hold
/// the decoded SFNT or TTC payload consumed by the runtime font collection.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct FontBlobArtifact {
    schema_version: u32,
    source_format: FontAssetSourceFormat,
    content_hash: [u8; 32],
    bytes: Arc<[u8]>,
}

impl FontBlobArtifact {
    pub(crate) fn from_decoded_bytes(source_format: FontAssetSourceFormat, bytes: Vec<u8>) -> Self {
        let content_hash = *blake3::hash(&bytes).as_bytes();
        Self {
            schema_version: FONT_BLOB_ARTIFACT_SCHEMA_VERSION,
            source_format,
            content_hash,
            bytes: Arc::from(bytes.into_boxed_slice()),
        }
    }

    pub fn schema_version(&self) -> u32 {
        self.schema_version
    }

    pub fn source_format(&self) -> FontAssetSourceFormat {
        self.source_format
    }

    pub fn content_hash(&self) -> [u8; 32] {
        self.content_hash
    }

    pub fn bytes(&self) -> &[u8] {
        &self.bytes
    }

    pub(crate) fn shared_bytes(&self) -> Arc<[u8]> {
        Arc::clone(&self.bytes)
    }

    pub(crate) fn is_valid_for_runtime(&self) -> bool {
        self.schema_version == FONT_BLOB_ARTIFACT_SCHEMA_VERSION && self.has_valid_content_hash()
    }

    pub fn has_valid_content_hash(&self) -> bool {
        self.content_hash == *blake3::hash(&self.bytes).as_bytes()
    }
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
    #[serde(default)]
    pub metrics: FontAssetFaceMetrics,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub variation_axes: Vec<FontAssetVariationAxis>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub named_instances: Vec<FontAssetVariableInstance>,
    pub cmap: FontAssetCmapCoverage,
}

/// Font-unit metrics shared by layout and decoration rendering.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct FontAssetFaceMetrics {
    pub units_per_em: u16,
    pub ascender: i16,
    pub descender: i16,
    pub line_gap: i16,
    pub uses_typographic_metrics: bool,
    pub windows_ascender: i16,
    pub windows_descender: i16,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub underline: Option<FontAssetLineMetrics>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub strikeout: Option<FontAssetLineMetrics>,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct FontAssetLineMetrics {
    pub position: i16,
    pub thickness: i16,
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

#[cfg(test)]
mod contract_owner_tests {
    use std::path::Path;

    #[test]
    fn composite_font_contract_is_owned_by_the_font_asset_schema() {
        let asset = include_str!("font.rs")
            .split("#[cfg(test)]")
            .next()
            .expect("font asset production source must precede its tests");
        let cache = include_str!("../artifact/cache_payload/font.rs");
        let text_family = include_str!("../../text/model/font/family.rs");
        let text_font = include_str!("../../text/model/font/mod.rs");

        assert!(asset.contains("pub struct CompositeFontDescriptor"));
        assert!(asset.contains("pub struct FontFamilyName"));
        assert!(!asset.contains("crate::text"));
        assert!(!cache.contains("crate::text"));
        assert!(!text_family.contains("pub struct FontFamilyName"));
        assert!(text_family.contains("use crate::asset::assets::FontFamilyName;"));
        assert!(text_font.contains("pub use crate::asset::assets::{"));
        for contract in [
            "CompositeFontDescriptor",
            "FontCultureTag",
            "FontFamilyName",
            "FontScript",
            "FontScriptTag",
            "SubFontRange",
        ] {
            assert!(text_font.contains(contract));
        }
        assert!(!Path::new(env!("CARGO_MANIFEST_DIR"))
            .join("src/text/model/font/composite.rs")
            .exists());
    }
}
