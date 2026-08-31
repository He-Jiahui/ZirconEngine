use std::sync::{Mutex, MutexGuard};

use zircon_runtime_interface::ui::{layout::UiPoint, surface::UiHitTestGrid};

const MIN_RETAINED_QUERY_ENTRIES: usize = 1_024;
const RETAINED_QUERY_SCALE: usize = 4;
const RETAINED_BYTES_PER_ENTRY: usize = std::mem::size_of::<u32>() + std::mem::size_of::<usize>();

#[derive(Debug, Default)]
pub(super) struct UiHitQueryScratch {
    generation: u32,
    marks: Vec<u32>,
    pub(super) candidates: Vec<usize>,
    #[cfg(test)]
    dedupe_probes: usize,
    #[cfg(test)]
    entry_count: usize,
    #[cfg(test)]
    sort_comparisons: usize,
}

impl UiHitQueryScratch {
    fn begin(&mut self, entry_count: usize) {
        self.release_excess_capacity(entry_count);
        self.candidates.clear();
        #[cfg(test)]
        {
            self.dedupe_probes = 0;
            self.entry_count = entry_count;
            self.sort_comparisons = 0;
        }
        self.generation = self.generation.wrapping_add(1);
        if self.generation == 0 {
            self.marks.fill(0);
            self.generation = 1;
        }
        if self.marks.len() < entry_count {
            self.marks.resize(entry_count, 0);
        }
    }

    fn release_excess_capacity(&mut self, entry_count: usize) {
        let retained_entry_budget = Self::retained_entry_budget(entry_count);
        if self.marks.capacity() > retained_entry_budget {
            let mut marks = Vec::with_capacity(entry_count);
            marks.resize(entry_count, 0);
            self.marks = marks;
        }
        if self.candidates.capacity() > retained_entry_budget {
            self.candidates = Vec::with_capacity(entry_count);
        }
    }

    fn retained_entry_budget(entry_count: usize) -> usize {
        entry_count
            .saturating_mul(RETAINED_QUERY_SCALE)
            .max(MIN_RETAINED_QUERY_ENTRIES)
    }

    fn retained_byte_budget(entry_count: usize) -> usize {
        Self::retained_entry_budget(entry_count).saturating_mul(RETAINED_BYTES_PER_ENTRY)
    }

    fn retained_bytes(&self) -> usize {
        self.marks
            .capacity()
            .saturating_mul(std::mem::size_of::<u32>())
            .saturating_add(
                self.candidates
                    .capacity()
                    .saturating_mul(std::mem::size_of::<usize>()),
            )
    }

    fn insert_candidate(&mut self, entry_index: usize) {
        let Some(mark) = self.marks.get_mut(entry_index) else {
            return;
        };
        #[cfg(test)]
        {
            self.dedupe_probes += 1;
        }
        if *mark == self.generation {
            return;
        }
        *mark = self.generation;
        self.candidates.push(entry_index);
    }
}

#[derive(Debug, Default)]
pub(super) struct UiHitQueryScratchCell(Mutex<UiHitQueryScratch>);

impl Clone for UiHitQueryScratchCell {
    fn clone(&self) -> Self {
        Self::default()
    }
}

impl UiHitQueryScratchCell {
    pub(super) fn collect<'a>(
        &'a self,
        grid: &UiHitTestGrid,
        point: UiPoint,
        cursor_radius: f32,
    ) -> MutexGuard<'a, UiHitQueryScratch> {
        let mut scratch = self.lock();
        scratch.begin(grid.entries.len());
        let Some((left, right, top, bottom)) =
            super::cell_bounds_for_query(grid, point, cursor_radius)
        else {
            return scratch;
        };
        for row in top..=bottom {
            for column in left..=right {
                let cell_index = (row * grid.columns + column) as usize;
                let Some(cell) = grid.cells.get(cell_index) else {
                    continue;
                };
                for entry_index in &cell.entries {
                    scratch.insert_candidate(*entry_index);
                }
            }
        }
        #[cfg(test)]
        let mut sort_comparisons = 0usize;
        scratch.candidates.sort_by(|left, right| {
            #[cfg(test)]
            {
                sort_comparisons += 1;
            }
            let left_entry = grid.entries.get(*left);
            let right_entry = grid.entries.get(*right);
            match (left_entry, right_entry) {
                (Some(left_entry), Some(right_entry)) => {
                    super::entry_sort_key(right_entry).cmp(&super::entry_sort_key(left_entry))
                }
                _ => right.cmp(left),
            }
        });
        #[cfg(test)]
        {
            scratch.sort_comparisons = sort_comparisons;
        }
        scratch
    }

    fn lock(&self) -> MutexGuard<'_, UiHitQueryScratch> {
        self.0
            .lock()
            .unwrap_or_else(|poisoned| poisoned.into_inner())
    }

    #[cfg(test)]
    pub(super) fn stats(&self) -> UiHitQueryScratchStats {
        let scratch = self.lock();
        UiHitQueryScratchStats {
            generation: scratch.generation,
            dedupe_probes: scratch.dedupe_probes,
            sort_comparisons: scratch.sort_comparisons,
            unique_candidates: scratch.candidates.len(),
            mark_capacity: scratch.marks.capacity(),
            candidate_capacity: scratch.candidates.capacity(),
            retained_bytes: scratch.retained_bytes(),
            retained_byte_budget: UiHitQueryScratch::retained_byte_budget(scratch.entry_count),
        }
    }
}

#[cfg(test)]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) struct UiHitQueryScratchStats {
    pub generation: u32,
    pub dedupe_probes: usize,
    pub sort_comparisons: usize,
    pub unique_candidates: usize,
    pub mark_capacity: usize,
    pub candidate_capacity: usize,
    pub retained_bytes: usize,
    pub retained_byte_budget: usize,
}

#[cfg(test)]
mod tests {
    use super::UiHitQueryScratch;

    #[test]
    fn historical_high_water_is_released_to_the_current_entry_byte_budget() {
        let mut scratch = UiHitQueryScratch::default();
        scratch.begin(16_384);
        for entry_index in 0..16_384 {
            scratch.insert_candidate(entry_index);
        }
        let high_water_bytes = scratch.retained_bytes();

        scratch.begin(32);
        let retained_after_shrink = scratch.retained_bytes();
        let small_entry_budget = UiHitQueryScratch::retained_byte_budget(32);
        let mark_capacity = scratch.marks.capacity();
        let candidate_capacity = scratch.candidates.capacity();

        assert!(retained_after_shrink < high_water_bytes);
        assert!(retained_after_shrink <= small_entry_budget);
        assert!(scratch.candidates.is_empty());

        for entry_index in 0..32 {
            scratch.insert_candidate(entry_index);
        }
        scratch.begin(32);

        assert_eq!(scratch.marks.capacity(), mark_capacity);
        assert_eq!(scratch.candidates.capacity(), candidate_capacity);
        assert!(scratch.retained_bytes() <= small_entry_budget);
    }

    #[test]
    fn dedupe_probe_count_scales_linearly_through_ten_thousand_entries() {
        const CELL_REFERENCES_PER_ENTRY: usize = 4;

        for entry_count in [1, 100, 1_000, 10_000] {
            let mut scratch = UiHitQueryScratch::default();
            scratch.begin(entry_count);
            for _ in 0..CELL_REFERENCES_PER_ENTRY {
                for entry_index in 0..entry_count {
                    scratch.insert_candidate(entry_index);
                }
            }

            assert_eq!(scratch.candidates.len(), entry_count);
            assert_eq!(
                scratch.dedupe_probes,
                entry_count * CELL_REFERENCES_PER_ENTRY
            );
            assert!(
                scratch.retained_bytes() <= UiHitQueryScratch::retained_byte_budget(entry_count)
            );
        }
    }
}
