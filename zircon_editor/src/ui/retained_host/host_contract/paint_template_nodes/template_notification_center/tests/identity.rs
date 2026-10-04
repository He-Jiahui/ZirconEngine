use std::hint::black_box;
use std::time::Instant;

use super::*;

const SAMPLE_PAIRS: usize = 17;
const HEADERS_PER_SAMPLE: usize = 262_144;

#[test]
fn header_uses_generation_metadata_without_scanning_rows() {
    let node = TemplatePaneNodeData {
        text: "Notifications".into(),
        notification_unread_count: 3,
        notification_overflow_count: 12,
        ..TemplatePaneNodeData::default()
    };

    assert_eq!(header_text(&node), "Notifications (3) +12 omitted");

    let source = include_str!("../identity.rs");
    let row_collection = ["structured_", "options"].concat();
    let cloning_access = ["row_", "data"].concat();
    assert!(!source.contains(&row_collection));
    assert!(!source.contains(&cloning_access));
}

#[test]
fn header_pixels_keep_the_existing_label_when_no_rows_were_dropped() {
    let unread = TemplatePaneNodeData {
        text: "Notifications".into(),
        notification_unread_count: 2,
        ..TemplatePaneNodeData::default()
    };
    let empty = TemplatePaneNodeData {
        text: "Notifications".into(),
        ..TemplatePaneNodeData::default()
    };

    assert_eq!(header_text(&unread), "Notifications (2)");
    assert_eq!(header_text(&empty), "Notifications");
}

#[test]
fn optimization_batch_ey_editor387_preserves_notification_header_bytes() {
    for (title, unread_count, overflow_count) in [
        ("Notifications", 0, 0),
        ("Notifications", 3, 0),
        ("Notifications", 0, 12),
        ("Build alerts", 37, 4_096),
        ("N", usize::MAX, usize::MAX),
    ] {
        assert_eq!(
            notification_header_text(title, unread_count, overflow_count),
            legacy_notification_header_text(title, unread_count, overflow_count)
        );
    }

    let production = include_str!("../identity.rs")
        .split("#[cfg(test)]")
        .next()
        .expect("production source");
    assert!(!production.contains("format!("));
    assert!(production.contains("String::with_capacity(capacity)"));
}

#[test]
#[ignore = "release performance gate"]
fn optimization_batch_ey_editor387_direct_notification_header_benchmark() {
    for _ in 0..4 {
        black_box(measure_headers(legacy_notification_header_text));
        black_box(measure_headers(notification_header_text));
    }
    let mut legacy_samples = Vec::with_capacity(SAMPLE_PAIRS);
    let mut optimized_samples = Vec::with_capacity(SAMPLE_PAIRS);
    for pair_index in 0..SAMPLE_PAIRS {
        if pair_index % 2 == 0 {
            legacy_samples.push(measure_headers(legacy_notification_header_text));
            optimized_samples.push(measure_headers(notification_header_text));
        } else {
            optimized_samples.push(measure_headers(notification_header_text));
            legacy_samples.push(measure_headers(legacy_notification_header_text));
        }
    }

    report_performance(&legacy_samples, &optimized_samples);
}

fn legacy_notification_header_text(
    title: &str,
    unread_count: usize,
    overflow_count: usize,
) -> String {
    match (unread_count, overflow_count) {
        (0, 0) => title.to_string(),
        (unread_count, 0) => format!("{title} ({unread_count})"),
        (0, overflow_count) => format!("{title} +{overflow_count} omitted"),
        (unread_count, overflow_count) => {
            format!("{title} ({unread_count}) +{overflow_count} omitted")
        }
    }
}

fn measure_headers(mut build: impl FnMut(&str, usize, usize) -> String) -> u128 {
    const TITLE: &str = "Notifications";
    let started = Instant::now();
    let mut total_len = 0_usize;
    for index in 0..HEADERS_PER_SAMPLE {
        let unread_count = black_box(1_000 + index % 97);
        let overflow_count = black_box(10_000 + index % 389);
        let header = build(black_box(TITLE), unread_count, overflow_count);
        total_len += black_box(header.len());
        black_box(header);
    }
    black_box(total_len);
    started.elapsed().as_nanos().max(1)
}

fn report_performance(legacy_samples: &[u128], optimized_samples: &[u128]) {
    let legacy_p95 = nearest_rank_p95(legacy_samples);
    let optimized_p95 = nearest_rank_p95(optimized_samples);
    let improvement_percent =
        legacy_p95.saturating_sub(optimized_p95).saturating_mul(100) / legacy_p95.max(1);
    println!(
        "EDITOR387_DIRECT_NOTIFICATION_HEADER_BENCH_V1 sample_pairs={SAMPLE_PAIRS} headers_per_sample={HEADERS_PER_SAMPLE} legacy_ns={} optimized_ns={} legacy_p95_ns={legacy_p95} optimized_p95_ns={optimized_p95} improvement_percent={improvement_percent} threshold_percent=25",
        csv(legacy_samples),
        csv(optimized_samples),
    );
    assert!(
        optimized_p95 <= legacy_p95.saturating_mul(75) / 100,
        "direct notification header construction must reduce P95 by at least 25%"
    );
}

fn nearest_rank_p95(samples: &[u128]) -> u128 {
    let mut sorted = samples.to_vec();
    sorted.sort_unstable();
    let rank = (sorted.len() * 95).div_ceil(100);
    sorted[rank.saturating_sub(1)]
}

fn csv(samples: &[u128]) -> String {
    samples
        .iter()
        .map(u128::to_string)
        .collect::<Vec<_>>()
        .join(",")
}
