pub(super) fn sorted_index_by_key<T, F>(values: &[T], target: &str, key: F) -> Option<usize>
where
    F: Fn(&T) -> &str,
{
    values
        .binary_search_by(|value| key(value).cmp(target))
        .ok()
        .or_else(|| values.iter().position(|value| key(value) == target))
}

#[cfg(test)]
mod tests {
    use std::{hint::black_box, time::Instant};

    use super::sorted_index_by_key;

    #[test]
    fn runtime52_sorted_lookup_finds_canonical_rows() {
        let values = ["slot.000001", "slot.000002", "slot.000003"];

        assert_eq!(
            sorted_index_by_key(&values, "slot.000002", |value| *value),
            Some(1)
        );
        assert_eq!(
            sorted_index_by_key(&values, "slot.000004", |value| *value),
            None
        );
    }

    #[test]
    fn runtime52_sorted_lookup_preserves_unsorted_input() {
        let values = ["slot.z", "slot.a", "slot.m"];

        assert_eq!(
            sorted_index_by_key(&values, "slot.z", |value| *value),
            Some(0)
        );
    }

    #[test]
    #[ignore = "release-only performance evidence"]
    fn runtime52_sorted_lookup_evidence() {
        const SLOT_COUNT: usize = 100_000;
        const QUERY_COUNT: usize = 100;
        const SAMPLE_PAIRS: usize = 21;
        const RUNTIME52_SORTED_LOOKUP_BENCH_V1: &str = "RUNTIME52_SORTED_LOOKUP_BENCH_V1";

        let slot_ids = (0..SLOT_COUNT)
            .map(|index| format!("slot.{index:06}"))
            .collect::<Vec<_>>();
        let query_indices = (0..QUERY_COUNT)
            .map(|index| index * (SLOT_COUNT - 1) / (QUERY_COUNT - 1))
            .collect::<Vec<_>>();
        let query_ids = query_indices
            .iter()
            .map(|index| slot_ids[*index].clone())
            .collect::<Vec<_>>();
        let mut legacy_samples_ns = Vec::with_capacity(SAMPLE_PAIRS);
        let mut optimized_samples_ns = Vec::with_capacity(SAMPLE_PAIRS);
        for pair in 0..SAMPLE_PAIRS {
            if pair % 2 == 0 {
                legacy_samples_ns.push(measure_legacy(&slot_ids, &query_ids));
                optimized_samples_ns.push(measure_optimized(&slot_ids, &query_ids));
            } else {
                optimized_samples_ns.push(measure_optimized(&slot_ids, &query_ids));
                legacy_samples_ns.push(measure_legacy(&slot_ids, &query_ids));
            }
        }
        let legacy_p95_ns = nearest_rank_p95(&mut legacy_samples_ns);
        let optimized_p95_ns = nearest_rank_p95(&mut optimized_samples_ns);
        let p95_reduction_pct =
            100.0 * (legacy_p95_ns - optimized_p95_ns) as f64 / legacy_p95_ns as f64;
        let legacy_comparisons = query_indices.iter().map(|index| index + 1).sum::<usize>();
        let binary_comparison_ceiling = QUERY_COUNT
            * usize::BITS
                .checked_sub((SLOT_COUNT - 1).leading_zeros())
                .expect("the non-empty slot fixture has a comparison ceiling")
                as usize;

        println!(
            "{RUNTIME52_SORTED_LOOKUP_BENCH_V1} sample_pairs={SAMPLE_PAIRS} slots={SLOT_COUNT} queries={QUERY_COUNT} legacy_comparisons={legacy_comparisons} binary_comparison_ceiling={binary_comparison_ceiling} legacy_p95_ns={legacy_p95_ns} optimized_p95_ns={optimized_p95_ns} p95_reduction_pct={p95_reduction_pct:.3}"
        );

        assert!(
            optimized_p95_ns <= legacy_p95_ns.saturating_mul(20) / 100,
            "100k-slot manifest lookup missed the 80% P95 reduction gate: legacy={legacy_p95_ns}ns optimized={optimized_p95_ns}ns"
        );
    }

    fn measure_legacy(values: &[String], queries: &[String]) -> u128 {
        let started = Instant::now();
        for query in queries {
            black_box(values.iter().position(|value| value == query));
        }
        started.elapsed().as_nanos()
    }

    fn measure_optimized(values: &[String], queries: &[String]) -> u128 {
        let started = Instant::now();
        for query in queries {
            black_box(sorted_index_by_key(values, query, String::as_str));
        }
        started.elapsed().as_nanos()
    }

    fn nearest_rank_p95(samples: &mut [u128]) -> u128 {
        samples.sort_unstable();
        let index = samples
            .len()
            .saturating_mul(95)
            .div_ceil(100)
            .saturating_sub(1);
        samples[index]
    }
}
