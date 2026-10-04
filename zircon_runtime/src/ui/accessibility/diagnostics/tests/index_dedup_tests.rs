use std::collections::{btree_map::Entry, BTreeMap, BTreeSet};
use std::hint::black_box;
use std::time::Instant;

use zircon_runtime_interface::ui::{
    accessibility::{
        UiAccessibilityDiagnosticCode, UiAccessibilityNode, UiAccessibilityTreeSnapshot,
    },
    event_ui::{UiNodeId, UiTreeId},
};

use super::validate_snapshot_bounded;

const SAMPLE_PAIRS: usize = 17;
const NODE_COUNT: usize = 4_096;

#[test]
fn diagnostic_index_preserves_duplicate_order_and_first_index() {
    let duplicate = UiNodeId::new(7);
    let mut snapshot = UiAccessibilityTreeSnapshot {
        tree_id: UiTreeId::new("runtime819-diagnostic-index"),
        nodes: vec![
            UiAccessibilityNode {
                node_id: duplicate,
                ..UiAccessibilityNode::default()
            },
            UiAccessibilityNode {
                node_id: duplicate,
                ..UiAccessibilityNode::default()
            },
            UiAccessibilityNode {
                node_id: UiNodeId::new(9),
                ..UiAccessibilityNode::default()
            },
        ],
        ..UiAccessibilityTreeSnapshot::default()
    };
    let mut observed = Vec::new();

    validate_snapshot_bounded(&mut snapshot, |count| {
        observed.push(count);
        Ok::<_, ()>(())
    })
    .expect("accessibility snapshot validation");

    assert_eq!(&observed[..3], &[0, 1, 0]);
    assert_eq!(observed.iter().sum::<usize>(), 1);
    assert_eq!(snapshot.diagnostics.len(), 1);
    assert_eq!(
        snapshot.diagnostics[0].code,
        UiAccessibilityDiagnosticCode::DuplicateNodeId
    );
    assert_eq!(snapshot.diagnostics[0].node_id, Some(duplicate));
}

#[test]
#[ignore = "managed Windows release performance evidence"]
fn runtime819_a11y_diagnostic_index_capacity_benchmark() {
    let node_ids = (0..NODE_COUNT)
        .map(|index| UiNodeId::new(index as u64))
        .collect::<Vec<_>>();
    let mut legacy_samples = Vec::with_capacity(SAMPLE_PAIRS);
    let mut optimized_samples = Vec::with_capacity(SAMPLE_PAIRS);
    for pair in 0..SAMPLE_PAIRS {
        if pair % 2 == 0 {
            legacy_samples.push(measure_index(&node_ids, false));
            optimized_samples.push(measure_index(&node_ids, true));
        } else {
            optimized_samples.push(measure_index(&node_ids, true));
            legacy_samples.push(measure_index(&node_ids, false));
        }
    }

    let legacy_p95 = percentile(&legacy_samples, 95);
    let optimized_p95 = percentile(&optimized_samples, 95);
    const MARKER: &str = "RUNTIME819_A11Y_DIAGNOSTIC_INDEX_BENCH_V1";
    println!(
        "{MARKER} node_count={NODE_COUNT} sample_pairs={SAMPLE_PAIRS} \
         legacy_index_allocations=2 optimized_index_allocations=1 \
         legacy_p95_ns={legacy_p95} optimized_p95_ns={optimized_p95} \
         legacy_ns={} optimized_ns={}",
        csv(&legacy_samples),
        csv(&optimized_samples),
    );
    assert!(optimized_p95 > 0);
}

fn measure_index(node_ids: &[UiNodeId], optimized: bool) -> u128 {
    let started = Instant::now();
    let mut checksum = 0usize;
    for _ in 0..8 {
        if optimized {
            let mut nodes = BTreeMap::new();
            for (index, node_id) in node_ids.iter().copied().enumerate() {
                if let Entry::Vacant(entry) = nodes.entry(node_id) {
                    entry.insert(index);
                }
            }
            checksum ^= nodes.len();
        } else {
            let mut seen = BTreeSet::new();
            let mut nodes = BTreeMap::new();
            for (index, node_id) in node_ids.iter().copied().enumerate() {
                if seen.insert(node_id) {
                    nodes.insert(node_id, index);
                }
            }
            checksum ^= seen.len() ^ nodes.len();
        }
    }
    black_box(checksum);
    started.elapsed().as_nanos().max(1)
}

fn percentile(samples: &[u128], percentile: usize) -> u128 {
    let mut sorted = samples.to_vec();
    sorted.sort_unstable();
    let rank = sorted.len().saturating_mul(percentile).div_ceil(100);
    sorted[rank.saturating_sub(1)]
}

fn csv(samples: &[u128]) -> String {
    samples
        .iter()
        .map(u128::to_string)
        .collect::<Vec<_>>()
        .join(",")
}
