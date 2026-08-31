use std::hint::black_box;
use std::time::Instant;

use super::{FrameRect, PresentDamage};

const SAMPLE_PAIRS: usize = 21;
const FORMATS_PER_SAMPLE: usize = 131_072;

#[test]
fn optimization_batch_20260826dt_editor109_present_damage_preserves_summary() {
    let frame = FrameRect {
        x: 12.25,
        y: 7.75,
        width: 640.0,
        height: 360.5,
    };
    assert_eq!(
        PresentDamage(Some(&frame)).to_string(),
        "12.2,7.8,640.0,360.5"
    );
    assert_eq!(PresentDamage(None).to_string(), "full");
}

#[test]
fn optimization_batch_20260826dt_editor109_present_damage_writes_through_display() {
    let source = include_str!("../log.rs");
    assert!(source.contains("PresentDamage(outcome.damage.as_ref())"));
    assert!(source.contains("impl fmt::Display for PresentDamage<'_>"));
    assert!(!source.contains(".map(frame_summary)"));
    assert!(!source.contains(".unwrap_or_else(|| \"full\".to_string())"));
}

#[test]
#[ignore = "managed Windows release performance evidence"]
fn optimization_batch_20260826dt_editor109_present_damage_display_adapter_bench() {
    let mut legacy_samples = Vec::with_capacity(SAMPLE_PAIRS);
    let mut optimized_samples = Vec::with_capacity(SAMPLE_PAIRS);

    for pair in 0..SAMPLE_PAIRS {
        if pair % 2 == 0 {
            legacy_samples.push(measure(legacy_full_present_log));
            optimized_samples.push(measure(optimized_full_present_log));
        } else {
            optimized_samples.push(measure(optimized_full_present_log));
            legacy_samples.push(measure(legacy_full_present_log));
        }
    }

    let legacy_p50_ns = percentile(&legacy_samples, 50);
    let optimized_p50_ns = percentile(&optimized_samples, 50);
    let legacy_p95_ns = percentile(&legacy_samples, 95);
    let optimized_p95_ns = percentile(&optimized_samples, 95);
    println!(
        "EDITOR109_PRESENT_DAMAGE_DISPLAY_ADAPTER_BENCH_V1 path=full_repaint sample_pairs={SAMPLE_PAIRS} \
formats_per_sample={FORMATS_PER_SAMPLE} legacy_allocations_per_format=2 \
optimized_allocations_per_format=1 legacy_p50_ns={legacy_p50_ns} \
optimized_p50_ns={optimized_p50_ns} legacy_p95_ns={legacy_p95_ns} \
optimized_p95_ns={optimized_p95_ns} legacy_raw_ns={} optimized_raw_ns={}",
        sample_csv(&legacy_samples),
        sample_csv(&optimized_samples),
    );
    assert!(
        optimized_p95_ns.saturating_mul(100) <= legacy_p95_ns.saturating_mul(70),
        "present damage Display adapter P95 {optimized_p95_ns}ns must be at most 70% of intermediate summary formatting P95 {legacy_p95_ns}ns"
    );
}

fn legacy_full_present_log() -> String {
    let damage = "full".to_string();
    format!("present damage={damage}")
}

fn optimized_full_present_log() -> String {
    format!("present damage={}", PresentDamage(None))
}

fn measure(render: fn() -> String) -> u128 {
    let started = Instant::now();
    let mut checksum = 0usize;
    for _ in 0..FORMATS_PER_SAMPLE {
        checksum ^= black_box(render()).len();
    }
    black_box(checksum);
    started.elapsed().as_nanos().max(1)
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
