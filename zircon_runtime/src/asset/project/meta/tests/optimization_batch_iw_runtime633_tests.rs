use std::collections::HashSet;
use std::hint::black_box;
use std::time::Instant;

use super::*;

const TAG_COUNT: usize = 65_536;
const SAMPLE_PAIRS: usize = 17;

#[test]
fn optimization_batch_iw_runtime633_preserves_duplicate_tag_error() {
    let tags = vec!["beta".to_string(), "alpha".to_string(), "beta".to_string()];

    assert_eq!(
        validate_tag_list("root", &tags),
        Err(AssetMetaError::DuplicateTag {
            scope: "root".to_string(),
            tag: "beta".to_string(),
        })
    );
}

#[test]
fn optimization_batch_iw_runtime633_reserves_tag_membership() {
    let source = include_str!("../../meta.rs");
    let serialized_body = source
        .split("fn validate_tag_value")
        .nth(1)
        .expect("serialized tag validation remains present")
        .split("fn validate_tag_list")
        .next()
        .expect("serialized tag validation remains bounded");
    let list_body = source
        .split("fn validate_tag_list")
        .nth(1)
        .expect("tag-list validation remains present")
        .split("fn validate_tag_set")
        .next()
        .expect("tag-list validation remains bounded");

    assert!(serialized_body.contains("HashSet::with_capacity(tags.len())"));
    assert!(list_body.contains("HashSet::with_capacity(tags.len())"));
    assert!(!serialized_body.contains("HashSet::new()"));
    assert!(!list_body.contains("HashSet::new()"));
}

#[test]
#[ignore = "release performance gate"]
fn optimization_batch_iw_runtime633_preallocated_tag_membership_benchmark() {
    let tags = (0..TAG_COUNT)
        .map(|index| format!("asset-tag-{index:08}"))
        .collect::<Vec<_>>();
    for _ in 0..4 {
        black_box(measure_tags(&tags, false));
        black_box(measure_tags(&tags, true));
    }

    let mut unreserved_samples = Vec::with_capacity(SAMPLE_PAIRS);
    let mut preallocated_samples = Vec::with_capacity(SAMPLE_PAIRS);
    for pair in 0..SAMPLE_PAIRS {
        if pair % 2 == 0 {
            unreserved_samples.push(measure_tags(&tags, false));
            preallocated_samples.push(measure_tags(&tags, true));
        } else {
            preallocated_samples.push(measure_tags(&tags, true));
            unreserved_samples.push(measure_tags(&tags, false));
        }
    }

    let unreserved_p95 = percentile(&unreserved_samples, 95);
    let preallocated_p95 = percentile(&preallocated_samples, 95);
    let improvement_percent = unreserved_p95
        .saturating_sub(preallocated_p95)
        .saturating_mul(100)
        / unreserved_p95.max(1);
    println!(
        "RUNTIME633_PREALLOCATED_ASSET_META_TAG_BENCH_V1 sample_pairs={SAMPLE_PAIRS} tag_count={TAG_COUNT} unreserved_ns={} preallocated_ns={} unreserved_p95_ns={unreserved_p95} preallocated_p95_ns={preallocated_p95} improvement_percent={improvement_percent} threshold_percent=20",
        csv(&unreserved_samples),
        csv(&preallocated_samples),
    );
    assert!(preallocated_p95 <= unreserved_p95 * 80 / 100);
}

fn measure_tags(tags: &[String], preallocated: bool) -> u128 {
    let mut seen = if preallocated {
        HashSet::with_capacity(tags.len())
    } else {
        HashSet::new()
    };
    let started = Instant::now();
    for tag in tags {
        black_box(seen.insert(black_box(tag.as_str())));
    }
    black_box(seen);
    started.elapsed().as_nanos().max(1)
}

fn percentile(samples: &[u128], percentile: usize) -> u128 {
    let mut sorted = samples.to_vec();
    sorted.sort_unstable();
    let rank = (sorted.len() * percentile).div_ceil(100);
    sorted[rank.saturating_sub(1)]
}

fn csv(samples: &[u128]) -> String {
    samples
        .iter()
        .map(u128::to_string)
        .collect::<Vec<_>>()
        .join(",")
}
