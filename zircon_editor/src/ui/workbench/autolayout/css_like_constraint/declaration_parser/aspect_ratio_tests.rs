use std::hint::black_box;
use std::time::Instant;

use super::{
    finite_non_negative, invalid_value, parse_aspect_ratio, parse_non_negative_number,
    CssLikeConstraintError,
};

const SAMPLE_PAIRS: usize = 21;
const PARSES_PER_SAMPLE: usize = 262_144;
const RATIOS: [&str; 4] = ["16 / 9", "4/3", "1.777", "21 / 9"];

#[test]
fn optimization_batch_20260826df_editor95_aspect_ratio_preserves_bounded_grammar() {
    assert_eq!(parse_aspect_ratio("16 / 9").unwrap(), 16.0 / 9.0);
    assert_eq!(parse_aspect_ratio(" 1.5 ").unwrap(), 1.5);
    for invalid in ["", "/", "1/", "/2", "1/0", "1/2/3"] {
        assert!(parse_aspect_ratio(invalid).is_err(), "value={invalid}");
    }
}

#[test]
fn optimization_batch_20260826df_editor95_aspect_ratio_uses_bounded_iterator() {
    let source = include_str!("../declaration_parser.rs");

    assert!(source.contains("let mut parts = value.split('/').map(str::trim)"));
    assert!(source.contains("if parts.next().is_some()"));
    assert!(!source.contains("value.split('/').map(str::trim).collect::<Vec<_>>()"));
}

#[test]
#[ignore = "managed Windows release performance evidence"]
fn optimization_batch_20260826df_editor95_aspect_ratio_bounded_parse_bench() {
    let mut legacy_samples = Vec::with_capacity(SAMPLE_PAIRS);
    let mut optimized_samples = Vec::with_capacity(SAMPLE_PAIRS);

    for pair in 0..SAMPLE_PAIRS {
        if pair % 2 == 0 {
            legacy_samples.push(measure(legacy_parse_aspect_ratio));
            optimized_samples.push(measure(parse_aspect_ratio));
        } else {
            optimized_samples.push(measure(parse_aspect_ratio));
            legacy_samples.push(measure(legacy_parse_aspect_ratio));
        }
    }

    let legacy_p50_ns = percentile(&legacy_samples, 50);
    let optimized_p50_ns = percentile(&optimized_samples, 50);
    let legacy_p95_ns = percentile(&legacy_samples, 95);
    let optimized_p95_ns = percentile(&optimized_samples, 95);
    println!(
        "EDITOR95_ASPECT_RATIO_BOUNDED_PARSE_BENCH_V1 sample_pairs={SAMPLE_PAIRS} \
parses_per_sample={PARSES_PER_SAMPLE} ratio_variants={} \
legacy_temporary_vec_allocations_per_sample={PARSES_PER_SAMPLE} \
optimized_temporary_vec_allocations_per_sample=0 legacy_p50_ns={legacy_p50_ns} \
optimized_p50_ns={optimized_p50_ns} legacy_p95_ns={legacy_p95_ns} \
optimized_p95_ns={optimized_p95_ns} legacy_raw_ns={} optimized_raw_ns={}",
        RATIOS.len(),
        sample_csv(&legacy_samples),
        sample_csv(&optimized_samples),
    );
    assert!(
        optimized_p95_ns.saturating_mul(100) <= legacy_p95_ns.saturating_mul(70),
        "bounded aspect-ratio parse P95 {optimized_p95_ns}ns must be at most 70% of temporary-Vec parse P95 {legacy_p95_ns}ns"
    );
}

fn legacy_parse_aspect_ratio(value: &str) -> Result<f32, CssLikeConstraintError> {
    let parts = value.split('/').map(str::trim).collect::<Vec<_>>();
    let ratio = match parts.as_slice() {
        [single] if !single.is_empty() => parse_non_negative_number(single, "aspect-ratio")?,
        [numerator, denominator] if !numerator.is_empty() && !denominator.is_empty() => {
            let numerator = parse_non_negative_number(numerator, "aspect-ratio")?;
            let denominator = parse_non_negative_number(denominator, "aspect-ratio")?;
            if denominator == 0.0 {
                return Err(invalid_value("aspect-ratio", value));
            }
            numerator / denominator
        }
        _ => return Err(invalid_value("aspect-ratio", value)),
    };
    finite_non_negative(ratio, "aspect-ratio")
}

fn measure(parse: fn(&str) -> Result<f32, CssLikeConstraintError>) -> u128 {
    let started = Instant::now();
    let mut checksum = 0u32;
    for index in 0..PARSES_PER_SAMPLE {
        checksum ^= black_box(parse(black_box(RATIOS[index % RATIOS.len()])).unwrap()).to_bits();
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
