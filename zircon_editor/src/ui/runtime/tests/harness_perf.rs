use std::hint::black_box;
use std::time::Instant;

use super::{render_number, trim_rendered_fraction};

const FRACTION_TEXT: &str = "340282346638528859811704183484516925440.200";
const TRIMS_PER_SAMPLE: usize = 131_072;
const SAMPLE_PAIRS: usize = 17;

#[test]
fn optimization_batch_fv_editor408_fraction_trimming_preserves_rendered_values() {
    let mut rendered = FRACTION_TEXT.to_string();
    trim_rendered_fraction(&mut rendered);

    assert_eq!(rendered, "340282346638528859811704183484516925440.2");
    assert_eq!(render_number(1.25), "1.25");
    assert_eq!(render_number(2.0), "2");
    assert_eq!(render_number(f32::INFINITY), "inf");
    assert_eq!(render_number(f32::NAN), "NaN");
}

#[test]
#[ignore = "release performance gate"]
fn optimization_batch_fv_editor408_fraction_tail_trim_benchmark() {
    for _ in 0..4 {
        black_box(measure_trims(false));
        black_box(measure_trims(true));
    }

    let mut legacy_samples = Vec::with_capacity(SAMPLE_PAIRS);
    let mut optimized_samples = Vec::with_capacity(SAMPLE_PAIRS);
    for pair in 0..SAMPLE_PAIRS {
        if pair % 2 == 0 {
            legacy_samples.push(measure_trims(false));
            optimized_samples.push(measure_trims(true));
        } else {
            optimized_samples.push(measure_trims(true));
            legacy_samples.push(measure_trims(false));
        }
    }

    let legacy_p95 = percentile(&legacy_samples, 95);
    let optimized_p95 = percentile(&optimized_samples, 95);
    let improvement_percent =
        legacy_p95.saturating_sub(optimized_p95).saturating_mul(100) / legacy_p95.max(1);
    println!(
        "EDITOR408_FRACTION_TAIL_TRIM_BENCH_V1 sample_pairs={SAMPLE_PAIRS} trims_per_sample={TRIMS_PER_SAMPLE} input_bytes={} legacy_full_scans_per_call=3 optimized_full_scans_per_call=0 legacy_ns={} optimized_ns={} legacy_p95_ns={legacy_p95} optimized_p95_ns={optimized_p95} improvement_percent={improvement_percent} threshold_percent=40",
        FRACTION_TEXT.len(),
        csv(&legacy_samples),
        csv(&optimized_samples),
    );
    assert!(optimized_p95 <= legacy_p95 * 60 / 100);
}

fn measure_trims(optimized: bool) -> u128 {
    let mut rendered = String::with_capacity(FRACTION_TEXT.len());
    let started = Instant::now();
    for _ in 0..TRIMS_PER_SAMPLE {
        rendered.clear();
        rendered.push_str(black_box(FRACTION_TEXT));
        if optimized {
            trim_rendered_fraction(&mut rendered);
        } else {
            while rendered.contains('.') && rendered.ends_with('0') {
                rendered.pop();
            }
            if rendered.ends_with('.') {
                rendered.pop();
            }
        }
        black_box(rendered.len());
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
