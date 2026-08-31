use std::hint::black_box;
use std::path::PathBuf;
use std::time::Instant;

use super::{push_candidate, push_svg_variants};

const SAMPLE_PAIRS: usize = 21;
const PUSHES_PER_SAMPLE: usize = 8_192;
const CANDIDATE_COUNT: usize = 256;

#[test]
fn optimization_batch_20260826eh_editor123_candidate_variants_preserve_order_and_deduplication() {
    let mut candidates = Vec::new();
    push_svg_variants(&mut candidates, PathBuf::new());
    assert!(candidates.is_empty());

    push_svg_variants(&mut candidates, PathBuf::from("icons/add"));
    assert_eq!(
        candidates,
        vec![PathBuf::from("icons/add.svg"), PathBuf::from("icons/add")]
    );
    push_candidate(&mut candidates, PathBuf::from("icons/add"));
    push_candidate(&mut candidates, PathBuf::from("icons/add.svg"));
    assert_eq!(candidates.len(), 2);

    push_svg_variants(&mut candidates, PathBuf::from("icons/remove.png"));
    assert_eq!(candidates.last(), Some(&PathBuf::from("icons/remove.png")));
}

#[test]
fn optimization_batch_20260826eh_editor123_candidate_dedup_checks_last_before_scan() {
    let source = include_str!("../variants.rs");
    let function_start = source.find("pub(super) fn push_candidate").unwrap();
    let function_end = source[function_start..]
        .find("#[cfg(test)]")
        .map(|offset| function_start + offset)
        .unwrap();
    let function_source = &source[function_start..function_end];
    let last_check = function_source.find("candidates.last()").unwrap();
    let full_scan = function_source.find("candidates.iter().any").unwrap();
    assert!(last_check < full_scan);
    assert_eq!(function_source.matches("candidates.push(path)").count(), 1);
}

#[test]
#[ignore = "managed Windows release performance evidence"]
fn optimization_batch_20260826eh_editor123_visual_asset_candidate_last_hit_bench() {
    let (candidates, duplicate) = candidate_fixture();
    let mut legacy_samples = Vec::with_capacity(SAMPLE_PAIRS);
    let mut optimized_samples = Vec::with_capacity(SAMPLE_PAIRS);
    for pair in 0..SAMPLE_PAIRS {
        if pair % 2 == 0 {
            legacy_samples.push(measure_legacy(&candidates, &duplicate));
            optimized_samples.push(measure_optimized(&candidates, &duplicate));
        } else {
            optimized_samples.push(measure_optimized(&candidates, &duplicate));
            legacy_samples.push(measure_legacy(&candidates, &duplicate));
        }
    }
    let legacy_p50_ns = percentile(&legacy_samples, 50);
    let optimized_p50_ns = percentile(&optimized_samples, 50);
    let legacy_p95_ns = percentile(&legacy_samples, 95);
    let optimized_p95_ns = percentile(&optimized_samples, 95);
    println!(
        "EDITOR123_VISUAL_ASSET_CANDIDATE_LAST_HIT_BENCH_V1 sample_pairs={SAMPLE_PAIRS} \
pushes_per_sample={PUSHES_PER_SAMPLE} candidate_count={CANDIDATE_COUNT} \
legacy_path_comparisons_per_last_hit={CANDIDATE_COUNT} optimized_path_comparisons_per_last_hit=1 \
legacy_p50_ns={legacy_p50_ns} optimized_p50_ns={optimized_p50_ns} \
legacy_p95_ns={legacy_p95_ns} optimized_p95_ns={optimized_p95_ns} \
legacy_raw_ns={} optimized_raw_ns={}",
        sample_csv(&legacy_samples),
        sample_csv(&optimized_samples),
    );
    assert!(
        optimized_p95_ns.saturating_mul(100) <= legacy_p95_ns.saturating_mul(70),
        "last-candidate fast hit P95 {optimized_p95_ns}ns must be at most 70% of full duplicate scan P95 {legacy_p95_ns}ns"
    );
}

fn candidate_fixture() -> (Vec<PathBuf>, PathBuf) {
    let candidates = (0..CANDIDATE_COUNT)
        .map(|index| {
            PathBuf::from(format!(
                "project/assets/generated/material_icons/category_{:04}/icon_{index:04}.svg",
                index / 16
            ))
        })
        .collect::<Vec<_>>();
    let duplicate = candidates.last().unwrap().clone();
    (candidates, duplicate)
}

fn legacy_push_candidate(candidates: &mut Vec<PathBuf>, path: PathBuf) {
    if !candidates.iter().any(|candidate| candidate == &path) {
        candidates.push(path);
    }
}

fn measure_legacy(candidates: &[PathBuf], duplicate: &PathBuf) -> u128 {
    let mut candidates = candidates.to_vec();
    let started = Instant::now();
    for _ in 0..PUSHES_PER_SAMPLE {
        legacy_push_candidate(black_box(&mut candidates), black_box(duplicate.clone()));
    }
    black_box(candidates.len());
    started.elapsed().as_nanos().max(1)
}

fn measure_optimized(candidates: &[PathBuf], duplicate: &PathBuf) -> u128 {
    let mut candidates = candidates.to_vec();
    let started = Instant::now();
    for _ in 0..PUSHES_PER_SAMPLE {
        push_candidate(black_box(&mut candidates), black_box(duplicate.clone()));
    }
    black_box(candidates.len());
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
