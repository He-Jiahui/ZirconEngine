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
#[path = "tests/normalize.rs"]
mod tests;
