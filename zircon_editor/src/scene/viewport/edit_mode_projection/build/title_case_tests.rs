use std::hint::black_box;
use std::time::Instant;

use super::title_case_identifier;

const SAMPLE_PAIRS: usize = 21;
const LABELS_PER_SAMPLE: usize = 131_072;
const FIXTURE: &str = "angular_velocity_constraint_override_priority_blend_weight";

#[test]
fn optimization_batch_20260826dk_editor100_viewport_field_label_preserves_title_contract() {
    assert_eq!(
        title_case_identifier("angular_velocity"),
        "Angular Velocity"
    );
    assert_eq!(
        title_case_identifier("_angular__velocity_"),
        "Angular Velocity"
    );
    assert_eq!(title_case_identifier("already_Title"), "Already Title");
    assert_eq!(title_case_identifier("élan_mode"), "élan Mode");
    assert_eq!(title_case_identifier("___"), "");
}

#[test]
fn optimization_batch_20260826dk_editor100_viewport_field_label_uses_one_buffer() {
    let title = title_case_identifier(FIXTURE);
    assert_eq!(title.len(), title.capacity());

    let source = include_str!("../build.rs");
    assert!(source.contains("let mut title = String::with_capacity(value.len());"));
    assert!(source.contains("title.push(first.to_ascii_uppercase());"));
    assert!(!source.contains("let mut word = first.to_ascii_uppercase().to_string();"));
    assert!(!source.contains(".collect::<Vec<_>>()\n        .join(\" \")"));
}

#[test]
#[ignore = "managed Windows release performance evidence"]
fn optimization_batch_20260826dk_editor100_viewport_field_label_single_buffer_bench() {
    let mut legacy_samples = Vec::with_capacity(SAMPLE_PAIRS);
    let mut optimized_samples = Vec::with_capacity(SAMPLE_PAIRS);

    for pair in 0..SAMPLE_PAIRS {
        if pair % 2 == 0 {
            legacy_samples.push(measure(legacy_title_case_identifier));
            optimized_samples.push(measure(title_case_identifier));
        } else {
            optimized_samples.push(measure(title_case_identifier));
            legacy_samples.push(measure(legacy_title_case_identifier));
        }
    }

    let legacy_p50_ns = percentile(&legacy_samples, 50);
    let optimized_p50_ns = percentile(&optimized_samples, 50);
    let legacy_p95_ns = percentile(&legacy_samples, 95);
    let optimized_p95_ns = percentile(&optimized_samples, 95);
    println!(
        "EDITOR100_VIEWPORT_FIELD_LABEL_SINGLE_BUFFER_BENCH_V1 sample_pairs={SAMPLE_PAIRS} \
labels_per_sample={LABELS_PER_SAMPLE} segments_per_label=7 \
legacy_minimum_allocations_per_label=9 optimized_allocations_per_label=1 \
legacy_p50_ns={legacy_p50_ns} optimized_p50_ns={optimized_p50_ns} \
legacy_p95_ns={legacy_p95_ns} optimized_p95_ns={optimized_p95_ns} \
legacy_raw_ns={} optimized_raw_ns={}",
        sample_csv(&legacy_samples),
        sample_csv(&optimized_samples),
    );
    assert!(
        optimized_p95_ns.saturating_mul(100) <= legacy_p95_ns.saturating_mul(70),
        "single-buffer viewport field label P95 {optimized_p95_ns}ns must be at most 70% of segmented collector P95 {legacy_p95_ns}ns"
    );
}

fn legacy_title_case_identifier(value: &str) -> String {
    value
        .split('_')
        .filter(|part| !part.is_empty())
        .map(|part| {
            let mut chars = part.chars();
            match chars.next() {
                Some(first) => {
                    let mut word = first.to_ascii_uppercase().to_string();
                    word.push_str(chars.as_str());
                    word
                }
                None => String::new(),
            }
        })
        .collect::<Vec<_>>()
        .join(" ")
}

fn measure(render: fn(&str) -> String) -> u128 {
    let started = Instant::now();
    let mut checksum = 0usize;
    for _ in 0..LABELS_PER_SAMPLE {
        checksum ^= black_box(render(black_box(FIXTURE))).len();
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
