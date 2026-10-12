use std::hint::black_box;
use std::sync::Arc;
use std::time::{Duration, Instant};

use super::joined;

const SAMPLE_PAIRS: usize = 101;
const PATHS_PER_SAMPLE: usize = 4_096;
const SEGMENTS_PER_PATH: usize = 64;

#[test]
fn editor892_settings_path_single_buffer_preserves_exact_text() {
    for values in [
        Vec::<Arc<str>>::new(),
        vec![Arc::from("")],
        vec![Arc::from("graphics")],
        vec![Arc::from(""), Arc::from("rendering"), Arc::from("")],
        vec![
            Arc::from("编辑器"),
            Arc::from("外观"),
            Arc::from("高对比度"),
        ],
    ] {
        assert_eq!(joined(&values), legacy_joined(&values));
    }
}

#[test]
fn editor892_settings_path_single_buffer_uses_exact_capacity() {
    let values = vec![
        Arc::from("a"),
        Arc::from("a-substantially-longer-segment"),
        Arc::from("中"),
    ];
    let output = joined(&values);

    assert_eq!(output.capacity(), output.len());
}

#[test]
#[ignore = "release percentile evidence; run through the managed Windows validation lane"]
fn editor892_settings_path_single_buffer_release_percentiles() {
    let values = (0..SEGMENTS_PER_PATH)
        .map(|index| Arc::<str>::from(format!("settings-segment-{index:02}-{}", "x".repeat(16))))
        .collect::<Vec<_>>();
    let mut legacy_samples = Vec::with_capacity(SAMPLE_PAIRS);
    let mut optimized_samples = Vec::with_capacity(SAMPLE_PAIRS);

    for pair in 0..SAMPLE_PAIRS {
        if pair % 2 == 0 {
            legacy_samples.push(measure(|| render_batch(&values, legacy_joined)));
            optimized_samples.push(measure(|| render_batch(&values, joined)));
        } else {
            optimized_samples.push(measure(|| render_batch(&values, joined)));
            legacy_samples.push(measure(|| render_batch(&values, legacy_joined)));
        }
    }

    let legacy_p50_ns = percentile_ns(&mut legacy_samples.clone(), 50);
    let legacy_p95_ns = percentile_ns(&mut legacy_samples.clone(), 95);
    let legacy_p99_ns = percentile_ns(&mut legacy_samples, 99);
    let optimized_p50_ns = percentile_ns(&mut optimized_samples.clone(), 50);
    let optimized_p95_ns = percentile_ns(&mut optimized_samples.clone(), 95);
    let optimized_p99_ns = percentile_ns(&mut optimized_samples, 99);
    println!(
        "EDITOR892_SETTINGS_PATH_SINGLE_BUFFER_BENCH_V1 \
legacy_p50_ns={legacy_p50_ns} legacy_p95_ns={legacy_p95_ns} \
legacy_p99_ns={legacy_p99_ns} optimized_p50_ns={optimized_p50_ns} \
optimized_p95_ns={optimized_p95_ns} optimized_p99_ns={optimized_p99_ns}"
    );
    assert!(
        optimized_p95_ns <= legacy_p95_ns.saturating_mul(110).div_ceil(100),
        "single-buffer p95 regressed beyond 10%: legacy={legacy_p95_ns}ns optimized={optimized_p95_ns}ns"
    );
}

fn legacy_joined(values: &[Arc<str>]) -> String {
    values
        .iter()
        .map(|value| value.as_ref())
        .collect::<Vec<_>>()
        .join("/")
}

fn render_batch(values: &[Arc<str>], render: fn(&[Arc<str>]) -> String) -> usize {
    (0..PATHS_PER_SAMPLE)
        .map(|_| black_box(render(black_box(values))).len())
        .sum()
}

fn measure<T>(run: impl FnOnce() -> T) -> Duration {
    let started = Instant::now();
    black_box(run());
    started.elapsed()
}

fn percentile_ns(samples: &mut [Duration], percentile: usize) -> u128 {
    samples.sort_unstable();
    let rank = (samples.len() * percentile).div_ceil(100);
    samples[rank.saturating_sub(1)].as_nanos()
}
