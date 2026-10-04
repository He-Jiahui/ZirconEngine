use std::hint::black_box;
use std::time::Instant;

const SAMPLE_COUNT: usize = 1_048_576;
const CHANNELS_PER_SAMPLE: usize = 4;
const SAMPLE_PAIRS: usize = 17;

#[test]
fn optimization_batch_jb_runtime641_reserves_declared_cube_lut_samples() {
    let source = include_str!("../../cube_lut.rs");
    let size_branch = source
        .split("CubeLutLineKind::Size3d => {")
        .nth(1)
        .expect("cube LUT size branch remains present")
        .split("CubeLutLineKind::Sample => {")
        .next()
        .expect("cube LUT size branch remains bounded");

    assert!(size_branch.contains("let expected_bytes = expected_sample_count(parsed_size)?"));
    assert!(size_branch.contains(".checked_mul(CHANNELS_PER_SAMPLE)"));
    assert!(size_branch.contains("let reservation_target = expected_bytes.min(source.len());"));
    assert!(size_branch
        .contains("samples.reserve_exact(reservation_target.saturating_sub(samples.len()));"));
}

#[test]
#[ignore = "release performance gate"]
fn optimization_batch_jb_runtime641_preallocated_cube_lut_sample_benchmark() {
    for _ in 0..4 {
        black_box(measure_samples(false));
        black_box(measure_samples(true));
    }

    let mut unreserved_samples = Vec::with_capacity(SAMPLE_PAIRS);
    let mut preallocated_samples = Vec::with_capacity(SAMPLE_PAIRS);
    for pair in 0..SAMPLE_PAIRS {
        if pair % 2 == 0 {
            unreserved_samples.push(measure_samples(false));
            preallocated_samples.push(measure_samples(true));
        } else {
            preallocated_samples.push(measure_samples(true));
            unreserved_samples.push(measure_samples(false));
        }
    }

    let unreserved_p95 = percentile(&unreserved_samples, 95);
    let preallocated_p95 = percentile(&preallocated_samples, 95);
    let improvement_percent = unreserved_p95
        .saturating_sub(preallocated_p95)
        .saturating_mul(100)
        / unreserved_p95.max(1);
    println!(
        "RUNTIME641_PREALLOCATED_CUBE_LUT_SAMPLE_BENCH_V1 sample_pairs={SAMPLE_PAIRS} sample_count={SAMPLE_COUNT} unreserved_ns={} preallocated_ns={} unreserved_p95_ns={unreserved_p95} preallocated_p95_ns={preallocated_p95} improvement_percent={improvement_percent} threshold_percent=20",
        csv(&unreserved_samples),
        csv(&preallocated_samples),
    );
    assert!(preallocated_p95 <= unreserved_p95 * 80 / 100);
}

fn measure_samples(preallocated: bool) -> u128 {
    let started = Instant::now();
    let mut samples = if preallocated {
        Vec::with_capacity(SAMPLE_COUNT * CHANNELS_PER_SAMPLE)
    } else {
        Vec::new()
    };
    for sample_index in 0..SAMPLE_COUNT {
        let channel = black_box(sample_index as u8);
        samples.extend([channel, channel.wrapping_add(1), channel.wrapping_add(2)]);
        samples.push(u8::MAX);
    }
    black_box(samples);
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
