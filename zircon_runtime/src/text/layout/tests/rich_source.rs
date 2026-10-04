use super::{
    checked_source_range, validate_rich_text_layout_source, RichTextLayoutRun, RichTextLayoutSource,
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
