use unicode_segmentation::UnicodeSegmentation;

use super::{compiled_unicode_data_snapshot_id, TextRange, UnicodeDataSnapshotId};

/// Zero-copy UAX #29 word-boundary view tied to the Unicode data identity that produced it.
///
/// The view deliberately retains the source string instead of materializing every boundary. UI
/// navigation and one-shot overflow queries therefore share one policy owner without adding a
/// second allocation proportional to paragraph length. A retained paragraph analysis can cache a
/// materialized form later without changing the query semantics defined here.
#[derive(Clone, Copy, Debug)]
pub(crate) struct WordBoundaryMap<'text> {
    text: &'text str,
    unicode_data_snapshot: UnicodeDataSnapshotId,
}

impl<'text> WordBoundaryMap<'text> {
    pub(crate) fn new(text: &'text str) -> Self {
        Self::for_snapshot(text, compiled_unicode_data_snapshot_id())
    }

    pub(crate) const fn for_snapshot(
        text: &'text str,
        unicode_data_snapshot: UnicodeDataSnapshotId,
    ) -> Self {
        Self {
            text,
            unicode_data_snapshot,
        }
    }

    pub(crate) const fn unicode_data_snapshot(self) -> UnicodeDataSnapshotId {
        self.unicode_data_snapshot
    }

    pub(crate) fn previous_word_start(self, offset: usize) -> Option<usize> {
        let offset = floor_utf8_boundary(self.text, offset);
        self.ranges()
            .rev()
            .find(|range| range.start < offset)
            .map(|range| range.start)
    }

    pub(crate) fn next_word_end(self, offset: usize) -> Option<usize> {
        let offset = floor_utf8_boundary(self.text, offset);
        self.ranges()
            .find(|range| range.end > offset)
            .map(|range| range.end)
    }

    pub(crate) fn word_range_at(self, offset: usize) -> Option<TextRange> {
        let offset = floor_utf8_boundary(self.text, offset);
        self.ranges()
            .take_while(|range| range.start <= offset)
            .find(|range| offset <= range.end)
    }

    /// Returns the end of the last complete Unicode word within a fitted source prefix.
    ///
    /// Separator and punctuation segments are not retained past the completed word. This keeps an
    /// EndWord marker from publishing dangling whitespace or punctuation as if it were a word.
    pub(crate) fn completed_prefix_end(self, fitted_end: usize) -> usize {
        let fitted_end = floor_utf8_boundary(self.text, fitted_end);
        self.ranges()
            .take_while(|range| range.end <= fitted_end)
            .last()
            .map(|range| range.end)
            .unwrap_or_default()
    }

    pub(crate) fn ranges(self) -> impl DoubleEndedIterator<Item = TextRange> + 'text {
        self.text
            .unicode_word_indices()
            .map(|(start, word)| TextRange {
                start,
                end: start + word.len(),
            })
    }
}

fn floor_utf8_boundary(text: &str, offset: usize) -> usize {
    // 导航与完整词前缀查询共用向下取整，保证非边界字节偏移也按同一 UTF-8 规则解释。
    let mut offset = offset.min(text.len());
    while offset > 0 && !text.is_char_boundary(offset) {
        offset -= 1;
    }
    offset
}

#[cfg(test)]
#[path = "tests/word_boundary.rs"]
mod tests;
