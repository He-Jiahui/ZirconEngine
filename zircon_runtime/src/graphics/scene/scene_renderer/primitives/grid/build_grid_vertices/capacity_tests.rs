use std::hint::black_box;
use std::time::Instant;

use super::{GRID_INDEX_COUNT, GRID_VERTICES_PER_INDEX, build_grid_vertices};

const SAMPLE_PAIRS: usize = 21;
const GRID_BUILDS_PER_SAMPLE: usize = 32_768;

#[test]
fn optimization_batch_20260826gh_runtime228_grid_capacity_covers_all_vertices() {
    let vertices = build_grid_vertices();
    let expected = GRID_INDEX_COUNT * GRID_VERTICES_PER_INDEX;

    assert_eq!(vertices.len(), expected);
    assert!(vertices.capacity() >= vertices.len());
}

#[test]
fn optimization_batch_20260826gh_runtime228_grid_uses_extent_derived_exact_capacity() {
    let source = include_str!("../build_grid_vertices.rs");

    assert!(source.contains("const GRID_HALF_EXTENT: i32 = 10;"));
    assert!(source.contains("Vec::with_capacity(GRID_INDEX_COUNT * GRID_VERTICES_PER_INDEX)"));
    assert!(source.contains("for index in -GRID_HALF_EXTENT..=GRID_HALF_EXTENT"));
    assert!(!source.contains("let mut vertices = Vec::new();"));
    assert!(!source.contains("Vec3::new(-10.0"));
}

#[test]
#[ignore = "managed Windows release performance evidence"]
fn optimization_batch_20260826gh_runtime228_grid_vertex_capacity_bench() {
    let vertex_count = GRID_INDEX_COUNT * GRID_VERTICES_PER_INDEX;
    let mut legacy_samples = Vec::with_capacity(SAMPLE_PAIRS);
    let mut optimized_samples = Vec::with_capacity(SAMPLE_PAIRS);
    for pair in 0..SAMPLE_PAIRS {
        if pair % 2 == 0 {
            legacy_samples.push(measure(vertex_count, false));
            optimized_samples.push(measure(vertex_count, true));
        } else {
            optimized_samples.push(measure(vertex_count, true));
            legacy_samples.push(measure(vertex_count, false));
        }
    }
    let legacy_p50_ns = percentile(&legacy_samples, 50);
    let optimized_p50_ns = percentile(&optimized_samples, 50);
    let legacy_p95_ns = percentile(&legacy_samples, 95);
    let optimized_p95_ns = percentile(&optimized_samples, 95);
    println!(
        "RUNTIME228_GRID_VERTEX_CAPACITY_BENCH_V1 sample_pairs={SAMPLE_PAIRS} \
grid_builds_per_sample={GRID_BUILDS_PER_SAMPLE} vertices_per_grid={vertex_count} \
legacy_exact_capacity_builds=0 optimized_exact_capacity_builds={GRID_BUILDS_PER_SAMPLE} \
legacy_p50_ns={legacy_p50_ns} optimized_p50_ns={optimized_p50_ns} \
legacy_p95_ns={legacy_p95_ns} optimized_p95_ns={optimized_p95_ns} \
legacy_raw_ns={} optimized_raw_ns={}",
        sample_csv(&legacy_samples),
        sample_csv(&optimized_samples),
    );
    assert!(optimized_p95_ns.saturating_mul(100) <= legacy_p95_ns.saturating_mul(70));
}

fn measure(vertex_count: usize, reserve: bool) -> u128 {
    let started = Instant::now();
    let mut checksum = 0usize;
    for grid in 0..GRID_BUILDS_PER_SAMPLE {
        let mut vertices = if reserve {
            Vec::with_capacity(vertex_count)
        } else {
            Vec::new()
        };
        for vertex in 0..vertex_count {
            let value = (grid ^ vertex) as f32;
            vertices.push([value; 7]);
        }
        checksum ^= black_box(vertices.capacity() ^ vertices.len());
        black_box(&vertices);
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
