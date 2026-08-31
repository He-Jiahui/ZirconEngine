use std::hint::black_box;
use std::time::Instant;

use super::extension_from_locator;

const CHECKS_PER_SAMPLE: usize = 8192;
const LOCATOR_BYTES: usize = 4096;
const SAMPLE_PAIRS: usize = 31;

fn legacy_extension_from_locator(locator: &str) -> String {
    let file_name = locator
        .rsplit(|ch| ch == '/' || ch == '\\')
        .next()
        .unwrap_or(locator);
    file_name
        .rsplit_once('.')
        .map(|(_, extension)| extension)
        .filter(|extension| !extension.is_empty())
        .unwrap_or_default()
        .to_string()
}

fn measure(locator: &str, optimized: bool) -> u128 {
    let started = Instant::now();
    let mut extension_bytes = 0;
    for _ in 0..CHECKS_PER_SAMPLE {
        let extension = if optimized {
            extension_from_locator(black_box(locator))
        } else {
            legacy_extension_from_locator(black_box(locator))
        };
        extension_bytes += extension.len();
        black_box(extension);
    }
    black_box(extension_bytes);
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

#[test]
fn optimization_batch_20260829bg_editor279_single_scan_extensions_preserve_results() {
    for locator in [
        "project/assets/mesh.zmesh",
        "project\\assets\\mesh.zmesh",
        "mesh.zmesh",
        "mesh",
        "project.with.dot/mesh",
        "project/.hidden",
        "project/mesh.",
        "project/\u{4f8b}.texture",
        "",
    ] {
        assert_eq!(
            extension_from_locator(locator),
            legacy_extension_from_locator(locator),
            "{locator:?}"
        );
    }
}

#[test]
fn optimization_batch_20260829bg_editor279_extension_parser_uses_one_reverse_scan() {
    let source = include_str!("../reference.rs");
    let production = source.split_once("#[cfg(test)]").expect("test boundary").0;

    assert!(production.contains("locator.bytes().enumerate().rev()"));
    assert!(!production.contains(".rsplit("));
    assert!(!production.contains(".rsplit_once("));
}

#[test]
#[ignore = "managed Windows release performance evidence"]
fn optimization_batch_20260829bg_editor279_single_scan_asset_extension_bench() {
    let prefix = "project/";
    let suffix = ".bin";
    let locator = format!(
        "{prefix}{}{suffix}",
        "a".repeat(LOCATOR_BYTES - prefix.len() - suffix.len())
    );
    assert_eq!(locator.len(), LOCATOR_BYTES);
    let mut baseline_samples = Vec::with_capacity(SAMPLE_PAIRS);
    let mut candidate_samples = Vec::with_capacity(SAMPLE_PAIRS);
    for pair in 0..SAMPLE_PAIRS {
        if pair % 2 == 0 {
            baseline_samples.push(measure(&locator, false));
            candidate_samples.push(measure(&locator, true));
        } else {
            candidate_samples.push(measure(&locator, true));
            baseline_samples.push(measure(&locator, false));
        }
    }

    let baseline_p50_ns = percentile(&baseline_samples, 50);
    let candidate_p50_ns = percentile(&candidate_samples, 50);
    let baseline_p95_ns = percentile(&baseline_samples, 95);
    let candidate_p95_ns = percentile(&candidate_samples, 95);
    println!(
        "EDITOR279_SINGLE_SCAN_ASSET_EXTENSION_BENCH_V1 sample_pairs={SAMPLE_PAIRS} \
checks_per_sample={CHECKS_PER_SAMPLE} locator_bytes={LOCATOR_BYTES} \
baseline_reverse_scans=2 candidate_reverse_scans=1 \
baseline_p50_ns={baseline_p50_ns} candidate_p50_ns={candidate_p50_ns} \
baseline_p95_ns={baseline_p95_ns} candidate_p95_ns={candidate_p95_ns} \
baseline_raw_ns={} candidate_raw_ns={}",
        sample_csv(&baseline_samples),
        sample_csv(&candidate_samples),
    );
    assert!(candidate_p95_ns.saturating_mul(100) <= baseline_p95_ns.saturating_mul(70));
}
