use std::collections::{BTreeMap, BTreeSet};
use std::hint::black_box;
use std::time::{Duration, Instant};

use super::*;

const ENTRY_COUNT: usize = 8_192;
const SAMPLE_COUNT: usize = 17;

fn entries() -> Vec<UiAssetSourceOutlineEntry> {
    (0..ENTRY_COUNT)
        .map(|index| {
            let line = i32::try_from(index * 4 + 1).expect("benchmark line fits i32");
            UiAssetSourceOutlineEntry {
                node_id: format!("node-{index}"),
                block_label: format!("[nodes.node-{index}]"),
                line,
                end_line: line + 2,
                excerpt: String::new(),
            }
        })
        .collect()
}

fn legacy_build_line_segments(
    entries: &[UiAssetSourceOutlineEntry],
) -> Vec<UiAssetSourceOutlineLineSegment> {
    let mut events = BTreeMap::<i32, UiAssetSourceOutlineLineEvents>::new();
    for (entry_index, entry) in entries.iter().enumerate() {
        if entry.end_line < entry.line {
            continue;
        }
        let end_line = entry.end_line;
        let priority = UiAssetSourceOutlineLinePriority {
            start_line: entry.line,
            span: std::cmp::Reverse(end_line.saturating_sub(entry.line)),
            entry_index,
        };
        events.entry(entry.line).or_default().starts.push(priority);
        if let Some(after_end_line) = end_line.checked_add(1) {
            events
                .entry(after_end_line)
                .or_default()
                .ends
                .push(priority);
        }
    }

    let boundaries = events.keys().copied().collect::<Vec<_>>();
    let mut active = BTreeSet::new();
    let mut segments = Vec::new();
    for (boundary_index, start_line) in boundaries.iter().copied().enumerate() {
        let events_at_line = &events[&start_line];
        for priority in &events_at_line.ends {
            active.remove(priority);
        }
        for priority in &events_at_line.starts {
            active.insert(*priority);
        }
        let Some(next_start_line) = boundaries.get(boundary_index + 1).copied() else {
            if let Some(priority) = active.last().copied() {
                segments.push(UiAssetSourceOutlineLineSegment {
                    start_line,
                    end_line: i32::MAX,
                    entry_index: priority.entry_index,
                });
            }
            continue;
        };
        let Some(priority) = active.last().copied() else {
            continue;
        };
        let end_line = next_start_line.saturating_sub(1);
        if start_line <= end_line {
            segments.push(UiAssetSourceOutlineLineSegment {
                start_line,
                end_line,
                entry_index: priority.entry_index,
            });
        }
    }
    segments
}

fn segment_values(segments: &[UiAssetSourceOutlineLineSegment]) -> Vec<(i32, i32, usize)> {
    segments
        .iter()
        .map(|segment| (segment.start_line, segment.end_line, segment.entry_index))
        .collect()
}

fn percentile_95(samples: &mut [Duration]) -> Duration {
    samples.sort_unstable();
    let rank = (samples.len() * 95).div_ceil(100);
    samples[rank.saturating_sub(1)]
}

#[test]
fn optimization_batch_jt_editor659_streamed_boundaries_preserve_nested_segments() {
    let entries = vec![
        UiAssetSourceOutlineEntry {
            node_id: "root".to_string(),
            block_label: "[nodes.root]".to_string(),
            line: 1,
            end_line: 12,
            excerpt: String::new(),
        },
        UiAssetSourceOutlineEntry {
            node_id: "child".to_string(),
            block_label: "[nodes.child]".to_string(),
            line: 4,
            end_line: 8,
            excerpt: String::new(),
        },
        UiAssetSourceOutlineEntry {
            node_id: "leaf".to_string(),
            block_label: "[nodes.leaf]".to_string(),
            line: 4,
            end_line: 6,
            excerpt: String::new(),
        },
    ];

    assert_eq!(
        segment_values(&build_line_segments(&entries)),
        segment_values(&legacy_build_line_segments(&entries)),
    );
}

#[test]
fn optimization_batch_jt_editor659_streams_owned_boundary_events() {
    let source = include_str!("../../source_sync.rs");
    let implementation = source
        .split("fn build_line_segments")
        .nth(1)
        .and_then(|body| {
            body.split("pub(crate) fn build_source_selection_summary")
                .next()
        })
        .expect("build_line_segments implementation");

    assert!(implementation.contains("events.into_iter().peekable()"));
    assert!(implementation.contains("events.peek()"));
    assert!(!implementation.contains("events.keys().copied().collect"));
    assert!(!implementation.contains("events[&start_line]"));
}

#[test]
#[ignore = "release performance evidence; run through the validation coordinator"]
fn optimization_batch_jt_editor659_streamed_boundary_performance_evidence() {
    let entries = entries();
    assert_eq!(
        segment_values(&build_line_segments(&entries)),
        segment_values(&legacy_build_line_segments(&entries)),
    );
    for _ in 0..4 {
        black_box(legacy_build_line_segments(black_box(&entries)));
        black_box(build_line_segments(black_box(&entries)));
    }

    let mut legacy_samples = Vec::with_capacity(SAMPLE_COUNT);
    let mut optimized_samples = Vec::with_capacity(SAMPLE_COUNT);
    for sample_index in 0..SAMPLE_COUNT {
        if sample_index % 2 == 0 {
            let started = Instant::now();
            black_box(legacy_build_line_segments(black_box(&entries)));
            legacy_samples.push(started.elapsed());

            let started = Instant::now();
            black_box(build_line_segments(black_box(&entries)));
            optimized_samples.push(started.elapsed());
        } else {
            let started = Instant::now();
            black_box(build_line_segments(black_box(&entries)));
            optimized_samples.push(started.elapsed());

            let started = Instant::now();
            black_box(legacy_build_line_segments(black_box(&entries)));
            legacy_samples.push(started.elapsed());
        }
    }

    let legacy_p95 = percentile_95(&mut legacy_samples);
    let optimized_p95 = percentile_95(&mut optimized_samples);
    println!(
        "EDITOR659_STREAMED_SOURCE_OUTLINE_BOUNDARIES_BENCH_V1 entry_count={ENTRY_COUNT} boundary_count={} sample_count={SAMPLE_COUNT} legacy_boundary_vector_allocations=1 optimized_boundary_vector_allocations=0 legacy_tree_lookups={} optimized_tree_lookups=0 legacy_p95_ns={} optimized_p95_ns={} target_ratio_bp=8000",
        ENTRY_COUNT * 2,
        ENTRY_COUNT * 2,
        legacy_p95.as_nanos(),
        optimized_p95.as_nanos(),
    );
    assert!(
        optimized_p95.as_nanos() * 100 <= legacy_p95.as_nanos() * 80,
        "streamed boundary P95 {optimized_p95:?} exceeded 80% of legacy P95 {legacy_p95:?}",
    );
}
