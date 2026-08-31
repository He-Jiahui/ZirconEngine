use std::hint::black_box;
use std::time::Instant;

use super::*;

const ACTIVE_DRAG_COUNT: usize = 2_048;
const REMOVED_NODE_COUNT: usize = 1_024;
const OPERATIONS_PER_SAMPLE: usize = 4;
const SAMPLE_PAIRS: usize = 21;

#[test]
fn optimization_batch_20260826hd_runtime250_preserves_pointer_drag_clear_contract() {
    let mut state = pointer_drag_state(128);
    let removed = (0..64)
        .map(|index| UiNodeId::new((index * 2) as u64))
        .collect::<Vec<_>>();

    state.clear_pointer_drags_for_nodes(&removed);

    assert_eq!(state.pointer_drags.len(), 64);
    assert!(
        removed
            .iter()
            .all(|node_id| !state.pointer_drags.contains_key(node_id))
    );
    assert!((0..64).all(|index| {
        state
            .pointer_drags
            .contains_key(&UiNodeId::new((index * 2 + 1) as u64))
    }));
}

#[test]
fn optimization_batch_20260826hd_runtime250_hashes_large_clear_sets_only() {
    let source = include_str!("../pointer_drag.rs");
    let start = source
        .find("pub fn clear_pointer_drags_for_nodes(")
        .expect("clear_pointer_drags_for_nodes function");
    let end = source[start..]
        .find("\n    }")
        .map(|offset| start + offset)
        .expect("function boundary");
    let body = &source[start..end];

    assert!(body.contains("POINTER_DRAG_HASH_CLEAR_THRESHOLD"));
    assert!(body.contains("HashSet::with_capacity"));
    assert!(body.contains("node_ids.contains(owner)"));
}

#[test]
#[ignore = "managed release performance evidence"]
fn optimization_batch_20260826hd_runtime250_pointer_drag_hash_clear_release_benchmark() {
    let base = pointer_drag_state(ACTIVE_DRAG_COUNT);
    let removed = (0..REMOVED_NODE_COUNT)
        .map(|index| UiNodeId::new(index as u64))
        .collect::<Vec<_>>();
    let mut expected = base.clone();
    legacy_clear_pointer_drags_for_nodes(&mut expected, &removed);
    let mut actual = base.clone();
    actual.clear_pointer_drags_for_nodes(&removed);
    assert_eq!(actual, expected);

    let mut legacy_ns = Vec::with_capacity(SAMPLE_PAIRS);
    let mut optimized_ns = Vec::with_capacity(SAMPLE_PAIRS);
    for sample_index in 0..SAMPLE_PAIRS {
        let legacy_inputs = (0..OPERATIONS_PER_SAMPLE)
            .map(|_| base.clone())
            .collect::<Vec<_>>();
        let optimized_inputs = (0..OPERATIONS_PER_SAMPLE)
            .map(|_| base.clone())
            .collect::<Vec<_>>();
        let measure_legacy = || {
            let started = Instant::now();
            for mut state in legacy_inputs {
                legacy_clear_pointer_drags_for_nodes(&mut state, black_box(&removed));
                black_box(state);
            }
            started.elapsed().as_nanos().max(1)
        };
        let measure_optimized = || {
            let started = Instant::now();
            for mut state in optimized_inputs {
                state.clear_pointer_drags_for_nodes(black_box(&removed));
                black_box(state);
            }
            started.elapsed().as_nanos().max(1)
        };
        if sample_index % 2 == 0 {
            legacy_ns.push(measure_legacy());
            optimized_ns.push(measure_optimized());
        } else {
            optimized_ns.push(measure_optimized());
            legacy_ns.push(measure_legacy());
        }
    }

    let legacy_p50_ns = percentile(&legacy_ns, 50);
    let legacy_p95_ns = percentile(&legacy_ns, 95);
    let optimized_p50_ns = percentile(&optimized_ns, 50);
    let optimized_p95_ns = percentile(&optimized_ns, 95);
    println!(
        "RUNTIME250_POINTER_DRAG_HASH_CLEAR_BENCH_V1 active_drags={ACTIVE_DRAG_COUNT} \
         removed_nodes={REMOVED_NODE_COUNT} operations_per_sample={OPERATIONS_PER_SAMPLE} \
         sample_pairs={SAMPLE_PAIRS} legacy_p50_ns={legacy_p50_ns} \
         legacy_p95_ns={legacy_p95_ns} optimized_p50_ns={optimized_p50_ns} \
         optimized_p95_ns={optimized_p95_ns} legacy_ns={} optimized_ns={}",
        samples(&legacy_ns),
        samples(&optimized_ns),
    );
    assert!(
        optimized_p95_ns.saturating_mul(100) <= legacy_p95_ns.saturating_mul(70),
        "optimized P95 {optimized_p95_ns}ns must be at most 70% of legacy P95 {legacy_p95_ns}ns"
    );
}

fn pointer_drag_state(count: usize) -> UiSurfaceInputState {
    let mut state = UiSurfaceInputState::default();
    for index in 0..count {
        state.begin_pointer_drag(UiNodeId::new(index as u64), UiPoint::default());
    }
    state
}

fn legacy_clear_pointer_drags_for_nodes(state: &mut UiSurfaceInputState, node_ids: &[UiNodeId]) {
    state
        .pointer_drags
        .retain(|owner, _| !node_ids.contains(owner));
}

fn percentile(samples: &[u128], percentile: usize) -> u128 {
    let mut ordered = samples.to_vec();
    ordered.sort_unstable();
    let rank = ordered.len().saturating_mul(percentile).div_ceil(100);
    ordered[rank.saturating_sub(1)]
}

fn samples(samples: &[u128]) -> String {
    samples
        .iter()
        .map(u128::to_string)
        .collect::<Vec<_>>()
        .join(",")
}
