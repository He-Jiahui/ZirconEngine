use std::hint::black_box;
use std::sync::Arc;
use std::time::{Duration, Instant};

use crate::core::editor_extension::{
    ViewportOverlayProvider, ViewportOverlayProviderContext, ViewportOverlayProviderRegistration,
};
use crate::core::extension::{
    ContributionBatch, ContributionSource, ContributionStore, PluginContributionId,
};
use zircon_runtime::core::framework::render::{SceneGizmoKind, SceneGizmoOverlayExtract};
use zircon_runtime::scene::Scene;

use super::ViewportOverlayProviderRegistry;

const SAMPLE_PAIRS: usize = 101;
const EXTRACTS_PER_SAMPLE: usize = 1_024;

struct FixedOverlayProvider {
    owner: u64,
    count: usize,
}

impl ViewportOverlayProvider for FixedOverlayProvider {
    fn extract(
        &self,
        _context: &ViewportOverlayProviderContext<'_>,
    ) -> Vec<SceneGizmoOverlayExtract> {
        vec![
            SceneGizmoOverlayExtract {
                owner: self.owner,
                kind: SceneGizmoKind::NavigationMesh,
                selected: false,
                lines: Vec::new(),
                wire_shapes: Vec::new(),
                icons: Vec::new(),
                pick_shapes: Vec::new(),
            };
            self.count
        ]
    }
}

fn registration(
    provider_id: &str,
    owner: u64,
    count: usize,
) -> ViewportOverlayProviderRegistration {
    ViewportOverlayProviderRegistration::new(provider_id, move || {
        Arc::new(FixedOverlayProvider { owner, count }) as Arc<dyn ViewportOverlayProvider>
    })
}

#[test]
fn editor898_viewport_overlay_provider_order_and_empty_capacity() {
    let scene = Scene::new();
    assert_eq!(
        ViewportOverlayProviderRegistry::default()
            .extract(&scene, None)
            .capacity(),
        0
    );

    let mut store = ContributionStore::default();
    let source = ContributionSource::Plugin(PluginContributionId::parse("test").unwrap());
    let ticket = store
        .contribute(source.clone(), ContributionBatch::default())
        .unwrap();
    let mut registry = ViewportOverlayProviderRegistry::default()
        .prepare_contribution(
            ticket,
            source,
            "test",
            [
                registration("c.second", 22, 32),
                registration("a.empty", 1, 0),
                registration("d.last", 33, 8),
                registration("b.first", 11, 16),
            ],
        )
        .unwrap();
    for id in ["a.empty", "b.first", "c.second", "d.last"] {
        assert!(registry.toggle(id).unwrap());
    }

    let output = registry.extract(&scene, None);
    let mut expected = vec![11_u64; 16];
    expected.extend([22_u64; 32]);
    expected.extend([33_u64; 8]);
    assert_eq!(
        output.iter().map(|gizmo| gizmo.owner).collect::<Vec<_>>(),
        expected
    );
    assert!(output.capacity() >= expected.len());
}

#[test]
fn editor898_batch_append_matches_flatten_for_sparse_and_dense_sources() {
    for batches in [
        vec![],
        vec![vec![]],
        vec![vec![], vec![1, 2, 3], vec![], vec![4]],
        vec![vec![0; 16], vec![1; 32], vec![2; 8]],
    ] {
        assert_eq!(optimized_extract(&batches), legacy_extract(&batches));
    }
}

#[test]
#[ignore = "release percentile evidence; run through the managed Windows validation lane"]
fn editor898_viewport_overlay_batch_append_release_percentiles() {
    let batches = (0..8).map(|index| vec![index; 16]).collect::<Vec<_>>();
    let mut legacy = Vec::with_capacity(SAMPLE_PAIRS);
    let mut optimized = Vec::with_capacity(SAMPLE_PAIRS);
    for pair in 0..SAMPLE_PAIRS {
        if pair % 2 == 0 {
            legacy.push(measure(&batches, legacy_extract));
            optimized.push(measure(&batches, optimized_extract));
        } else {
            optimized.push(measure(&batches, optimized_extract));
            legacy.push(measure(&batches, legacy_extract));
        }
    }
    let legacy_p50_ns = percentile(&mut legacy.clone(), 50);
    let legacy_p95_ns = percentile(&mut legacy.clone(), 95);
    let legacy_p99_ns = percentile(&mut legacy, 99);
    let optimized_p50_ns = percentile(&mut optimized.clone(), 50);
    let optimized_p95_ns = percentile(&mut optimized.clone(), 95);
    let optimized_p99_ns = percentile(&mut optimized, 99);
    println!(
        "EDITOR898_VIEWPORT_OVERLAY_BATCH_APPEND_BENCH_V1 legacy_p50_ns={legacy_p50_ns} legacy_p95_ns={legacy_p95_ns} legacy_p99_ns={legacy_p99_ns} optimized_p50_ns={optimized_p50_ns} optimized_p95_ns={optimized_p95_ns} optimized_p99_ns={optimized_p99_ns}"
    );
    assert!(optimized_p95_ns <= legacy_p95_ns.saturating_mul(110).div_ceil(100));
}

fn legacy_extract(batches: &[Vec<usize>]) -> Vec<usize> {
    batches.iter().flat_map(|batch| batch.clone()).collect()
}

fn optimized_extract(batches: &[Vec<usize>]) -> Vec<usize> {
    let mut collected = Vec::new();
    for batch in batches {
        let gizmos = batch.clone();
        if !gizmos.is_empty() {
            collected.reserve(gizmos.len());
            collected.extend(gizmos);
        }
    }
    collected
}

fn measure(batches: &[Vec<usize>], extract: fn(&[Vec<usize>]) -> Vec<usize>) -> Duration {
    let started = Instant::now();
    let checksum = (0..EXTRACTS_PER_SAMPLE)
        .map(|_| black_box(extract(black_box(batches))).len())
        .sum::<usize>();
    black_box(checksum);
    started.elapsed()
}

fn percentile(samples: &mut [Duration], percent: usize) -> u128 {
    samples.sort_unstable();
    samples[(samples.len() * percent).div_ceil(100).saturating_sub(1)].as_nanos()
}
