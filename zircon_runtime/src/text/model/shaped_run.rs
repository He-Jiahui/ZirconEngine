use std::{collections::BTreeMap, sync::Arc};

use crate::core::framework::text::{
    TextDirection, TextLayoutError, TextVerticalGlyphDecisionBasis,
};
use crate::text::language::{
    TextLanguageFallbackKey, TextLanguageScriptSubtag, canonical_text_language,
};
use crate::text::unicode_data::{UnicodeDataSnapshotId, compiled_unicode_data_snapshot_id};
use crate::text::{TextRange, TextStyle};
use serde::{Deserialize, Deserializer, Serialize, Serializer, de::Error as _};

use super::font::{FontFaceId, InstancedFaceId};
use super::shaping_receipt::TextHorizontalCompositionReceipt;

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

    pub(crate) const fn as_bytes(&self) -> &[u8; 4] {
        &self.0
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

/// Provenance for reshaping at the beginning of a backend cluster.
///
/// This is a receipt, not a line-break policy. `RequiresReshape` means a break
/// remains legal only if the final-line owner reshapes both sides.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
#[repr(u8)]
pub enum ShapedGlyphBreakSafety {
    #[default]
    Unknown,
    Safe,
    RequiresReshape,
}

impl ShapedGlyphBreakSafety {
    pub(crate) const fn from_direct_backend(unsafe_to_break: bool) -> Self {
        if unsafe_to_break {
            Self::RequiresReshape
        } else {
            Self::Safe
        }
    }
}

/// Compiled line-break tailoring profile used to analyze a shaped cluster.
///
/// The owning `ShapedGlyphRun::unicode_data_snapshot` supplies the provider data version. This
/// compact profile receipt distinguishes current UAX #14 defaults from legacy or unknown data
/// without copying a locale tag into every glyph.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
#[repr(u8)]
pub enum LineBreakTailoringProfile {
    #[default]
    Unknown,
    UnicodeDefault,
}

/// Opportunity selected for a cluster boundary by the recorded line-break profile.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
#[repr(u8)]
pub enum ShapedGlyphLineBreakOpportunity {
    #[default]
    None,
    ProviderAllowed,
    ProviderMandatory,
    MandatoryControl,
}

/// Per-cluster line-break provenance retained with the shaped artifact.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct ShapedGlyphLineBreakReceipt {
    pub profile: LineBreakTailoringProfile,
    pub opportunity: ShapedGlyphLineBreakOpportunity,
}

impl ShapedGlyphLineBreakReceipt {
    pub(crate) const fn mandatory_control() -> Self {
        Self {
            profile: LineBreakTailoringProfile::UnicodeDefault,
            opportunity: ShapedGlyphLineBreakOpportunity::MandatoryControl,
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
    /// Meaningful only when `cluster_start` is true.
    #[serde(default)]
    pub break_safety: ShapedGlyphBreakSafety,
    /// Meaningful only when `cluster_start` is true. The run-level Unicode snapshot versions the
    /// provider named by this receipt.
    #[serde(default)]
    pub line_break: ShapedGlyphLineBreakReceipt,
    /// Present only on the first glyph of a vertical cluster.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub vertical_decision: Option<TextVerticalGlyphDecisionBasis>,
}

impl ShapedGlyphClusterFlags {
    pub(crate) const fn with_direct_break_safety(mut self, unsafe_to_break: bool) -> Self {
        if self.cluster_start {
            self.break_safety = ShapedGlyphBreakSafety::from_direct_backend(unsafe_to_break);
        }
        self
    }

    pub(crate) const fn with_vertical_decision(
        mut self,
        decision: TextVerticalGlyphDecisionBasis,
    ) -> Self {
        if self.cluster_start {
            self.vertical_decision = Some(decision);
        }
        self
    }
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
    let mut values_by_tag = BTreeMap::new();
    for feature in features {
        values_by_tag.insert(feature.tag, feature.value);
    }
    values_by_tag
        .into_iter()
        .map(|(tag, value)| OpenTypeFeature::new(tag, value))
        .collect()
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

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct VerticalGlyphDecision {
    pub basis: TextVerticalGlyphDecisionBasis,
    pub rotation: ShapedGlyphRotation,
    pub font_id: Option<FontFaceId>,
    pub font_instance_id: Option<InstancedFaceId>,
}

impl ShapedGlyph {
    pub fn vertical_glyph_decision(&self) -> Option<VerticalGlyphDecision> {
        let basis = self
            .cluster_flags
            .cluster_start
            .then_some(self.cluster_flags.vertical_decision)
            .flatten()?;
        Some(VerticalGlyphDecision {
            basis,
            rotation: self.rotation,
            font_id: self.font_id,
            font_instance_id: self.font_instance_id,
        })
    }
}

/// One explicit hard-line projection produced by shaping before wrap and overflow layout.
///
/// This owner may contain a hard-break separator glyph. It is not a visual/layout line; wrapping,
/// ellipsis, and final placement remain with the layout layer.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct ShapedHardLine {
    pub line_index: usize,
    pub source_range: TextRange,
    pub visual_range: TextRange,
    pub measured_width: f32,
    pub baseline: f32,
    pub line_height: f32,
    pub glyphs: Vec<ShapedGlyph>,
}

/// Backend-neutral raw vertical metrics for one horizontal shaped line.
///
/// This sidecar preserves the selected-face content envelope before a composite UI line policy
/// chooses its public baseline. It is deliberately separate from `ShapedHardLine::baseline`,
/// which remains the current single-fragment layout result for compatibility.
#[derive(Clone, Copy, Debug, PartialEq, Serialize, Deserialize)]
pub(crate) struct HorizontalLineRawMetrics {
    ascent: f32,
    descent: f32,
    line_spacing_gap: f32,
}

impl HorizontalLineRawMetrics {
    pub(crate) fn new(ascent: f32, descent: f32, line_spacing_gap: f32) -> Option<Self> {
        (ascent.is_finite()
            && ascent >= 0.0
            && descent.is_finite()
            && descent >= 0.0
            && line_spacing_gap.is_finite()
            && line_spacing_gap >= 0.0)
            .then_some(Self {
                ascent,
                descent,
                line_spacing_gap,
            })
    }

    pub(crate) const fn ascent(self) -> f32 {
        self.ascent
    }

    pub(crate) const fn descent(self) -> f32 {
        self.descent
    }

    pub(crate) const fn line_spacing_gap(self) -> f32 {
        self.line_spacing_gap
    }
}

/// Raw selected-face metrics for one contiguous glyph range in a horizontal shaped line.
///
/// This stays crate-private and serde-skipped with its owning shaped run. It is captured while
/// direct shaping still owns the selected face, so later composite line policy never needs to
/// rediscover a glyph's vertical metrics from the font database.
#[derive(Clone, Copy, Debug, PartialEq)]
pub(crate) struct HorizontalGlyphMetricSpan {
    pub(crate) line_index: usize,
    pub(crate) glyph_start: usize,
    pub(crate) glyph_end: usize,
    pub(crate) metrics: HorizontalLineRawMetrics,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct ShapedGlyphRun {
    #[serde(with = "arc_str_serde")]
    pub source_text: Arc<str>,
    pub source_range: TextRange,
    pub unicode_data_snapshot: UnicodeDataSnapshotId,
    /// Query primary face selected before fallback itemization.
    ///
    /// This remains distinct from every glyph's selected fallback face so physical-line policy
    /// can use the primary line-gap rule without re-resolving the font chain.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub primary_face_id: Option<FontFaceId>,
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
    /// Present when a horizontal run uses an alternate backend for all or part of the source.
    /// Empty alternate ranges identify whole-run alternate output.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub horizontal_composition_receipt: Option<Box<TextHorizontalCompositionReceipt>>,
    /// One entry per shaped hard line when a horizontal backend preserves raw selected-face
    /// extents. Empty means the backend did not provide this optional sidecar.
    #[serde(skip)]
    pub(crate) horizontal_line_raw_metrics: Vec<Option<HorizontalLineRawMetrics>>,
    /// Contiguous direct-shaping face spans for horizontal glyph-origin policy.
    ///
    /// A line is usable only when these spans cover all of its glyphs in order. Backends that
    /// cannot preserve same-pass face metrics leave this sidecar empty and later policy falls
    /// back without guessing from `font_id`.
    #[serde(skip)]
    pub(crate) horizontal_glyph_metric_spans: Vec<HorizontalGlyphMetricSpan>,
    pub lines: Vec<ShapedHardLine>,
}

impl ShapedGlyphRun {
    pub fn hard_line_text(&self, line: &ShapedHardLine) -> Option<&str> {
        let start = line
            .source_range
            .start
            .checked_sub(self.source_range.start)?;
        let end = line.source_range.end.checked_sub(self.source_range.start)?;
        self.source_text.get(start..end)
    }

    pub(crate) fn horizontal_line_raw_metrics_at(
        &self,
        line_index: usize,
    ) -> Option<HorizontalLineRawMetrics> {
        (self.orientation == TextOrientation::Horizontal
            && self.horizontal_line_raw_metrics.len() == self.lines.len())
        .then(|| {
            self.horizontal_line_raw_metrics
                .get(line_index)
                .copied()
                .flatten()
        })
        .flatten()
    }

    /// Returns a line's complete ordered selected-face span coverage when it is safe to consume.
    pub(crate) fn horizontal_glyph_metric_spans_for_line(
        &self,
        line_index: usize,
    ) -> Option<&[HorizontalGlyphMetricSpan]> {
        if self.orientation != TextOrientation::Horizontal {
            return None;
        }
        let glyph_count = self.lines.get(line_index)?.glyphs.len();
        if glyph_count == 0 {
            return None;
        }
        let first = self
            .horizontal_glyph_metric_spans
            .partition_point(|span| span.line_index < line_index);
        let after_last = self
            .horizontal_glyph_metric_spans
            .partition_point(|span| span.line_index <= line_index);
        let spans = self.horizontal_glyph_metric_spans.get(first..after_last)?;
        let mut expected_start = 0_usize;
        for span in spans {
            if span.glyph_start != expected_start
                || span.glyph_start >= span.glyph_end
                || span.glyph_end > glyph_count
            {
                return None;
            }
            expected_start = span.glyph_end;
        }
        (expected_start == glyph_count).then_some(spans)
    }
}

#[derive(Clone, Copy, Debug)]
pub(crate) struct BackendShapeRequest<'a> {
    pub text: &'a str,
    /// Reuses an owning parallel-request allocation when it exactly covers `text`.
    source_owner: Option<&'a Arc<str>>,
    pub style: &'a TextStyle,
    pub base_direction: TextDirection,
    /// Absolute source identity for the local `text` view; canonicalization requires equal byte
    /// spans, while allowing a non-zero document offset.
    pub source_range: TextRange,
    pub orientation: TextOrientation,
    pub vertical_mode: VerticalMode,
    pub include_kerning: bool,
    pub language: Option<&'a str>,
    language_is_canonical: bool,
    language_fallback_key: Option<TextLanguageFallbackKey>,
    features: &'a [OpenTypeFeature],
    features_are_normalized: bool,
    unicode_data_snapshot: UnicodeDataSnapshotId,
}

pub(crate) struct CanonicalBackendShapeRequest<'a> {
    request: BackendShapeRequest<'a>,
    normalized_features: Option<Vec<OpenTypeFeature>>,
    normalized_language: Option<String>,
    language_fallback_key: Option<TextLanguageFallbackKey>,
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
        let language = normalized_style_language(style);
        Self {
            text,
            source_owner: None,
            style,
            base_direction,
            source_range,
            orientation: TextOrientation::Horizontal,
            vertical_mode: VerticalMode::Mixed,
            include_kerning,
            language,
            language_is_canonical: language.is_none(),
            language_fallback_key: None,
            features: style.features.as_ref(),
            features_are_normalized: style.features.is_empty(),
            unicode_data_snapshot: compiled_unicode_data_snapshot_id(),
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
        let language = normalized_style_language(style);
        Self {
            text,
            source_owner: None,
            style,
            base_direction,
            source_range,
            orientation: TextOrientation::Vertical,
            vertical_mode,
            include_kerning,
            language,
            language_is_canonical: language.is_none(),
            language_fallback_key: None,
            features: style.features.as_ref(),
            features_are_normalized: style.features.is_empty(),
            unicode_data_snapshot: compiled_unicode_data_snapshot_id(),
        }
    }

    pub fn with_language(mut self, language: Option<&'a str>) -> Self {
        self.language = language
            .map(str::trim)
            .filter(|language| !language.is_empty());
        self.language_is_canonical = self.language.is_none();
        self.language_fallback_key = None;
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

    pub(crate) const fn has_exact_source_owner(&self) -> bool {
        self.source_owner.is_some()
    }

    pub(crate) fn features(&self) -> &[OpenTypeFeature] {
        self.features
    }

    pub(crate) const fn unicode_data_snapshot(&self) -> UnicodeDataSnapshotId {
        self.unicode_data_snapshot
    }

    pub(crate) fn explicit_language_script(&self) -> Option<TextLanguageScriptSubtag> {
        self.language_fallback_key
            .and_then(TextLanguageFallbackKey::explicit_script)
    }

    pub(crate) const fn language_fallback_key(&self) -> Option<TextLanguageFallbackKey> {
        self.language_fallback_key
    }

    #[cfg(test)]
    pub(crate) fn with_unicode_data_snapshot_for_test(
        mut self,
        unicode_data_snapshot: UnicodeDataSnapshotId,
    ) -> Self {
        self.unicode_data_snapshot = unicode_data_snapshot;
        self
    }

    fn reborrow_canonical<'b>(
        &'b self,
        language: Option<&'b str>,
        features: &'b [OpenTypeFeature],
        language_fallback_key: Option<TextLanguageFallbackKey>,
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
            language,
            language_is_canonical: true,
            language_fallback_key,
            features,
            features_are_normalized: true,
            unicode_data_snapshot: self.unicode_data_snapshot,
        }
    }

    pub(crate) const fn features_are_normalized(&self) -> bool {
        self.features_are_normalized
    }

    pub(crate) const fn language_is_canonical(&self) -> bool {
        self.language_is_canonical
    }

    pub(crate) fn canonicalized(self) -> Result<CanonicalBackendShapeRequest<'a>, TextLayoutError> {
        // The local text view and absolute identity must describe the same UTF-8 byte span;
        // otherwise backend clusters cannot be translated back to document coordinates safely.
        let Some(source_span) = self.source_range.end.checked_sub(self.source_range.start) else {
            return Err(TextLayoutError::BidiInvariant);
        };
        if source_span != self.text.len() {
            return Err(TextLayoutError::BidiInvariant);
        }
        let (normalized_language, language_fallback_key) =
            match (self.language_is_canonical, self.language) {
                (true, _) => (None, self.language_fallback_key),
                (false, Some(language)) => {
                    let canonical = canonical_text_language(language)
                        .map_err(|_| TextLayoutError::InvalidLanguage)?;
                    let fallback_key = canonical.fallback_key();
                    let normalized = match canonical.into_tag() {
                        std::borrow::Cow::Borrowed(_) => None,
                        std::borrow::Cow::Owned(language) => Some(language),
                    };
                    (normalized, Some(fallback_key))
                }
                (false, None) => (None, None),
            };
        Ok(CanonicalBackendShapeRequest {
            language_fallback_key,
            normalized_features: (!self.features_are_normalized)
                .then(|| normalized_open_type_features(self.features)),
            normalized_language,
            request: self,
        })
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
        self.request.reborrow_canonical(
            self.normalized_language
                .as_deref()
                .or(self.request.language),
            self.normalized_features
                .as_deref()
                .unwrap_or(self.request.features),
            self.language_fallback_key,
        )
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
mod tests;
