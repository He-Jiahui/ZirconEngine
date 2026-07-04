use serde::{Deserialize, Serialize};
use zircon_runtime_interface::ui::surface::{UiResolvedStyle, UiTextDirection, UiTextRange};

use super::font::FontFaceId;

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

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct ShapedGlyph {
    pub glyph_id: u32,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub font_id: Option<FontFaceId>,
    pub source_range: UiTextRange,
    pub visual_range: UiTextRange,
    pub advance: f32,
    pub x: f32,
    pub y: f32,
    pub offset_x: f32,
    pub offset_y: f32,
    pub direction: UiTextDirection,
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
    pub source_range: UiTextRange,
    pub visual_range: UiTextRange,
    pub measured_width: f32,
    pub baseline: f32,
    pub line_height: f32,
    pub glyphs: Vec<ShapedGlyph>,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct ShapedGlyphRun {
    pub source_text: String,
    pub source_range: UiTextRange,
    pub direction: UiTextDirection,
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
pub struct TextShapeRequest<'a> {
    pub text: &'a str,
    pub style: &'a UiResolvedStyle,
    pub base_direction: UiTextDirection,
    pub source_range: UiTextRange,
    pub orientation: TextOrientation,
    pub vertical_mode: VerticalMode,
    pub include_kerning: bool,
}

impl<'a> TextShapeRequest<'a> {
    pub fn horizontal(
        text: &'a str,
        style: &'a UiResolvedStyle,
        base_direction: UiTextDirection,
        source_range: UiTextRange,
    ) -> Self {
        Self::horizontal_with_kerning(text, style, base_direction, source_range, true)
    }

    pub fn horizontal_with_kerning(
        text: &'a str,
        style: &'a UiResolvedStyle,
        base_direction: UiTextDirection,
        source_range: UiTextRange,
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
        }
    }
}

const fn default_include_kerning() -> bool {
    true
}

fn is_default_include_kerning(value: &bool) -> bool {
    *value
}
