use crate::core::framework::text::TextDirection;
use crate::text::{TextRange, TextStyle};
use serde::{Deserialize, Serialize};

use super::font::{FontFaceId, InstancedFaceId};

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
pub enum TextOrientation {
    #[default]
    Horizontal,
    Vertical,
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
pub enum VerticalMode {
    Upright,
    #[default]
    Mixed,
    Sideways,
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
pub enum ShapedGlyphRotation {
    #[default]
    None,
    Cw90,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct ShapedGlyphScript {
    pub iso15924: String,
}

impl Default for ShapedGlyphScript {
    fn default() -> Self {
        Self {
            iso15924: "Zyyy".to_string(),
        }
    }
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct ShapedGlyphClusterFlags {
    pub cluster_start: bool,
    pub rtl: bool,
    pub whitespace: bool,
    pub space: bool,
    pub tab: bool,
    pub mandatory_break: bool,
    pub soft_break: bool,
    pub virtual_glyph: bool,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
pub struct OpenTypeFeature {
    pub tag: [u8; 4],
    pub value: u32,
}

impl OpenTypeFeature {
    pub const fn new(tag: [u8; 4], value: u32) -> Self {
        Self { tag, value }
    }
}

pub fn normalized_open_type_features(features: &[OpenTypeFeature]) -> Vec<OpenTypeFeature> {
    let mut normalized = features.to_vec();
    normalized.sort_unstable();
    normalized.dedup();
    normalized
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct ShapedGlyph {
    pub glyph_id: u32,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub font_id: Option<FontFaceId>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub font_instance_id: Option<InstancedFaceId>,
    pub source_range: TextRange,
    pub visual_range: TextRange,
    pub advance: f32,
    pub x: f32,
    pub y: f32,
    pub offset_x: f32,
    pub offset_y: f32,
    pub direction: TextDirection,
    #[serde(default)]
    pub bidi_level: u8,
    #[serde(default)]
    pub cluster_flags: ShapedGlyphClusterFlags,
    #[serde(default)]
    pub rotation: ShapedGlyphRotation,
    #[serde(default)]
    pub script: ShapedGlyphScript,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct ShapedTextLine {
    pub line_index: usize,
    pub text: String,
    pub source_range: TextRange,
    pub visual_range: TextRange,
    pub measured_width: f32,
    pub baseline: f32,
    pub line_height: f32,
    pub glyphs: Vec<ShapedGlyph>,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct ShapedGlyphRun {
    pub source_text: String,
    pub source_range: TextRange,
    pub direction: TextDirection,
    pub orientation: TextOrientation,
    pub vertical_mode: VerticalMode,
    #[serde(
        default = "default_include_kerning",
        skip_serializing_if = "is_default_include_kerning"
    )]
    pub include_kerning: bool,
    pub measured_width: f32,
    pub measured_height: f32,
    pub lines: Vec<ShapedTextLine>,
}

#[derive(Clone, Copy, Debug)]
pub(crate) struct BackendShapeRequest<'a> {
    pub text: &'a str,
    pub style: &'a TextStyle,
    pub base_direction: TextDirection,
    pub source_range: TextRange,
    pub orientation: TextOrientation,
    pub vertical_mode: VerticalMode,
    pub include_kerning: bool,
    pub language: Option<&'a str>,
    pub features: &'a [OpenTypeFeature],
}

impl<'a> BackendShapeRequest<'a> {
    pub fn horizontal(
        text: &'a str,
        style: &'a TextStyle,
        base_direction: TextDirection,
        source_range: TextRange,
    ) -> Self {
        Self::horizontal_with_kerning(text, style, base_direction, source_range, true)
    }

    pub fn horizontal_with_kerning(
        text: &'a str,
        style: &'a TextStyle,
        base_direction: TextDirection,
        source_range: TextRange,
        include_kerning: bool,
    ) -> Self {
        Self {
            text,
            style,
            base_direction,
            source_range,
            orientation: TextOrientation::Horizontal,
            vertical_mode: VerticalMode::Mixed,
            include_kerning,
            language: normalized_style_language(style),
            features: &[],
        }
    }

    pub fn vertical(
        text: &'a str,
        style: &'a TextStyle,
        base_direction: TextDirection,
        source_range: TextRange,
        vertical_mode: VerticalMode,
    ) -> Self {
        Self::vertical_with_kerning(
            text,
            style,
            base_direction,
            source_range,
            vertical_mode,
            true,
        )
    }

    pub fn vertical_with_kerning(
        text: &'a str,
        style: &'a TextStyle,
        base_direction: TextDirection,
        source_range: TextRange,
        vertical_mode: VerticalMode,
        include_kerning: bool,
    ) -> Self {
        Self {
            text,
            style,
            base_direction,
            source_range,
            orientation: TextOrientation::Vertical,
            vertical_mode,
            include_kerning,
            language: normalized_style_language(style),
            features: &[],
        }
    }

    pub fn with_language(mut self, language: Option<&'a str>) -> Self {
        self.language = language
            .map(str::trim)
            .filter(|language| !language.is_empty());
        self
    }

    pub fn with_kerning(mut self, include_kerning: bool) -> Self {
        self.include_kerning = include_kerning;
        self
    }

    pub fn with_features(mut self, features: &'a [OpenTypeFeature]) -> Self {
        self.features = features;
        self
    }
}

fn normalized_style_language(style: &TextStyle) -> Option<&str> {
    style
        .language
        .as_deref()
        .map(str::trim)
        .filter(|language| !language.is_empty())
}

const fn default_include_kerning() -> bool {
    true
}

fn is_default_include_kerning(value: &bool) -> bool {
    *value
}
