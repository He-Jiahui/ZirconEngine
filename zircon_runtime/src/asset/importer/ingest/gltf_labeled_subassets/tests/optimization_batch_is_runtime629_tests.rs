use std::collections::HashSet;
use std::hint::black_box;
use std::time::Instant;

const DEPENDENCY_COUNT: usize = 32_768;
const SAMPLE_PAIRS: usize = 31;

#[test]
fn optimization_batch_is_runtime629_reserves_mesh_dependency_membership() {
    let source = include_str!("../../gltf_labeled_subassets.rs");
    let mesh_body = source
        .split("pub(crate) fn add_gltf_mesh_subassets")
        .nth(1)
        .expect("mesh subasset builder remains present")
        .split("pub(crate) fn add_gltf_scene_subassets")
        .next()
        .expect("mesh subasset builder remains bounded");

    assert!(mesh_body.contains("HashSet::with_capacity(mesh.primitives.len().saturating_mul(2))"));
    assert!(!mesh_body.contains("let mut dependency_index = HashSet::new();"));
}

#[test]
#[ignore = "release helper microbenchmark; real glTF caller evidence is required"]
fn optimization_batch_is_runtime629_preallocated_mesh_dependency_benchmark() {
    let dependencies = (0..DEPENDENCY_COUNT)
        .map(|index| format!("res://mesh/dependency/{index:08}"))
        .collect::<Vec<_>>();
    for _ in 0..4 {
        black_box(measure_dependency_index(&dependencies, false));
        black_box(measure_dependency_index(&dependencies, true));
    }

    let mut unreserved_samples = Vec::with_capacity(SAMPLE_PAIRS);
    let mut preallocated_samples = Vec::with_capacity(SAMPLE_PAIRS);
    for pair in 0..SAMPLE_PAIRS {
        if pair % 2 == 0 {
            unreserved_samples.push(measure_dependency_index(&dependencies, false));
            preallocated_samples.push(measure_dependency_index(&dependencies, true));
        } else {
            preallocated_samples.push(measure_dependency_index(&dependencies, true));
            unreserved_samples.push(measure_dependency_index(&dependencies, false));
        }
    }

    let unreserved_p50 = percentile(&unreserved_samples, 50);
    let unreserved_p95 = percentile(&unreserved_samples, 95);
    let unreserved_p99 = percentile(&unreserved_samples, 99);
    let preallocated_p50 = percentile(&preallocated_samples, 50);
    let preallocated_p95 = percentile(&preallocated_samples, 95);
    let preallocated_p99 = percentile(&preallocated_samples, 99);
    let improvement_percent = unreserved_p95
        .saturating_sub(preallocated_p95)
        .saturating_mul(100)
        / unreserved_p95.max(1);
    println!(
        "RUNTIME629_PREALLOCATED_GLTF_MESH_DEPENDENCY_BENCH_V2 \
         sample_pairs={SAMPLE_PAIRS} dependency_count={DEPENDENCY_COUNT} \
         benchmark_scope=microbenchmark real_caller_required=true \
         percentile_method=nearest_rank unreserved_p50_ns={unreserved_p50} \
         unreserved_p95_ns={unreserved_p95} unreserved_p99_ns={unreserved_p99} \
         preallocated_p50_ns={preallocated_p50} preallocated_p95_ns={preallocated_p95} \
         preallocated_p99_ns={preallocated_p99} unreserved_ns={} preallocated_ns={} \
         improvement_percent={improvement_percent} threshold_percent=15",
        csv(&unreserved_samples),
        csv(&preallocated_samples),
    );
    assert!(preallocated_p95 <= unreserved_p95 * 85 / 100);
}

fn measure_dependency_index(dependencies: &[String], preallocated: bool) -> u128 {
    let mut index = if preallocated {
        HashSet::with_capacity(dependencies.len())
    } else {
        HashSet::new()
    };
    let started = Instant::now();
    for dependency in dependencies {
        black_box(index.insert(black_box(dependency.clone())));
    }
    black_box(index);
    started.elapsed().as_nanos().max(1)
}

fn percentile(samples: &[u128], percentile: usize) -> u128 {
    let mut sorted = samples.to_vec();
    sorted.sort_unstable();
    let rank = (sorted.len() * percentile).div_ceil(100);
    sorted[rank.saturating_sub(1)]
}

fn csv(samples: &[u128]) -> String {
    samples
        .iter()
        .map(u128::to_string)
        .collect::<Vec<_>>()
        .join(",")
}
