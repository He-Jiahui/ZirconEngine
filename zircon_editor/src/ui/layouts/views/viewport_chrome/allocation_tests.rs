use std::hint::black_box;
use std::time::Instant;

use super::*;

const SAMPLE_PAIRS: usize = 21;
const CHROME_STATES_PER_SAMPLE: usize = 131_072;

#[test]
fn optimization_batch_20260826gl_editor178_single_pass_snap_labels_preserve_precision() {
    assert_eq!(format_step_label('T', 1.0), "T 1");
    assert_eq!(format_step_label('R', 1.5), "R 1.5");
    assert_eq!(format_step_label('S', 0.25), "S 0.25");
}

#[test]
fn optimization_batch_20260826gl_editor178_viewport_chrome_formats_final_labels_once() {
    let source = include_str!("../viewport_chrome.rs");

    assert_eq!(source.matches("format_step_label(").count(), 4);
    assert!(source.contains("format!(\"{prefix} {value:.0}\")"));
    assert!(!source.contains("format!(\"T {}\", format_step("));
    assert!(!source.contains("format!(\"R {}\", format_step("));
    assert!(!source.contains("format!(\"S {}\", format_step("));
}

#[test]
#[ignore = "managed Windows release performance evidence"]
fn optimization_batch_20260826gl_editor178_viewport_snap_label_single_pass_formatting_bench() {
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
        "EDITOR178_VIEWPORT_SNAP_LABEL_SINGLE_PASS_FORMATTING_BENCH_V1 sample_pairs={SAMPLE_PAIRS} \
chrome_states_per_sample={CHROME_STATES_PER_SAMPLE} labels_per_state=3 \
legacy_string_allocations_per_state=6 optimized_string_allocations_per_state=3 \
legacy_p50_ns={legacy_p50_ns} optimized_p50_ns={optimized_p50_ns} \
legacy_p95_ns={legacy_p95_ns} optimized_p95_ns={optimized_p95_ns} \
legacy_raw_ns={} optimized_raw_ns={}",
        sample_csv(&legacy_samples),
        sample_csv(&optimized_samples),
    );
    assert!(optimized_p95_ns.saturating_mul(100) <= legacy_p95_ns.saturating_mul(70));
}

fn measure(single_pass: bool) -> u128 {
    let started = Instant::now();
    let mut checksum = 0usize;
    for state in 0..CHROME_STATES_PER_SAMPLE {
        let values = black_box(match state % 3 {
            0 => [1.0_f32, 15.0, 0.0],
            1 => [1.5_f32, 22.5, 0.5],
            _ => [0.25_f32, 7.25, 0.05],
        });
        let labels = if single_pass {
            [
                format_step_label('T', values[0]),
                format_step_label('R', values[1]),
                format_step_label('S', values[2]),
            ]
        } else {
            [
                format!("T {}", legacy_format_step(values[0])),
                format!("R {}", legacy_format_step(values[1])),
                format!("S {}", legacy_format_step(values[2])),
            ]
        };
        checksum ^= black_box(labels.iter().map(String::len).sum::<usize>() ^ state);
        black_box(labels);
    }
    black_box(checksum);
    started.elapsed().as_nanos().max(1)
}

fn legacy_format_step(value: f32) -> String {
    if value.fract().abs() <= f32::EPSILON {
        format!("{value:.0}")
    } else if (value * 10.0).fract().abs() <= f32::EPSILON {
        format!("{value:.1}")
    } else {
        format!("{value:.2}")
    }
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
