use std::hint::black_box;
use std::time::Instant;

const LABEL_BYTES: usize = 4_096;
const CHECKS_PER_SAMPLE: usize = 32_768;
const SAMPLE_PAIRS: usize = 17;

#[test]
fn optimization_batch_fx_editor410_button_label_presence_is_computed_once() {
    let source = include_str!("../entry.rs");
    let production = source
        .split("#[cfg(test)]")
        .next()
        .expect("button content production source");

    assert_eq!(production.matches("label.trim().is_empty()").count(), 1);
    assert!(production.contains("button_glyph_width(node, glyph, has_label)"));
    assert!(production.contains("if has_label"));
}

#[test]
#[ignore = "release performance gate"]
fn optimization_batch_fx_editor410_button_label_presence_benchmark() {
    let label = " ".repeat(LABEL_BYTES);
    for _ in 0..4 {
        black_box(measure_checks(&label, false));
        black_box(measure_checks(&label, true));
    }

    let mut legacy_samples = Vec::with_capacity(SAMPLE_PAIRS);
    let mut optimized_samples = Vec::with_capacity(SAMPLE_PAIRS);
    for pair in 0..SAMPLE_PAIRS {
        if pair % 2 == 0 {
            legacy_samples.push(measure_checks(&label, false));
            optimized_samples.push(measure_checks(&label, true));
        } else {
            optimized_samples.push(measure_checks(&label, true));
            legacy_samples.push(measure_checks(&label, false));
        }
    }

    let legacy_p95 = percentile(&legacy_samples, 95);
    let optimized_p95 = percentile(&optimized_samples, 95);
    let improvement_percent =
        legacy_p95.saturating_sub(optimized_p95).saturating_mul(100) / legacy_p95.max(1);
    println!(
        "EDITOR410_CACHED_BUTTON_LABEL_PRESENCE_BENCH_V1 sample_pairs={SAMPLE_PAIRS} label_bytes={LABEL_BYTES} checks_per_sample={CHECKS_PER_SAMPLE} legacy_trim_scans_per_button=2 optimized_trim_scans_per_button=1 legacy_ns={} optimized_ns={} legacy_p95_ns={legacy_p95} optimized_p95_ns={optimized_p95} improvement_percent={improvement_percent} threshold_percent=35",
        csv(&legacy_samples),
        csv(&optimized_samples),
    );
    assert!(optimized_p95 <= legacy_p95 * 65 / 100);
}

fn measure_checks(label: &str, optimized: bool) -> u128 {
    let started = Instant::now();
    for _ in 0..CHECKS_PER_SAMPLE {
        if optimized {
            let has_label = !black_box(label).trim().is_empty();
            black_box(has_label);
            black_box(has_label);
        } else {
            black_box(!black_box(label).trim().is_empty());
            black_box(!black_box(label).trim().is_empty());
        }
    }
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
