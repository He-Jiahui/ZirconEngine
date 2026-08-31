use std::ops::Range;

use crate::core::framework::text::TextDirection;
use crate::text::{TextRange, UnicodeDataSnapshotId, compiled_unicode_data_snapshot_id};
use unicode_bidi::{BidiClass, BidiInfo, Level};
use unicode_bidi_mirroring::get_mirrored;

/// An internal UAX#9 request violated the paragraph-local source contract.
///
/// These values are deliberately offset-only. Callers may aggregate their category, but must not
/// persist raw text or silently reinterpret an invalid range in logical order.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum BidiInvariantError {
    InvalidResolvedRange {
        start: usize,
        end: usize,
    },
    InvalidLineRange {
        start: usize,
        end: usize,
    },
    LineOutsideParagraph {
        start: usize,
        end: usize,
    },
    GlyphOutsideLine {
        glyph_index: usize,
        start: usize,
        end: usize,
        line_start: usize,
        line_end: usize,
    },
    MissingResolvedLevel {
        glyph_index: usize,
        offset: usize,
    },
    MissingResolvedRangeLevel {
        offset: usize,
    },
    MissingSignatureScalar {
        offset: usize,
    },
    NonMonotonicGlyphRange {
        glyph_index: usize,
        start: usize,
        previous_start: usize,
    },
    ProjectionCardinalityMismatch {
        cluster_count: usize,
        visual_index_count: usize,
        level_count: usize,
    },
    AdvanceCardinalityMismatch {
        cluster_count: usize,
        advance_count: usize,
    },
    MissingLogicalCluster {
        logical_index: usize,
        cluster_count: usize,
    },
}

/// Paragraph-owned UAX#9 analysis. Levels remain in logical/source order; line
/// consumers apply L1/L2 only after a wrapping boundary is known.
pub(crate) struct BidiParagraph<'text> {
    text: &'text str,
    info: BidiInfo<'text>,
    resolved_base_direction: TextDirection,
    unicode_data_snapshot: UnicodeDataSnapshotId,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub(crate) struct BidiLineOrder {
    pub(crate) resolved_base_direction: TextDirection,
    pub(crate) logical_levels: Vec<u8>,
    pub(crate) visual_indices: Vec<usize>,
    pub(crate) unicode_data_snapshot: UnicodeDataSnapshotId,
}

/// A source-free UAX#9 paragraph projection that can reproduce L1/L2 after wrapping.
///
/// This retains only directional metadata. Consumers can discard the original string and still
/// resolve each physical line without reinterpreting a display substitution such as a password
/// mask. Ranges are byte offsets in the source that was used to create the signature.
#[derive(Clone, Debug, PartialEq, Eq)]
pub(crate) struct BidiLineSignature {
    resolved_base_direction: TextDirection,
    paragraph_level: u8,
    source_range: TextRange,
    scalars: Vec<BidiSignatureScalar>,
    unicode_data_snapshot: UnicodeDataSnapshotId,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
struct BidiSignatureScalar {
    source_range: TextRange,
    resolved_level: u8,
    l1_class: BidiL1Class,
}

/// The L1-relevant projection of a Unicode bidi class. This is intentionally not the source
/// character or its full class: reordering needs only these categories once embedding levels have
/// already been resolved by `BidiInfo`.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum BidiL1Class {
    SegmentSeparator,
    WhitespaceOrIsolate,
    ExplicitFormatting,
    Other,
}

impl BidiLineSignature {
    pub(crate) const fn unicode_data_snapshot(&self) -> UnicodeDataSnapshotId {
        self.unicode_data_snapshot
    }

    /// Resolves one physical line from this paragraph-local directional signature.
    ///
    /// `line` and `logical_glyph_ranges` must remain at scalar boundaries from the source used to
    /// construct this signature. The result preserves `visual_index -> logical_glyph_index`.
    pub(crate) fn line_order(
        &self,
        line: Range<usize>,
        logical_glyph_ranges: &[TextRange],
    ) -> Result<BidiLineOrder, BidiInvariantError> {
        if logical_glyph_ranges.is_empty()
            || line.start >= line.end
            || line.start < self.source_range.start
            || line.end > self.source_range.end
        {
            return Err(BidiInvariantError::InvalidLineRange {
                start: line.start,
                end: line.end,
            });
        }

        let first_scalar = self
            .scalars
            .iter()
            .position(|scalar| scalar.source_range.start == line.start)
            .ok_or(BidiInvariantError::MissingSignatureScalar { offset: line.start })?;
        let after_last_scalar = self
            .scalars
            .iter()
            .position(|scalar| scalar.source_range.end == line.end)
            .map(|index| index.saturating_add(1))
            .ok_or(BidiInvariantError::MissingSignatureScalar { offset: line.end })?;
        let line_scalars = self.scalars.get(first_scalar..after_last_scalar).ok_or(
            BidiInvariantError::InvalidLineRange {
                start: line.start,
                end: line.end,
            },
        )?;

        let mut levels = line_scalars
            .iter()
            .map(|scalar| scalar.resolved_level)
            .collect::<Vec<_>>();
        apply_l1_to_signature_levels(line_scalars, &mut levels, self.paragraph_level);

        let mut glyph_levels = Vec::with_capacity(logical_glyph_ranges.len());
        let mut scalar_index = 0;
        let mut previous_start = line.start;
        for (glyph_index, range) in logical_glyph_ranges.iter().enumerate() {
            if range.start < line.start || range.end > line.end || range.start >= range.end {
                return Err(BidiInvariantError::GlyphOutsideLine {
                    glyph_index,
                    start: range.start,
                    end: range.end,
                    line_start: line.start,
                    line_end: line.end,
                });
            }
            if range.start < previous_start {
                return Err(BidiInvariantError::NonMonotonicGlyphRange {
                    glyph_index,
                    start: range.start,
                    previous_start,
                });
            }
            while line_scalars
                .get(scalar_index)
                .is_some_and(|scalar| scalar.source_range.start < range.start)
            {
                scalar_index = scalar_index.saturating_add(1);
            }
            let glyph_scalar_index = scalar_index;
            let Some(scalar) = line_scalars.get(glyph_scalar_index) else {
                return Err(BidiInvariantError::MissingSignatureScalar {
                    offset: range.start,
                });
            };
            if scalar.source_range.start != range.start {
                return Err(BidiInvariantError::MissingSignatureScalar {
                    offset: range.start,
                });
            }
            while line_scalars
                .get(scalar_index)
                .is_some_and(|scalar| scalar.source_range.end < range.end)
            {
                scalar_index = scalar_index.saturating_add(1);
            }
            let Some(last_scalar) = line_scalars.get(scalar_index) else {
                return Err(BidiInvariantError::MissingSignatureScalar { offset: range.end });
            };
            if last_scalar.source_range.end != range.end {
                return Err(BidiInvariantError::MissingSignatureScalar { offset: range.end });
            }
            let Some(level) = levels.get(glyph_scalar_index).copied() else {
                return Err(BidiInvariantError::MissingResolvedLevel {
                    glyph_index,
                    offset: range.start,
                });
            };
            glyph_levels.push(level);
            previous_start = range.start;
        }

        Ok(BidiLineOrder {
            resolved_base_direction: self.resolved_base_direction,
            visual_indices: reorder_visual_signature_levels(&glyph_levels),
            logical_levels: glyph_levels,
            unicode_data_snapshot: self.unicode_data_snapshot,
        })
    }
}

impl<'text> BidiParagraph<'text> {
    pub(crate) fn new(text: &'text str, requested_direction: TextDirection) -> Self {
        Self::for_snapshot(
            text,
            requested_direction,
            compiled_unicode_data_snapshot_id(),
        )
    }

    pub(crate) fn for_snapshot(
        text: &'text str,
        requested_direction: TextDirection,
        unicode_data_snapshot: UnicodeDataSnapshotId,
    ) -> Self {
        #[cfg(any(test, feature = "profiling", feature = "profiling-tracy"))]
        let profile_started = super::analysis_profile::start_build();
        let requested_level = requested_base_level(requested_direction);
        let info = BidiInfo::new(text, requested_level);
        let resolved_base_direction = info
            .paragraphs
            .first()
            .map(|paragraph| direction_for_level(paragraph.level.number()))
            .unwrap_or_else(|| {
                requested_level
                    .map(|level| direction_for_level(level.number()))
                    .unwrap_or(TextDirection::LeftToRight)
            });
        let paragraph = Self {
            text,
            info,
            resolved_base_direction,
            unicode_data_snapshot,
        };
        #[cfg(any(test, feature = "profiling", feature = "profiling-tracy"))]
        super::analysis_profile::record_bidi_build(text.len(), profile_started);
        paragraph
    }

    pub(crate) const fn unicode_data_snapshot(&self) -> UnicodeDataSnapshotId {
        self.unicode_data_snapshot
    }

    pub(crate) const fn resolved_base_direction(&self) -> TextDirection {
        self.resolved_base_direction
    }

    /// Resolves the UAX#9 embedding level for a source cluster without reinterpreting invalid
    /// offsets as the paragraph base direction.
    pub(crate) fn level_for_range(&self, range: TextRange) -> Result<u8, BidiInvariantError> {
        if range.start >= range.end
            || range.end > self.text.len()
            || !self.text.is_char_boundary(range.start)
            || !self.text.is_char_boundary(range.end)
        {
            return Err(BidiInvariantError::InvalidResolvedRange {
                start: range.start,
                end: range.end,
            });
        }
        self.info.levels.get(range.start).map(Level::number).ok_or(
            BidiInvariantError::MissingResolvedRangeLevel {
                offset: range.start,
            },
        )
    }

    /// Returns visual-index -> logical-glyph-index for a single laid-out line.
    /// The input ranges are paragraph-local UTF-8 byte ranges in logical order.
    pub(crate) fn visual_order_for_line(
        &self,
        line: Range<usize>,
        logical_glyph_ranges: &[TextRange],
    ) -> Result<Vec<usize>, BidiInvariantError> {
        Ok(self.line_order(line, logical_glyph_ranges)?.visual_indices)
    }

    pub(crate) fn line_order(
        &self,
        line: Range<usize>,
        logical_glyph_ranges: &[TextRange],
    ) -> Result<BidiLineOrder, BidiInvariantError> {
        if logical_glyph_ranges.is_empty()
            || line.start >= line.end
            || line.end > self.text.len()
            || !self.text.is_char_boundary(line.start)
            || !self.text.is_char_boundary(line.end)
        {
            return Err(BidiInvariantError::InvalidLineRange {
                start: line.start,
                end: line.end,
            });
        }
        let Some(paragraph) = self.info.paragraphs.iter().find(|paragraph| {
            line.start >= paragraph.range.start && line.end <= paragraph.range.end
        }) else {
            return Err(BidiInvariantError::LineOutsideParagraph {
                start: line.start,
                end: line.end,
            });
        };
        let reordered_levels = self.info.reordered_levels(paragraph, line.clone());
        let mut glyph_levels = Vec::with_capacity(logical_glyph_ranges.len());
        for (glyph_index, range) in logical_glyph_ranges.iter().enumerate() {
            if range.start < line.start || range.end > line.end || range.start > range.end {
                return Err(BidiInvariantError::GlyphOutsideLine {
                    glyph_index,
                    start: range.start,
                    end: range.end,
                    line_start: line.start,
                    line_end: line.end,
                });
            }
            let offset = level_index_for_range(*range, &line);
            let Some(level) = reordered_levels.get(offset) else {
                return Err(BidiInvariantError::MissingResolvedLevel {
                    glyph_index,
                    offset,
                });
            };
            glyph_levels.push(*level);
        }
        let visual_indices = BidiInfo::reorder_visual(&glyph_levels);
        let logical_levels = glyph_levels
            .into_iter()
            .map(|level| level.number())
            .collect();
        Ok(BidiLineOrder {
            resolved_base_direction: self.resolved_base_direction,
            logical_levels,
            visual_indices,
            unicode_data_snapshot: self.unicode_data_snapshot,
        })
    }

    /// Captures the paragraph's resolved UAX#9 state for later wrapped-line ordering without
    /// retaining the source text itself.
    pub(crate) fn line_signature(
        &self,
        line: Range<usize>,
    ) -> Result<BidiLineSignature, BidiInvariantError> {
        if line.start >= line.end
            || line.end > self.text.len()
            || !self.text.is_char_boundary(line.start)
            || !self.text.is_char_boundary(line.end)
        {
            return Err(BidiInvariantError::InvalidLineRange {
                start: line.start,
                end: line.end,
            });
        }
        let Some(paragraph) = self.info.paragraphs.iter().find(|paragraph| {
            line.start >= paragraph.range.start && line.end <= paragraph.range.end
        }) else {
            return Err(BidiInvariantError::LineOutsideParagraph {
                start: line.start,
                end: line.end,
            });
        };

        let mut scalars = Vec::new();
        for (relative_start, character) in self.text[line.clone()].char_indices() {
            let start = line.start + relative_start;
            let end = start + character.len_utf8();
            let Some(resolved_level) = self.info.levels.get(start).map(Level::number) else {
                return Err(BidiInvariantError::MissingResolvedRangeLevel { offset: start });
            };
            let Some(class) = self.info.original_classes.get(start).copied() else {
                return Err(BidiInvariantError::MissingResolvedRangeLevel { offset: start });
            };
            scalars.push(BidiSignatureScalar {
                source_range: TextRange { start, end },
                resolved_level,
                l1_class: l1_class(class),
            });
        }
        Ok(BidiLineSignature {
            resolved_base_direction: self.resolved_base_direction,
            paragraph_level: paragraph.level.number(),
            source_range: TextRange {
                start: line.start,
                end: line.end,
            },
            scalars,
            unicode_data_snapshot: self.unicode_data_snapshot,
        })
    }
}

fn l1_class(class: BidiClass) -> BidiL1Class {
    use BidiClass::*;

    match class {
        B | S => BidiL1Class::SegmentSeparator,
        WS | FSI | LRI | RLI | PDI => BidiL1Class::WhitespaceOrIsolate,
        RLE | LRE | RLO | LRO | PDF | BN => BidiL1Class::ExplicitFormatting,
        _ => BidiL1Class::Other,
    }
}

/// Mirrors `unicode_bidi::BidiInfo::reordered_levels` at scalar granularity. The signature is
/// captured only at Unicode scalar boundaries, so applying L1 to scalar levels preserves the
/// level used by this engine's grapheme-level visual projection.
fn apply_l1_to_signature_levels(
    scalars: &[BidiSignatureScalar],
    levels: &mut [u8],
    paragraph_level: u8,
) {
    debug_assert_eq!(scalars.len(), levels.len());
    let mut reset_from = Some(0usize);
    let mut reset_to = None;
    let mut previous_level = paragraph_level;
    for (index, scalar) in scalars.iter().enumerate() {
        match scalar.l1_class {
            BidiL1Class::SegmentSeparator => {
                reset_to = Some(index.saturating_add(1));
                if reset_from.is_none() {
                    reset_from = Some(index);
                }
            }
            BidiL1Class::WhitespaceOrIsolate => {
                if reset_from.is_none() {
                    reset_from = Some(index);
                }
            }
            BidiL1Class::ExplicitFormatting => {
                if reset_from.is_none() {
                    reset_from = Some(index);
                }
                levels[index] = previous_level;
            }
            BidiL1Class::Other => reset_from = None,
        }
        if let (Some(start), Some(end)) = (reset_from, reset_to) {
            for level in &mut levels[start..end] {
                *level = paragraph_level;
            }
            reset_from = None;
            reset_to = None;
        }
        previous_level = levels[index];
    }
    if let Some(start) = reset_from {
        for level in &mut levels[start..] {
            *level = paragraph_level;
        }
    }
}

/// Applies UAX#9 L2 to one level per logical grapheme. `unicode_bidi::reorder_visual` exposes
/// the same result for `Level`, but the presentation signature deliberately stores compact `u8`
/// metadata after source disposal.
fn reorder_visual_signature_levels(levels: &[u8]) -> Vec<usize> {
    let mut visual_indices = (0..levels.len()).collect::<Vec<_>>();
    let Some((&first, remaining)) = levels.split_first() else {
        return visual_indices;
    };
    let (minimum, maximum) = remaining
        .iter()
        .copied()
        .fold((first, first), |range, level| {
            (range.0.min(level), range.1.max(level))
        });
    if minimum == maximum && minimum % 2 == 0 {
        return visual_indices;
    }
    let lowest_odd_level = if minimum % 2 == 0 {
        minimum.saturating_add(1)
    } else {
        minimum
    };
    if lowest_odd_level > maximum {
        return visual_indices;
    }

    for level in (lowest_odd_level..=maximum).rev() {
        let mut index = 0;
        while index < levels.len() {
            while index < levels.len() && levels[index] < level {
                index += 1;
            }
            let start = index;
            while index < levels.len() && levels[index] >= level {
                index += 1;
            }
            visual_indices[start..index].reverse();
        }
    }
    visual_indices
}

pub(crate) fn analyze_bidi_line(
    paragraph_text: &str,
    requested_direction: TextDirection,
    line_range: TextRange,
    logical_ranges: &[TextRange],
) -> Result<BidiLineOrder, BidiInvariantError> {
    BidiParagraph::new(paragraph_text, requested_direction)
        .line_order(line_range.start..line_range.end, logical_ranges)
}

/// Captures source-free directional metadata for a non-empty paragraph-local line.
pub(crate) fn capture_bidi_line_signature(
    paragraph_text: &str,
    requested_direction: TextDirection,
    line_range: TextRange,
) -> Result<BidiLineSignature, BidiInvariantError> {
    BidiParagraph::new(paragraph_text, requested_direction)
        .line_signature(line_range.start..line_range.end)
}

pub(crate) fn resolve_bidi_base_direction(
    text: &str,
    requested_direction: TextDirection,
) -> TextDirection {
    BidiParagraph::new(text, requested_direction).resolved_base_direction()
}

pub(crate) fn mirrored_bidi_char(ch: char, bidi_level: u8) -> Option<char> {
    (bidi_level % 2 == 1).then(|| get_mirrored(ch)).flatten()
}

#[cfg(test)]
mod tests {
    use crate::core::framework::text::TextDirection;
    use crate::text::{TextRange, compiled_unicode_data_snapshot_id};
    use unicode_segmentation::UnicodeSegmentation;

    use super::{BidiInvariantError, BidiParagraph, mirrored_bidi_char};

    #[test]
    fn mirrors_unicode_bidi_pairs_only_on_odd_levels() {
        assert_eq!(mirrored_bidi_char('\u{27E8}', 1), Some('\u{27E9}'));
        assert_eq!(mirrored_bidi_char('\u{29C4}', 1), Some('\u{29C5}'));
        assert_eq!(mirrored_bidi_char('\u{27E8}', 2), None);
    }

    #[test]
    fn bidi_line_order_rejects_glyph_ranges_outside_the_requested_line() {
        let bidi = BidiParagraph::new("abc", TextDirection::LeftToRight);

        assert_eq!(
            bidi.line_order(
                0..2,
                &[
                    TextRange { start: 0, end: 1 },
                    TextRange { start: 2, end: 3 }
                ],
            ),
            Err(BidiInvariantError::GlyphOutsideLine {
                glyph_index: 1,
                start: 2,
                end: 3,
                line_start: 0,
                line_end: 2,
            })
        );
    }

    #[test]
    fn bidi_line_order_rejects_a_line_outside_the_source_paragraph() {
        let bidi = BidiParagraph::new("abc", TextDirection::LeftToRight);

        assert_eq!(
            bidi.line_order(0..4, &[TextRange { start: 0, end: 1 }]),
            Err(BidiInvariantError::InvalidLineRange { start: 0, end: 4 })
        );
    }

    #[test]
    fn source_free_signature_replays_l1_for_a_wrapped_rtl_line() {
        let source = "\u{05D0}\u{05D1} abc ";
        let logical_ranges = source
            .grapheme_indices(true)
            .map(|(start, grapheme)| TextRange {
                start,
                end: start + grapheme.len(),
            })
            .collect::<Vec<_>>();
        let wrapped_end = logical_ranges[2].end;
        let bidi = BidiParagraph::new(source, TextDirection::Auto);
        let signature = bidi.line_signature(0..source.len()).unwrap();

        let expected = bidi
            .line_order(0..wrapped_end, &logical_ranges[..3])
            .unwrap();
        let actual = signature
            .line_order(0..wrapped_end, &logical_ranges[..3])
            .unwrap();

        // The third cluster is a trailing space in this physical line. This is the case that
        // cannot be recovered by slicing an already-reordered hard-line result.
        assert_eq!(actual, expected);
        assert_eq!(actual.resolved_base_direction, TextDirection::RightToLeft);
    }

    #[test]
    fn source_free_signature_rejects_out_of_order_logical_glyph_ranges() {
        let bidi = BidiParagraph::new("abc", TextDirection::LeftToRight);
        let signature = bidi.line_signature(0..3).unwrap();

        assert_eq!(
            signature.line_order(
                0..3,
                &[
                    TextRange { start: 1, end: 2 },
                    TextRange { start: 0, end: 1 },
                ],
            ),
            Err(BidiInvariantError::NonMonotonicGlyphRange {
                glyph_index: 1,
                start: 0,
                previous_start: 1,
            })
        );
    }

    #[test]
    fn source_free_signature_rejects_a_glyph_range_ending_inside_a_scalar() {
        let bidi = BidiParagraph::new("a\u{4E2D}", TextDirection::LeftToRight);
        let signature = bidi.line_signature(0..4).unwrap();

        assert_eq!(
            signature.line_order(0..4, &[TextRange { start: 0, end: 2 }]),
            Err(BidiInvariantError::MissingSignatureScalar { offset: 2 })
        );
    }

    #[test]
    fn bidi_artifacts_retain_request_unicode_snapshot_identity() {
        let current = compiled_unicode_data_snapshot_id();
        let next = current.with_generation_for_test(current.generation() + 1);
        let paragraph = BidiParagraph::for_snapshot("abc", TextDirection::LeftToRight, next);
        let signature = paragraph
            .line_signature(0..3)
            .expect("valid paragraph signature");
        let order = signature
            .line_order(
                0..3,
                &[
                    TextRange { start: 0, end: 1 },
                    TextRange { start: 1, end: 3 },
                ],
            )
            .expect("valid line order");

        assert_eq!(paragraph.unicode_data_snapshot(), next);
        assert_eq!(signature.unicode_data_snapshot(), next);
        assert_eq!(order.unicode_data_snapshot, next);
    }
}

fn requested_base_level(direction: TextDirection) -> Option<Level> {
    match direction {
        TextDirection::LeftToRight => Some(Level::ltr()),
        TextDirection::RightToLeft => Some(Level::rtl()),
        TextDirection::Auto | TextDirection::Mixed => None,
    }
}

fn direction_for_level(level: u8) -> TextDirection {
    if level % 2 == 1 {
        TextDirection::RightToLeft
    } else {
        TextDirection::LeftToRight
    }
}

fn level_index_for_range(range: TextRange, line: &Range<usize>) -> usize {
    if range.start < line.end {
        range.start
    } else {
        range.start.saturating_sub(1).max(line.start)
    }
}
