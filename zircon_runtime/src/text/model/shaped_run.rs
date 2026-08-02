use std::sync::Arc;

use crate::core::framework::text::TextDirection;
use crate::text::{TextRange, TextStyle};
use serde::{de::Error as _, Deserialize, Deserializer, Serialize, Serializer};

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

/// Inline script identity that retains the existing four-character serde contract.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
#[repr(transparent)]
pub struct Iso15924Tag([u8; 4]);

impl Iso15924Tag {
    pub const COMMON: Self = Self(*b"Zyyy");
    pub const EMOJI: Self = Self(*b"Zsye");

    pub fn parse(value: &str) -> Option<Self> {
        let bytes: [u8; 4] = value.as_bytes().try_into().ok()?;
        bytes
            .iter()
            .all(u8::is_ascii_alphabetic)
            .then_some(Self(bytes))
    }

    pub fn as_str(&self) -> &str {
        match self.0 {
            [b'L', b'a', b't', b'n'] => "Latn",
            [b'C', b'y', b'r', b'l'] => "Cyrl",
            [b'G', b'r', b'e', b'k'] => "Grek",
            [b'H', b'a', b'n', b'i'] => "Hani",
            [b'H', b'i', b'r', b'a'] => "Hira",
            [b'K', b'a', b'n', b'a'] => "Kana",
            [b'H', b'a', b'n', b'g'] => "Hang",
            [b'A', b'r', b'a', b'b'] => "Arab",
            [b'H', b'e', b'b', b'r'] => "Hebr",
            [b'D', b'e', b'v', b'a'] => "Deva",
            [b'Z', b's', b'y', b'e'] => "Zsye",
            [b'Z', b'y', b'y', b'y'] => "Zyyy",
            _ => match std::str::from_utf8(&self.0) {
                Ok(tag) => tag,
                Err(_) => "Zyyy",
            },
        }
    }
}

impl Default for Iso15924Tag {
    fn default() -> Self {
        Self::COMMON
    }
}

impl PartialEq<&str> for Iso15924Tag {
    fn eq(&self, other: &&str) -> bool {
        self.as_str() == *other
    }
}

impl Serialize for Iso15924Tag {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        serializer.serialize_str(self.as_str())
    }
}

impl<'de> Deserialize<'de> for Iso15924Tag {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        let value = String::deserialize(deserializer)?;
        Self::parse(&value)
            .ok_or_else(|| D::Error::custom("ISO15924 tag must contain exactly four ASCII letters"))
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct ShapedGlyphScript {
    pub iso15924: Iso15924Tag,
}

impl Default for ShapedGlyphScript {
    fn default() -> Self {
        Self {
            iso15924: Iso15924Tag::COMMON,
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
    pub source_range: TextRange,
    pub visual_range: TextRange,
    pub measured_width: f32,
    pub baseline: f32,
    pub line_height: f32,
    pub glyphs: Vec<ShapedGlyph>,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct ShapedGlyphRun {
    #[serde(with = "arc_str_serde")]
    pub source_text: Arc<str>,
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

impl ShapedGlyphRun {
    pub fn line_text(&self, line: &ShapedTextLine) -> Option<&str> {
        let start = line
            .source_range
            .start
            .checked_sub(self.source_range.start)?;
        let end = line.source_range.end.checked_sub(self.source_range.start)?;
        self.source_text.get(start..end)
    }
}

#[derive(Clone, Copy, Debug)]
pub(crate) struct BackendShapeRequest<'a> {
    pub text: &'a str,
    /// Reuses an owning parallel-request allocation when it exactly covers `text`.
    source_owner: Option<&'a Arc<str>>,
    pub style: &'a TextStyle,
    pub base_direction: TextDirection,
    pub source_range: TextRange,
    pub orientation: TextOrientation,
    pub vertical_mode: VerticalMode,
    pub include_kerning: bool,
    pub language: Option<&'a str>,
    features: &'a [OpenTypeFeature],
    features_are_normalized: bool,
}

pub(crate) struct CanonicalBackendShapeRequest<'a> {
    request: BackendShapeRequest<'a>,
    normalized_features: Option<Vec<OpenTypeFeature>>,
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
            source_owner: None,
            style,
            base_direction,
            source_range,
            orientation: TextOrientation::Horizontal,
            vertical_mode: VerticalMode::Mixed,
            include_kerning,
            language: normalized_style_language(style),
            features: &[],
            features_are_normalized: true,
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
            source_owner: None,
            style,
            base_direction,
            source_range,
            orientation: TextOrientation::Vertical,
            vertical_mode,
            include_kerning,
            language: normalized_style_language(style),
            features: &[],
            features_are_normalized: true,
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
        self.features_are_normalized = features.is_empty();
        self
    }

    pub(crate) fn with_source_owner(mut self, source_owner: &'a Arc<str>) -> Self {
        self.source_owner = (source_owner.as_ref() == self.text).then_some(source_owner);
        self
    }

    pub(crate) fn features(&self) -> &[OpenTypeFeature] {
        self.features
    }

    fn reborrow_with_normalized_features<'b>(
        &'b self,
        features: &'b [OpenTypeFeature],
    ) -> BackendShapeRequest<'b> {
        BackendShapeRequest {
            text: self.text,
            source_owner: self.source_owner,
            style: self.style,
            base_direction: self.base_direction,
            source_range: self.source_range,
            orientation: self.orientation,
            vertical_mode: self.vertical_mode,
            include_kerning: self.include_kerning,
            language: self.language,
            features,
            features_are_normalized: true,
        }
    }

    pub(crate) const fn features_are_normalized(&self) -> bool {
        self.features_are_normalized
    }

    pub(crate) fn canonicalized(self) -> CanonicalBackendShapeRequest<'a> {
        CanonicalBackendShapeRequest {
            normalized_features: (!self.features_are_normalized)
                .then(|| normalized_open_type_features(self.features)),
            request: self,
        }
    }

    pub(crate) fn shared_source_text(&self) -> Arc<str> {
        if let Some(source) = self.source_owner {
            if source.as_ref() == self.text {
                return Arc::clone(source);
            }
        }
        Arc::from(self.text)
    }
}

impl CanonicalBackendShapeRequest<'_> {
    pub(crate) fn request(&self) -> BackendShapeRequest<'_> {
        self.normalized_features
            .as_deref()
            .map_or(self.request, |features| {
                self.request.reborrow_with_normalized_features(features)
            })
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

mod arc_str_serde {
    use std::sync::Arc;

    use serde::{Deserialize, Deserializer, Serializer};

    pub(super) fn serialize<S>(value: &Arc<str>, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        serializer.serialize_str(value)
    }

    pub(super) fn deserialize<'de, D>(deserializer: D) -> Result<Arc<str>, D::Error>
    where
        D: Deserializer<'de>,
    {
        String::deserialize(deserializer).map(Arc::from)
    }
}

#[cfg(test)]
mod tests {
    use std::{mem::size_of, sync::Arc};

    use super::{
        Iso15924Tag, ShapedGlyph, ShapedGlyphClusterFlags, ShapedGlyphRotation, ShapedGlyphRun,
        ShapedGlyphScript, ShapedTextLine, TextOrientation, VerticalMode,
    };
    use crate::{core::framework::text::TextDirection, text::TextRange};

    #[test]
    fn iso15924_tag_is_inline_copy_and_serde_remains_string_compatible() {
        let script = ShapedGlyphScript {
            iso15924: Iso15924Tag::parse("Latn").expect("valid ISO15924 fixture"),
        };

        assert_eq!(size_of::<Iso15924Tag>(), 4);
        assert_eq!(script.iso15924, "Latn");
        let json = serde_json::to_string(&script).expect("script serializes");
        assert_eq!(json, r#"{"iso15924":"Latn"}"#);
        assert_eq!(
            serde_json::from_str::<ShapedGlyphScript>(&json).expect("script deserializes"),
            script
        );
        assert!(serde_json::from_str::<ShapedGlyphScript>(r#"{"iso15924":"Latin"}"#).is_err());
        assert!(serde_json::from_str::<ShapedGlyphScript>(r#"{"iso15924":"La1n"}"#).is_err());
    }

    #[test]
    fn shaped_lines_borrow_absolute_ranges_from_one_shared_source() {
        let source: Arc<str> = Arc::from("alpha beta");
        let run = ShapedGlyphRun {
            source_text: Arc::clone(&source),
            source_range: TextRange { start: 40, end: 50 },
            direction: TextDirection::LeftToRight,
            orientation: TextOrientation::Horizontal,
            vertical_mode: VerticalMode::Mixed,
            include_kerning: true,
            measured_width: 10.0,
            measured_height: 16.0,
            lines: vec![ShapedTextLine {
                line_index: 0,
                source_range: TextRange { start: 46, end: 50 },
                visual_range: TextRange { start: 6, end: 10 },
                measured_width: 4.0,
                baseline: 12.0,
                line_height: 16.0,
                glyphs: vec![ShapedGlyph {
                    glyph_id: 7,
                    font_id: None,
                    font_instance_id: None,
                    source_range: TextRange { start: 46, end: 50 },
                    visual_range: TextRange { start: 6, end: 10 },
                    advance: 4.0,
                    x: 0.0,
                    y: 0.0,
                    offset_x: 0.0,
                    offset_y: 0.0,
                    direction: TextDirection::LeftToRight,
                    bidi_level: 0,
                    cluster_flags: ShapedGlyphClusterFlags {
                        cluster_start: true,
                        ..ShapedGlyphClusterFlags::default()
                    },
                    rotation: ShapedGlyphRotation::None,
                    script: ShapedGlyphScript {
                        iso15924: Iso15924Tag::parse("Latn").expect("valid ISO15924 fixture"),
                    },
                }],
            }],
        };
        let cloned = run.clone();
        let json = serde_json::to_string(&run).expect("shaped run serializes");
        let roundtrip =
            serde_json::from_str::<ShapedGlyphRun>(&json).expect("shaped run deserializes");

        assert_eq!(run.line_text(&run.lines[0]), Some("beta"));
        assert!(Arc::ptr_eq(&run.source_text, &cloned.source_text));
        assert_eq!(roundtrip, run);
        assert_eq!(roundtrip.line_text(&roundtrip.lines[0]), Some("beta"));
    }
}
