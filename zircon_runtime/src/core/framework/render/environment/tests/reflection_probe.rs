use std::hint::black_box;
use std::time::Instant;

use super::*;

#[test]
fn reflection_probe_blend_selects_top_two_without_candidate_vec_or_sort() {
    let source = include_str!("../reflection_probe.rs");
    let selection = source
        .split("pub fn select_reflection_probe_blend")
        .nth(1)
        .and_then(|text| text.split("fn validate_blend_distance").next())
        .expect("read reflection probe selection helper");

    assert!(
            !selection.contains("collect::<Vec") && !selection.contains("sort_by"),
            "per-position reflection probe selection must retain only the best two candidates instead of allocating and sorting every eligible probe"
        );

    let probes = [probe(20, 0), probe(10, 0), probe(30, 2)];
    let blend = select_reflection_probe_blend(&probes, Vec3::ZERO, &RenderLayerSet::default());

    assert_eq!(blend.primary.map(|entry| entry.probe_index), Some(2));
    assert_eq!(blend.secondary.map(|entry| entry.probe_index), Some(1));
}

#[test]
fn optimization_batch_dz_sphere_weight_uses_squared_distance_broad_phase() {
    let source = include_str!("../reflection_probe.rs");
    let weight = source
        .split("pub fn reflection_probe_influence_weight")
        .nth(1)
        .and_then(|text| {
            text.split("pub fn reflection_probe_box_project_direction")
                .next()
        })
        .expect("read reflection probe influence helper");

    let position_delta = weight
        .find("let position_delta = world_position - probe.position;")
        .expect("probe influence must retain the world-space center delta");
    let box_branch = weight
        .find("ProbeInfluenceShape::Box {")
        .expect("probe influence must retain box weighting");
    let box_rotation = weight
        .find("let local_position = probe.rotation.conjugate() * position_delta;")
        .expect("box weighting must retain local-space rotation");
    let sphere_branch = weight
        .find("ProbeInfluenceShape::Sphere {")
        .expect("probe influence must retain sphere weighting");
    let sphere_distance_squared = weight
        .find("let distance_squared = position_delta.length_squared();")
        .expect("sphere weighting must compute squared center distance once");
    let sphere_rejection = weight
        .find("if radius > 0.0 && distance_squared >= radius * radius")
        .expect("far sphere probes must be rejected before square root");
    let sphere_distance = weight
        .find("boundary_weight(radius - distance_squared.sqrt(), blend_distance)")
        .expect("sphere weighting must use its rotation-invariant center distance");

    assert!(
        position_delta < box_branch
            && box_branch < box_rotation
            && box_rotation < sphere_branch
            && sphere_branch < sphere_distance_squared
            && sphere_distance_squared < sphere_rejection
            && sphere_rejection < sphere_distance,
        "only box influences may rotate the world-space center delta"
    );
}

#[test]
fn optimization_batch_dz_far_sphere_probe_keeps_zero_boundary_weight() {
    let probe = probe(40, 0);

    assert_eq!(
        reflection_probe_influence_weight(&probe, Vec3::new(4.0, 0.0, 0.0)),
        0.0
    );
    assert_eq!(
        reflection_probe_influence_weight(&probe, Vec3::new(40.0, 0.0, 0.0)),
        0.0
    );
    assert_eq!(
        reflection_probe_influence_weight(&probe, Vec3::new(3.5, 0.0, 0.0)),
        0.5
    );
}

#[test]
#[ignore = "release-only far sphere reflection probe benchmark"]
fn optimization_batch_dz_far_sphere_probe_release_benchmark_evidence() {
    const SAMPLE_PAIRS: usize = 17;
    const PASSES_PER_SAMPLE: usize = 32;
    const PROBES_PER_PASS: usize = 2_048;

    fn legacy_weight(probe: &ReflectionProbeData, world_position: Vec3) -> Real {
        let position_delta = world_position - probe.position;
        match probe.shape {
            ProbeInfluenceShape::Box {
                half_extents,
                blend_distance,
            } => {
                let local_position = probe.rotation.conjugate() * position_delta;
                let edge_distance = (half_extents - local_position.abs()).min_element();
                boundary_weight(edge_distance, blend_distance)
            }
            ProbeInfluenceShape::Sphere {
                radius,
                blend_distance,
            } => boundary_weight(radius - position_delta.length(), blend_distance),
        }
    }

    fn measure_legacy(probes: &[ReflectionProbeData]) -> u128 {
        let world_position = black_box(Vec3::new(16_384.0, 8_192.0, -4_096.0));
        let started = Instant::now();
        let mut checksum = 0.0;
        for _ in 0..PASSES_PER_SAMPLE {
            for probe in probes {
                checksum += legacy_weight(black_box(probe), world_position);
            }
        }
        black_box(checksum);
        started.elapsed().as_nanos().max(1)
    }

    fn measure_optimized(probes: &[ReflectionProbeData]) -> u128 {
        let world_position = black_box(Vec3::new(16_384.0, 8_192.0, -4_096.0));
        let started = Instant::now();
        let mut checksum = 0.0;
        for _ in 0..PASSES_PER_SAMPLE {
            for probe in probes {
                checksum += reflection_probe_influence_weight(black_box(probe), world_position);
            }
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

    fn raw(samples: &[u128]) -> String {
        samples
            .iter()
            .map(u128::to_string)
            .collect::<Vec<_>>()
            .join(",")
    }

    let probes = (0..PROBES_PER_PASS)
        .map(|index| {
            ReflectionProbeData::try_new(
                index as u64,
                Vec3::new(index as Real * 0.125, 0.0, 0.0),
                Quat::IDENTITY,
                ProbeInfluenceShape::sphere(4.0, 1.0).expect("valid benchmark sphere"),
                Vec3::splat(4.0),
            )
            .expect("valid benchmark probe")
        })
        .collect::<Vec<_>>();

    for _ in 0..4 {
        black_box(measure_legacy(&probes));
        black_box(measure_optimized(&probes));
    }

    let mut legacy_samples = Vec::with_capacity(SAMPLE_PAIRS);
    let mut optimized_samples = Vec::with_capacity(SAMPLE_PAIRS);
    for pair in 0..SAMPLE_PAIRS {
        if pair % 2 == 0 {
            legacy_samples.push(measure_legacy(&probes));
            optimized_samples.push(measure_optimized(&probes));
        } else {
            optimized_samples.push(measure_optimized(&probes));
            legacy_samples.push(measure_legacy(&probes));
        }
    }

    let legacy_p50_ns = percentile(&legacy_samples, 50);
    let optimized_p50_ns = percentile(&optimized_samples, 50);
    let legacy_p95_ns = percentile(&legacy_samples, 95);
    let optimized_p95_ns = percentile(&optimized_samples, 95);

    println!(
        "RUNTIME434_FAR_SPHERE_PROBE_BROAD_PHASE_BENCH_V1 sample_pairs={SAMPLE_PAIRS} \
passes_per_sample={PASSES_PER_SAMPLE} probes_per_pass={PROBES_PER_PASS} \
pair_order=alternating_legacy_even legacy_first_pairs=9 optimized_first_pairs=8 \
legacy_sqrt_per_sample={} optimized_sqrt_per_sample=0 legacy_p50_ns={legacy_p50_ns} \
optimized_p50_ns={optimized_p50_ns} legacy_p95_ns={legacy_p95_ns} \
optimized_p95_ns={optimized_p95_ns} legacy_raw_ns={} optimized_raw_ns={}",
        PASSES_PER_SAMPLE * PROBES_PER_PASS,
        raw(&legacy_samples),
        raw(&optimized_samples),
    );

    assert!(
            optimized_p95_ns.saturating_mul(100) <= legacy_p95_ns.saturating_mul(70),
            "far sphere broad phase must reduce P95 by at least 30%: legacy={legacy_p95_ns}ns optimized={optimized_p95_ns}ns"
        );
}

fn probe(probe_id: u64, priority: i32) -> ReflectionProbeData {
    ReflectionProbeData::try_new(
        probe_id,
        Vec3::ZERO,
        Quat::IDENTITY,
        ProbeInfluenceShape::sphere(4.0, 1.0).expect("valid sphere"),
        Vec3::splat(4.0),
    )
    .expect("valid probe")
    .with_baked_cubemap(Some(ResourceId::from_stable_label(&format!(
        "res://probe/{probe_id}.zcube"
    ))))
    .with_priority(priority)
}
