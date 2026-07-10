use std::ops::Range;

use unicode_bidi::{BidiInfo, Level};
use zircon_runtime_interface::ui::surface::{UiTextDirection, UiTextRange};

/// Paragraph-owned UAX#9 analysis. Levels remain in logical/source order; line
/// consumers apply L1/L2 only after a wrapping boundary is known.
pub(crate) struct BidiParagraph<'text> {
    text: &'text str,
    info: BidiInfo<'text>,
    resolved_base_direction: UiTextDirection,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub(crate) struct BidiLineOrder {
    pub(crate) resolved_base_direction: UiTextDirection,
    pub(crate) logical_levels: Vec<u8>,
    pub(crate) visual_indices: Vec<usize>,
}

impl<'text> BidiParagraph<'text> {
    pub(crate) fn new(text: &'text str, requested_direction: UiTextDirection) -> Self {
        let requested_level = requested_base_level(requested_direction);
        let info = BidiInfo::new(text, requested_level);
        let resolved_base_direction = info
            .paragraphs
            .first()
            .map(|paragraph| direction_for_level(paragraph.level.number()))
            .unwrap_or_else(|| {
                requested_level
                    .map(|level| direction_for_level(level.number()))
                    .unwrap_or(UiTextDirection::LeftToRight)
            });
        Self {
            text,
            info,
            resolved_base_direction,
        }
    }

    pub(crate) const fn resolved_base_direction(&self) -> UiTextDirection {
        self.resolved_base_direction
    }

    pub(crate) fn level_for_range(&self, range: UiTextRange) -> u8 {
        let fallback = match self.resolved_base_direction {
            UiTextDirection::RightToLeft => 1,
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
        logical_glyph_ranges: &[UiTextRange],
    ) -> Vec<usize> {
        let identity = || (0..logical_glyph_ranges.len()).collect::<Vec<_>>();
        if logical_glyph_ranges.is_empty()
            || line.start >= line.end
            || line.end > self.text.len()
            || !self.text.is_char_boundary(line.start)
            || !self.text.is_char_boundary(line.end)
        {
            return identity();
        }

        let Some(paragraph) = self.info.paragraphs.iter().find(|paragraph| {
            line.start >= paragraph.range.start && line.end <= paragraph.range.end
        }) else {
            return identity();
        };
        let line_levels = self.info.reordered_levels(paragraph, line.clone());
        let mut glyph_levels = Vec::with_capacity(logical_glyph_ranges.len());
        for range in logical_glyph_ranges {
            if range.start < line.start || range.end > line.end || range.start > range.end {
                return identity();
            }
            let Some(level) = line_levels.get(level_index_for_range(*range, &line)) else {
                return identity();
            };
            glyph_levels.push(*level);
        }
        BidiInfo::reorder_visual(&glyph_levels)
    }

    pub(crate) fn line_order(
        &self,
        line: Range<usize>,
        logical_glyph_ranges: &[UiTextRange],
    ) -> BidiLineOrder {
        let visual_indices = self.visual_order_for_line(line.clone(), logical_glyph_ranges);
        let logical_levels = self.line_levels(line, logical_glyph_ranges);
        BidiLineOrder {
            resolved_base_direction: self.resolved_base_direction,
            logical_levels,
            visual_indices,
        }
    }

    fn line_levels(&self, line: Range<usize>, logical_glyph_ranges: &[UiTextRange]) -> Vec<u8> {
        let fallback = match self.resolved_base_direction {
            UiTextDirection::RightToLeft => 1,
            _ => 0,
        };
        let Some(paragraph) = self.info.paragraphs.iter().find(|paragraph| {
            line.start >= paragraph.range.start && line.end <= paragraph.range.end
        }) else {
            return vec![fallback; logical_glyph_ranges.len()];
        };
        let levels = self.info.reordered_levels(paragraph, line.clone());
        logical_glyph_ranges
            .iter()
            .map(|range| {
                levels
                    .get(level_index_for_range(*range, &line))
                    .map(Level::number)
                    .unwrap_or(fallback)
            })
            .collect()
    }
}

pub(crate) fn analyze_bidi_line(
    paragraph_text: &str,
    requested_direction: UiTextDirection,
    line_range: UiTextRange,
    logical_ranges: &[UiTextRange],
) -> BidiLineOrder {
    BidiParagraph::new(paragraph_text, requested_direction)
        .line_order(line_range.start..line_range.end, logical_ranges)
}

pub(crate) fn resolve_bidi_base_direction(
    text: &str,
    requested_direction: UiTextDirection,
) -> UiTextDirection {
    BidiParagraph::new(text, requested_direction).resolved_base_direction()
}

pub(crate) fn mirrored_bidi_char(ch: char, bidi_level: u8) -> Option<char> {
    if bidi_level % 2 == 0 {
        return None;
    }
    Some(match ch {
        '(' => ')',
        ')' => '(',
        '[' => ']',
        ']' => '[',
        '{' => '}',
        '}' => '{',
        '<' => '>',
        '>' => '<',
        '«' => '»',
        '»' => '«',
        '‹' => '›',
        '›' => '‹',
        '≤' => '≥',
        '≥' => '≤',
        '∈' => '∋',
        '∋' => '∈',
        '⊂' => '⊃',
        '⊃' => '⊂',
        '⊆' => '⊇',
        '⊇' => '⊆',
        '←' => '→',
        '→' => '←',
        _ => return None,
    })
}

fn requested_base_level(direction: UiTextDirection) -> Option<Level> {
    match direction {
        UiTextDirection::LeftToRight => Some(Level::ltr()),
        UiTextDirection::RightToLeft => Some(Level::rtl()),
        UiTextDirection::Auto | UiTextDirection::Mixed => None,
    }
}

fn direction_for_level(level: u8) -> UiTextDirection {
    if level % 2 == 1 {
        UiTextDirection::RightToLeft
    } else {
        UiTextDirection::LeftToRight
    }
}

fn level_index_for_range(range: UiTextRange, line: &Range<usize>) -> usize {
    if range.start < line.end {
        range.start
    } else {
        range.start.saturating_sub(1).max(line.start)
    }
}
