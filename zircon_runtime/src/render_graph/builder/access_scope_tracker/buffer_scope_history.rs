use std::collections::BTreeMap;
use std::ops::Bound::{Excluded, Unbounded};

use super::{AccessScopeWorkReceipt, ResourceAccessHistory};
use crate::render_graph::RenderGraphError;

#[cfg(test)]
thread_local! {
    static WHOLE_MAP_COALESCING: std::cell::Cell<bool> = const { std::cell::Cell::new(false) };
}

#[cfg(test)]
pub(in crate::render_graph::builder) fn with_whole_map_coalescing<T>(run: impl FnOnce() -> T) -> T {
    struct Restore(bool);
    impl Drop for Restore {
        fn drop(&mut self) {
            WHOLE_MAP_COALESCING.set(self.0);
        }
    }
    let _restore = Restore(WHOLE_MAP_COALESCING.replace(true));
    run()
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub(super) struct BufferSegment {
    pub(super) end: u64,
    pub(super) history: ResourceAccessHistory,
}

#[cfg(test)]
#[path = "tests/buffer_scope_history.rs"]
mod tests;

#[derive(Clone, Debug, Default)]
pub(super) struct BufferScopeHistory {
    pub(super) segments: BTreeMap<u64, BufferSegment>,
}

impl BufferScopeHistory {
    pub(super) fn new(end: u64) -> Self {
        let mut segments = BTreeMap::new();
        segments.insert(
            0,
            BufferSegment {
                end,
                history: ResourceAccessHistory::default(),
            },
        );
        Self { segments }
    }

    pub(super) fn ensure_boundaries(
        &mut self,
        start: u64,
        end: u64,
        identity: usize,
        work: &mut AccessScopeWorkReceipt,
    ) -> Result<(), RenderGraphError> {
        self.split_at(start, identity, work)?;
        self.split_at(end, identity, work)
    }

    pub(super) fn overlapping_segments(
        &self,
        start: u64,
        end: u64,
    ) -> impl Iterator<Item = (&u64, &BufferSegment)> {
        let first = self
            .segments
            .range(..=start)
            .next_back()
            .map_or(start, |(&key, _)| key);
        self.segments
            .range(first..end)
            .filter(move |(_, segment)| segment.end > start)
    }

    fn split_at(
        &mut self,
        boundary: u64,
        identity: usize,
        work: &mut AccessScopeWorkReceipt,
    ) -> Result<(), RenderGraphError> {
        work.lookup_visits += 1;
        let Some((&start, segment)) = self.segments.range(..=boundary).next_back() else {
            return Err(RenderGraphError::AccessScopeTrackerStateMismatch { identity });
        };
        if boundary == start || boundary >= segment.end {
            return Ok(());
        }
        let end = segment.end;
        let history = segment.history.clone();
        let Some(segment) = self.segments.get_mut(&start) else {
            return Err(RenderGraphError::AccessScopeTrackerStateMismatch { identity });
        };
        segment.end = boundary;
        self.segments
            .insert(boundary, BufferSegment { end, history });
        work.split_visits += 1;
        Ok(())
    }

    pub(super) fn merge_adjacent_equal_around(
        &mut self,
        start: u64,
        end: u64,
        work: &mut AccessScopeWorkReceipt,
    ) {
        #[cfg(test)]
        if WHOLE_MAP_COALESCING.get() {
            self.merge_adjacent_equal_whole_map(work);
            return;
        }
        let mut left_start = self
            .segments
            .range(..start)
            .next_back()
            .map(|(key, _)| *key)
            .unwrap_or(start);
        loop {
            let Some(right_start) = self
                .segments
                .range((Excluded(left_start), Unbounded))
                .next()
                .map(|(key, _)| *key)
            else {
                return;
            };
            work.merge_visits += 1;
            let should_merge = self
                .segments
                .get(&left_start)
                .zip(self.segments.get(&right_start))
                .is_some_and(|(left, right)| {
                    left.end == right_start && left.history == right.history
                });
            if should_merge {
                let Some(right) = self.segments.remove(&right_start) else {
                    return;
                };
                if let Some(left) = self.segments.get_mut(&left_start) {
                    left.end = right.end;
                }
            } else {
                left_start = right_start;
            }
            if right_start >= end {
                return;
            }
        }
    }

    #[cfg(test)]
    fn merge_adjacent_equal_whole_map(&mut self, work: &mut AccessScopeWorkReceipt) {
        let starts: Vec<_> = self.segments.keys().copied().collect();
        let Some(&first) = starts.first() else {
            return;
        };
        let mut left_start = first;
        for &right_start in &starts[1..] {
            work.merge_visits += 1;
            let should_merge = self
                .segments
                .get(&left_start)
                .zip(self.segments.get(&right_start))
                .is_some_and(|(left, right)| {
                    left.end == right_start && left.history == right.history
                });
            if should_merge {
                let right = self.segments.remove(&right_start).unwrap();
                self.segments.get_mut(&left_start).unwrap().end = right.end;
            } else {
                left_start = right_start;
            }
        }
    }
}
