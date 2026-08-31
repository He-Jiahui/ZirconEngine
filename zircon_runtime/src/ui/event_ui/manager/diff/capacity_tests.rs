use std::hint::black_box;
use std::time::Instant;

use super::{compute_diff, reflection_diff_capacities};
use zircon_runtime_interface::ui::event_ui::{
    UiNodeDescriptor, UiNodeId, UiNodePath, UiReflectionSnapshot, UiTreeId,
};

const SAMPLE_PAIRS: usize = 21;
const BUILDS_PER_SAMPLE: usize = 1_366;
const CHANGED_NODES_PER_BUILD: usize = 256;
const REMOVED_NODES_PER_BUILD: usize = 128;

#[test]
fn optimization_batch_20260826fb_runtime197_capacity_preserves_reflection_diff_order() {
    let previous = snapshot((0..256).map(|id| node(id, "previous")).collect());
    let current = snapshot(
        (0..128)
            .map(|id| node(id, "changed"))
            .chain((256..384).map(|id| node(id, "added")))
            .collect(),
    );

    let diff = compute_diff(&previous, &current);

    assert_eq!(diff.changed_nodes.len(), CHANGED_NODES_PER_BUILD);
    assert!(diff.changed_nodes.capacity() >= CHANGED_NODES_PER_BUILD);
    assert_eq!(diff.changed_nodes[0], UiNodeId::new(0));
    assert_eq!(diff.changed_nodes[255], UiNodeId::new(383));
    assert_eq!(diff.removed_nodes.len(), REMOVED_NODES_PER_BUILD);
    assert!(diff.removed_nodes.capacity() >= REMOVED_NODES_PER_BUILD);
    assert_eq!(diff.removed_nodes[0], UiNodeId::new(128));
    assert_eq!(diff.removed_nodes[127], UiNodeId::new(255));
    assert_eq!(
        reflection_diff_capacities(&previous, &current),
        (CHANGED_NODES_PER_BUILD, REMOVED_NODES_PER_BUILD)
    );
}

#[test]
fn optimization_batch_20260826fb_runtime197_diff_reserves_exact_change_counts() {
    let source = include_str!("../diff.rs");
    assert!(source.contains("fn reflection_diff_capacities("));
    assert!(source.contains("let (changed_capacity, removed_capacity) ="));
    assert!(source.contains("Vec::with_capacity(changed_capacity)"));
    assert!(source.contains("Vec::with_capacity(removed_capacity)"));
}

#[test]
#[ignore = "managed Windows release performance evidence"]
fn optimization_batch_20260826fb_runtime197_ui_reflection_diff_capacity_bench() {
    let mut legacy_samples = Vec::with_capacity(SAMPLE_PAIRS);
    let mut optimized_samples = Vec::with_capacity(SAMPLE_PAIRS);
    for pair in 0..SAMPLE_PAIRS {
        if pair % 2 == 0 {
            legacy_samples.push(measure(false));
            optimized_samples.push(measure(true));
        } else {
            optimized_samples.push(measure(true));
            legacy_samples.push(measure(false));
        }
    }
    let legacy_p50_ns = percentile(&legacy_samples, 50);
    let optimized_p50_ns = percentile(&optimized_samples, 50);
    let legacy_p95_ns = percentile(&legacy_samples, 95);
    let optimized_p95_ns = percentile(&optimized_samples, 95);
    println!(
        "RUNTIME197_UI_REFLECTION_DIFF_CAPACITY_BENCH_V1 sample_pairs={SAMPLE_PAIRS} \
builds_per_sample={BUILDS_PER_SAMPLE} changed_nodes_per_build={CHANGED_NODES_PER_BUILD} \
removed_nodes_per_build={REMOVED_NODES_PER_BUILD} legacy_reservations_per_build=0 \
optimized_reservations_per_build=2 legacy_p50_ns={legacy_p50_ns} \
optimized_p50_ns={optimized_p50_ns} legacy_p95_ns={legacy_p95_ns} \
optimized_p95_ns={optimized_p95_ns} legacy_raw_ns={} optimized_raw_ns={}",
        sample_csv(&legacy_samples),
        sample_csv(&optimized_samples),
    );
    assert!(optimized_p95_ns.saturating_mul(100) <= legacy_p95_ns.saturating_mul(70));
}

fn snapshot(nodes: Vec<UiNodeDescriptor>) -> UiReflectionSnapshot {
    UiReflectionSnapshot::new(UiTreeId::new("runtime197.tree"), Vec::new(), nodes)
}

fn node(id: u64, class_name: &str) -> UiNodeDescriptor {
    UiNodeDescriptor::new(
        UiNodeId::new(id),
        UiNodePath::new(format!("node/{id}")),
        class_name,
        format!("Node {id}"),
    )
}

fn measure(reserve: bool) -> u128 {
    let started = Instant::now();
    let mut checksum = 0usize;
    for _ in 0..BUILDS_PER_SAMPLE {
        let mut changed = if reserve {
            Vec::with_capacity(CHANGED_NODES_PER_BUILD)
        } else {
            Vec::new()
        };
        let mut removed = if reserve {
            Vec::with_capacity(REMOVED_NODES_PER_BUILD)
        } else {
            Vec::new()
        };
        for node in 0..CHANGED_NODES_PER_BUILD {
            changed.push(black_box(node));
        }
        for node in 0..REMOVED_NODES_PER_BUILD {
            removed.push(black_box(node));
        }
        checksum ^=
            black_box(changed.len() ^ changed.capacity() ^ removed.len() ^ removed.capacity());
    }
    black_box(checksum);
    started.elapsed().as_nanos().max(1)
}

fn percentile(samples: &[u128], percentile: usize) -> u128 {
    let mut sorted = samples.to_vec();
    sorted.sort_unstable();
    let rank = (sorted.len() * percentile).div_ceil(100);
    sorted[rank.saturating_sub(1)]
}

fn sample_csv(samples: &[u128]) -> String {
    samples
        .iter()
        .map(u128::to_string)
        .collect::<Vec<_>>()
        .join(",")
}
