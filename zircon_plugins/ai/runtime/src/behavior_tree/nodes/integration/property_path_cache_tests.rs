use std::hint::black_box;
use std::time::Instant;

use zircon_runtime::core::framework::navigation::NAV_MESH_AGENT_COMPONENT_TYPE;
use zircon_runtime::core::framework::scene::ComponentPropertyPath;

use super::nav_destination_property_path;

const BENCHMARK_LOOKUP_COUNT: usize = 4_096;
const BENCHMARK_SAMPLE_COUNT: usize = 21;

#[test]
fn nav_destination_property_path_is_initialized_once_with_expected_segments() {
    let first = nav_destination_property_path().expect("valid static path");
    let second = nav_destination_property_path().expect("cached static path");

    assert!(std::ptr::eq(first, second));
    assert_eq!(first.component(), NAV_MESH_AGENT_COMPONENT_TYPE);
    assert_eq!(first.property_segments(), &["destination".to_string()]);
    assert_eq!(
        first.as_str(),
        format!("{NAV_MESH_AGENT_COMPONENT_TYPE}.destination")
    );
}

#[test]
fn nav_target_write_uses_cached_component_property_path() {
    let source = include_str!("../integration.rs");
    let write = source
        .split("fn write_nav_target(")
        .nth(1)
        .and_then(|body| body.split("fn clear_nav_target(").next())
        .expect("write_nav_target body");

    assert!(source.contains("OnceLock<Result<ComponentPropertyPath, String>>"));
    assert!(write.contains("nav_destination_property_path()?"));
    assert!(!write.contains("ComponentPropertyPath::new("));
}

#[test]
#[ignore = "release-only performance evidence"]
fn cached_nav_property_path_release_benchmark_evidence() {
    nav_destination_property_path().expect("prewarm cached path");
    assert_eq!(legacy_checksum(), cached_checksum());
    let (legacy_samples, optimized_samples) =
        benchmark_paired_samples(legacy_checksum, cached_checksum);
    let legacy_p50 = percentile(&legacy_samples, 50);
    let legacy_p95 = percentile(&legacy_samples, 95);
    let optimized_p50 = percentile(&optimized_samples, 50);
    let optimized_p95 = percentile(&optimized_samples, 95);
    let legacy_ns = benchmark_samples_csv(&legacy_samples);
    let optimized_ns = benchmark_samples_csv(&optimized_samples);

    println!(
        "PERF_RESULT plugins15_cached_nav_property_path lookups={BENCHMARK_LOOKUP_COUNT} samples={BENCHMARK_SAMPLE_COUNT} sample_pairs={BENCHMARK_SAMPLE_COUNT} sample_order=alternating percentile_method=nearest_rank legacy_path_constructions_per_sample={BENCHMARK_LOOKUP_COUNT} optimized_path_constructions_per_sample=0 legacy_p50_ns={legacy_p50} legacy_p95_ns={legacy_p95} optimized_p50_ns={optimized_p50} optimized_p95_ns={optimized_p95} legacy_ns={legacy_ns} optimized_ns={optimized_ns}"
    );
    assert!(
        optimized_p95 * 10 <= legacy_p95,
        "optimized P95 {optimized_p95}ns must be no more than 10% of legacy P95 {legacy_p95}ns"
    );
}

fn legacy_checksum() -> usize {
    let mut checksum = 0_usize;
    for _ in 0..BENCHMARK_LOOKUP_COUNT {
        let path = ComponentPropertyPath::new(
            NAV_MESH_AGENT_COMPONENT_TYPE,
            vec!["destination".to_string()],
        )
        .expect("valid benchmark path");
        checksum += black_box(path.as_str().len());
    }
    checksum
}

fn cached_checksum() -> usize {
    let mut checksum = 0_usize;
    for _ in 0..BENCHMARK_LOOKUP_COUNT {
        let path = nav_destination_property_path().expect("cached benchmark path");
        checksum += black_box(path.as_str().len());
    }
    checksum
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
