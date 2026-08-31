use std::hint::black_box;
use std::time::Instant;

use super::normalize_custom_extension_token;

const SAMPLE_PAIRS: usize = 21;
const TOKENS_PER_SAMPLE: usize = 16_384;

#[test]
fn optimization_batch_20260826dh_runtime151_extension_token_preserves_canonicalization() {
    let mut token = " \tCuStOm:Cel_Shading\n ".to_string();
    assert!(normalize_custom_extension_token(&mut token));
    assert_eq!(token, "custom:cel_shading");

    for invalid in ["custom:", "builtin:lit", "", " custom: "] {
        let mut token = invalid.to_string();
        assert!(!normalize_custom_extension_token(&mut token));
        assert_eq!(
            token, invalid,
            "invalid inputs remain available for diagnostics"
        );
    }
}

#[test]
fn optimization_batch_20260826dh_runtime151_extension_token_reuses_descriptor_buffer() {
    let mut token = String::with_capacity(128);
    token.push_str("  CUSTOM:TerrainBlend  ");
    let allocation = token.as_ptr();
    let capacity = token.capacity();

    assert!(normalize_custom_extension_token(&mut token));

    assert_eq!(token, "custom:terrainblend");
    assert_eq!(token.as_ptr(), allocation);
    assert_eq!(token.capacity(), capacity);

    let source = include_str!("../metadata.rs");
    assert_eq!(
        source
            .matches("validate_geometry_source_descriptor(&mut descriptor)")
            .count(),
        1
    );
    assert_eq!(
        source
            .matches("validate_shading_model_descriptor(&mut descriptor)")
            .count(),
        1
    );
    assert!(!source.contains("descriptor.token.trim().to_ascii_lowercase()"));
    assert!(!source.contains("descriptor.token = key.clone()"));
}

#[test]
#[ignore = "managed Windows release performance evidence"]
fn optimization_batch_20260826dh_runtime151_extension_token_in_place_normalization_bench() {
    let mut legacy_samples = Vec::with_capacity(SAMPLE_PAIRS);
    let mut optimized_samples = Vec::with_capacity(SAMPLE_PAIRS);

    for pair in 0..SAMPLE_PAIRS {
        let legacy = fixture_tokens();
        let optimized = legacy.clone();
        if pair % 2 == 0 {
            legacy_samples.push(measure(legacy, legacy_normalize));
            optimized_samples.push(measure(optimized, optimized_normalize));
        } else {
            optimized_samples.push(measure(optimized, optimized_normalize));
            legacy_samples.push(measure(legacy, legacy_normalize));
        }
    }

    let legacy_p50_ns = percentile(&legacy_samples, 50);
    let optimized_p50_ns = percentile(&optimized_samples, 50);
    let legacy_p95_ns = percentile(&legacy_samples, 95);
    let optimized_p95_ns = percentile(&optimized_samples, 95);
    println!(
        "RUNTIME151_EXTENSION_TOKEN_IN_PLACE_NORMALIZATION_BENCH_V1 sample_pairs={SAMPLE_PAIRS} \
tokens_per_sample={TOKENS_PER_SAMPLE} legacy_normalization_allocations_per_sample={TOKENS_PER_SAMPLE} \
optimized_normalization_allocations_per_sample=0 legacy_p50_ns={legacy_p50_ns} \
optimized_p50_ns={optimized_p50_ns} legacy_p95_ns={legacy_p95_ns} \
optimized_p95_ns={optimized_p95_ns} legacy_raw_ns={} optimized_raw_ns={}",
        sample_csv(&legacy_samples),
        sample_csv(&optimized_samples),
    );
    assert!(
        optimized_p95_ns.saturating_mul(100) <= legacy_p95_ns.saturating_mul(70),
        "in-place token normalization P95 {optimized_p95_ns}ns must be at most 70% of allocated normalization P95 {legacy_p95_ns}ns"
    );
}

fn fixture_tokens() -> Vec<String> {
    (0..TOKENS_PER_SAMPLE)
        .map(|index| format!("  CUSTOM:Extension_{index:05}  "))
        .collect()
}

fn legacy_normalize(token: String) -> String {
    token.trim().to_ascii_lowercase()
}

fn optimized_normalize(mut token: String) -> String {
    assert!(normalize_custom_extension_token(&mut token));
    token
}

fn measure(tokens: Vec<String>, normalize: fn(String) -> String) -> u128 {
    let started = Instant::now();
    let mut checksum = 0usize;
    for token in tokens {
        checksum ^= black_box(normalize(black_box(token))).len();
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
