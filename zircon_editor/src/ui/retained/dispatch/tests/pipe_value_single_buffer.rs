use std::hint::black_box;
use std::time::{Duration, Instant};

use super::pipe_value;

const SAMPLE_PAIRS: usize = 101;
const VALUES_PER_SAMPLE: usize = 4_096;
const TOKENS_PER_VALUE: usize = 64;

#[test]
fn editor893_notification_pipe_value_single_buffer_preserves_exact_text() {
    for value in [
        "",
        "plain",
        "  leading and trailing  ",
        "alpha|beta=gamma\tdelta\nepsilon\rzeta",
        "alpha|||===\t\n\r beta",
        "编辑器\u{00a0}通知\u{2003}消息",
        "emoji🙂|value=完成",
    ] {
        assert_eq!(pipe_value(value), legacy_pipe_value(value));
    }
}

#[test]
#[ignore = "release percentile evidence; run through the managed Windows validation lane"]
fn editor893_notification_pipe_value_single_buffer_release_percentiles() {
    let value = (0..TOKENS_PER_VALUE)
        .map(|index| format!("\u{2003}notification-{index:02}-{}|", "值".repeat(8)))
        .collect::<String>();
    let mut legacy_samples = Vec::with_capacity(SAMPLE_PAIRS);
    let mut optimized_samples = Vec::with_capacity(SAMPLE_PAIRS);

    for pair in 0..SAMPLE_PAIRS {
        if pair % 2 == 0 {
            legacy_samples.push(measure(|| render_batch(&value, legacy_pipe_value)));
            optimized_samples.push(measure(|| render_batch(&value, pipe_value)));
        } else {
            optimized_samples.push(measure(|| render_batch(&value, pipe_value)));
            legacy_samples.push(measure(|| render_batch(&value, legacy_pipe_value)));
        }
    }

    let legacy_p50_ns = percentile_ns(&mut legacy_samples.clone(), 50);
    let legacy_p95_ns = percentile_ns(&mut legacy_samples.clone(), 95);
    let legacy_p99_ns = percentile_ns(&mut legacy_samples, 99);
    let optimized_p50_ns = percentile_ns(&mut optimized_samples.clone(), 50);
    let optimized_p95_ns = percentile_ns(&mut optimized_samples.clone(), 95);
    let optimized_p99_ns = percentile_ns(&mut optimized_samples, 99);
    println!(
        "EDITOR893_NOTIFICATION_PIPE_VALUE_SINGLE_BUFFER_BENCH_V1 \
legacy_p50_ns={legacy_p50_ns} legacy_p95_ns={legacy_p95_ns} \
legacy_p99_ns={legacy_p99_ns} optimized_p50_ns={optimized_p50_ns} \
optimized_p95_ns={optimized_p95_ns} optimized_p99_ns={optimized_p99_ns}"
    );
    assert!(
        optimized_p95_ns <= legacy_p95_ns.saturating_mul(110).div_ceil(100),
        "single-buffer p95 regressed beyond 10%: legacy={legacy_p95_ns}ns optimized={optimized_p95_ns}ns"
    );
}

fn legacy_pipe_value(value: &str) -> String {
    value
        .chars()
        .map(|ch| match ch {
            '|' | '=' | '\n' | '\r' | '\t' => ' ',
            _ => ch,
        })
        .collect::<String>()
        .split_whitespace()
        .collect::<Vec<_>>()
        .join(" ")
}

fn render_batch(value: &str, render: fn(&str) -> String) -> usize {
    (0..VALUES_PER_SAMPLE)
        .map(|_| black_box(render(black_box(value))).len())
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
