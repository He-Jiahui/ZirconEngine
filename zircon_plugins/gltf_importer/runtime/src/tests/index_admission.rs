use super::*;
use std::hint::black_box;
use std::time::Instant;

const BENCHMARK_TRIANGLES: usize = 65_536;
const BENCHMARK_ITERATIONS: usize = 8;
const BENCHMARK_SAMPLE_PAIRS: usize = 21;
const BENCHMARK_TIME_RATIO_THRESHOLD_BPS: u128 = 11_000;

fn legacy_generate_normals(positions: &[f32], indices: &[u32]) -> Vec<f32> {
    let vertex_count = positions.len() / 3;
    let mut normals = vec![0.0_f32; vertex_count * 3];

    for triangle in indices.chunks_exact(3) {
        let a = triangle[0] as usize;
        let b = triangle[1] as usize;
        let c = triangle[2] as usize;
        let position = |index: usize| -> Vec3 {
            Vec3::new(
                positions[index * 3],
                positions[index * 3 + 1],
                positions[index * 3 + 2],
            )
        };
        let face_normal = (position(b) - position(a))
            .cross(position(c) - position(a))
            .normalize_or_zero();
        for index in [a, b, c] {
            normals[index * 3] += face_normal.x;
            normals[index * 3 + 1] += face_normal.y;
            normals[index * 3 + 2] += face_normal.z;
        }
    }

    normals
}

fn measure_normal_generation(iterations: usize, mut generate: impl FnMut() -> Vec<f32>) -> u128 {
    let timer = Instant::now();
    let mut checksum = 0.0_f32;
    for _ in 0..iterations {
        checksum += black_box(generate())[2];
    }
    black_box(checksum);
    timer.elapsed().as_nanos()
}

fn nearest_rank_p95(samples: &mut [u128]) -> u128 {
    samples.sort_unstable();
    samples[(samples.len() * 95 - 1) / 100]
}

fn sample_csv(samples: &[u128]) -> String {
    samples
        .iter()
        .map(u128::to_string)
        .collect::<Vec<_>>()
        .join(",")
}

#[test]
fn gltf_index_admission_rejects_out_of_range_vertices_without_panicking() {
    let result = std::panic::catch_unwind(|| {
        let mut budget = MeshSdfCookBudget::default();
        primitive_from_indexed_mesh(
            &[0.0, 0.0, 0.0, 1.0, 0.0, 0.0, 0.0, 1.0, 0.0],
            &[],
            &[],
            &[],
            &[0, 1, 3],
            &[],
            &[],
            Some("malformed"),
            "gltf-index-admission-test",
            None,
            &mut budget,
        )
    });

    assert!(result.is_ok(), "malformed glTF indices must not unwind");
    let error = result
        .unwrap()
        .expect_err("out-of-range glTF index must be rejected");
    assert!(matches!(
        error,
        AssetImportError::Parse(message)
            if message.contains("mesh index 3") && message.contains("vertex count 3")
    ));
}

#[test]
#[ignore = "release-only performance evidence"]
fn benchmark_index_admitted_normal_generation() {
    let positions = [0.0_f32, 0.0, 0.0, 1.0, 0.0, 0.0, 0.0, 1.0, 0.0];
    let indices = [0_u32, 1, 2].repeat(BENCHMARK_TRIANGLES);
    assert_eq!(
        legacy_generate_normals(&positions, &indices),
        generate_normals(&positions, &indices).unwrap()
    );

    let mut legacy_samples = Vec::with_capacity(BENCHMARK_SAMPLE_PAIRS);
    let mut admitted_samples = Vec::with_capacity(BENCHMARK_SAMPLE_PAIRS);
    for sample_index in 0..BENCHMARK_SAMPLE_PAIRS {
        if sample_index % 2 == 0 {
            legacy_samples.push(measure_normal_generation(BENCHMARK_ITERATIONS, || {
                legacy_generate_normals(black_box(&positions), black_box(&indices))
            }));
            admitted_samples.push(measure_normal_generation(BENCHMARK_ITERATIONS, || {
                generate_normals(black_box(&positions), black_box(&indices)).unwrap()
            }));
        } else {
            admitted_samples.push(measure_normal_generation(BENCHMARK_ITERATIONS, || {
                generate_normals(black_box(&positions), black_box(&indices)).unwrap()
            }));
            legacy_samples.push(measure_normal_generation(BENCHMARK_ITERATIONS, || {
                legacy_generate_normals(black_box(&positions), black_box(&indices))
            }));
        }
    }

    let legacy_raw = legacy_samples.clone();
    let admitted_raw = admitted_samples.clone();
    let legacy_p95_ns = nearest_rank_p95(&mut legacy_samples);
    let admitted_p95_ns = nearest_rank_p95(&mut admitted_samples);
    let ratio_bps = admitted_p95_ns.saturating_mul(10_000) / legacy_p95_ns.max(1);
    let legacy_position_component_reads = BENCHMARK_TRIANGLES * 12 * BENCHMARK_ITERATIONS;
    let admitted_position_component_reads = BENCHMARK_TRIANGLES * 9 * BENCHMARK_ITERATIONS;

    println!(
        "PERF_RESULT plugins07_index_admitted_normal_generation triangles={} iterations_per_sample={} sample_pairs={} order=alternating_legacy_first_even percentile_method=nearest_rank legacy_position_component_reads={} admitted_position_component_reads={} reduction_bps=2500 legacy_index_admission_checks=0 admitted_index_admission_checks={} legacy_p95_ns={} admitted_p95_ns={} ratio_bps={} threshold_bps={} legacy_samples_ns={} admitted_samples_ns={}",
        BENCHMARK_TRIANGLES,
        BENCHMARK_ITERATIONS,
        BENCHMARK_SAMPLE_PAIRS,
        legacy_position_component_reads,
        admitted_position_component_reads,
        BENCHMARK_TRIANGLES * 3 * BENCHMARK_ITERATIONS,
        legacy_p95_ns,
        admitted_p95_ns,
        ratio_bps,
        BENCHMARK_TIME_RATIO_THRESHOLD_BPS,
        sample_csv(&legacy_raw),
        sample_csv(&admitted_raw),
    );

    assert_eq!(BENCHMARK_SAMPLE_PAIRS, legacy_raw.len());
    assert_eq!(BENCHMARK_SAMPLE_PAIRS, admitted_raw.len());
    assert!(
        ratio_bps <= BENCHMARK_TIME_RATIO_THRESHOLD_BPS,
        "index-admitted normal generation P95 regression: ratio_bps={ratio_bps}"
    );
}
