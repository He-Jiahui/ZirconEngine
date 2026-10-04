use std::sync::{Mutex, MutexGuard};

use zircon_runtime_interface::ui::{event_ui::UiNodeId, layout::UiPoint, surface::UiHitTestGrid};

const MIN_RETAINED_QUERY_ENTRIES: usize = 1_024;
const RETAINED_QUERY_SCALE: usize = 4;
pub(super) type UiHitRadiusHit = (f32, UiNodeId, usize);

const RETAINED_BYTES_PER_ENTRY: usize = std::mem::size_of::<u32>()
    + std::mem::size_of::<usize>()
    + std::mem::size_of::<UiHitRadiusHit>();

#[derive(Debug, Default)]
pub(super) struct UiHitQueryScratch {
    generation: u32,
    marks: Vec<u32>,
    pub(super) candidates: Vec<usize>,
    pub(super) radius_hits: Vec<UiHitRadiusHit>,
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
        self.radius_hits.clear();
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
        if self.radius_hits.capacity() > retained_entry_budget {
            self.radius_hits = Vec::with_capacity(entry_count);
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
            .saturating_add(
                self.radius_hits
                    .capacity()
                    .saturating_mul(std::mem::size_of::<UiHitRadiusHit>()),
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
            radius_hit_count: scratch.radius_hits.len(),
            radius_hit_capacity: scratch.radius_hits.capacity(),
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
    pub radius_hit_count: usize,
    pub radius_hit_capacity: usize,
    pub retained_bytes: usize,
    pub retained_byte_budget: usize,
}

#[cfg(test)]
#[path = "tests/query_scratch.rs"]
mod tests;
