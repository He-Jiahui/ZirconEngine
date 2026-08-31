use std::hint::black_box;
use std::time::Instant;

use super::{valid_notification_id_syntax, NotificationId, MAX_NOTIFICATION_ID_BYTES};

const SAMPLE_PAIRS: usize = 31;
const VALIDATIONS_PER_SAMPLE: usize = 16_384;

#[test]
fn optimization_batch_20260829at_editor265_single_pass_notification_ids_match_legacy_syntax() {
    for value in [
        "editor.asset.saved",
        "runtime_2.render.frame_17",
        "a.b.c",
        "workspace.document.close_requested",
    ] {
        assert!(valid_notification_id_syntax(value));
        assert!(legacy_valid_notification_id_syntax(value));
        assert_eq!(NotificationId::parse(value).unwrap().as_str(), value);
    }
}

#[test]
fn optimization_batch_20260829at_editor265_single_pass_notification_ids_reject_invalid_segments() {
    for value in [
        "",
        "a.b",
        ".a.b.c",
        "a..b.c",
        "a.b.c.",
        "Editor.asset.saved",
        "editor.asset-saved.event",
        "editor.asset.sav\u{00e9}d",
    ] {
        assert_eq!(
            valid_notification_id_syntax(value),
            legacy_valid_notification_id_syntax(value),
            "syntax result changed for {value:?}"
        );
        assert!(NotificationId::parse(value).is_err());
    }

    let boundary = "ab.".repeat(63) + "ab";
    assert_eq!(boundary.len(), MAX_NOTIFICATION_ID_BYTES - 1);
    assert!(NotificationId::parse(&boundary).is_ok());
    assert!(NotificationId::parse(boundary + "__").is_err());
}

#[test]
#[ignore = "managed Windows release performance evidence"]
fn optimization_batch_20260829at_editor265_single_pass_notification_id_validation_bench() {
    let value = "ab.".repeat(63) + "ab";
    assert_eq!(value.len(), 191);
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
        "EDITOR265_SINGLE_PASS_NOTIFICATION_ID_VALIDATION_BENCH_V1 sample_pairs={SAMPLE_PAIRS} \
validations_per_sample={VALIDATIONS_PER_SAMPLE} id_bytes={} id_segments=64 \
legacy_p50_ns={legacy_p50_ns} optimized_p50_ns={optimized_p50_ns} \
legacy_p95_ns={legacy_p95_ns} optimized_p95_ns={optimized_p95_ns} \
legacy_raw_ns={} optimized_raw_ns={}",
        value.len(),
        sample_csv(&legacy_samples),
        sample_csv(&optimized_samples),
    );
    assert!(optimized_p95_ns.saturating_mul(100) <= legacy_p95_ns.saturating_mul(70));
}

fn measure(value: &str, optimized: bool) -> u128 {
    let started = Instant::now();
    let mut checksum = 0usize;
    for _ in 0..VALIDATIONS_PER_SAMPLE {
        let valid = if optimized {
            valid_notification_id_syntax(black_box(value))
        } else {
            legacy_valid_notification_id_syntax(black_box(value))
        };
        checksum ^= usize::from(valid);
    }
    black_box(checksum);
    started.elapsed().as_nanos().max(1)
}

fn legacy_valid_notification_id_syntax(value: &str) -> bool {
    let mut segment_count = 0usize;
    let invalid_segment = value.split('.').any(|segment| {
        segment_count += 1;
        segment.is_empty()
            || !segment.chars().all(|character| {
                character.is_ascii_lowercase() || character.is_ascii_digit() || character == '_'
            })
    });
    segment_count >= 3 && !invalid_segment
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
