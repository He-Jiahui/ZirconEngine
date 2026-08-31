use std::borrow::Cow;
use std::hint::black_box;
use std::time::Instant;

use super::normalize_runtime_plugin_key;

const SAMPLE_PAIRS: usize = 31;
const KEYS_PER_SAMPLE: usize = 10_000;

#[test]
fn optimization_batch_20260829ad_runtime303_plugin_key_normalization_preserves_results() {
    assert_eq!(
        normalize_runtime_plugin_key("  third_party.weather-sim  "),
        Some(Cow::Borrowed("third_party.weather-sim"))
    );
    assert_eq!(
        normalize_runtime_plugin_key("Third_Party.Weather-Sim"),
        Some(Cow::Owned("third_party.weather-sim".to_string()))
    );
    for invalid in ["", "   ", ".leading", "bad key", "bad/key", "snow_\u{96ea}"] {
        assert_eq!(normalize_runtime_plugin_key(invalid), None);
    }
}

#[test]
fn optimization_batch_20260829ad_runtime303_plugin_key_normalization_uses_one_scan() {
    let source = include_str!("../plugin_id.rs");
    let implementation = source.split("#[cfg(test)]").next().expect("implementation");
    let normalization = implementation
        .split("fn normalize_runtime_plugin_key")
        .nth(1)
        .expect("plugin key normalizer");

    assert!(normalization.contains("let first = bytes.next()?"));
    assert!(normalization.contains("for byte in bytes"));
    assert!(normalization.contains("has_uppercase |= byte.is_ascii_uppercase()"));
    assert!(!normalization.contains(".all("));
    assert!(!normalization.contains(".any("));
}

#[test]
#[ignore = "managed Windows release performance evidence"]
fn optimization_batch_20260829ad_runtime303_single_pass_plugin_key_normalization_bench() {
    let key = format!("plugin.{}tailZ", "segment_".repeat(64));
    assert_eq!(
        normalize_runtime_plugin_key(&key),
        legacy_normalize_runtime_plugin_key(&key)
    );

    let mut legacy_samples = Vec::with_capacity(SAMPLE_PAIRS);
    let mut optimized_samples = Vec::with_capacity(SAMPLE_PAIRS);
    for pair in 0..SAMPLE_PAIRS {
        if pair % 2 == 0 {
            legacy_samples.push(measure(false, &key));
            optimized_samples.push(measure(true, &key));
        } else {
            optimized_samples.push(measure(true, &key));
            legacy_samples.push(measure(false, &key));
        }
    }
    let legacy_p50_ns = percentile(&legacy_samples, 50);
    let optimized_p50_ns = percentile(&optimized_samples, 50);
    let legacy_p95_ns = percentile(&legacy_samples, 95);
    let optimized_p95_ns = percentile(&optimized_samples, 95);
    println!(
        "RUNTIME303_SINGLE_PASS_PLUGIN_KEY_NORMALIZATION_BENCH_V1 sample_pairs={SAMPLE_PAIRS} \
keys_per_sample={KEYS_PER_SAMPLE} key_bytes={} uppercase_byte_index={} \
legacy_full_scans_per_key=2 optimized_full_scans_per_key=1 \
legacy_p50_ns={legacy_p50_ns} optimized_p50_ns={optimized_p50_ns} \
legacy_p95_ns={legacy_p95_ns} optimized_p95_ns={optimized_p95_ns} \
legacy_raw_ns={} optimized_raw_ns={}",
        key.len(),
        key.len().saturating_sub(1),
        sample_csv(&legacy_samples),
        sample_csv(&optimized_samples),
    );
    assert!(optimized_p95_ns.saturating_mul(100) <= legacy_p95_ns.saturating_mul(70));
}

fn legacy_normalize_runtime_plugin_key(raw: &str) -> Option<Cow<'_, str>> {
    let trimmed = raw.trim();
    if trimmed.is_empty()
        || !trimmed
            .bytes()
            .all(|byte| byte.is_ascii_alphanumeric() || matches!(byte, b'_' | b'-' | b'.'))
        || !trimmed
            .bytes()
            .next()
            .is_some_and(|byte| byte.is_ascii_alphanumeric())
    {
        return None;
    }
    if trimmed.bytes().any(|byte| byte.is_ascii_uppercase()) {
        Some(Cow::Owned(trimmed.to_ascii_lowercase()))
    } else {
        Some(Cow::Borrowed(trimmed))
    }
}

fn measure(optimized: bool, key: &str) -> u128 {
    let started = Instant::now();
    let mut checksum = 0usize;
    for _ in 0..KEYS_PER_SAMPLE {
        let normalized = if optimized {
            normalize_runtime_plugin_key(black_box(key))
        } else {
            legacy_normalize_runtime_plugin_key(black_box(key))
        };
        checksum = checksum.wrapping_add(black_box(normalized).map_or(0, |key| key.len()));
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
