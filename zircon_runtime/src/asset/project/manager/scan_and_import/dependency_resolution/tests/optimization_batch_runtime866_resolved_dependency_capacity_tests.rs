use std::hint::black_box;
use std::time::Instant;

use crate::asset::AssetUri;
use crate::core::resource::ResourceRegistry;

use super::resolve_dependencies;

const DEPENDENCY_COUNT: usize = 4_096;
const SAMPLE_PAIRS: usize = 101;

#[test]
fn runtime866_resolved_dependency_capacity_preserves_empty_and_missing_paths() {
    let registry = ResourceRegistry::default();
    let empty = resolve_dependencies(&[], &registry);
    assert!(empty.dependency_ids.is_empty());
    assert_eq!(empty.dependency_ids.capacity(), 0);
    assert!(empty.diagnostics.is_empty());
    assert_eq!(empty.diagnostics.capacity(), 0);

    let missing = (0..64)
        .map(|index| {
            AssetUri::parse(&format!("res://runtime866/missing-{index:03}.asset"))
                .expect("valid missing dependency URI")
        })
        .collect::<Vec<_>>();
    let resolved = resolve_dependencies(&missing, &registry);
    assert!(resolved.dependency_ids.is_empty());
    assert!(resolved.dependency_ids.capacity() >= missing.len());
    assert_eq!(resolved.diagnostics.len(), missing.len());
    assert!(resolved.diagnostics.capacity() >= missing.len());

    let source = include_str!("../../dependency_resolution.rs");
    let resolution = source
        .split("fn resolve_dependencies")
        .nth(1)
        .expect("dependency resolution owner must exist")
        .split("fn admit_resolved_dependency_id")
        .next()
        .expect("dependency resolution owner must stay bounded");
    assert!(resolution.contains("dependency_ids: Vec::with_capacity(dependencies.len())"));
    assert!(resolution.contains("if resolved.diagnostics.is_empty()"));
}

#[test]
#[ignore = "managed Windows release performance evidence"]
fn runtime866_resolved_dependency_output_capacity_benchmark() {
    let values = (0..DEPENDENCY_COUNT).collect::<Vec<_>>();
    assert_eq!(legacy_projection(&values), values);
    assert_eq!(optimized_projection(&values), values);

    let mut legacy = Vec::with_capacity(SAMPLE_PAIRS);
    let mut optimized = Vec::with_capacity(SAMPLE_PAIRS);
    for pair in 0..SAMPLE_PAIRS {
        if pair % 2 == 0 {
            legacy.push(measure(|| legacy_projection(&values)));
            optimized.push(measure(|| optimized_projection(&values)));
        } else {
            optimized.push(measure(|| optimized_projection(&values)));
            legacy.push(measure(|| legacy_projection(&values)));
        }
    }

    let legacy_p50 = percentile(&legacy, 50);
    let legacy_p95 = percentile(&legacy, 95);
    let legacy_p99 = percentile(&legacy, 99);
    let optimized_p50 = percentile(&optimized, 50);
    let optimized_p95 = percentile(&optimized, 95);
    let optimized_p99 = percentile(&optimized, 99);
    println!(
        "RUNTIME866_RESOLVED_DEPENDENCY_OUTPUT_CAPACITY_BENCH_V1 sample_pairs={SAMPLE_PAIRS} dependencies={DEPENDENCY_COUNT} legacy_growth_events=11 optimized_growth_events=0 legacy_p50_ns={legacy_p50} legacy_p95_ns={legacy_p95} legacy_p99_ns={legacy_p99} optimized_p50_ns={optimized_p50} optimized_p95_ns={optimized_p95} optimized_p99_ns={optimized_p99}"
    );
    assert_eq!(growth_events(DEPENDENCY_COUNT, 0), 11);
    assert_eq!(growth_events(DEPENDENCY_COUNT, DEPENDENCY_COUNT), 0);
}

fn legacy_projection(values: &[usize]) -> Vec<usize> {
    let mut projected = Vec::new();
    projected.extend_from_slice(values);
    projected
}

fn optimized_projection(values: &[usize]) -> Vec<usize> {
    let mut projected = Vec::with_capacity(values.len());
    projected.extend_from_slice(values);
    projected
}

fn growth_events(item_count: usize, initial_capacity: usize) -> usize {
    let mut capacity = initial_capacity;
    let mut events = 0;
    for length in 1..=item_count {
        if length > capacity {
            capacity = if capacity == 0 {
                4
            } else {
                capacity.saturating_mul(2)
            };
            events += 1;
        }
    }
    events
}

fn measure<T>(work: impl FnOnce() -> T) -> u128 {
    let started = Instant::now();
    black_box(work());
    started.elapsed().as_nanos().max(1)
}

fn percentile(samples: &[u128], percentile: usize) -> u128 {
    let mut sorted = samples.to_vec();
    sorted.sort_unstable();
    let rank = sorted.len().saturating_mul(percentile).div_ceil(100);
    sorted[rank.saturating_sub(1)]
}
