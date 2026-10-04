use std::collections::HashSet;
use std::hint::black_box;
use std::time::Instant;

const REMOVED_COUNT: usize = 32_768;
const READY_COUNT: usize = 32_768;
const SAMPLE_PAIRS: usize = 17;

#[test]
fn optimization_batch_ix_runtime634_reserves_shader_replacement_paths() {
    let source = include_str!("../../shader_import_dependencies.rs");
    let replacement_body = source
        .split("pub(super) fn prepare_source_replacement")
        .nth(1)
        .expect("shader source replacement remains present")
        .split("pub(super) fn dependency_locators")
        .next()
        .expect("shader source replacement remains bounded");

    assert!(replacement_body.contains("let ready_shaders = ready_shaders.into_iter();"));
    assert!(replacement_body.contains("let replacement_path_capacity ="));
    assert!(replacement_body.contains("removed_ids.len().saturating_add(ready_capacity)"));
    assert!(replacement_body.contains("HashSet::with_capacity(replacement_path_capacity)"));
    assert!(!replacement_body.contains("let mut affected_paths = HashSet::new();"));
}

#[test]
#[ignore = "release performance gate"]
fn optimization_batch_ix_runtime634_preallocated_shader_replacement_benchmark() {
    let paths = (0..REMOVED_COUNT.saturating_add(READY_COUNT))
        .map(|index| format!("shader.import.path.{index:08}"))
        .collect::<Vec<_>>();
    for _ in 0..4 {
        black_box(measure_paths(&paths, false));
        black_box(measure_paths(&paths, true));
    }

    let mut unreserved_samples = Vec::with_capacity(SAMPLE_PAIRS);
    let mut preallocated_samples = Vec::with_capacity(SAMPLE_PAIRS);
    for pair in 0..SAMPLE_PAIRS {
        if pair % 2 == 0 {
            unreserved_samples.push(measure_paths(&paths, false));
            preallocated_samples.push(measure_paths(&paths, true));
        } else {
            preallocated_samples.push(measure_paths(&paths, true));
            unreserved_samples.push(measure_paths(&paths, false));
        }
    }

    let unreserved_p95 = percentile(&unreserved_samples, 95);
    let preallocated_p95 = percentile(&preallocated_samples, 95);
    let improvement_percent = unreserved_p95
        .saturating_sub(preallocated_p95)
        .saturating_mul(100)
        / unreserved_p95.max(1);
    println!(
        "RUNTIME634_PREALLOCATED_SHADER_REPLACEMENT_PATH_BENCH_V1 sample_pairs={SAMPLE_PAIRS} removed_count={REMOVED_COUNT} ready_count={READY_COUNT} unreserved_ns={} preallocated_ns={} unreserved_p95_ns={unreserved_p95} preallocated_p95_ns={preallocated_p95} improvement_percent={improvement_percent} threshold_percent=20",
        csv(&unreserved_samples),
        csv(&preallocated_samples),
    );
    assert!(preallocated_p95 <= unreserved_p95 * 80 / 100);
}

fn measure_paths(paths: &[String], preallocated: bool) -> u128 {
    let mut affected = if preallocated {
        HashSet::with_capacity(paths.len())
    } else {
        HashSet::new()
    };
    let started = Instant::now();
    for path in paths {
        black_box(affected.insert(black_box(path.clone())));
    }
    black_box(affected);
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
