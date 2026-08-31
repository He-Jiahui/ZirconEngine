use std::hint::black_box;
use std::time::Instant;

use super::{normalize_metadata_tags, trim_tag_in_place};

const SAMPLE_PAIRS: usize = 21;
const BATCHES_PER_SAMPLE: usize = 64;
const TAGS_PER_BATCH: usize = 256;

#[test]
fn optimization_batch_20260826db_runtime145_metadata_tags_preserve_canonical_order() {
    let mut tags = vec![
        "  gameplay  ".to_string(),
        "\tui\n".to_string(),
        "gameplay".to_string(),
        "   ".to_string(),
        "世界 ".to_string(),
    ];

    normalize_metadata_tags(&mut tags);

    assert_eq!(tags, ["gameplay", "ui", "世界"]);
}

#[test]
fn optimization_batch_20260826db_runtime145_metadata_tag_trim_reuses_owned_buffer() {
    let mut tag = String::with_capacity(128);
    tag.push_str("  retained-ui  ");
    let allocation = tag.as_ptr();
    let capacity = tag.capacity();

    trim_tag_in_place(&mut tag);

    assert_eq!(tag, "retained-ui");
    assert_eq!(tag.as_ptr(), allocation);
    assert_eq!(tag.capacity(), capacity);

    let source = include_str!("../tags.rs");
    assert!(source.contains("trim_tag_in_place(tag)"));
    assert!(!source.contains("*tag = tag.trim().to_string()"));
}

#[test]
#[ignore = "managed Windows release performance evidence"]
fn optimization_batch_20260826db_runtime145_metadata_tag_in_place_trim_bench() {
    let mut legacy_samples = Vec::with_capacity(SAMPLE_PAIRS);
    let mut optimized_samples = Vec::with_capacity(SAMPLE_PAIRS);

    for pair in 0..SAMPLE_PAIRS {
        let legacy = fixture_batches();
        let optimized = legacy.clone();
        let measure_legacy = || measure(legacy, legacy_normalize_metadata_tags);
        let measure_optimized = || measure(optimized, normalize_metadata_tags);
        if pair % 2 == 0 {
            legacy_samples.push(measure_legacy());
            optimized_samples.push(measure_optimized());
        } else {
            optimized_samples.push(measure_optimized());
            legacy_samples.push(measure_legacy());
        }
    }

    let legacy_p50_ns = percentile(&legacy_samples, 50);
    let optimized_p50_ns = percentile(&optimized_samples, 50);
    let legacy_p95_ns = percentile(&legacy_samples, 95);
    let optimized_p95_ns = percentile(&optimized_samples, 95);
    println!(
        "RUNTIME145_METADATA_TAG_IN_PLACE_TRIM_BENCH_V1 sample_pairs={SAMPLE_PAIRS} \
batches_per_sample={BATCHES_PER_SAMPLE} tags_per_batch={TAGS_PER_BATCH} \
legacy_trim_allocations_per_sample={} optimized_trim_allocations_per_sample=0 \
legacy_p50_ns={legacy_p50_ns} optimized_p50_ns={optimized_p50_ns} \
legacy_p95_ns={legacy_p95_ns} optimized_p95_ns={optimized_p95_ns} \
legacy_raw_ns={} optimized_raw_ns={}",
        BATCHES_PER_SAMPLE * TAGS_PER_BATCH,
        sample_csv(&legacy_samples),
        sample_csv(&optimized_samples),
    );
    assert!(
        optimized_p95_ns.saturating_mul(100) <= legacy_p95_ns.saturating_mul(70),
        "in-place tag trim P95 {optimized_p95_ns}ns must be at most 70% of copied trim P95 {legacy_p95_ns}ns"
    );
}

fn fixture_batches() -> Vec<Vec<String>> {
    (0..BATCHES_PER_SAMPLE)
        .map(|batch| {
            (0..TAGS_PER_BATCH)
                .map(|tag| format!("  tag-{:03}-{:03}  ", batch % 8, tag % 64))
                .collect()
        })
        .collect()
}

fn legacy_normalize_metadata_tags(tags: &mut Vec<String>) {
    for tag in tags.iter_mut() {
        *tag = tag.trim().to_string();
    }
    tags.retain(|tag| !tag.is_empty());
    tags.sort();
    tags.dedup();
}

fn measure(mut batches: Vec<Vec<String>>, normalize: fn(&mut Vec<String>)) -> u128 {
    let started = Instant::now();
    for tags in &mut batches {
        normalize(black_box(tags));
    }
    black_box(batches);
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
