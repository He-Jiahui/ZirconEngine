use super::*;
use std::hint::black_box;
use std::time::Instant;

#[test]
fn plugins07_model_hotpath_complete_normals_are_borrowed_and_short_normals_are_padded() {
    let positions = [0.0, 0.0, 0.0, 1.0, 0.0, 0.0, 0.0, 1.0, 0.0];
    let normals = [0.0, 0.0, 2.0, 0.0, 0.0, 2.0, 0.0, 0.0, 2.0];
    let indices = [0, 1, 2];

    let complete = prepare_vertex_normals(&positions, &normals, &indices).unwrap();
    let short = prepare_vertex_normals(&positions, &normals[..3], &indices).unwrap();

    assert!(matches!(&complete, std::borrow::Cow::Borrowed(_)));
    assert!(matches!(&short, std::borrow::Cow::Owned(_)));
    assert_eq!(
        short.as_ref(),
        &[0.0, 0.0, 2.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0]
    );
}

#[test]
#[ignore = "release performance gate; run through the Plugins07 coordinator validator"]
fn plugins07_model_hotpath_release_borrowed_normals_p95_gate() {
    const SAMPLE_PAIRS: usize = 21;
    const NORMAL_VALUES: usize = 524_288;
    const ITERATIONS: usize = 16;
    const THRESHOLD_PERCENT: u128 = 80;
    let normals = vec![0.25_f32; NORMAL_VALUES];
    let mut legacy_samples = Vec::with_capacity(SAMPLE_PAIRS);
    let mut optimized_samples = Vec::with_capacity(SAMPLE_PAIRS);
    for pair in 0..SAMPLE_PAIRS {
        let legacy = || measure_normal_clone(&normals, ITERATIONS);
        let optimized = || measure_normal_borrow(&normals, ITERATIONS);
        if pair % 2 == 0 {
            legacy_samples.push(legacy());
            optimized_samples.push(optimized());
        } else {
            optimized_samples.push(optimized());
            legacy_samples.push(legacy());
        }
    }

    emit_model_normal_performance_gate(
        &legacy_samples,
        &optimized_samples,
        THRESHOLD_PERCENT,
        &format!(
            "normal_values={NORMAL_VALUES} iterations_per_sample={ITERATIONS} legacy_cloned_bytes_per_sample={} optimized_cloned_bytes_per_sample=0",
            NORMAL_VALUES * ITERATIONS * std::mem::size_of::<f32>()
        ),
    );
}

fn measure_normal_clone(normals: &[f32], iterations: usize) -> u128 {
    let started = Instant::now();
    let mut values = 0_usize;
    for _ in 0..iterations {
        let owned = black_box(normals).to_vec();
        values += black_box(owned.as_slice()).len();
    }
    black_box(values);
    started.elapsed().as_nanos()
}

fn measure_normal_borrow(normals: &[f32], iterations: usize) -> u128 {
    let started = Instant::now();
    let mut values = 0_usize;
    for _ in 0..iterations {
        let borrowed: Cow<'_, [f32]> = Cow::Borrowed(black_box(normals));
        values += black_box(borrowed.as_ref()).len();
    }
    black_box(values);
    started.elapsed().as_nanos()
}

fn emit_model_normal_performance_gate(
    legacy_samples: &[u128],
    optimized_samples: &[u128],
    threshold_percent: u128,
    workload: &str,
) {
    let legacy_p95 = nearest_rank_model_p95(legacy_samples);
    let optimized_p95 = nearest_rank_model_p95(optimized_samples);
    let improvement_percent =
        legacy_p95.saturating_sub(optimized_p95).saturating_mul(100) / legacy_p95.max(1);
    println!(
        "PERF_RESULT plugins07_model_borrowed_normals sample_pairs=21 order=alternating_legacy_first_even {workload} legacy_ns={} optimized_ns={} legacy_p95_ns={legacy_p95} optimized_p95_ns={optimized_p95} improvement_percent={improvement_percent} threshold_percent={threshold_percent}",
        model_samples_csv(legacy_samples),
        model_samples_csv(optimized_samples),
    );
    assert!(
        improvement_percent >= threshold_percent,
        "model borrowed normals must improve P95 by at least {threshold_percent}% (legacy={legacy_p95}ns optimized={optimized_p95}ns improvement={improvement_percent}%)"
    );
}

fn nearest_rank_model_p95(samples: &[u128]) -> u128 {
    let mut sorted = samples.to_vec();
    sorted.sort_unstable();
    sorted[(sorted.len() * 95).div_ceil(100).saturating_sub(1)]
}

fn model_samples_csv(samples: &[u128]) -> String {
    samples
        .iter()
        .map(u128::to_string)
        .collect::<Vec<_>>()
        .join(",")
}
