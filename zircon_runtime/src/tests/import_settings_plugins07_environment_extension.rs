use std::hint::black_box;
use std::time::Instant;

use super::*;

const SAMPLE_PAIRS: usize = 21;
const CHECKS_PER_SAMPLE: usize = 240_000;
const EXTENSIONS: [&str; 6] = ["hdr", "HDR", "ExR", "png", "ktx2", "jpeg"];

#[test]
fn borrowed_import_keyword_contract_environment_ibl_extension() {
    assert!(is_environment_ibl_extension("HDR"));
    assert!(is_environment_ibl_extension("eXr"));
    assert!(!is_environment_ibl_extension("png"));
    assert!(!is_environment_ibl_extension("hdr.backup"));
}

#[test]
#[ignore = "release performance gate"]
fn borrowed_import_keyword_performance_release_environment_ibl_extension() {
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
        "PERF_RESULT plugins07_environment_ibl_borrowed_extension sample_pairs={SAMPLE_PAIRS} checks_per_sample={CHECKS_PER_SAMPLE} legacy_ns={} optimized_ns={} legacy_p95_ns={legacy_p95} optimized_p95_ns={optimized_p95} improvement_percent={improvement_percent} threshold_percent=50 legacy_allocations_per_sample={CHECKS_PER_SAMPLE} optimized_allocations_per_sample=0 order=alternating_legacy_first_even legacy_first_pairs=11 optimized_first_pairs=10",
        csv(&legacy_samples),
        csv(&optimized_samples),
    );
    assert!(
        improvement_percent >= 50,
        "borrowed environment IBL extension classification must improve P95 by at least 50%"
    );
}

fn measure_legacy() -> u128 {
    let started = Instant::now();
    let mut matched = 0_u64;
    for check in 0..CHECKS_PER_SAMPLE {
        let normalized = black_box(EXTENSIONS[check % EXTENSIONS.len()]).to_ascii_lowercase();
        matched += u64::from(matches!(normalized.as_str(), "hdr" | "exr"));
        black_box(normalized);
    }
    black_box(matched);
    started.elapsed().as_nanos().max(1)
}

fn measure_borrowed() -> u128 {
    let started = Instant::now();
    let mut matched = 0_u64;
    for check in 0..CHECKS_PER_SAMPLE {
        matched += u64::from(is_environment_ibl_extension(black_box(
            EXTENSIONS[check % EXTENSIONS.len()],
        )));
    }
    black_box(matched);
    started.elapsed().as_nanos().max(1)
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
