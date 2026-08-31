use std::hint::black_box;
use std::time::Instant;

use super::{uv_value_issue, SpriteAtlasUvValueIssue};

const CHECKS_PER_SAMPLE: usize = 500_000;
const SAMPLE_PAIRS: usize = 31;

fn legacy_uv_value_issue(min: [f32; 2], max: [f32; 2]) -> Option<SpriteAtlasUvValueIssue> {
    if !min.iter().chain(max.iter()).all(|value| value.is_finite()) {
        return Some(SpriteAtlasUvValueIssue::NonFinite);
    }
    if !min
        .iter()
        .chain(max.iter())
        .all(|value| (0.0..=1.0).contains(value))
    {
        return Some(SpriteAtlasUvValueIssue::OutOfRange);
    }
    None
}

fn measure(min: [f32; 2], max: [f32; 2], optimized: bool) -> u128 {
    let started = Instant::now();
    let mut issue = None;
    for _ in 0..CHECKS_PER_SAMPLE {
        issue = if optimized {
            uv_value_issue(black_box(min), black_box(max))
        } else {
            legacy_uv_value_issue(black_box(min), black_box(max))
        };
    }
    black_box(issue);
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
fn optimization_batch_20260829bu_runtime348_uv_value_issues_preserve_results() {
    for (min, max) in [
        ([0.0, 0.25], [0.75, 1.0]),
        ([-0.1, 0.25], [0.75, 1.0]),
        ([-0.1, 0.25], [f32::NAN, 1.0]),
        ([0.0, f32::INFINITY], [0.75, 1.0]),
    ] {
        assert_eq!(
            uv_value_issue(min, max),
            legacy_uv_value_issue(min, max),
            "min={min:?} max={max:?}"
        );
    }
}

#[test]
fn optimization_batch_20260829bu_runtime348_uv_values_use_one_scan() {
    let source = include_str!("../validation.rs");
    let production = source.split_once("#[cfg(test)]").expect("test boundary").0;
    let function = production
        .split_once("fn uv_value_issue")
        .expect("UV classifier")
        .1
        .split_once("fn uv_rects_match")
        .expect("UV match boundary")
        .0;
    assert!(function.contains("for value in min.into_iter().chain(max)"));
    assert!(function.contains("out_of_range |= "));
    assert_eq!(function.matches(".all(").count(), 0);
}

#[test]
#[ignore = "managed Windows release performance evidence"]
fn optimization_batch_20260829bu_runtime348_single_pass_sprite_uv_bench() {
    let min = [0.125, 0.25];
    let max = [0.75, 0.875];
    let mut baseline = Vec::with_capacity(SAMPLE_PAIRS);
    let mut candidate = Vec::with_capacity(SAMPLE_PAIRS);
    for pair in 0..SAMPLE_PAIRS {
        if pair % 2 == 0 {
            baseline.push(measure(min, max, false));
            candidate.push(measure(min, max, true));
        } else {
            candidate.push(measure(min, max, true));
            baseline.push(measure(min, max, false));
        }
    }
    let baseline_p50_ns = percentile(&baseline, 50);
    let candidate_p50_ns = percentile(&candidate, 50);
    let baseline_p95_ns = percentile(&baseline, 95);
    let candidate_p95_ns = percentile(&candidate, 95);
    println!(
        "RUNTIME348_SINGLE_PASS_SPRITE_UV_BENCH_V1 sample_pairs={SAMPLE_PAIRS} checks_per_sample={CHECKS_PER_SAMPLE} uv_values=4 baseline_value_scans=2 candidate_value_scans=1 baseline_p50_ns={baseline_p50_ns} candidate_p50_ns={candidate_p50_ns} baseline_p95_ns={baseline_p95_ns} candidate_p95_ns={candidate_p95_ns} baseline_raw_ns={} candidate_raw_ns={}",
        sample_csv(&baseline),
        sample_csv(&candidate)
    );
    assert!(candidate_p95_ns.saturating_mul(100) <= baseline_p95_ns.saturating_mul(70));
}
