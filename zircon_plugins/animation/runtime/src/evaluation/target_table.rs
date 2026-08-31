use std::collections::HashMap;

use zircon_runtime::core::framework::animation::AnimationTargetId;

use super::{TargetSlot, TargetTableError};

/// Per-evaluation dense binding table from stable identity to a resolved runtime target.
#[derive(Clone, Debug)]
pub struct TargetTable<T> {
    slots: TargetSlotMap,
    targets: Vec<T>,
}

const INLINE_TARGET_CAPACITY: usize = 32;

#[derive(Clone, Debug)]
enum TargetSlotMap {
    Inline(Vec<(AnimationTargetId, TargetSlot)>),
    Indexed(HashMap<AnimationTargetId, TargetSlot>),
}

impl Default for TargetSlotMap {
    fn default() -> Self {
        Self::Inline(Vec::new())
    }
}

impl TargetSlotMap {
    fn get(&self, target_id: AnimationTargetId) -> Option<TargetSlot> {
        match self {
            Self::Inline(entries) => entries
                .iter()
                .find_map(|(id, slot)| (*id == target_id).then_some(*slot)),
            Self::Indexed(indexed) => indexed.get(&target_id).copied(),
        }
    }

    fn insert(&mut self, target_id: AnimationTargetId, slot: TargetSlot) {
        match self {
            Self::Inline(entries) if entries.len() < INLINE_TARGET_CAPACITY => {
                entries.push((target_id, slot));
            }
            Self::Inline(entries) => {
                let mut indexed = HashMap::with_capacity(entries.len().saturating_mul(2));
                indexed.extend(entries.drain(..));
                indexed.insert(target_id, slot);
                *self = Self::Indexed(indexed);
            }
            Self::Indexed(indexed) => {
                indexed.insert(target_id, slot);
            }
        }
    }
}

impl<T> Default for TargetTable<T> {
    fn default() -> Self {
        Self {
            slots: TargetSlotMap::default(),
            targets: Vec::new(),
        }
    }
}

impl<T> TargetTable<T>
where
    T: Clone + Eq,
{
    pub fn new() -> Self {
        Self::default()
    }

    pub fn bind(
        &mut self,
        target_id: AnimationTargetId,
        target: T,
    ) -> Result<TargetSlot, TargetTableError> {
        if let Some(slot) = self.slots.get(target_id) {
            let existing = &self.targets[slot.index() as usize];
            return if existing == &target {
                Ok(slot)
            } else {
                Err(TargetTableError::ConflictingBinding { target_id })
            };
        }

        let index =
            u32::try_from(self.targets.len()).map_err(|_| TargetTableError::CapacityExceeded)?;
        let slot = TargetSlot::new(index);
        self.targets.push(target);
        self.slots.insert(target_id, slot);
        Ok(slot)
    }

    pub fn slot(&self, target_id: AnimationTargetId) -> Option<TargetSlot> {
        self.slots.get(target_id)
    }

    pub fn target(&self, slot: TargetSlot) -> Option<&T> {
        self.targets.get(slot.index() as usize)
    }
}

#[cfg(test)]
mod optimization_batch_20260830co_tests {
    use std::collections::BTreeMap;
    use std::hint::black_box;
    use std::time::Instant;

    use super::{TargetSlotMap, TargetTable, INLINE_TARGET_CAPACITY};
    use zircon_runtime::core::framework::animation::AnimationTargetId;

    const SAMPLE_PAIRS: usize = 21;
    const BENCH_TARGETS: usize = 1_024;
    const BENCH_LOOKUPS: usize = 262_144;

    #[test]
    fn optimization_batch_20260830co_target_table_keeps_small_sets_inline() {
        let mut table = TargetTable::new();
        for index in 0..INLINE_TARGET_CAPACITY {
            table.bind(target_id(index), index).unwrap();
        }

        assert!(matches!(table.slots, TargetSlotMap::Inline(_)));
        for index in 0..INLINE_TARGET_CAPACITY {
            assert_eq!(table.slot(target_id(index)).unwrap().index(), index as u32);
        }
    }

    #[test]
    fn optimization_batch_20260830co_target_table_promotes_once_and_preserves_conflicts() {
        let mut table = TargetTable::new();
        for index in 0..=INLINE_TARGET_CAPACITY {
            table.bind(target_id(index), index).unwrap();
        }

        assert!(matches!(table.slots, TargetSlotMap::Indexed(_)));
        let duplicate = target_id(INLINE_TARGET_CAPACITY);
        assert_eq!(
            table.bind(duplicate, INLINE_TARGET_CAPACITY).unwrap(),
            table.slot(duplicate).unwrap()
        );
        assert!(table.bind(duplicate, usize::MAX).is_err());
    }

    #[test]
    #[ignore = "release-only adaptive target-table lookup benchmark"]
    fn optimization_batch_20260830co_target_table_release_benchmark() {
        let ids = (0..BENCH_TARGETS).map(target_id).collect::<Vec<_>>();
        let mut baseline = BTreeMap::new();
        let mut optimized = TargetTable::new();
        for (index, id) in ids.iter().copied().enumerate() {
            baseline.insert(id, index as u32);
            optimized.bind(id, index).unwrap();
        }
        let queries = (0..BENCH_LOOKUPS)
            .map(|query| (query.wrapping_mul(17).wrapping_add(query / 7 * 13)) % ids.len())
            .collect::<Vec<_>>();
        let mut baseline_samples = Vec::with_capacity(SAMPLE_PAIRS);
        let mut optimized_samples = Vec::with_capacity(SAMPLE_PAIRS);

        for pair in 0..SAMPLE_PAIRS {
            let (first, second) = if pair % 2 == 0 {
                (
                    measure_lookup(&queries, |index| baseline[&ids[index]] as u64),
                    measure_lookup(&queries, |index| {
                        optimized.slot(ids[index]).unwrap().index() as u64
                    }),
                )
            } else {
                let optimized = measure_lookup(&queries, |index| {
                    optimized.slot(ids[index]).unwrap().index() as u64
                });
                let baseline = measure_lookup(&queries, |index| baseline[&ids[index]] as u64);
                (baseline, optimized)
            };
            assert_eq!(first.1, second.1);
            baseline_samples.push(first.0);
            optimized_samples.push(second.0);
        }

        let baseline_p95_ns = percentile(&baseline_samples, 95);
        let optimized_p95_ns = percentile(&optimized_samples, 95);
        println!(
            "RUNTIME170_TARGET_TABLE_ADAPTIVE_INDEX_BENCH_V1 sample_pairs={SAMPLE_PAIRS} target_count={BENCH_TARGETS} lookup_count={BENCH_LOOKUPS} baseline_p95_ns={baseline_p95_ns} optimized_p95_ns={optimized_p95_ns} baseline_raw_ns={} optimized_raw_ns={}",
            raw(&baseline_samples),
            raw(&optimized_samples),
        );
        assert!(
            optimized_p95_ns.saturating_mul(4) <= baseline_p95_ns.saturating_mul(3),
            "adaptive target-table lookup must reduce 1,024-target P95 by at least 25%: baseline={baseline_p95_ns}ns optimized={optimized_p95_ns}ns"
        );
    }

    fn target_id(index: usize) -> AnimationTargetId {
        AnimationTargetId::from_segments([format!("Bone{index:04}")])
    }

    fn measure_lookup(queries: &[usize], mut lookup: impl FnMut(usize) -> u64) -> (u128, u64) {
        let started = Instant::now();
        let checksum = queries.iter().copied().fold(0_u64, |checksum, index| {
            checksum.wrapping_add(black_box(lookup(black_box(index))))
        });
        (started.elapsed().as_nanos(), black_box(checksum))
    }

    fn percentile(samples: &[u128], percentile: usize) -> u128 {
        let mut sorted = samples.to_vec();
        sorted.sort_unstable();
        sorted[(sorted.len() - 1) * percentile / 100]
    }

    fn raw(samples: &[u128]) -> String {
        samples
            .iter()
            .map(u128::to_string)
            .collect::<Vec<_>>()
            .join(",")
    }
}
