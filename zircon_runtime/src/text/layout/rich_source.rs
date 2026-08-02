use crate::text::{InlineObjectRef, RichParseResult, StyleOverride};

#[derive(Clone, Copy, Debug)]
pub(crate) struct RichTextLayoutRun<'a> {
    pub(crate) source_index: u32,
    pub(crate) byte_range: (u32, u32),
    pub(crate) style: &'a StyleOverride,
    pub(crate) inline: Option<&'a InlineObjectRef>,
}

/// Borrowed input contract for rich layout. Implementations may expose a full
/// parse result or a projected range without copying text or run metadata.
///
/// `run(index)` must enumerate local byte ranges in strictly increasing,
/// non-overlapping order, and every range must remain within `text()`. The
/// `source_index` remains the stable index in the parent compiled artifact.
/// Advance indexing relies on these invariants for its monotonic cursor.
pub(crate) trait RichTextLayoutSource {
    fn text(&self) -> &str;
    fn run_count(&self) -> usize;
    fn run(&self, index: usize) -> Option<RichTextLayoutRun<'_>>;
}

impl RichTextLayoutSource for RichParseResult {
    fn text(&self) -> &str {
        &self.text
    }

    fn run_count(&self) -> usize {
        self.runs.len()
    }

    fn run(&self, index: usize) -> Option<RichTextLayoutRun<'_>> {
        let run = self.runs.get(index)?;
        Some(RichTextLayoutRun {
            source_index: u32::try_from(index).unwrap_or(u32::MAX),
            byte_range: run.byte_range,
            style: &run.style,
            inline: run.inline.as_ref(),
        })
    }
}
