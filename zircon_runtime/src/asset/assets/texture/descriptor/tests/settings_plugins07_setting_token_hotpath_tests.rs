use std::hint::black_box;
use std::time::Instant;

use super::*;

const SAMPLE_PAIRS: usize = 21;
const LOOKUPS_PER_SAMPLE: usize = 80_000;
const TOKENS: [&str; 3] = ["render_target", "copy_src", "copy_dst"];

#[test]
fn borrowed_texture_metadata_contract_setting_token() {
    assert!(matches!(
        normalized_token(" render_target "),
        std::borrow::Cow::Borrowed("render_target")
    ));
    assert!(matches!(
        normalized_token("Render-Target"),
        std::borrow::Cow::Owned(ref value) if value == "render_target"
    ));
    assert_eq!(
        parse_usage("Render-Target"),
        Ok(RenderImageUsage::RenderTarget)
    );
}

#[test]
#[ignore = "release performance gate"]
fn borrowed_texture_metadata_performance_release_setting_token() {
    let mut legacy_samples = Vec::with_capacity(SAMPLE_PAIRS);
    let mut optimized_samples = Vec::with_capacity(SAMPLE_PAIRS);
    for pair_index in 0..SAMPLE_PAIRS {
        let (legacy_ns, optimized_ns) = if pair_index % 2 == 0 {
            (measure_legacy(), measure_borrowed())
        } else {
            let optimized_ns = measure_borrowed();
            (measure_legacy(), optimized_ns)
        };
        legacy_samples.push(legacy_ns);
        optimized_samples.push(optimized_ns);
    }

    let legacy_p95 = nearest_rank_p95(&legacy_samples);
    let optimized_p95 = nearest_rank_p95(&optimized_samples);
    let improvement_percent =
        legacy_p95.saturating_sub(optimized_p95).saturating_mul(100) / legacy_p95.max(1);
    println!(
        "PERF_RESULT plugins07_texture_setting_token_borrow sample_pairs={SAMPLE_PAIRS} legacy_ns={} optimized_ns={} legacy_p95_ns={legacy_p95} optimized_p95_ns={optimized_p95} improvement_percent={improvement_percent} threshold_percent=50 legacy_allocations_per_sample={} optimized_allocations_per_sample=0 order=alternating_legacy_first_even legacy_first_pairs=11 optimized_first_pairs=10",
        csv(&legacy_samples),
        csv(&optimized_samples),
        LOOKUPS_PER_SAMPLE * TOKENS.len() * 2,
    );
    assert!(
        improvement_percent >= 50,
        "borrowed canonical texture setting tokens must improve P95 by at least 50%"
    );
}

fn measure_legacy() -> u128 {
    let started = Instant::now();
    let mut parsed = 0_u64;
    for _ in 0..LOOKUPS_PER_SAMPLE {
        for token in TOKENS {
            let normalized = black_box(token)
                .trim()
                .to_ascii_lowercase()
                .replace('-', "_");
            parsed += u64::from(matches!(
                normalized.as_str(),
                "render_target" | "copy_src" | "copy_dst"
            ));
            black_box(normalized);
        }
    }
    black_box(parsed);
    started.elapsed().as_nanos()
}

fn measure_borrowed() -> u128 {
    let started = Instant::now();
    let mut parsed = 0_u64;
    for _ in 0..LOOKUPS_PER_SAMPLE {
        for token in TOKENS {
            parsed += u64::from(parse_usage(black_box(token)).is_ok());
        }
    }
    black_box(parsed);
    started.elapsed().as_nanos()
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
