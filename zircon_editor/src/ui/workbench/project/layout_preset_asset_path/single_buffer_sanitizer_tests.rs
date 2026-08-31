use std::hint::black_box;
use std::time::Instant;

use super::sanitize_layout_preset_name;

const SAMPLE_PAIRS: usize = 31;
const NAMES_PER_SAMPLE: usize = 100_000;

#[test]
fn optimization_batch_20260829ad_editor249_layout_preset_sanitizer_preserves_bytes() {
    for (input, expected) in [
        ("--Project Layout--", "Project-Layout"),
        ("  \u{96ea} Layout  ", "Layout"),
        ("alpha---beta", "alpha---beta"),
        ("valid_name", "valid_name"),
        ("---", "preset"),
    ] {
        assert_eq!(sanitize_layout_preset_name(input), expected);
        assert_eq!(
            sanitize_layout_preset_name(input),
            legacy_sanitize_layout_preset_name(input)
        );
    }
}

#[test]
fn optimization_batch_20260829ad_editor249_layout_preset_sanitizer_uses_one_buffer() {
    let source = include_str!("../layout_preset_asset_path.rs");
    let implementation = source.split("#[cfg(test)]").next().expect("implementation");
    let sanitizer = implementation
        .split("fn sanitize_layout_preset_name")
        .nth(1)
        .expect("layout preset sanitizer");

    assert!(sanitizer.contains("String::with_capacity(name.len())"));
    assert!(sanitizer.contains("pending_hyphens"));
    assert!(sanitizer.contains("sanitized.push"));
    assert!(!sanitizer.contains("collect::<String>"));
    assert!(!sanitizer.contains("trim_matches"));
}

#[test]
#[ignore = "managed Windows release performance evidence"]
fn optimization_batch_20260829ad_editor249_single_buffer_layout_preset_sanitizer_bench() {
    let name = "---Production Layout / Animation Graph / Final Review---";
    assert_eq!(
        sanitize_layout_preset_name(name),
        legacy_sanitize_layout_preset_name(name)
    );

    let mut legacy_samples = Vec::with_capacity(SAMPLE_PAIRS);
    let mut optimized_samples = Vec::with_capacity(SAMPLE_PAIRS);
    for pair in 0..SAMPLE_PAIRS {
        if pair % 2 == 0 {
            legacy_samples.push(measure(false, name));
            optimized_samples.push(measure(true, name));
        } else {
            optimized_samples.push(measure(true, name));
            legacy_samples.push(measure(false, name));
        }
    }
    let legacy_p50_ns = percentile(&legacy_samples, 50);
    let optimized_p50_ns = percentile(&optimized_samples, 50);
    let legacy_p95_ns = percentile(&legacy_samples, 95);
    let optimized_p95_ns = percentile(&optimized_samples, 95);
    println!(
        "EDITOR249_SINGLE_BUFFER_LAYOUT_PRESET_SANITIZER_BENCH_V1 sample_pairs={SAMPLE_PAIRS} \
names_per_sample={NAMES_PER_SAMPLE} input_bytes={} \
legacy_result_allocations_per_name=2 optimized_result_allocations_per_name=1 \
legacy_p50_ns={legacy_p50_ns} optimized_p50_ns={optimized_p50_ns} \
legacy_p95_ns={legacy_p95_ns} optimized_p95_ns={optimized_p95_ns} \
legacy_raw_ns={} optimized_raw_ns={}",
        name.len(),
        sample_csv(&legacy_samples),
        sample_csv(&optimized_samples),
    );
    assert!(optimized_p95_ns.saturating_mul(100) <= legacy_p95_ns.saturating_mul(70));
}

fn legacy_sanitize_layout_preset_name(name: &str) -> String {
    let sanitized = name
        .chars()
        .map(|ch| match ch {
            'a'..='z' | 'A'..='Z' | '0'..='9' | '-' | '_' => ch,
            _ => '-',
        })
        .collect::<String>()
        .trim_matches('-')
        .to_string();
    if sanitized.is_empty() {
        "preset".to_string()
    } else {
        sanitized
    }
}

fn measure(optimized: bool, name: &str) -> u128 {
    let started = Instant::now();
    let mut checksum = 0usize;
    for _ in 0..NAMES_PER_SAMPLE {
        let sanitized = if optimized {
            sanitize_layout_preset_name(black_box(name))
        } else {
            legacy_sanitize_layout_preset_name(black_box(name))
        };
        checksum = checksum.wrapping_add(black_box(sanitized).len());
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
