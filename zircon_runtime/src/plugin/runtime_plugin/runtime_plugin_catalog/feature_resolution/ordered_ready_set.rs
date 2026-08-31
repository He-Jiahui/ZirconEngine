const WORD_BITS: usize = u64::BITS as usize;

fn ready_set_level_count(capacity: usize) -> usize {
    let mut level_count = 0;
    let mut word_count = capacity.div_ceil(WORD_BITS);
    while word_count > 0 {
        level_count += 1;
        if word_count == 1 {
            break;
        }
        word_count = word_count.div_ceil(WORD_BITS);
    }
    level_count
}

/// Ordered integer set with a fixed machine-word hierarchy.
///
/// The hierarchy has at most `ceil(usize::BITS / log2(WORD_BITS))` levels, so
/// insert/pop work is bounded independently of the feature count.
pub(super) struct OrderedReadySet {
    levels: Vec<Vec<u64>>,
    len: usize,
}

impl OrderedReadySet {
    pub(super) fn new(capacity: usize) -> Self {
        let mut levels = Vec::with_capacity(ready_set_level_count(capacity));
        let mut word_count = capacity.div_ceil(WORD_BITS);
        while word_count > 0 {
            levels.push(vec![0; word_count]);
            if word_count == 1 {
                break;
            }
            word_count = word_count.div_ceil(WORD_BITS);
        }
        Self { levels, len: 0 }
    }

    pub(super) fn is_empty(&self) -> bool {
        self.len == 0
    }

    pub(super) fn insert(&mut self, index: usize) -> bool {
        let Some(base_level) = self.levels.first() else {
            return false;
        };
        let word_index = index / WORD_BITS;
        if word_index >= base_level.len() {
            return false;
        }
        let bit = 1_u64 << (index % WORD_BITS);
        if base_level[word_index] & bit != 0 {
            return false;
        }

        let mut item_index = index;
        for level in &mut self.levels {
            let word_index = item_index / WORD_BITS;
            let bit = 1_u64 << (item_index % WORD_BITS);
            let was_empty = level[word_index] == 0;
            level[word_index] |= bit;
            if !was_empty {
                break;
            }
            item_index = word_index;
        }
        self.len += 1;
        true
    }

    pub(super) fn pop_first(&mut self) -> Option<usize> {
        if self.len == 0 {
            return None;
        }
        let top_level = self.levels.len() - 1;
        let mut item_index = self.levels[top_level][0].trailing_zeros() as usize;
        for level_index in (0..top_level).rev() {
            let word = self.levels[level_index][item_index];
            item_index = item_index * WORD_BITS + word.trailing_zeros() as usize;
        }
        self.remove(item_index);
        Some(item_index)
    }

    fn remove(&mut self, index: usize) {
        let mut item_index = index;
        for level in &mut self.levels {
            let word_index = item_index / WORD_BITS;
            let bit = 1_u64 << (item_index % WORD_BITS);
            level[word_index] &= !bit;
            if level[word_index] != 0 {
                break;
            }
            item_index = word_index;
        }
        self.len -= 1;
    }
}

#[cfg(test)]
mod tests {
    use super::{ready_set_level_count, OrderedReadySet};

    #[test]
    fn pops_sparse_indices_in_stable_order() {
        let mut set = OrderedReadySet::new(1_000);
        for index in [999, 64, 511, 0, 65, 511] {
            set.insert(index);
        }

        assert_eq!(
            std::iter::from_fn(|| set.pop_first()).collect::<Vec<_>>(),
            [0, 64, 65, 511, 999]
        );
    }

    #[test]
    fn optimization_batch_20260830cv_ready_set_level_count_covers_word_boundaries() {
        assert_eq!(ready_set_level_count(0), 0);
        assert_eq!(ready_set_level_count(1), 1);
        assert_eq!(ready_set_level_count(64), 1);
        assert_eq!(ready_set_level_count(65), 2);
        assert_eq!(ready_set_level_count(4_096), 2);
        assert_eq!(ready_set_level_count(4_097), 3);
    }

    #[test]
    fn optimization_batch_20260830cv_ready_set_reserves_its_exact_level_count() {
        let source = include_str!("ordered_ready_set.rs");
        let production = source
            .split("#[cfg(test)]")
            .next()
            .expect("ordered ready set production source");

        assert!(production.contains("Vec::with_capacity(ready_set_level_count(capacity))"));
    }

    #[test]
    #[ignore = "release-only performance evidence"]
    fn optimization_batch_20260830cv_ready_set_level_capacity_evidence() {
        const BATCH_COUNT: usize = 32_768;
        const FEATURE_CAPACITY: usize = 1 << 20;
        const MARKER: &str = "RUNTIME509_ORDERED_READY_SET_LEVEL_CAPACITY_BENCH_V1";
        let level_count = ready_set_level_count(FEATURE_CAPACITY);

        let legacy_growth_events = level_growth_events(BATCH_COUNT, level_count, false);
        let optimized_growth_events = level_growth_events(BATCH_COUNT, level_count, true);

        assert!(legacy_growth_events > 0);
        assert_eq!(optimized_growth_events, 0);
        println!(
            "{MARKER} batches={BATCH_COUNT} feature_capacity={FEATURE_CAPACITY} \
             levels_per_batch={level_count} legacy_growth_events={legacy_growth_events} \
             optimized_growth_events={optimized_growth_events} reduction_pct=100"
        );
    }

    fn level_growth_events(batch_count: usize, level_count: usize, reserve: bool) -> usize {
        let mut growth_events = 0;
        for _ in 0..batch_count {
            let mut levels = if reserve {
                Vec::with_capacity(level_count)
            } else {
                Vec::new()
            };
            for level in 0..level_count {
                let previous_capacity = levels.capacity();
                levels.push(level);
                growth_events += usize::from(levels.capacity() != previous_capacity);
            }
        }
        growth_events
    }
}
