use std::hint::black_box;
use std::sync::Arc;
use std::time::Instant;

const SCHEMA_KEY_COUNT: usize = 4_096;
const BENCHMARK_SAMPLE_COUNT: usize = 21;

#[test]
fn set_entries_borrows_schema_descriptor_and_clones_only_layout_arc() {
    let source = include_str!("../blackboard.rs");
    let set_entries = source
        .split("pub(super) fn set_entries(")
        .nth(1)
        .and_then(|body| body.split("pub(super) fn entries(").next())
        .expect("set_entries body");

    assert!(set_entries.contains("validate_blackboard_entries(Some(&schema.descriptor), &entries)"));
    assert!(set_entries.contains("Arc::clone(&schema.layout)"));
    assert!(
        !set_entries.contains(".find(|schema| schema.id == schema_id)\n                .cloned()")
    );
}

#[test]
#[ignore = "release-only performance evidence"]
fn blackboard_schema_layout_arc_clone_release_benchmark_evidence() {
    let schema = synthetic_schema();
    assert_eq!(legacy_checksum(&schema), layout_arc_checksum(&schema));
    let (legacy_samples, optimized_samples) = benchmark_paired_samples(
        || legacy_checksum(black_box(&schema)),
        || layout_arc_checksum(black_box(&schema)),
    );
    let legacy_p50 = percentile(&legacy_samples, 50);
    let legacy_p95 = percentile(&legacy_samples, 95);
    let optimized_p50 = percentile(&optimized_samples, 50);
    let optimized_p95 = percentile(&optimized_samples, 95);
    let legacy_ns = benchmark_samples_csv(&legacy_samples);
    let optimized_ns = benchmark_samples_csv(&optimized_samples);

    println!(
        "PERF_RESULT plugins15_blackboard_schema_layout_arc_clone schema_keys={SCHEMA_KEY_COUNT} samples={BENCHMARK_SAMPLE_COUNT} sample_pairs={BENCHMARK_SAMPLE_COUNT} sample_order=alternating percentile_method=nearest_rank legacy_descriptor_string_clones_per_sample={SCHEMA_KEY_COUNT} optimized_descriptor_string_clones_per_sample=0 legacy_layout_arc_clones_per_sample=1 optimized_layout_arc_clones_per_sample=1 legacy_p50_ns={legacy_p50} legacy_p95_ns={legacy_p95} optimized_p50_ns={optimized_p50} optimized_p95_ns={optimized_p95} legacy_ns={legacy_ns} optimized_ns={optimized_ns}"
    );
    assert!(
        optimized_p95 * 20 <= legacy_p95,
        "optimized P95 {optimized_p95}ns must be no more than 5% of legacy P95 {legacy_p95}ns"
    );
}

#[derive(Clone)]
struct SyntheticSchema {
    descriptor: Vec<String>,
    layout: Arc<usize>,
}

fn synthetic_schema() -> SyntheticSchema {
    SyntheticSchema {
        descriptor: (0..SCHEMA_KEY_COUNT)
            .map(|index| format!("blackboard.schema.key.{index:04}.{}", "x".repeat(48)))
            .collect(),
        layout: Arc::new(73),
    }
}

fn legacy_checksum(schema: &SyntheticSchema) -> usize {
    let cloned = schema.clone();
    black_box(&cloned);
    cloned.descriptor.len() + *cloned.layout
}

fn layout_arc_checksum(schema: &SyntheticSchema) -> usize {
    let layout = Arc::clone(&schema.layout);
    black_box(&layout);
    schema.descriptor.len() + *layout
}

fn benchmark_paired_samples(
    mut legacy: impl FnMut() -> usize,
    mut optimized: impl FnMut() -> usize,
) -> (Vec<u128>, Vec<u128>) {
    black_box(legacy());
    black_box(optimized());
    let mut legacy_samples = Vec::with_capacity(BENCHMARK_SAMPLE_COUNT);
    let mut optimized_samples = Vec::with_capacity(BENCHMARK_SAMPLE_COUNT);
    for sample_index in 0..BENCHMARK_SAMPLE_COUNT {
        if sample_index % 2 == 0 {
            legacy_samples.push(benchmark_sample(&mut legacy));
            optimized_samples.push(benchmark_sample(&mut optimized));
        } else {
            optimized_samples.push(benchmark_sample(&mut optimized));
            legacy_samples.push(benchmark_sample(&mut legacy));
        }
    }
    (legacy_samples, optimized_samples)
}

fn benchmark_sample(operation: &mut impl FnMut() -> usize) -> u128 {
    let started = Instant::now();
    black_box(operation());
    started.elapsed().as_nanos()
}

fn benchmark_samples_csv(samples: &[u128]) -> String {
    samples
        .iter()
        .map(u128::to_string)
        .collect::<Vec<_>>()
        .join(",")
}

fn percentile(samples: &[u128], percentile: usize) -> u128 {
    let mut sorted = samples.to_vec();
    sorted.sort_unstable();
    assert!(!sorted.is_empty());
    assert!((1..=100).contains(&percentile));
    let index = (sorted.len() * percentile).div_ceil(100) - 1;
    sorted[index]
}
