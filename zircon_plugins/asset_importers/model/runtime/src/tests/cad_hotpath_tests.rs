use super::*;
use std::hint::black_box;
use std::time::Instant;

#[test]
fn plugins07_model_hotpath_surface_point_count_preserves_triangles_and_quads() {
    let first = Point::new(0.0, 0.0, 0.0);
    let second = Point::new(1.0, 0.0, 0.0);
    let third = Point::new(0.0, 1.0, 0.0);
    let fourth = Point::new(1.0, 1.0, 0.0);

    assert_eq!(surface_point_count([&first, &second, &third, &first]), 3);
    assert_eq!(surface_point_count([&first, &second, &third, &fourth]), 4);
}

#[test]
#[ignore = "release performance gate; run through the Plugins07 coordinator validator"]
fn plugins07_model_hotpath_release_stack_surface_points_p95_gate() {
    const SAMPLE_PAIRS: usize = 21;
    const SURFACES: usize = 262_144;
    const THRESHOLD_PERCENT: u128 = 80;
    let first = Point::new(0.0, 0.0, 0.0);
    let second = Point::new(1.0, 0.0, 0.0);
    let third = Point::new(0.0, 1.0, 0.0);
    let fourth = Point::new(1.0, 1.0, 0.0);
    let points = [&first, &second, &third, &fourth];
    let mut legacy_samples = Vec::with_capacity(SAMPLE_PAIRS);
    let mut optimized_samples = Vec::with_capacity(SAMPLE_PAIRS);
    for pair in 0..SAMPLE_PAIRS {
        let legacy = || measure_heap_surface_points(points, SURFACES);
        let optimized = || measure_stack_surface_points(points, SURFACES);
        if pair % 2 == 0 {
            legacy_samples.push(legacy());
            optimized_samples.push(optimized());
        } else {
            optimized_samples.push(optimized());
            legacy_samples.push(legacy());
        }
    }

    emit_cad_performance_gate(
        &legacy_samples,
        &optimized_samples,
        THRESHOLD_PERCENT,
        &format!(
            "surfaces_per_sample={SURFACES} points_per_surface=4 legacy_temporary_vec_allocations_per_sample={SURFACES} optimized_temporary_vec_allocations_per_sample=0"
        ),
    );
}

fn measure_heap_surface_points(points: [&Point; 4], surfaces: usize) -> u128 {
    let started = Instant::now();
    let mut point_count = 0_usize;
    for _ in 0..surfaces {
        let points = black_box(points);
        let mut polygon = vec![points[0], points[1], points[2]];
        if surface_point_count(points) == 4 {
            polygon.push(points[3]);
        }
        point_count += black_box(polygon.as_slice()).len();
    }
    black_box(point_count);
    started.elapsed().as_nanos()
}

fn measure_stack_surface_points(points: [&Point; 4], surfaces: usize) -> u128 {
    let started = Instant::now();
    let mut point_count = 0_usize;
    for _ in 0..surfaces {
        let points = black_box(points);
        let count = surface_point_count(points);
        point_count += black_box(&points[..count]).len();
    }
    black_box(point_count);
    started.elapsed().as_nanos()
}

fn emit_cad_performance_gate(
    legacy_samples: &[u128],
    optimized_samples: &[u128],
    threshold_percent: u128,
    workload: &str,
) {
    let legacy_p95 = nearest_rank_cad_p95(legacy_samples);
    let optimized_p95 = nearest_rank_cad_p95(optimized_samples);
    let improvement_percent =
        legacy_p95.saturating_sub(optimized_p95).saturating_mul(100) / legacy_p95.max(1);
    println!(
        "PERF_RESULT plugins07_cad_stack_surface_points sample_pairs=21 order=alternating_legacy_first_even {workload} legacy_ns={} optimized_ns={} legacy_p95_ns={legacy_p95} optimized_p95_ns={optimized_p95} improvement_percent={improvement_percent} threshold_percent={threshold_percent}",
        cad_samples_csv(legacy_samples),
        cad_samples_csv(optimized_samples),
    );
    assert!(
        improvement_percent >= threshold_percent,
        "CAD stack surface points must improve P95 by at least {threshold_percent}% (legacy={legacy_p95}ns optimized={optimized_p95}ns improvement={improvement_percent}%)"
    );
}

fn nearest_rank_cad_p95(samples: &[u128]) -> u128 {
    let mut sorted = samples.to_vec();
    sorted.sort_unstable();
    sorted[(sorted.len() * 95).div_ceil(100).saturating_sub(1)]
}

fn cad_samples_csv(samples: &[u128]) -> String {
    samples
        .iter()
        .map(u128::to_string)
        .collect::<Vec<_>>()
        .join(",")
}
