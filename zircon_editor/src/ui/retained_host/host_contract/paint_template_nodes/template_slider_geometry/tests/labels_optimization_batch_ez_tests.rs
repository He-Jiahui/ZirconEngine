use std::hint::black_box;
use std::time::Instant;

use super::*;

const SAMPLE_PAIRS: usize = 17;
const LABELS_PER_SAMPLE: usize = 262_144;

#[test]
fn optimization_batch_ez_editor388_preserves_slider_percent_labels() {
    let mut values = vec![
        f32::NEG_INFINITY,
        -1.0,
        -0.0,
        0.0,
        0.004,
        0.005,
        0.015,
        0.125,
        0.995,
        1.0,
        2.0,
        f32::INFINITY,
        f32::NAN,
    ];
    for boundary_index in 0..100_u32 {
        let boundary = ((f64::from(boundary_index) + 0.5) / 100.0) as f32;
        values.extend([
            f32::from_bits(boundary.to_bits() - 1),
            boundary,
            f32::from_bits(boundary.to_bits() + 1),
        ]);
    }

    for percent in values {
        assert_eq!(
            slider_range_min_label(percent),
            legacy_percent_label(percent),
            "percent bits {:08x}",
            percent.to_bits()
        );
    }
}

#[test]
#[ignore = "release performance gate"]
fn optimization_batch_ez_editor388_direct_slider_percent_label_benchmark() {
    for _ in 0..4 {
        black_box(measure(legacy_percent_label));
        black_box(measure(slider_range_min_label));
    }
    let mut legacy_samples = Vec::with_capacity(SAMPLE_PAIRS);
    let mut optimized_samples = Vec::with_capacity(SAMPLE_PAIRS);
    for pair_index in 0..SAMPLE_PAIRS {
        if pair_index % 2 == 0 {
            legacy_samples.push(measure(legacy_percent_label));
            optimized_samples.push(measure(slider_range_min_label));
        } else {
            optimized_samples.push(measure(slider_range_min_label));
            legacy_samples.push(measure(legacy_percent_label));
        }
    }

    report_performance(&legacy_samples, &optimized_samples);
}

fn legacy_percent_label(percent: f32) -> String {
    format!("{:.2}", percent.clamp(0.0, 1.0))
}

fn measure(mut label: impl FnMut(f32) -> String) -> u128 {
    let started = Instant::now();
    let mut checksum = 0_usize;
    for index in 0..LABELS_PER_SAMPLE {
        let percent = black_box((index % 1_001) as f32 / 1_000.0);
        let value = label(percent);
        checksum = checksum.wrapping_add(black_box(value.len()));
        black_box(value);
    }
    black_box(checksum);
    started.elapsed().as_nanos().max(1)
}

fn report_performance(legacy_samples: &[u128], optimized_samples: &[u128]) {
    let legacy_p95 = nearest_rank_p95(legacy_samples);
    let optimized_p95 = nearest_rank_p95(optimized_samples);
    let improvement_percent =
        legacy_p95.saturating_sub(optimized_p95).saturating_mul(100) / legacy_p95.max(1);
    println!(
        "EDITOR388_DIRECT_SLIDER_PERCENT_LABEL_BENCH_V1 sample_pairs={SAMPLE_PAIRS} labels_per_sample={LABELS_PER_SAMPLE} legacy_ns={} optimized_ns={} legacy_p95_ns={legacy_p95} optimized_p95_ns={optimized_p95} improvement_percent={improvement_percent} threshold_percent=30",
        csv(legacy_samples),
        csv(optimized_samples),
    );
    assert!(
        optimized_p95 <= legacy_p95.saturating_mul(70) / 100,
        "direct slider percent labels must reduce P95 by at least 30%"
    );
}

fn nearest_rank_p95(samples: &[u128]) -> u128 {
    let mut sorted = samples.to_vec();
    sorted.sort_unstable();
    let rank = (sorted.len() * 95).div_ceil(100);
    sorted[rank.saturating_sub(1)]
}

fn csv(samples: &[u128]) -> String {
    samples
        .iter()
        .map(u128::to_string)
        .collect::<Vec<_>>()
        .join(",")
}
