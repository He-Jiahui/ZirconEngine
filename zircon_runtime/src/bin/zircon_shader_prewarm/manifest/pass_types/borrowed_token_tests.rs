use std::hint::black_box;
use std::time::Instant;

use super::{pass_type_from_token, ShaderPassType, ASSET_SCAN_FULL_MATERIAL_PASSES};

const CHECKS_PER_SAMPLE: usize = 8192;
const SAMPLE_PAIRS: usize = 31;
const TOKEN_BYTES: usize = 4096;

fn legacy_pass_type_from_token(token: &str) -> Option<ShaderPassType> {
    let token = token.trim().to_ascii_lowercase();
    ASSET_SCAN_FULL_MATERIAL_PASSES
        .iter()
        .copied()
        .find(|pass_type| pass_type.token() == token)
}

fn measure(token: &str, optimized: bool) -> u128 {
    let started = Instant::now();
    let mut matches = 0;
    for _ in 0..CHECKS_PER_SAMPLE {
        matches += usize::from(if optimized {
            pass_type_from_token(black_box(token)).is_some()
        } else {
            legacy_pass_type_from_token(black_box(token)).is_some()
        });
    }
    black_box(matches);
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
fn optimization_batch_20260829bi_runtime336_borrowed_pass_tokens_preserve_results() {
    for token in [
        "forward",
        " GBUFFER ",
        "DepthPrepass",
        "shadow",
        "VELOCITY",
        "taa_reactive_mask",
        "",
        "unknown-pass",
        "\u{4f8b}",
    ] {
        assert_eq!(
            pass_type_from_token(token),
            legacy_pass_type_from_token(token),
            "{token:?}"
        );
    }
}

#[test]
fn optimization_batch_20260829bi_runtime336_pass_token_lookup_stays_borrowed() {
    let source = include_str!("../pass_types.rs");
    let production = source.split_once("#[cfg(test)]").expect("test boundary").0;
    let lookup = production
        .split_once("fn pass_type_from_token")
        .expect("lookup function")
        .1;

    assert!(lookup.contains("token.eq_ignore_ascii_case(pass_type.token())"));
    assert!(!lookup.contains("to_ascii_lowercase()"));
}

#[test]
#[ignore = "managed Windows release performance evidence"]
fn optimization_batch_20260829bi_runtime336_borrowed_shader_pass_token_bench() {
    let token = "x".repeat(TOKEN_BYTES);
    let mut baseline_samples = Vec::with_capacity(SAMPLE_PAIRS);
    let mut candidate_samples = Vec::with_capacity(SAMPLE_PAIRS);
    for pair in 0..SAMPLE_PAIRS {
        if pair % 2 == 0 {
            baseline_samples.push(measure(&token, false));
            candidate_samples.push(measure(&token, true));
        } else {
            candidate_samples.push(measure(&token, true));
            baseline_samples.push(measure(&token, false));
        }
    }

    let baseline_p50_ns = percentile(&baseline_samples, 50);
    let candidate_p50_ns = percentile(&candidate_samples, 50);
    let baseline_p95_ns = percentile(&baseline_samples, 95);
    let candidate_p95_ns = percentile(&candidate_samples, 95);
    println!(
        "RUNTIME336_BORROWED_SHADER_PASS_TOKEN_BENCH_V1 sample_pairs={SAMPLE_PAIRS} \
checks_per_sample={CHECKS_PER_SAMPLE} token_bytes={TOKEN_BYTES} \
baseline_lowercase_allocations={CHECKS_PER_SAMPLE} candidate_lowercase_allocations=0 \
baseline_p50_ns={baseline_p50_ns} candidate_p50_ns={candidate_p50_ns} \
baseline_p95_ns={baseline_p95_ns} candidate_p95_ns={candidate_p95_ns} \
baseline_raw_ns={} candidate_raw_ns={}",
        sample_csv(&baseline_samples),
        sample_csv(&candidate_samples),
    );
    assert!(candidate_p95_ns.saturating_mul(100) <= baseline_p95_ns.saturating_mul(70));
}
