use std::ops::Range;

pub(super) struct ShapingTextView<'a> {
    original: &'a str,
}

impl<'a> ShapingTextView<'a> {
    pub(super) const fn v1_disabled(original: &'a str) -> Self {
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
    fn text_normalization_v1_keeps_decomposed_source_bytes_unchanged() {
        let source = "a\u{0304}\u{0301}b";
        let view = ShapingTextView::v1_disabled(source);

        assert_eq!(view.shaping_text(), source);
        assert_eq!(view.shaping_text().len(), source.len());
    }

    #[test]
    fn text_normalization_v1_maps_shaping_offsets_to_original_source_offsets() {
        let source = "a\u{0304}\u{0301}b";
        let view = ShapingTextView::v1_disabled(source);

        assert_eq!(view.source_range_for_shaping_range(1..5), 1..5);
        assert_eq!(
            view.source_range_for_shaping_range(source.len()..source.len()),
            source.len()..source.len()
        );
    }
}
