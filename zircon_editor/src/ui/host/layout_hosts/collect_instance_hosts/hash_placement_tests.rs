use std::collections::{BTreeMap, HashMap};
use std::hint::black_box;
use std::time::{Duration, Instant};

use crate::ui::workbench::layout::{DocumentNode, MainPageId, SplitAxis, TabStackLayout};
use crate::ui::workbench::view::{ViewHost, ViewInstanceId};

use super::super::collect_document_hosts::collect_document_hosts;

const BENCHMARK_PLACEMENT_COUNT: usize = 16_384;
const BENCHMARK_LOOKUP_COUNT: usize = 4_096;
const BENCHMARK_ITERATIONS: usize = 16;
const BENCHMARK_SAMPLES: usize = 17;

#[test]
fn optimization_batch_20260826ch_layout_host_hash_placements_preserve_last_host() {
    let repeated = ViewInstanceId::new("editor.scene#1");
    let first_only = ViewInstanceId::new("editor.inspector#1");
    let workspace = DocumentNode::SplitNode {
        axis: SplitAxis::Horizontal,
        ratio: 0.5,
        first: Box::new(DocumentNode::Tabs(TabStackLayout {
            tabs: vec![repeated.clone(), first_only.clone()],
            active_tab: Some(first_only.clone()),
        })),
        second: Box::new(DocumentNode::Tabs(TabStackLayout {
            tabs: vec![repeated.clone()],
            active_tab: Some(repeated.clone()),
        })),
    };
    let mut placements = HashMap::new();

    collect_document_hosts(&workspace, &mut placements, |path| {
        ViewHost::Document(MainPageId::new("main"), path)
    });

    assert_eq!(placements.len(), 2);
    assert_eq!(
        placements.get(&first_only),
        Some(&ViewHost::Document(MainPageId::new("main"), vec![0]))
    );
    assert_eq!(
        placements.get(&repeated),
        Some(&ViewHost::Document(MainPageId::new("main"), vec![1]))
    );
}

#[test]
fn optimization_batch_20260826ch_layout_host_hash_placements_have_no_order_projection() {
    let document_source = include_str!("../collect_document_hosts.rs");
    let instance_source = include_str!("../collect_instance_hosts.rs");

    assert!(document_source.contains("placements: &mut HashMap<ViewInstanceId, ViewHost>"));
    assert!(instance_source.contains(") -> HashMap<ViewInstanceId, ViewHost>"));
    assert!(instance_source.contains("let mut placements = HashMap::new()"));
    assert!(!document_source.contains("BTreeMap"));
    assert!(!instance_source.contains("BTreeMap"));
    assert!(!instance_source.contains("sort"));
}

#[test]
#[ignore = "release-only performance evidence"]
fn optimization_batch_20260826ch_layout_host_hash_placements_p95() {
    let placements = (0..BENCHMARK_PLACEMENT_COUNT)
        .map(|index| {
            ViewInstanceId::new(format!(
                "editor.workspace.instance.{index:05}.{}",
                "shared-prefix".repeat(6)
            ))
        })
        .collect::<Vec<_>>();
    let lookup_keys = placements
        .iter()
        .rev()
        .take(BENCHMARK_LOOKUP_COUNT)
        .cloned()
        .collect::<Vec<_>>();
    let mut legacy_samples = Vec::with_capacity(BENCHMARK_SAMPLES);
    let mut optimized_samples = Vec::with_capacity(BENCHMARK_SAMPLES);

    for sample in 0..BENCHMARK_SAMPLES {
        if sample % 2 == 0 {
            legacy_samples.push(measure_legacy(&placements, &lookup_keys));
            optimized_samples.push(measure_optimized(&placements, &lookup_keys));
        } else {
            optimized_samples.push(measure_optimized(&placements, &lookup_keys));
            legacy_samples.push(measure_legacy(&placements, &lookup_keys));
        }
    }

    let legacy_p50 = percentile(&mut legacy_samples, 50);
    let legacy_p95 = percentile(&mut legacy_samples, 95);
    let optimized_p50 = percentile(&mut optimized_samples, 50);
    let optimized_p95 = percentile(&mut optimized_samples, 95);
    let reduction_basis_points = 10_000_u128.saturating_sub(
        optimized_p95.as_nanos().saturating_mul(10_000) / legacy_p95.as_nanos().max(1),
    );
    eprintln!(
        "EDITOR13_LAYOUT_HOST_HASH_PLACEMENTS_BENCH_V1 samples={BENCHMARK_SAMPLES} \
iterations={BENCHMARK_ITERATIONS} placements={BENCHMARK_PLACEMENT_COUNT} \
lookups={BENCHMARK_LOOKUP_COUNT} legacy_p50_ns={} legacy_p95_ns={} \
optimized_p50_ns={} optimized_p95_ns={} reduction_basis_points={reduction_basis_points}",
        legacy_p50.as_nanos(),
        legacy_p95.as_nanos(),
        optimized_p50.as_nanos(),
        optimized_p95.as_nanos(),
    );
    assert!(
        optimized_p95.as_nanos().saturating_mul(100) <= legacy_p95.as_nanos().saturating_mul(70),
        "hash layout-host placement must reduce build-and-lookup P95 by at least 30%: \
legacy={legacy_p95:?}, optimized={optimized_p95:?}"
    );
}

fn measure_legacy(placements: &[ViewInstanceId], lookup_keys: &[ViewInstanceId]) -> Duration {
    measure_placements(placements, lookup_keys, |placements| {
        placements
            .iter()
            .cloned()
            .enumerate()
            .map(|(index, id)| (id, index))
            .collect::<BTreeMap<_, _>>()
    })
}

fn measure_optimized(placements: &[ViewInstanceId], lookup_keys: &[ViewInstanceId]) -> Duration {
    measure_placements(placements, lookup_keys, |placements| {
        placements
            .iter()
            .cloned()
            .enumerate()
            .map(|(index, id)| (id, index))
            .collect::<HashMap<_, _>>()
    })
}

fn measure_placements<M>(
    placements: &[ViewInstanceId],
    lookup_keys: &[ViewInstanceId],
    mut build: impl FnMut(&[ViewInstanceId]) -> M,
) -> Duration
where
    M: PlacementIndex,
{
    let started = Instant::now();
    let mut checksum = 0_usize;
    for _ in 0..BENCHMARK_ITERATIONS {
        let index = build(black_box(placements));
        for key in lookup_keys {
            checksum ^= index.placement(black_box(key)).unwrap_or_default();
        }
        black_box(index);
    }
    black_box(checksum);
    started.elapsed()
}

trait PlacementIndex {
    fn placement(&self, key: &ViewInstanceId) -> Option<usize>;
}

impl PlacementIndex for BTreeMap<ViewInstanceId, usize> {
    fn placement(&self, key: &ViewInstanceId) -> Option<usize> {
        self.get(key).copied()
    }
}

impl PlacementIndex for HashMap<ViewInstanceId, usize> {
    fn placement(&self, key: &ViewInstanceId) -> Option<usize> {
        self.get(key).copied()
    }
}

fn percentile(samples: &mut [Duration], percentile: usize) -> Duration {
    samples.sort_unstable();
    let index = (samples.len() - 1).saturating_mul(percentile) / 100;
    samples[index]
}
