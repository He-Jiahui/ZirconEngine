use std::collections::BTreeSet;

pub(super) fn partition_pruned_slot_ids(
    all_slot_ids: impl IntoIterator<Item = String>,
    scoped_slot_ids: &BTreeSet<String>,
    kept_slot_ids: &BTreeSet<String>,
) -> (Vec<String>, Vec<String>) {
    let mut scoped_slot_ids = scoped_slot_ids.iter().peekable();
    let mut kept_slot_ids = kept_slot_ids.iter().peekable();
    all_slot_ids.into_iter().partition(|slot_id| {
        while scoped_slot_ids
            .peek()
            .is_some_and(|candidate| candidate.as_str() < slot_id.as_str())
        {
            scoped_slot_ids.next();
        }
        let is_scoped = scoped_slot_ids
            .peek()
            .is_some_and(|candidate| candidate.as_str() == slot_id.as_str());
        if !is_scoped {
            return true;
        }

        while kept_slot_ids
            .peek()
            .is_some_and(|candidate| candidate.as_str() < slot_id.as_str())
        {
            kept_slot_ids.next();
        }
        kept_slot_ids
            .peek()
            .is_some_and(|candidate| candidate.as_str() == slot_id.as_str())
    })
}

#[cfg(test)]
mod tests {
    use std::{collections::BTreeSet, time::Instant};

    use super::partition_pruned_slot_ids;

    #[test]
    fn runtime52_retention_partition_preserves_order() {
        let all_slot_ids = slot_ids(0, 6);
        let scoped_slot_ids = ["slot.000001", "slot.000002", "slot.000003", "slot.000004"]
            .into_iter()
            .map(str::to_string)
            .collect::<BTreeSet<_>>();
        let kept_slot_ids = ["slot.000001", "slot.000004"]
            .into_iter()
            .map(str::to_string)
            .collect::<BTreeSet<_>>();

        let (retained_slot_ids, removed_slot_ids) =
            partition_pruned_slot_ids(all_slot_ids, &scoped_slot_ids, &kept_slot_ids);

        assert_eq!(
            retained_slot_ids,
            ["slot.000000", "slot.000001", "slot.000004", "slot.000005"]
                .into_iter()
                .map(str::to_string)
                .collect::<Vec<_>>()
        );
        assert_eq!(
            removed_slot_ids,
            ["slot.000002", "slot.000003"]
                .into_iter()
                .map(str::to_string)
                .collect::<Vec<_>>()
        );
    }

    #[test]
    #[ignore = "release-only performance evidence"]
    fn runtime52_retention_partition_evidence() {
        const SLOT_COUNT: usize = 100_000;
        const KEPT_COUNT: usize = 50_000;
        const SAMPLE_PAIRS: usize = 21;
        const RUNTIME52_RETENTION_PARTITION_BENCH_V1: &str =
            "RUNTIME52_RETENTION_PARTITION_BENCH_V1";

        let source_slot_ids = slot_ids(0, SLOT_COUNT);
        let scoped_slot_ids = source_slot_ids.iter().cloned().collect::<BTreeSet<_>>();
        let kept_slot_ids = source_slot_ids[..KEPT_COUNT]
            .iter()
            .cloned()
            .collect::<BTreeSet<_>>();
        let mut legacy_samples_ns = Vec::with_capacity(SAMPLE_PAIRS);
        let mut optimized_samples_ns = Vec::with_capacity(SAMPLE_PAIRS);
        let mut retained_count = 0;
        let mut removed_count = 0;
        for pair in 0..SAMPLE_PAIRS {
            if pair % 2 == 0 {
                let (elapsed, retained, removed) =
                    measure_legacy(source_slot_ids.clone(), &scoped_slot_ids, &kept_slot_ids);
                legacy_samples_ns.push(elapsed);
                retained_count = retained;
                removed_count = removed;
                optimized_samples_ns.push(measure_optimized(
                    source_slot_ids.clone(),
                    &scoped_slot_ids,
                    &kept_slot_ids,
                ));
            } else {
                optimized_samples_ns.push(measure_optimized(
                    source_slot_ids.clone(),
                    &scoped_slot_ids,
                    &kept_slot_ids,
                ));
                let (elapsed, retained, removed) =
                    measure_legacy(source_slot_ids.clone(), &scoped_slot_ids, &kept_slot_ids);
                legacy_samples_ns.push(elapsed);
                retained_count = retained;
                removed_count = removed;
            }
        }
        let legacy_p95_ns = nearest_rank_p95(&mut legacy_samples_ns);
        let optimized_p95_ns = nearest_rank_p95(&mut optimized_samples_ns);
        let p95_reduction_pct =
            100.0 * (legacy_p95_ns - optimized_p95_ns) as f64 / legacy_p95_ns as f64;
        let legacy_report_string_clones = SLOT_COUNT * 2 + removed_count;
        let partition_string_clones = SLOT_COUNT;
        let legacy_membership_queries = SLOT_COUNT * 3;
        let partition_ordered_cursor_rows = scoped_slot_ids.len() + kept_slot_ids.len();

        println!(
            "{RUNTIME52_RETENTION_PARTITION_BENCH_V1} sample_pairs={SAMPLE_PAIRS} slots={SLOT_COUNT} retained={retained_count} removed={removed_count} legacy_report_string_clones={legacy_report_string_clones} partition_string_clones={partition_string_clones} legacy_membership_queries={legacy_membership_queries} partition_ordered_cursor_rows={partition_ordered_cursor_rows} legacy_p95_ns={legacy_p95_ns} optimized_p95_ns={optimized_p95_ns} p95_reduction_pct={p95_reduction_pct:.3}"
        );

        assert_eq!(retained_count, KEPT_COUNT);
        assert_eq!(removed_count, SLOT_COUNT - KEPT_COUNT);
        assert!(
            optimized_p95_ns <= legacy_p95_ns.saturating_mul(80) / 100,
            "100k-slot report partition missed the 20% P95 reduction gate: legacy={legacy_p95_ns}ns optimized={optimized_p95_ns}ns"
        );
    }

    fn measure_legacy(
        all_slot_ids: Vec<String>,
        scoped_slot_ids: &BTreeSet<String>,
        kept_slot_ids: &BTreeSet<String>,
    ) -> (u128, usize, usize) {
        let started = Instant::now();
        let removed_slot_ids = all_slot_ids
            .iter()
            .filter(|slot_id| scoped_slot_ids.contains(*slot_id))
            .filter(|slot_id| !kept_slot_ids.contains(*slot_id))
            .cloned()
            .collect::<Vec<_>>();
        let removed_set = removed_slot_ids.iter().cloned().collect::<BTreeSet<_>>();
        let retained_slot_ids = all_slot_ids
            .into_iter()
            .filter(|slot_id| !removed_set.contains(slot_id))
            .collect::<Vec<_>>();
        (
            started.elapsed().as_nanos(),
            retained_slot_ids.len(),
            removed_slot_ids.len(),
        )
    }

    fn measure_optimized(
        all_slot_ids: Vec<String>,
        scoped_slot_ids: &BTreeSet<String>,
        kept_slot_ids: &BTreeSet<String>,
    ) -> u128 {
        let started = Instant::now();
        let _ = partition_pruned_slot_ids(all_slot_ids, scoped_slot_ids, kept_slot_ids);
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

    fn slot_ids(start: usize, end: usize) -> Vec<String> {
        (start..end)
            .map(|index| format!("slot.{index:06}"))
            .collect()
    }
}
