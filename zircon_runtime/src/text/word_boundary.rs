use unicode_segmentation::UnicodeSegmentation;

use super::{TextRange, UnicodeDataSnapshotId, compiled_unicode_data_snapshot_id};

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
    let mut offset = offset.min(text.len());
    while offset > 0 && !text.is_char_boundary(offset) {
        offset -= 1;
    }
    offset
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn map_retains_the_unicode_snapshot_identity() {
        let current = compiled_unicode_data_snapshot_id();
        let next = current.with_generation_for_test(current.generation() + 1);

        assert_eq!(
            WordBoundaryMap::for_snapshot("text", next).unicode_data_snapshot(),
            next
        );
    }

    #[test]
    fn completed_prefix_uses_unicode_words_instead_of_whitespace_tokens() {
        let hyphenated = WordBoundaryMap::new("alpha-beta");
        assert_eq!(hyphenated.completed_prefix_end(8), 5);

        let apostrophe = WordBoundaryMap::new("go can't");
        assert_eq!(apostrophe.completed_prefix_end(6), 2);
        assert_eq!(apostrophe.completed_prefix_end(8), 8);
    }

    #[test]
    fn completed_prefix_supports_text_without_spaces() {
        let text = "中文文本";
        let third_ideograph_end = "中文文".len();

        assert_eq!(
            WordBoundaryMap::new(text).completed_prefix_end(third_ideograph_end),
            third_ideograph_end
        );
    }

    #[test]
    fn navigation_queries_share_the_same_word_ranges() {
        let map = WordBoundaryMap::new("alpha-beta");

        assert_eq!(map.previous_word_start(8), Some(6));
        assert_eq!(map.previous_word_start(6), Some(0));
        assert_eq!(map.next_word_end(5), Some(10));
        assert_eq!(map.word_range_at(7), Some(TextRange { start: 6, end: 10 }));
    }
}
