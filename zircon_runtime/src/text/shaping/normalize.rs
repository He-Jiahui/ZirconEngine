use std::ops::Range;

pub(super) struct ShapingTextView<'a> {
    original: &'a str,
}

impl<'a> ShapingTextView<'a> {
    /// V1 keeps the exact source UTF-8 for both shaping and all published offsets.
    ///
    /// NFC/NFD must not be introduced here until a versioned bidirectional source map is
    /// available to selection, IME, accessibility, cache, and glyph projection consumers.
    pub(super) const fn source_preserving(original: &'a str) -> Self {
        Self { original }
    }

    pub(super) const fn shaping_text(&self) -> &'a str {
        self.original
    }

    pub(super) fn source_range_for_shaping_range(&self, range: Range<usize>) -> Range<usize> {
        let start = range.start.min(self.original.len());
        let end = range.end.clamp(start, self.original.len());
        start..end
    }
}

#[cfg(test)]
mod tests {
    use super::ShapingTextView;

    #[test]
    fn source_preserving_view_keeps_decomposed_source_bytes_unchanged() {
        let source = "a\u{0304}\u{0301}b";
        let view = ShapingTextView::source_preserving(source);

        assert_eq!(view.shaping_text(), source);
        assert_eq!(view.shaping_text().len(), source.len());
    }

    #[test]
    fn source_preserving_view_maps_shaping_offsets_to_original_source_offsets() {
        let source = "a\u{0304}\u{0301}b";
        let view = ShapingTextView::source_preserving(source);

        assert_eq!(view.source_range_for_shaping_range(1..5), 1..5);
        assert_eq!(
            view.source_range_for_shaping_range(source.len()..source.len()),
            source.len()..source.len()
        );
    }

    #[test]
    fn source_preserving_view_keeps_canonical_equivalents_as_distinct_source_bytes() {
        let composed = "\u{00E9}";
        let decomposed = "e\u{0301}";
        let composed_view = ShapingTextView::source_preserving(composed);
        let decomposed_view = ShapingTextView::source_preserving(decomposed);

        assert_ne!(composed_view.shaping_text(), decomposed_view.shaping_text());
        assert_eq!(
            composed_view.source_range_for_shaping_range(0..composed.len()),
            0..composed.len()
        );
        assert_eq!(
            decomposed_view.source_range_for_shaping_range(0..decomposed.len()),
            0..decomposed.len()
        );
    }
}
