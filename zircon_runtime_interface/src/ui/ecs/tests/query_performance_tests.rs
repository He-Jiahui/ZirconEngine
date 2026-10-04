use std::{hint::black_box, time::Instant};

use crate::ui::event_ui::{UiNodeId, UiNodePath};

use super::{
    UiEcsDirtyDomains, UiEcsProjectionChangeKind, UiEcsProjectionDelta, UiEcsProjectionNodeChange,
};

const CHANGE_COUNT: usize = 65_536;
const LOOKUP_COUNT: usize = 32;
const SAMPLE_COUNT: usize = 11;

fn delta() -> UiEcsProjectionDelta {
    UiEcsProjectionDelta {
        changes: (0..CHANGE_COUNT)
            .map(|index| UiEcsProjectionNodeChange {
                node_id: UiNodeId::new(index as u64 + 1),
                node_path: UiNodePath::new(format!("root/{index}")),
                kind: if index % 2 == 0 {
                    UiEcsProjectionChangeKind::Updated
                } else {
                    UiEcsProjectionChangeKind::Added
                },
                domains: UiEcsDirtyDomains::default(),
                reasons: Vec::new(),
            })
            .collect(),
        ..UiEcsProjectionDelta::default()
    }
}

#[test]
fn runtime_interface03_batch21_fused_change_id_query_preserves_order() {
    let delta = delta();
    for kind in [
        UiEcsProjectionChangeKind::Added,
        UiEcsProjectionChangeKind::Removed,
        UiEcsProjectionChangeKind::Updated,
    ] {
        assert_eq!(
            delta.node_ids_by_change_kind(kind),
            delta.node_ids_by_change_kind_staged(kind),
        );
    }
}

#[test]
fn runtime_interface03_batch23_binary_change_lookup_preserves_sorted_and_fallback_results() {
    let sorted = delta();
    for node_id in [
        UiNodeId::new(1),
        UiNodeId::new((CHANGE_COUNT / 2) as u64),
        UiNodeId::new(CHANGE_COUNT as u64),
        UiNodeId::new(CHANGE_COUNT as u64 + 1),
    ] {
        assert_eq!(sorted.change(node_id), sorted.change_linear(node_id));
    }

    let mut reordered = sorted.clone();
    reordered.changes.swap(0, CHANGE_COUNT - 1);
    for node_id in [
        UiNodeId::new(1),
        UiNodeId::new((CHANGE_COUNT / 2) as u64),
        UiNodeId::new(CHANGE_COUNT as u64),
        UiNodeId::new(CHANGE_COUNT as u64 + 1),
    ] {
        assert_eq!(reordered.change(node_id), reordered.change_linear(node_id));
    }
}

#[test]
#[ignore = "release-only fused ECS change-id query benchmark"]
fn runtime_interface03_batch21_fused_change_id_query_release_benchmark() {
    let delta = delta();
    let kind = UiEcsProjectionChangeKind::Updated;
    let mut staged_samples = Vec::with_capacity(SAMPLE_COUNT);
    let mut fused_samples = Vec::with_capacity(SAMPLE_COUNT);

    for sample in 0..SAMPLE_COUNT {
        let measure_staged = || {
            let started = Instant::now();
            for _ in 0..LOOKUP_COUNT {
                black_box(delta.node_ids_by_change_kind_staged(black_box(kind)));
            }
            started.elapsed().as_nanos()
        };
        let measure_fused = || {
            let started = Instant::now();
            for _ in 0..LOOKUP_COUNT {
                black_box(delta.node_ids_by_change_kind(black_box(kind)));
            }
            started.elapsed().as_nanos()
        };
        if sample % 2 == 0 {
            staged_samples.push(measure_staged());
            fused_samples.push(measure_fused());
        } else {
            fused_samples.push(measure_fused());
            staged_samples.push(measure_staged());
        }
    }

    staged_samples.sort_unstable();
    fused_samples.sort_unstable();
    let p95 = SAMPLE_COUNT - 1;
    eprintln!(
        "RUNTIME_INTERFACE03_FUSED_CHANGE_ID_QUERY_BENCH_V1 changes={CHANGE_COUNT} lookups={LOOKUP_COUNT} samples={SAMPLE_COUNT} staged_p95_ns={} fused_p95_ns={}",
        staged_samples[p95], fused_samples[p95],
    );
    assert!(
        fused_samples[p95].saturating_mul(5) <= staged_samples[p95].saturating_mul(4),
        "fused change-id query must improve P95 by at least 20%: staged={}ns fused={}ns",
        staged_samples[p95],
        fused_samples[p95],
    );
}

#[test]
#[ignore = "release-only binary ECS change lookup benchmark"]
fn runtime_interface03_batch23_binary_change_lookup_release_benchmark() {
    let delta = delta();
    let target = UiNodeId::new(CHANGE_COUNT as u64);
    let mut linear_samples = Vec::with_capacity(SAMPLE_COUNT);
    let mut binary_samples = Vec::with_capacity(SAMPLE_COUNT);

    for sample in 0..SAMPLE_COUNT {
        let measure_linear = || {
            let started = Instant::now();
            for _ in 0..LOOKUP_COUNT {
                black_box(delta.change_linear(black_box(target)));
            }
            started.elapsed().as_nanos()
        };
        let measure_binary = || {
            let started = Instant::now();
            for _ in 0..LOOKUP_COUNT {
                black_box(delta.change(black_box(target)));
            }
            started.elapsed().as_nanos()
        };
        if sample % 2 == 0 {
            linear_samples.push(measure_linear());
            binary_samples.push(measure_binary());
        } else {
            binary_samples.push(measure_binary());
            linear_samples.push(measure_linear());
        }
    }

    linear_samples.sort_unstable();
    binary_samples.sort_unstable();
    let p95 = SAMPLE_COUNT - 1;
    eprintln!(
        "RUNTIME_INTERFACE03_BINARY_ECS_CHANGE_LOOKUP_BENCH_V1 changes={CHANGE_COUNT} lookups={LOOKUP_COUNT} samples={SAMPLE_COUNT} linear_p95_ns={} binary_p95_ns={}",
        linear_samples[p95], binary_samples[p95],
    );
    assert!(
        binary_samples[p95].saturating_mul(5) <= linear_samples[p95],
        "binary change lookup must improve P95 by at least 80%: linear={}ns binary={}ns",
        linear_samples[p95],
        binary_samples[p95],
    );
}
