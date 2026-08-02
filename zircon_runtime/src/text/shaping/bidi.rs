use std::ops::Range;

use crate::core::framework::text::TextDirection;
use crate::text::TextRange;
use unicode_bidi::{BidiInfo, Level};
use unicode_bidi_mirroring::get_mirrored;

/// Paragraph-owned UAX#9 analysis. Levels remain in logical/source order; line
/// consumers apply L1/L2 only after a wrapping boundary is known.
pub(crate) struct BidiParagraph<'text> {
    text: &'text str,
    info: BidiInfo<'text>,
    resolved_base_direction: TextDirection,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub(crate) struct BidiLineOrder {
    pub(crate) resolved_base_direction: TextDirection,
    pub(crate) logical_levels: Vec<u8>,
    pub(crate) visual_indices: Vec<usize>,
}

impl<'text> BidiParagraph<'text> {
    pub(crate) fn new(text: &'text str, requested_direction: TextDirection) -> Self {
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
        Self {
            text,
            info,
            resolved_base_direction,
        }
    }

    pub(crate) const fn resolved_base_direction(&self) -> TextDirection {
        self.resolved_base_direction
    }

    pub(crate) fn level_for_range(&self, range: TextRange) -> u8 {
        let fallback = match self.resolved_base_direction {
            TextDirection::RightToLeft => 1,
            _ => 0,
        };
        if self.text.is_empty() {
            return fallback;
        }

        let byte_index = range.start.min(self.text.len().saturating_sub(1));
        self.info
            .levels
            .get(byte_index)
            .map(Level::number)
            .unwrap_or(fallback)
    }

    /// Returns visual-index -> logical-glyph-index for a single laid-out line.
    /// The input ranges are paragraph-local UTF-8 byte ranges in logical order.
    pub(crate) fn visual_order_for_line(
        &self,
        line: Range<usize>,
        logical_glyph_ranges: &[TextRange],
    ) -> Vec<usize> {
        self.line_order(line, logical_glyph_ranges).visual_indices
    }

    pub(crate) fn line_order(
        &self,
        line: Range<usize>,
        logical_glyph_ranges: &[TextRange],
    ) -> BidiLineOrder {
        let fallback = match self.resolved_base_direction {
            TextDirection::RightToLeft => 1,
            _ => 0,
        };
        let fallback_order = || BidiLineOrder {
            resolved_base_direction: self.resolved_base_direction,
            logical_levels: vec![fallback; logical_glyph_ranges.len()],
            visual_indices: (0..logical_glyph_ranges.len()).collect(),
        };
        if logical_glyph_ranges.is_empty()
            || line.start >= line.end
            || line.end > self.text.len()
            || !self.text.is_char_boundary(line.start)
            || !self.text.is_char_boundary(line.end)
        {
            return fallback_order();
        }
        let Some(paragraph) = self.info.paragraphs.iter().find(|paragraph| {
            line.start >= paragraph.range.start && line.end <= paragraph.range.end
        }) else {
            return fallback_order();
        };
        let reordered_levels = self.info.reordered_levels(paragraph, line.clone());
        let mut glyph_levels = Vec::with_capacity(logical_glyph_ranges.len());
        for range in logical_glyph_ranges {
            if range.start < line.start || range.end > line.end || range.start > range.end {
                return fallback_order();
            }
            let Some(level) = reordered_levels.get(level_index_for_range(*range, &line)) else {
                return fallback_order();
            };
            glyph_levels.push(*level);
        }
        let visual_indices = BidiInfo::reorder_visual(&glyph_levels);
        let logical_levels = glyph_levels
            .into_iter()
            .map(|level| level.number())
            .collect();
        BidiLineOrder {
            resolved_base_direction: self.resolved_base_direction,
            logical_levels,
            visual_indices,
        }
    }
}

pub(crate) fn analyze_bidi_line(
    paragraph_text: &str,
    requested_direction: TextDirection,
    line_range: TextRange,
    logical_ranges: &[TextRange],
) -> BidiLineOrder {
    BidiParagraph::new(paragraph_text, requested_direction)
        .line_order(line_range.start..line_range.end, logical_ranges)
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
    use super::mirrored_bidi_char;

    #[test]
    fn mirrors_unicode_bidi_pairs_only_on_odd_levels() {
        assert_eq!(mirrored_bidi_char('\u{27E8}', 1), Some('\u{27E9}'));
        assert_eq!(mirrored_bidi_char('\u{29C4}', 1), Some('\u{29C5}'));
        assert_eq!(mirrored_bidi_char('\u{27E8}', 2), None);
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
