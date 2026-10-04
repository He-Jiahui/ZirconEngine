use std::{
    collections::{BTreeSet, HashSet},
    hint::black_box,
    time::Instant,
};

use super::{finish_focus_chain, UiFocusChainCandidate};
use crate::ui::{event_ui::UiNodeId, navigation::UiTabIndex};

const CANDIDATE_COUNT: usize = 10_000;
const VISIT_COUNT: usize = 65_536;
const SAMPLE_PAIRS: usize = 21;

#[derive(Clone, Copy)]
struct LegacyUiFocusChainCandidate {
    node_id: UiNodeId,
    tab_index: Option<UiTabIndex>,
    pre_order: usize,
}

fn legacy_finish_focus_chain(mut candidates: Vec<LegacyUiFocusChainCandidate>) -> Vec<UiNodeId> {
    candidates.sort_by_key(|candidate| {
        (
            candidate.tab_index.is_none(),
            candidate.tab_index.map_or(0, |index| index.order),
            candidate.pre_order,
        )
    });
    candidates
        .into_iter()
        .map(|candidate| candidate.node_id)
        .collect()
}

fn p95(mut samples: Vec<u128>) -> u128 {
    samples.sort_unstable();
    samples[(samples.len() * 95).div_ceil(100) - 1]
}

fn measure<T>(run: impl FnOnce() -> T) -> (u128, T) {
    let start = Instant::now();
    let output = run();
    let elapsed = start.elapsed().as_nanos();
    (elapsed, output)
}

#[test]
#[ignore = "release performance evidence"]
fn focus_chain_default_partition_avoids_full_candidate_sort() {
    let default_candidates = (0..CANDIDATE_COUNT)
        .map(|index| UiNodeId::new(index as u64 + 1))
        .collect::<Vec<_>>();
    let legacy_candidates = default_candidates
        .iter()
        .copied()
        .enumerate()
        .map(|(pre_order, node_id)| LegacyUiFocusChainCandidate {
            node_id,
            tab_index: None,
            pre_order,
        })
        .collect::<Vec<_>>();

    for _ in 0..5 {
        black_box(legacy_finish_focus_chain(legacy_candidates.clone()));
        black_box(finish_focus_chain(default_candidates.clone(), Vec::new()));
    }

    let mut legacy_samples = Vec::with_capacity(SAMPLE_PAIRS);
    let mut optimized_samples = Vec::with_capacity(SAMPLE_PAIRS);
    for sample in 0..SAMPLE_PAIRS {
        let legacy_input = legacy_candidates.clone();
        let optimized_input = default_candidates.clone();
        if sample % 2 == 0 {
            let (elapsed, output) = measure(|| legacy_finish_focus_chain(legacy_input));
            black_box(output);
            legacy_samples.push(elapsed);
            let (elapsed, output) = measure(|| {
                finish_focus_chain(optimized_input, Vec::<UiFocusChainCandidate>::new())
            });
            black_box(output);
            optimized_samples.push(elapsed);
        } else {
            let (elapsed, output) = measure(|| {
                finish_focus_chain(optimized_input, Vec::<UiFocusChainCandidate>::new())
            });
            black_box(output);
            optimized_samples.push(elapsed);
            let (elapsed, output) = measure(|| legacy_finish_focus_chain(legacy_input));
            black_box(output);
            legacy_samples.push(elapsed);
        }
    }

    let legacy_p95_ns = p95(legacy_samples);
    let optimized_p95_ns = p95(optimized_samples);
    println!(
        "PERF_RESULT runtime_interface03_focus_chain_partition \
             candidates={CANDIDATE_COUNT} sample_pairs={SAMPLE_PAIRS} \
             legacy_p95_ns={legacy_p95_ns} optimized_p95_ns={optimized_p95_ns} \
             legacy_sorted_candidates=10000 optimized_sorted_candidates=0"
    );
    assert!(
        optimized_p95_ns * 100 <= legacy_p95_ns * 35,
        "partitioned focus-chain finalization must be <=35% of legacy P95: \
             optimized={optimized_p95_ns}ns legacy={legacy_p95_ns}ns"
    );
}

#[test]
fn runtime_interface03_batch22_hashed_focus_visit_set_preserves_membership() {
    let node_ids = (0..VISIT_COUNT)
        .chain(0..VISIT_COUNT)
        .map(|index| UiNodeId::new(index as u64 + 1));
    let ordered = node_ids.clone().collect::<BTreeSet<_>>();
    let hashed = node_ids.collect::<HashSet<_>>();

    assert_eq!(hashed.len(), ordered.len());
    assert!((0..VISIT_COUNT)
        .map(|index| UiNodeId::new(index as u64 + 1))
        .all(|node_id| ordered.contains(&node_id) && hashed.contains(&node_id)));
}

#[test]
#[ignore = "release-only hashed focus visit-set benchmark"]
fn runtime_interface03_batch22_hashed_focus_visit_set_release_benchmark() {
    let node_ids = (0..VISIT_COUNT)
        .map(|index| UiNodeId::new(index as u64 + 1))
        .collect::<Vec<_>>();
    let mut ordered_samples = Vec::with_capacity(SAMPLE_PAIRS);
    let mut hashed_samples = Vec::with_capacity(SAMPLE_PAIRS);

    let measure_ordered = || {
        measure(|| {
            let mut visited = BTreeSet::new();
            for node_id in &node_ids {
                black_box(visited.insert(*node_id));
            }
            visited.len()
        })
    };
    let measure_hashed = || {
        measure(|| {
            let mut visited = HashSet::with_capacity(node_ids.len());
            for node_id in &node_ids {
                black_box(visited.insert(*node_id));
            }
            visited.len()
        })
    };
    for sample in 0..SAMPLE_PAIRS {
        if sample % 2 == 0 {
            let (elapsed, count) = measure_ordered();
            assert_eq!(count, VISIT_COUNT);
            ordered_samples.push(elapsed);
            let (elapsed, count) = measure_hashed();
            assert_eq!(count, VISIT_COUNT);
            hashed_samples.push(elapsed);
        } else {
            let (elapsed, count) = measure_hashed();
            assert_eq!(count, VISIT_COUNT);
            hashed_samples.push(elapsed);
            let (elapsed, count) = measure_ordered();
            assert_eq!(count, VISIT_COUNT);
            ordered_samples.push(elapsed);
        }
    }

    let ordered_p95_ns = p95(ordered_samples);
    let hashed_p95_ns = p95(hashed_samples);
    println!(
        "RUNTIME_INTERFACE03_HASHED_FOCUS_VISIT_SET_BENCH_V1 visits={VISIT_COUNT} sample_pairs={SAMPLE_PAIRS} ordered_p95_ns={ordered_p95_ns} hashed_p95_ns={hashed_p95_ns}"
    );
    assert!(
        hashed_p95_ns.saturating_mul(10) <= ordered_p95_ns.saturating_mul(7),
        "hashed focus visit-set must improve P95 by at least 30%: ordered={ordered_p95_ns}ns hashed={hashed_p95_ns}ns",
    );
}
