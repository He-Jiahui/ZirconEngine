use std::hint::black_box;
use std::time::Instant;

use super::{PARTICLE_VELOCITY_VERTICES_PER_SPRITE, particle_velocity_vertex_capacity};

const SAMPLE_PAIRS: usize = 21;
const BUILDS_PER_SAMPLE: usize = 128;
const SPRITES_PER_BUILD: usize = 4_096;

#[test]
fn optimization_batch_20260826fr_runtime213_capacity_covers_particle_velocity_quads() {
    let capacity = particle_velocity_vertex_capacity(SPRITES_PER_BUILD);
    let mut vertices = Vec::with_capacity(capacity);
    for sprite in 0..SPRITES_PER_BUILD {
        vertices.extend(std::iter::repeat_n(
            sprite,
            PARTICLE_VELOCITY_VERTICES_PER_SPRITE,
        ));
    }

    assert_eq!(capacity, SPRITES_PER_BUILD * 6);
    assert_eq!(vertices.len(), capacity);
    assert!(vertices.capacity() >= capacity);
    assert_eq!(vertices[0], 0);
    assert_eq!(vertices[vertices.len() - 1], SPRITES_PER_BUILD - 1);
    assert_eq!(particle_velocity_vertex_capacity(usize::MAX), usize::MAX);
}

#[test]
fn optimization_batch_20260826fr_runtime213_particle_vertices_reserve_sprite_upper_bound() {
    let source = include_str!("../build_particle_velocity_vertices.rs");
    assert!(source.contains("Vec::with_capacity(particle_velocity_vertex_capacity("));
    assert!(source.contains("const PARTICLE_VELOCITY_VERTICES_PER_SPRITE: usize = 6;"));
    assert!(!source.contains("let mut vertices = Vec::new();"));
}

#[test]
#[ignore = "managed Windows release performance evidence"]
fn optimization_batch_20260826fr_runtime213_particle_velocity_vertex_capacity_bench() {
    const VERTICES_PER_BUILD: usize = SPRITES_PER_BUILD * PARTICLE_VELOCITY_VERTICES_PER_SPRITE;
    let mut legacy_samples = Vec::with_capacity(SAMPLE_PAIRS);
    let mut optimized_samples = Vec::with_capacity(SAMPLE_PAIRS);
    for pair in 0..SAMPLE_PAIRS {
        if pair % 2 == 0 {
            legacy_samples.push(measure(false));
            optimized_samples.push(measure(true));
        } else {
            optimized_samples.push(measure(true));
            legacy_samples.push(measure(false));
        }
    }
    let legacy_p50_ns = percentile(&legacy_samples, 50);
    let optimized_p50_ns = percentile(&optimized_samples, 50);
    let legacy_p95_ns = percentile(&legacy_samples, 95);
    let optimized_p95_ns = percentile(&optimized_samples, 95);
    println!(
        "RUNTIME213_PARTICLE_VELOCITY_VERTEX_CAPACITY_BENCH_V1 sample_pairs={SAMPLE_PAIRS} \
builds_per_sample={BUILDS_PER_SAMPLE} sprites_per_build={SPRITES_PER_BUILD} \
vertices_per_build={VERTICES_PER_BUILD} legacy_reservations_per_build=0 \
optimized_reservations_per_build=1 legacy_p50_ns={legacy_p50_ns} \
optimized_p50_ns={optimized_p50_ns} legacy_p95_ns={legacy_p95_ns} \
optimized_p95_ns={optimized_p95_ns} legacy_raw_ns={} optimized_raw_ns={}",
        sample_csv(&legacy_samples),
        sample_csv(&optimized_samples),
    );
    assert!(optimized_p95_ns.saturating_mul(100) <= legacy_p95_ns.saturating_mul(70));
}

#[derive(Clone, Copy)]
struct VelocityVertexFixture([usize; 4]);

fn measure(reserve: bool) -> u128 {
    let started = Instant::now();
    let mut checksum = 0usize;
    let capacity = particle_velocity_vertex_capacity(SPRITES_PER_BUILD);
    for build in 0..BUILDS_PER_SAMPLE {
        let mut vertices = if reserve {
            Vec::with_capacity(capacity)
        } else {
            Vec::new()
        };
        for sprite in 0..SPRITES_PER_BUILD {
            vertices.extend(std::iter::repeat_n(
                VelocityVertexFixture([black_box(build ^ sprite); 4]),
                PARTICLE_VELOCITY_VERTICES_PER_SPRITE,
            ));
        }
        checksum ^= black_box(vertices.len() ^ vertices.capacity() ^ vertices[capacity - 1].0[0]);
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
