use std::ops::Range;
use std::sync::Arc;

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub(super) struct ScreenSpaceUiTextSegmentRunSpan {
    pub(super) segment_index: usize,
    pub(super) native_run_base: usize,
    pub(super) native_run_count: usize,
    pub(super) sdf_run_base: usize,
    pub(super) sdf_run_count: usize,
}

#[derive(Debug, Default)]
pub(super) struct ScreenSpaceUiTextFrameRunIndex {
    spans: Arc<[ScreenSpaceUiTextSegmentRunSpan]>,
    native_run_count: usize,
    sdf_run_count: usize,
}

impl ScreenSpaceUiTextSegmentRunSpan {
    pub(super) fn native_run_range(self) -> Range<usize> {
        self.native_run_base..self.native_run_base.saturating_add(self.native_run_count)
    }

    pub(super) fn sdf_run_range(self) -> Range<usize> {
        self.sdf_run_base..self.sdf_run_base.saturating_add(self.sdf_run_count)
    }
}

impl ScreenSpaceUiTextFrameRunIndex {
    pub(super) fn from_segment_run_counts<Counts>(segment_run_counts: Counts) -> Self
    where
        Counts: IntoIterator<Item = [usize; 2]>,
    {
        let mut native_run_count = 0usize;
        let mut sdf_run_count = 0usize;
        let spans = segment_run_counts
            .into_iter()
            .enumerate()
            .map(|(segment_index, [native_count, sdf_count])| {
                let span = ScreenSpaceUiTextSegmentRunSpan {
                    segment_index,
                    native_run_base: native_run_count,
                    native_run_count: native_count,
                    sdf_run_base: sdf_run_count,
                    sdf_run_count: sdf_count,
                };
                native_run_count = native_run_count.saturating_add(native_count);
                sdf_run_count = sdf_run_count.saturating_add(sdf_count);
                span
            })
            .collect::<Vec<_>>();
        let index = Self {
            spans: Arc::from(spans),
            native_run_count,
            sdf_run_count,
        };
        debug_assert!(index.is_contiguous());
        index
    }

    pub(super) fn spans(&self) -> &[ScreenSpaceUiTextSegmentRunSpan] {
        &self.spans
    }

    pub(super) fn native_run_count(&self) -> usize {
        self.native_run_count
    }

    pub(super) fn sdf_run_count(&self) -> usize {
        self.sdf_run_count
    }

    fn is_contiguous(&self) -> bool {
        let mut native_base = 0usize;
        let mut sdf_base = 0usize;
        for (segment_index, span) in self.spans.iter().copied().enumerate() {
            if span.segment_index != segment_index
                || span.native_run_base != native_base
                || span.sdf_run_base != sdf_base
            {
                return false;
            }
            native_base = span.native_run_range().end;
            sdf_base = span.sdf_run_range().end;
        }
        native_base == self.native_run_count && sdf_base == self.sdf_run_count
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn frame_run_index_preserves_empty_segments_and_global_run_bases() {
        let index =
            ScreenSpaceUiTextFrameRunIndex::from_segment_run_counts([[2, 3], [0, 1], [4, 0]]);

        assert_eq!(index.native_run_count(), 6);
        assert_eq!(index.sdf_run_count(), 4);
        assert_eq!(
            index.spans(),
            &[
                ScreenSpaceUiTextSegmentRunSpan {
                    segment_index: 0,
                    native_run_base: 0,
                    native_run_count: 2,
                    sdf_run_base: 0,
                    sdf_run_count: 3,
                },
                ScreenSpaceUiTextSegmentRunSpan {
                    segment_index: 1,
                    native_run_base: 2,
                    native_run_count: 0,
                    sdf_run_base: 3,
                    sdf_run_count: 1,
                },
                ScreenSpaceUiTextSegmentRunSpan {
                    segment_index: 2,
                    native_run_base: 2,
                    native_run_count: 4,
                    sdf_run_base: 4,
                    sdf_run_count: 0,
                },
            ]
        );
    }

    #[test]
    fn empty_frame_run_index_has_zero_counts_and_no_spans() {
        let index = ScreenSpaceUiTextFrameRunIndex::from_segment_run_counts([]);

        assert!(index.spans().is_empty());
        assert_eq!(index.native_run_count(), 0);
        assert_eq!(index.sdf_run_count(), 0);
    }
}
