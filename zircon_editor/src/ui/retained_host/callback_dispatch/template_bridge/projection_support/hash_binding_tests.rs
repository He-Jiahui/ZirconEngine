use std::collections::{BTreeMap, HashMap};
use std::hint::black_box;
use std::time::{Duration, Instant};

use super::BindingIndex;

const BENCHMARK_BINDING_COUNT: usize = 16_384;
const BENCHMARK_LOOKUP_COUNT: usize = 4_096;
const BENCHMARK_ITERATIONS: usize = 32;
const BENCHMARK_SAMPLES: usize = 17;

#[test]
fn optimization_batch_20260826ci_surface_binding_hash_index_matches_ordered_lookup() {
    let ordered = BTreeMap::from([
        ("editor.open".to_string(), 3_u64),
        ("editor.save".to_string(), 7_u64),
    ]);
    let hashed = ordered.clone().into_iter().collect::<HashMap<_, _>>();

    assert_eq!(ordered.binding_by_id("editor.open"), Some(&3));
    assert_eq!(hashed.binding_by_id("editor.open"), Some(&3));
    assert_eq!(ordered.binding_by_id("editor.save"), Some(&7));
    assert_eq!(hashed.binding_by_id("editor.save"), Some(&7));
    assert_eq!(ordered.binding_by_id("editor.missing"), None);
    assert_eq!(hashed.binding_by_id("editor.missing"), None);
}

#[test]
fn optimization_batch_20260826ci_surface_binding_hash_index_keeps_workbench_order_owner() {
    let source = include_str!("../projection_support.rs");
    let asset_source = include_str!("../asset_surface/bridge.rs");
    let inspector_source = include_str!("../inspector_surface/bridge.rs");
    let pane_source = include_str!("../pane_surface/bridge.rs");
    let welcome_source = include_str!("../welcome_surface/bridge.rs");

    assert!(source.contains("pub(crate) trait BindingIndex<V>"));
    assert!(source.contains("impl<V> BindingIndex<V> for BTreeMap<String, V>"));
    assert!(source.contains("impl<V> BindingIndex<V> for HashMap<String, V>"));
    assert!(source.contains("pub(crate) fn build_bindings_by_id("));
    assert!(source.contains("pub(crate) fn build_surface_bindings_by_id("));
    assert!(source.contains(".collect::<HashMap<_, _>>()"));
    for bridge_source in [asset_source, inspector_source, pane_source, welcome_source] {
        assert!(bridge_source.contains("bindings_by_id: HashMap<String, EditorUiBinding>"));
        assert!(!bridge_source.contains("BTreeMap"));
    }
}

#[test]
#[ignore = "release-only performance evidence"]
fn optimization_batch_20260826ci_surface_binding_hash_index_p95() {
    let bindings = (0..BENCHMARK_BINDING_COUNT)
        .map(|index| {
            (
                format!(
                    "editor.surface.binding.{index:05}.{}",
                    "shared-route-prefix".repeat(5)
                ),
                index as u64,
            )
        })
        .collect::<Vec<_>>();
    let lookup_keys = bindings
        .iter()
        .rev()
        .take(BENCHMARK_LOOKUP_COUNT)
        .map(|(key, _)| key.clone())
        .collect::<Vec<_>>();
    let mut legacy_samples = Vec::with_capacity(BENCHMARK_SAMPLES);
    let mut optimized_samples = Vec::with_capacity(BENCHMARK_SAMPLES);

    for sample in 0..BENCHMARK_SAMPLES {
        if sample % 2 == 0 {
            legacy_samples.push(measure_legacy(&bindings, &lookup_keys));
            optimized_samples.push(measure_optimized(&bindings, &lookup_keys));
        } else {
            optimized_samples.push(measure_optimized(&bindings, &lookup_keys));
            legacy_samples.push(measure_legacy(&bindings, &lookup_keys));
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
        "EDITOR01_SURFACE_BINDING_HASH_INDEX_BENCH_V1 samples={BENCHMARK_SAMPLES} \
iterations={BENCHMARK_ITERATIONS} bindings={BENCHMARK_BINDING_COUNT} \
lookups={BENCHMARK_LOOKUP_COUNT} legacy_p50_ns={} legacy_p95_ns={} \
optimized_p50_ns={} optimized_p95_ns={} reduction_basis_points={reduction_basis_points}",
        legacy_p50.as_nanos(),
        legacy_p95.as_nanos(),
        optimized_p50.as_nanos(),
        optimized_p95.as_nanos(),
    );
    assert!(
        optimized_p95.as_nanos().saturating_mul(100) <= legacy_p95.as_nanos().saturating_mul(70),
        "hash surface binding lookup must reduce build-and-lookup P95 by at least 30%: \
legacy={legacy_p95:?}, optimized={optimized_p95:?}"
    );
}

fn measure_legacy(bindings: &[(String, u64)], lookup_keys: &[String]) -> Duration {
    measure_index(bindings, lookup_keys, |bindings| {
        bindings.iter().cloned().collect::<BTreeMap<_, _>>()
    })
}

fn measure_optimized(bindings: &[(String, u64)], lookup_keys: &[String]) -> Duration {
    measure_index(bindings, lookup_keys, |bindings| {
        bindings.iter().cloned().collect::<HashMap<_, _>>()
    })
}

fn measure_index<M>(
    bindings: &[(String, u64)],
    lookup_keys: &[String],
    mut build: impl FnMut(&[(String, u64)]) -> M,
) -> Duration
where
    M: BindingIndex<u64>,
{
    let started = Instant::now();
    let mut checksum = 0_u64;
    for _ in 0..BENCHMARK_ITERATIONS {
        let index = build(black_box(bindings));
        for key in lookup_keys {
            checksum ^= index
                .binding_by_id(black_box(key))
                .copied()
                .unwrap_or_default();
        }
        black_box(index);
    }
    black_box(checksum);
    started.elapsed()
}

fn percentile(samples: &mut [Duration], percentile: usize) -> Duration {
    samples.sort_unstable();
    let index = (samples.len() - 1).saturating_mul(percentile) / 100;
    samples[index]
}
