use std::ffi::OsString;
use std::hint::black_box;
use std::time::Instant;

use super::{codepoint_range, text, FontSdfCliError};

const CHECKS_PER_SAMPLE: usize = 8192;
const SAMPLE_PAIRS: usize = 31;

fn legacy_codepoint(value: OsString) -> Result<u32, FontSdfCliError> {
    let value = text(value, "--codepoint")?;
    let digits = value
        .strip_prefix("U+")
        .or_else(|| value.strip_prefix("u+"))
        .ok_or_else(|| FontSdfCliError(format!("invalid codepoint {value}")))?;
    let codepoint = u32::from_str_radix(digits, 16)
        .map_err(|_| FontSdfCliError(format!("invalid codepoint {value}")))?;
    char::from_u32(codepoint)
        .map(|_| codepoint)
        .ok_or_else(|| FontSdfCliError(format!("invalid Unicode scalar {value}")))
}

fn legacy_codepoint_range(value: OsString) -> Result<Vec<u32>, FontSdfCliError> {
    let value = text(value, "--codepoint-range")?;
    let (start, end) = value
        .split_once('-')
        .ok_or_else(|| FontSdfCliError(format!("invalid codepoint range {value}")))?;
    let start = legacy_codepoint(OsString::from(start))?;
    let end = legacy_codepoint(OsString::from(end))?;
    if end < start {
        return Err(FontSdfCliError(format!(
            "codepoint range is reversed: {value}"
        )));
    }
    let mut codepoints = Vec::with_capacity((end - start + 1) as usize);
    for scalar in start..=end {
        if char::from_u32(scalar).is_none() {
            return Err(FontSdfCliError(format!(
                "codepoint range contains a non-scalar value: {value}"
            )));
        }
        codepoints.push(scalar);
    }
    Ok(codepoints)
}

fn measure(value: &OsString, optimized: bool) -> u128 {
    let started = Instant::now();
    let mut scalars = 0;
    for _ in 0..CHECKS_PER_SAMPLE {
        let parsed = if optimized {
            codepoint_range(black_box(value.clone()))
        } else {
            legacy_codepoint_range(black_box(value.clone()))
        };
        scalars += parsed.expect("benchmark range should parse").len();
    }
    black_box(scalars);
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
fn optimization_batch_20260829br_runtime345_codepoint_ranges_preserve_results() {
    for value in [
        "U+0041-U+0041",
        "u+0041-U+0043",
        "U+0043-U+0041",
        "U+D7FF-U+E000",
    ] {
        let candidate = codepoint_range(OsString::from(value)).map_err(|error| error.to_string());
        let baseline =
            legacy_codepoint_range(OsString::from(value)).map_err(|error| error.to_string());
        assert_eq!(candidate, baseline, "{value:?}");
    }
}

#[test]
fn optimization_batch_20260829br_runtime345_codepoint_range_borrows_endpoints() {
    let source = include_str!("../args.rs");
    let production = source.split_once("#[cfg(test)]").expect("test boundary").0;
    let range = production
        .split_once("fn codepoint_range")
        .expect("range parser")
        .1
        .split_once("fn hash")
        .expect("hash boundary")
        .0;
    assert!(production.contains("fn codepoint_text(value: &str)"));
    assert!(range.contains("codepoint_text(start)"));
    assert!(range.contains("codepoint_text(end)"));
    assert!(!range.contains("OsString::from"));
}

#[test]
#[ignore = "managed Windows release performance evidence"]
fn optimization_batch_20260829br_runtime345_borrowed_codepoint_range_bench() {
    let value = OsString::from("U+0041-U+0041");
    let mut baseline = Vec::with_capacity(SAMPLE_PAIRS);
    let mut candidate = Vec::with_capacity(SAMPLE_PAIRS);
    for pair in 0..SAMPLE_PAIRS {
        if pair % 2 == 0 {
            baseline.push(measure(&value, false));
            candidate.push(measure(&value, true));
        } else {
            candidate.push(measure(&value, true));
            baseline.push(measure(&value, false));
        }
    }
    let baseline_p50_ns = percentile(&baseline, 50);
    let candidate_p50_ns = percentile(&candidate, 50);
    let baseline_p95_ns = percentile(&baseline, 95);
    let candidate_p95_ns = percentile(&candidate, 95);
    println!(
        "RUNTIME345_BORROWED_CODEPOINT_RANGE_BENCH_V1 sample_pairs={SAMPLE_PAIRS} checks_per_sample={CHECKS_PER_SAMPLE} baseline_endpoint_allocations_per_check=2 candidate_endpoint_allocations_per_check=0 baseline_p50_ns={baseline_p50_ns} candidate_p50_ns={candidate_p50_ns} baseline_p95_ns={baseline_p95_ns} candidate_p95_ns={candidate_p95_ns} baseline_raw_ns={} candidate_raw_ns={}",
        sample_csv(&baseline),
        sample_csv(&candidate)
    );
    assert!(candidate_p95_ns.saturating_mul(100) <= baseline_p95_ns.saturating_mul(70));
}
