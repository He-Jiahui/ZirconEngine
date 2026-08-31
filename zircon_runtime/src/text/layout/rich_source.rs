use crate::core::framework::text::TextLayoutError;
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
/// `source_index` remains the stable index in the parent compiled artifact and
/// must be strictly increasing in the projected run order. Advance indexing
/// and materialization rely on these invariants for monotonic cursors and
/// semantic run identity.
pub(crate) trait RichTextLayoutSource {
    fn text(&self) -> &str;
    fn run_count(&self) -> usize;
    fn run(&self, index: usize) -> Option<RichTextLayoutRun<'_>>;
}

/// Validates the source contract before any index or materialization owner can
/// turn a malformed run into silently missing geometry. Gaps are allowed: the
/// rich index deliberately fills them with the base style.
pub(crate) fn validate_rich_text_layout_source<S>(source: &S) -> Result<(), TextLayoutError>
where
    S: RichTextLayoutSource + ?Sized,
{
    for_each_validated_rich_run(source, |_, _, _| Ok(()))
}

/// Iterates rich runs once after applying the complete source contract. Layout owners can build
/// their projection from the checked byte offsets without performing a second metadata pass.
pub(crate) fn for_each_validated_rich_run<'a, S, F>(
    source: &'a S,
    mut visit: F,
) -> Result<(), TextLayoutError>
where
    S: RichTextLayoutSource + ?Sized,
    F: FnMut(RichTextLayoutRun<'a>, usize, usize) -> Result<(), TextLayoutError>,
{
    let text_end = u32::try_from(source.text().len()).map_err(|_| TextLayoutError::LayoutFailed)?;
    let mut previous_end = 0_u32;
    let mut previous_source_index = None;
    for index in 0..source.run_count() {
        let run = source.run(index).ok_or(TextLayoutError::LayoutFailed)?;
        if run.source_index == u32::MAX
            || previous_source_index.is_some_and(|previous| run.source_index <= previous)
        {
            return Err(TextLayoutError::LayoutFailed);
        }
        if run.byte_range.0 < previous_end
            || run.byte_range.1 <= run.byte_range.0
            || run.byte_range.1 > text_end
        {
            return Err(TextLayoutError::LayoutFailed);
        }
        let start = usize::try_from(run.byte_range.0).map_err(|_| TextLayoutError::LayoutFailed)?;
        let end = usize::try_from(run.byte_range.1).map_err(|_| TextLayoutError::LayoutFailed)?;
        if source.text().get(start..end).is_none() {
            return Err(TextLayoutError::LayoutFailed);
        }
        visit(run, start, end)?;
        previous_end = run.byte_range.1;
        previous_source_index = Some(run.source_index);
    }
    Ok(())
}

/// Converts an absolute source range only after checking integer, ordering, bounds, and UTF-8
/// invariants. Range-producing layout owners share this helper instead of recovering with a
/// sentinel offset or clamping into an empty slice.
pub(crate) fn checked_source_range(
    text: &str,
    range: (u32, u32),
) -> Result<(usize, usize), TextLayoutError> {
    let start = usize::try_from(range.0).map_err(|_| TextLayoutError::LayoutFailed)?;
    let end = usize::try_from(range.1).map_err(|_| TextLayoutError::LayoutFailed)?;
    if start > end
        || end > text.len()
        || !text.is_char_boundary(start)
        || !text.is_char_boundary(end)
    {
        return Err(TextLayoutError::LayoutFailed);
    }
    Ok((start, end))
}

/// Validates a UTF-8 source range and publishes it in the compact layout representation.
pub(crate) fn checked_source_range_to_u32(
    text: &str,
    start: usize,
    end: usize,
) -> Result<(u32, u32), TextLayoutError> {
    if start > end
        || end > text.len()
        || !text.is_char_boundary(start)
        || !text.is_char_boundary(end)
    {
        return Err(TextLayoutError::LayoutFailed);
    }
    Ok((
        u32::try_from(start).map_err(|_| TextLayoutError::LayoutFailed)?,
        u32::try_from(end).map_err(|_| TextLayoutError::LayoutFailed)?,
    ))
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

#[cfg(test)]
mod tests {
    use super::{
        RichTextLayoutRun, RichTextLayoutSource, checked_source_range,
        validate_rich_text_layout_source,
    };
    use crate::core::framework::text::TextLayoutError;
    use crate::text::StyleOverride;

    struct Fixture {
        text: String,
        ranges: Vec<(u32, u32)>,
        source_indices: Vec<u32>,
        style: StyleOverride,
    }

    impl RichTextLayoutSource for Fixture {
        fn text(&self) -> &str {
            &self.text
        }

        fn run_count(&self) -> usize {
            self.ranges.len()
        }

        fn run(&self, index: usize) -> Option<RichTextLayoutRun<'_>> {
            let byte_range = *self.ranges.get(index)?;
            Some(RichTextLayoutRun {
                source_index: *self.source_indices.get(index)?,
                byte_range,
                style: &self.style,
                inline: None,
            })
        }
    }

    #[test]
    fn source_contract_accepts_empty_and_partially_covered_text() {
        let empty = Fixture {
            text: String::new(),
            ranges: Vec::new(),
            source_indices: Vec::new(),
            style: StyleOverride::default(),
        };
        assert_eq!(validate_rich_text_layout_source(&empty), Ok(()));

        let covered = Fixture {
            text: "abc".to_string(),
            ranges: vec![(0, 1), (1, 3)],
            source_indices: vec![0, 1],
            style: StyleOverride::default(),
        };
        assert_eq!(validate_rich_text_layout_source(&covered), Ok(()));

        let partial = Fixture {
            text: "abc".to_string(),
            ranges: vec![(1, 2)],
            source_indices: vec![0],
            style: StyleOverride::default(),
        };
        assert_eq!(validate_rich_text_layout_source(&partial), Ok(()));
    }

    #[test]
    fn source_contract_rejects_missing_or_invalid_ranges() {
        for (ranges, source_indices) in [
            (vec![(0, 2), (1, 3)], vec![0, 1]),
            (vec![(2, 2)], vec![0]),
            (vec![(0, 4)], vec![0]),
            (vec![(0, 1), (1, 3)], vec![1, 1]),
            (vec![(0, 1), (1, 3)], vec![2, 1]),
            (vec![(0, 1)], vec![u32::MAX]),
        ] {
            let source = Fixture {
                text: "abc".to_string(),
                ranges,
                source_indices,
                style: StyleOverride::default(),
            };
            assert_eq!(
                validate_rich_text_layout_source(&source),
                Err(TextLayoutError::LayoutFailed)
            );
        }
    }

    #[test]
    fn checked_source_range_preserves_empty_boundaries_and_rejects_invalid_slices() {
        assert_eq!(checked_source_range("abc", (1, 1)), Ok((1, 1)));
        assert_eq!(
            checked_source_range("界", (1, 2)),
            Err(TextLayoutError::LayoutFailed)
        );
        assert_eq!(
            checked_source_range("abc", (2, 1)),
            Err(TextLayoutError::LayoutFailed)
        );
        assert_eq!(
            checked_source_range("abc", (0, 4)),
            Err(TextLayoutError::LayoutFailed)
        );
    }
}
