use std::hint::black_box;
use std::time::Instant;

use super::{topic_segment_byte_is_valid, EditorTopic, EditorTopicError};

const CHECKS_PER_SAMPLE: usize = 8192;
const SAMPLE_PAIRS: usize = 31;
const TOPIC_BYTES: usize = 4096;

fn legacy_parse(value: &str) -> Result<EditorTopic, EditorTopicError> {
    let value = value.to_owned();
    if value.is_empty() {
        return Err(EditorTopicError::Empty);
    }
    if !value.contains('.') {
        return Err(EditorTopicError::MissingSeparator { value });
    }
    for (index, segment) in value.split('.').enumerate() {
        if segment.is_empty() {
            return Err(EditorTopicError::EmptySegment { index });
        }
        if !segment.bytes().all(topic_segment_byte_is_valid) {
            return Err(EditorTopicError::InvalidSegment {
                segment: segment.to_owned(),
            });
        }
    }
    Ok(EditorTopic(value))
}

fn measure(value: &str, optimized: bool) -> u128 {
    let started = Instant::now();
    let mut valid = 0;
    for _ in 0..CHECKS_PER_SAMPLE {
        valid += usize::from(if optimized {
            EditorTopic::parse(black_box(value)).is_ok()
        } else {
            legacy_parse(black_box(value)).is_ok()
        });
    }
    black_box(valid);
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
fn optimization_batch_20260829be_editor277_single_scan_topics_preserve_results() {
    for value in [
        "editor.document",
        "editor.asset-changed",
        "",
        "editor",
        "Editor",
        ".editor",
        "editor.",
        "editor..document",
        "editor.Document",
        "editor.\u{4f8b}",
    ] {
        assert_eq!(EditorTopic::parse(value), legacy_parse(value), "{value:?}");
    }
}

#[test]
fn optimization_batch_20260829be_editor277_topic_parser_uses_one_byte_scan() {
    let source = include_str!("../topic.rs");
    let production = source.split_once("#[cfg(test)]").expect("test boundary").0;

    assert!(production.contains("for (offset, byte) in value.bytes().enumerate()"));
    assert!(!production.contains("value.contains('.')"));
    assert!(!production.contains("value.split('.')"));
    assert!(!production.contains("segment.bytes().all"));
}

#[test]
#[ignore = "managed Windows release performance evidence"]
fn optimization_batch_20260829be_editor277_single_scan_editor_topic_bench() {
    let suffix = ".leaf";
    let value = format!("{}{}", "a".repeat(TOPIC_BYTES - suffix.len()), suffix);
    assert_eq!(value.len(), TOPIC_BYTES);
    let mut legacy_samples = Vec::with_capacity(SAMPLE_PAIRS);
    let mut optimized_samples = Vec::with_capacity(SAMPLE_PAIRS);
    for pair in 0..SAMPLE_PAIRS {
        if pair % 2 == 0 {
            legacy_samples.push(measure(&value, false));
            optimized_samples.push(measure(&value, true));
        } else {
            optimized_samples.push(measure(&value, true));
            legacy_samples.push(measure(&value, false));
        }
    }

    let legacy_p50_ns = percentile(&legacy_samples, 50);
    let optimized_p50_ns = percentile(&optimized_samples, 50);
    let legacy_p95_ns = percentile(&legacy_samples, 95);
    let optimized_p95_ns = percentile(&optimized_samples, 95);
    println!(
        "EDITOR277_SINGLE_SCAN_EDITOR_TOPIC_BENCH_V1 sample_pairs={SAMPLE_PAIRS} \
checks_per_sample={CHECKS_PER_SAMPLE} topic_bytes={TOPIC_BYTES} \
legacy_topic_scans=3 optimized_topic_scans=1 \
legacy_p50_ns={legacy_p50_ns} optimized_p50_ns={optimized_p50_ns} \
legacy_p95_ns={legacy_p95_ns} optimized_p95_ns={optimized_p95_ns} \
legacy_raw_ns={} optimized_raw_ns={}",
        sample_csv(&legacy_samples),
        sample_csv(&optimized_samples),
    );
    assert!(optimized_p95_ns.saturating_mul(100) <= legacy_p95_ns.saturating_mul(70));
}
