use std::collections::{BTreeMap, HashMap};
use std::hint::black_box;
use std::time::Instant;

use serde_json::json;

use super::*;

const SAMPLE_PAIRS: usize = 17;
const LOOKUPS_PER_SAMPLE: usize = 4_096;

#[test]
fn optimization_batch_20260826cd_ui_delta_hash_coalescing_preserves_sorted_flush() {
    let mut queue = EditorUiDeltaQueue::default();
    let view = ViewInstanceId::new("workbench.root");
    for (path, value) in [
        ("editor/workbench/z-last", 1),
        ("editor/workbench/a-first", 2),
        ("editor/workbench/m-middle", 3),
        ("editor/workbench/m-middle", 4),
    ] {
        queue.push_patch(
            view.clone(),
            UiReflectionNodePatch::new(UiNodePath::new(path)).with_property("value", json!(value)),
        );
    }

    let batch = queue.drain();
    let patches = batch.reflection_patches();
    assert_eq!(
        patches
            .iter()
            .map(|patch| patch.node_path.0.as_str())
            .collect::<Vec<_>>(),
        vec![
            "editor/workbench/a-first",
            "editor/workbench/m-middle",
            "editor/workbench/z-last",
        ]
    );
    assert_eq!(patches[1].properties["value"], json!(4));
}

#[test]
fn optimization_batch_20260826cd_ui_delta_hash_coalescing_keeps_order_at_flush_only() {
    let source = include_str!("../editor_ui_delta.rs");

    assert!(source.contains("pending: HashMap<UiNodePath, EditorUiNodeDelta>"));
    assert!(source.contains("let mut deltas = std::mem::take(&mut self.pending)"));
    assert!(source.contains("deltas.sort_unstable_by"));
    assert!(source.contains("left.patch.node_path.cmp(&right.patch.node_path)"));
}

#[test]
#[ignore = "release performance evidence"]
fn optimization_batch_20260826cd_ui_delta_hash_coalescing_p95() {
    const PATHS: usize = 16_384;
    let paths = (0..PATHS)
        .map(|index| {
            UiNodePath::new(format!(
                "editor/workbench/reflection/shared/component/path/{index:05}"
            ))
        })
        .collect::<Vec<_>>();
    let legacy = paths
        .iter()
        .cloned()
        .enumerate()
        .map(|(index, path)| (path, index))
        .collect::<BTreeMap<_, _>>();
    let optimized = paths
        .iter()
        .cloned()
        .enumerate()
        .map(|(index, path)| (path, index))
        .collect::<HashMap<_, _>>();
    let target = paths.last().expect("benchmark path set must not be empty");

    let mut legacy_lookup = || repeated_lookup(&legacy, target);
    let mut optimized_lookup = || repeated_lookup(&optimized, target);
    assert_eq!(black_box(legacy_lookup()), black_box(optimized_lookup()));

    let mut legacy_ns = Vec::with_capacity(SAMPLE_PAIRS);
    let mut optimized_ns = Vec::with_capacity(SAMPLE_PAIRS);
    for sample_index in 0..SAMPLE_PAIRS {
        if sample_index % 2 == 0 {
            legacy_ns.push(measure_ns(&mut legacy_lookup));
            optimized_ns.push(measure_ns(&mut optimized_lookup));
        } else {
            optimized_ns.push(measure_ns(&mut optimized_lookup));
            legacy_ns.push(measure_ns(&mut legacy_lookup));
        }
    }

    let legacy_p50_ns = nearest_rank(&legacy_ns, 50);
    let legacy_p95_ns = nearest_rank(&legacy_ns, 95);
    let optimized_p50_ns = nearest_rank(&optimized_ns, 50);
    let optimized_p95_ns = nearest_rank(&optimized_ns, 95);
    assert!(
        optimized_p95_ns.saturating_mul(10) <= legacy_p95_ns.saturating_mul(7),
        "UI delta hash coalescing lookup P95 must be at least 30% below BTreeMap: legacy={legacy_p95_ns}ns optimized={optimized_p95_ns}ns"
    );

    println!(
        "EDITOR48_UI_DELTA_HASH_COALESCING_BENCH_V1 paths={PATHS} lookups_per_sample={LOOKUPS_PER_SAMPLE} sample_pairs={SAMPLE_PAIRS} pair_order=alternating_legacy_even legacy_first_pairs=9 optimized_first_pairs=8 legacy_p50_ns={legacy_p50_ns} legacy_p95_ns={legacy_p95_ns} optimized_p50_ns={optimized_p50_ns} optimized_p95_ns={optimized_p95_ns} legacy_ns={} optimized_ns={}",
        join_samples(&legacy_ns),
        join_samples(&optimized_ns),
    );
}

fn repeated_lookup<M>(map: &M, key: &UiNodePath) -> usize
where
    M: LookupMap,
{
    let mut checksum = 0usize;
    for _ in 0..LOOKUPS_PER_SAMPLE {
        checksum = checksum.wrapping_add(black_box(map.lookup(black_box(key))));
    }
    black_box(checksum)
}

trait LookupMap {
    fn lookup(&self, key: &UiNodePath) -> usize;
}

impl LookupMap for BTreeMap<UiNodePath, usize> {
    fn lookup(&self, key: &UiNodePath) -> usize {
        *self.get(key).expect("legacy benchmark path must exist")
    }
}

impl LookupMap for HashMap<UiNodePath, usize> {
    fn lookup(&self, key: &UiNodePath) -> usize {
        *self.get(key).expect("optimized benchmark path must exist")
    }
}

fn measure_ns(operation: &mut impl FnMut() -> usize) -> u128 {
    let started = Instant::now();
    black_box(operation());
    started.elapsed().as_nanos()
}

fn nearest_rank(samples: &[u128], percentile: usize) -> u128 {
    let mut sorted = samples.to_vec();
    sorted.sort_unstable();
    let rank = sorted.len().saturating_mul(percentile).div_ceil(100);
    sorted[rank.saturating_sub(1)]
}

fn join_samples(samples: &[u128]) -> String {
    samples
        .iter()
        .map(u128::to_string)
        .collect::<Vec<_>>()
        .join(",")
}
