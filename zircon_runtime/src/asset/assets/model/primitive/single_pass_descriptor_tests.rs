use std::hint::black_box;
use std::time::Instant;

use crate::asset::MeshVertex;
use crate::core::framework::render::RenderMeshBounds;
use crate::core::math::{Vec2, Vec3};

use super::mesh_bounds_and_planarity;

const CHECKS_PER_SAMPLE: usize = 64;
const SAMPLE_PAIRS: usize = 31;
const VERTEX_COUNT: usize = 8192;

fn vertex(position: [f32; 3]) -> MeshVertex {
    MeshVertex::new(
        Vec3::from_array(position),
        Vec3::new(0.0, 0.0, 1.0),
        Vec2::ZERO,
    )
}

fn legacy_bounds_and_planarity(vertices: &[MeshVertex]) -> (RenderMeshBounds, bool) {
    let bounds = RenderMeshBounds::from_positions(vertices.iter().map(|vertex| vertex.position));
    let is_planar = vertices.iter().all(|vertex| vertex.position[2] == 0.0);
    (bounds, is_planar)
}

fn measure(vertices: &[MeshVertex], optimized: bool) -> u128 {
    let started = Instant::now();
    let mut radius = 0.0;
    for _ in 0..CHECKS_PER_SAMPLE {
        let (bounds, is_planar) = if optimized {
            mesh_bounds_and_planarity(black_box(vertices))
        } else {
            legacy_bounds_and_planarity(black_box(vertices))
        };
        radius += bounds.radius + usize::from(is_planar) as f32;
    }
    black_box(radius);
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

#[test]
fn optimization_batch_20260829bh_runtime335_single_pass_descriptor_preserves_results() {
    for vertices in [
        Vec::new(),
        vec![vertex([1.0, 2.0, 0.0])],
        vec![vertex([-2.0, 4.0, 0.0]), vertex([5.0, -3.0, 0.0])],
        vec![vertex([-2.0, 4.0, 0.0]), vertex([5.0, -3.0, 7.0])],
    ] {
        assert_eq!(
            mesh_bounds_and_planarity(&vertices),
            legacy_bounds_and_planarity(&vertices)
        );
    }
}

#[test]
fn optimization_batch_20260829bh_runtime335_descriptor_uses_one_vertex_loop() {
    let source = include_str!("../primitive.rs");
    let production = source.split_once("#[cfg(test)]").expect("test boundary").0;

    assert!(production.contains("fn mesh_bounds_and_planarity"));
    assert_eq!(
        production.matches("for vertex in &vertices[1..]").count(),
        1
    );
    assert!(!production.contains("RenderMeshBounds::from_positions"));
    assert!(!production.contains("vertices.iter().all"));
}

#[test]
#[ignore = "managed Windows release performance evidence"]
fn optimization_batch_20260829bh_runtime335_single_pass_primitive_descriptor_bench() {
    let vertices = (0..VERTEX_COUNT)
        .map(|index| vertex([index as f32, (index % 127) as f32, 0.0]))
        .collect::<Vec<_>>();
    let mut baseline_samples = Vec::with_capacity(SAMPLE_PAIRS);
    let mut candidate_samples = Vec::with_capacity(SAMPLE_PAIRS);
    for pair in 0..SAMPLE_PAIRS {
        if pair % 2 == 0 {
            baseline_samples.push(measure(&vertices, false));
            candidate_samples.push(measure(&vertices, true));
        } else {
            candidate_samples.push(measure(&vertices, true));
            baseline_samples.push(measure(&vertices, false));
        }
    }

    let baseline_p50_ns = percentile(&baseline_samples, 50);
    let candidate_p50_ns = percentile(&candidate_samples, 50);
    let baseline_p95_ns = percentile(&baseline_samples, 95);
    let candidate_p95_ns = percentile(&candidate_samples, 95);
    println!(
        "RUNTIME335_SINGLE_PASS_PRIMITIVE_DESCRIPTOR_BENCH_V1 sample_pairs={SAMPLE_PAIRS} \
checks_per_sample={CHECKS_PER_SAMPLE} vertices={VERTEX_COUNT} \
baseline_vertex_passes=2 candidate_vertex_passes=1 \
baseline_p50_ns={baseline_p50_ns} candidate_p50_ns={candidate_p50_ns} \
baseline_p95_ns={baseline_p95_ns} candidate_p95_ns={candidate_p95_ns} \
baseline_raw_ns={} candidate_raw_ns={}",
        sample_csv(&baseline_samples),
        sample_csv(&candidate_samples),
    );
    assert!(candidate_p95_ns.saturating_mul(100) <= baseline_p95_ns.saturating_mul(70));
}
