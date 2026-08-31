use std::collections::HashSet;
use std::slice;

use crate::scene::EntityId;

use super::QueryEntityError;

const INLINE_UNIQUE_ENTITY_SCAN_LIMIT: usize = 16;

/// Fixed-size entity list that has been validated to contain no duplicate ids.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct UniqueEntityArray<const N: usize> {
    entities: [EntityId; N],
}

impl<const N: usize> UniqueEntityArray<N> {
    pub fn new(entities: [EntityId; N]) -> Result<Self, QueryEntityError> {
        validate_unique_entities(&entities)?;
        Ok(Self { entities })
    }

    /// Creates a unique entity array without checking for duplicate ids.
    ///
    /// # Safety
    ///
    /// `entities` must not contain duplicate ids.
    pub const unsafe fn from_unique_unchecked(entities: [EntityId; N]) -> Self {
        Self { entities }
    }

    pub fn as_slice(&self) -> &[EntityId] {
        &self.entities
    }

    pub fn into_inner(self) -> [EntityId; N] {
        self.entities
    }
}

impl<const N: usize> TryFrom<[EntityId; N]> for UniqueEntityArray<N> {
    type Error = QueryEntityError;

    fn try_from(entities: [EntityId; N]) -> Result<Self, Self::Error> {
        Self::new(entities)
    }
}

impl<const N: usize> IntoIterator for UniqueEntityArray<N> {
    type IntoIter = std::array::IntoIter<EntityId, N>;
    type Item = EntityId;

    fn into_iter(self) -> Self::IntoIter {
        self.entities.into_iter()
    }
}

impl<'entity, const N: usize> IntoIterator for &'entity UniqueEntityArray<N> {
    type IntoIter = slice::Iter<'entity, EntityId>;
    type Item = &'entity EntityId;

    fn into_iter(self) -> Self::IntoIter {
        self.entities.iter()
    }
}

pub(crate) fn first_duplicate_entity<const N: usize>(entities: &[EntityId; N]) -> Option<EntityId> {
    if N <= INLINE_UNIQUE_ENTITY_SCAN_LIMIT {
        for current in 0..N {
            for previous in 0..current {
                if entities[current] == entities[previous] {
                    return Some(entities[current]);
                }
            }
        }
        return None;
    }

    first_duplicate_entity_hashed(entities)
}

fn first_duplicate_entity_hashed<const N: usize>(entities: &[EntityId; N]) -> Option<EntityId> {
    let mut seen = HashSet::with_capacity(N);
    for &entity in entities {
        if !seen.insert(entity) {
            return Some(entity);
        }
    }
    None
}

#[cfg(test)]
fn first_duplicate_entity_sorted<const N: usize>(
    entities: &[EntityId; N],
    mut record_comparison: impl FnMut(),
) -> Option<EntityId> {
    let mut indexed: [(EntityId, usize); N] = std::array::from_fn(|index| (entities[index], index));
    indexed.sort_unstable_by(|left, right| {
        record_comparison();
        left.cmp(right)
    });
    indexed
        .windows(2)
        .filter(|pair| pair[0].0 == pair[1].0)
        .map(|pair| (pair[1].1, pair[1].0))
        .min_by_key(|(index, _)| *index)
        .map(|(_, entity)| entity)
}

pub(crate) fn validate_unique_entities<const N: usize>(
    entities: &[EntityId; N],
) -> Result<(), QueryEntityError> {
    if let Some(entity) = first_duplicate_entity(entities) {
        return Err(QueryEntityError::DuplicateEntity(entity));
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use std::hint::black_box;
    use std::time::Instant;

    use super::*;

    #[test]
    fn unique_entity_validation_keeps_the_first_duplicate_in_request_order() {
        assert_eq!(first_duplicate_entity(&[7, 3, 7, 3]), Some(7));
    }

    #[test]
    fn large_unique_entity_validation_keeps_the_first_duplicate_in_request_order() {
        let mut entities = std::array::from_fn::<_, 32, _>(|index| index as EntityId + 100);
        entities[10] = entities[8];
        entities[20] = entities[2];

        assert_eq!(first_duplicate_entity(&entities), Some(108));
        assert_eq!(
            UniqueEntityArray::new(entities),
            Err(QueryEntityError::DuplicateEntity(108))
        );
    }

    #[test]
    #[ignore = "managed release performance evidence"]
    fn unique_entity_validation_release_benchmark_evidence() {
        const ENTITY_COUNT: usize = 4_096;
        const SAMPLE_PAIRS: usize = 21;

        let entities = std::array::from_fn::<_, ENTITY_COUNT, _>(|index| {
            (index.wrapping_mul(2_654_435_761) & (ENTITY_COUNT - 1)) as EntityId + 1
        });
        let mut optimized_sort_comparisons: usize = 0;
        assert_eq!(
            first_duplicate_entity_sorted(&entities, || optimized_sort_comparisons += 1),
            None
        );
        let mut legacy_ns = Vec::with_capacity(SAMPLE_PAIRS);
        let mut optimized_ns = Vec::with_capacity(SAMPLE_PAIRS);
        for sample_index in 0..SAMPLE_PAIRS {
            let mut measure_legacy = || {
                let started = Instant::now();
                assert_eq!(legacy_first_duplicate(black_box(&entities)), None);
                legacy_ns.push(started.elapsed().as_nanos());
            };
            let mut measure_optimized = || {
                let started = Instant::now();
                assert_eq!(first_duplicate_entity(black_box(&entities)), None);
                optimized_ns.push(started.elapsed().as_nanos());
            };
            if sample_index % 2 == 0 {
                measure_legacy();
                measure_optimized();
            } else {
                measure_optimized();
                measure_legacy();
            }
        }

        let legacy_p50_ns = nearest_rank_percentile(&legacy_ns, 50);
        let legacy_p95_ns = nearest_rank_percentile(&legacy_ns, 95);
        let optimized_p50_ns = nearest_rank_percentile(&optimized_ns, 50);
        let optimized_p95_ns = nearest_rank_percentile(&optimized_ns, 95);
        let legacy_comparisons = ENTITY_COUNT * (ENTITY_COUNT - 1) / 2;
        println!(
            "UNIQUE_ENTITY_VALIDATION_BENCH_V1 entities={ENTITY_COUNT} sample_pairs={SAMPLE_PAIRS} \
             legacy_comparisons={legacy_comparisons} \
             optimized_sort_comparisons={optimized_sort_comparisons} legacy_p50_ns={legacy_p50_ns} \
             legacy_p95_ns={legacy_p95_ns} optimized_p50_ns={optimized_p50_ns} \
             optimized_p95_ns={optimized_p95_ns} legacy_ns={} optimized_ns={}",
            join_samples(&legacy_ns),
            join_samples(&optimized_ns),
        );
        assert!(
            optimized_sort_comparisons.saturating_mul(50) <= legacy_comparisons,
            "sort comparisons {optimized_sort_comparisons} must be at most 2% of legacy comparisons {legacy_comparisons}"
        );
        assert!(
            optimized_p95_ns.saturating_mul(4) <= legacy_p95_ns,
            "optimized P95 {optimized_p95_ns}ns must be at most 25% of legacy P95 {legacy_p95_ns}ns"
        );
    }

    fn legacy_first_duplicate(entities: &[EntityId]) -> Option<EntityId> {
        for current in 0..entities.len() {
            for previous in 0..current {
                if entities[current] == entities[previous] {
                    return Some(entities[current]);
                }
            }
        }
        None
    }

    fn nearest_rank_percentile(samples: &[u128], percentile: usize) -> u128 {
        assert!(!samples.is_empty());
        let mut ordered = samples.to_vec();
        ordered.sort_unstable();
        let index = samples
            .len()
            .saturating_mul(percentile)
            .div_ceil(100)
            .saturating_sub(1);
        ordered[index]
    }

    fn join_samples(samples: &[u128]) -> String {
        samples
            .iter()
            .map(ToString::to_string)
            .collect::<Vec<_>>()
            .join(",")
    }
}

#[cfg(test)]
#[path = "unique_entities/hash_scan_tests.rs"]
mod hash_scan_tests;
