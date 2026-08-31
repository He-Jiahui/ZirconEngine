use std::{hint::black_box, time::Instant};

use super::*;
use crate::core::framework::picking::{HitData, Pickable};
use crate::core::math::Real;

const SAMPLE_PAIRS: usize = 17;

#[test]
fn runtime77_picking_report_preserves_blocked_hover_and_non_hover_counts() {
    let hits = vec![
        hit(1, Pickable::NON_BLOCKING),
        hit(2, Pickable::IGNORE),
        hit(3, Pickable::default()),
        hit(4, Pickable::NON_BLOCKING),
        hit(5, Pickable::IGNORE),
    ];

    let report =
        PickingPointerPipelineReport::from_pointer(PointerId::new(7), 2, &hits, 3, hits.len());

    assert_eq!(report.sorted_hit_count, 5);
    assert_eq!(report.hovered_hit_count, 2);
    assert_eq!(report.non_hoverable_hit_count, 2);
    assert_eq!(report.top_target, Some(HitTarget::renderable(1)));
    assert_eq!(report.blocking_target, Some(HitTarget::renderable(3)));

    let unblocked = summarize_pointer_hits(&hits[..2]);
    assert_eq!(unblocked.hovered_hit_count, 1);
    assert_eq!(unblocked.non_hoverable_hit_count, 1);
    assert_eq!(unblocked.blocking_target, None);
}

#[test]
fn runtime77_picking_report_uses_one_hit_scan() {
    let source = include_str!("../report.rs");
    let pointer = bounded_source(source, "fn from_pointer(", "fn summarize_pointer_hits(");
    let summary = bounded_source(
        source,
        "fn summarize_pointer_hits(",
        "fn output_counts_by_pointer(",
    );

    assert!(pointer.contains("summarize_pointer_hits(sorted_hits)"));
    assert!(!pointer.contains(".position("));
    assert!(!pointer.contains(".filter("));
    assert_eq!(summary.matches("for hit in sorted_hits").count(), 1);
    assert!(!summary.contains(".position("));
    assert!(!summary.contains(".filter("));
}

#[test]
#[ignore = "release performance evidence"]
fn runtime77_picking_report_single_pass_p95() {
    const HIT_COUNT: usize = 65_536;
    const SUMMARIES: usize = 64;
    let hits = (0..HIT_COUNT)
        .map(|index| {
            hit(
                index as u64 + 1,
                if index % 2 == 0 {
                    Pickable::NON_BLOCKING
                } else {
                    Pickable::IGNORE
                },
            )
        })
        .collect::<Vec<_>>();
    let mut legacy_ns = Vec::with_capacity(SAMPLE_PAIRS);
    let mut optimized_ns = Vec::with_capacity(SAMPLE_PAIRS);

    for sample_index in 0..SAMPLE_PAIRS {
        if sample_index % 2 == 0 {
            legacy_ns.push(measure_ns(SUMMARIES, || legacy_summary(black_box(&hits))));
            optimized_ns.push(measure_ns(SUMMARIES, || {
                let summary = summarize_pointer_hits(black_box(&hits));
                summary.hovered_hit_count
                    + summary.non_hoverable_hit_count
                    + usize::from(summary.blocking_target.is_some())
            }));
        } else {
            optimized_ns.push(measure_ns(SUMMARIES, || {
                let summary = summarize_pointer_hits(black_box(&hits));
                summary.hovered_hit_count
                    + summary.non_hoverable_hit_count
                    + usize::from(summary.blocking_target.is_some())
            }));
            legacy_ns.push(measure_ns(SUMMARIES, || legacy_summary(black_box(&hits))));
        }
    }

    let legacy_p50_ns = nearest_rank(&legacy_ns, 50);
    let legacy_p95_ns = nearest_rank(&legacy_ns, 95);
    let optimized_p50_ns = nearest_rank(&optimized_ns, 50);
    let optimized_p95_ns = nearest_rank(&optimized_ns, 95);
    assert!(
        optimized_p95_ns.saturating_mul(2) <= legacy_p95_ns,
        "single-pass pointer summary P95 must be at least 50% below three scans: legacy={legacy_p95_ns}ns optimized={optimized_p95_ns}ns"
    );

    println!(
        "RUNTIME77_PICKING_REPORT_SINGLE_PASS_BENCH_V1 hits={HIT_COUNT} summaries_per_sample={SUMMARIES} sample_pairs={SAMPLE_PAIRS} pair_order=alternating_legacy_even legacy_hit_visits_per_sample={} optimized_hit_visits_per_sample={} legacy_hit_scans_per_summary=3 optimized_hit_scans_per_summary=1 legacy_p50_ns={legacy_p50_ns} legacy_p95_ns={legacy_p95_ns} optimized_p50_ns={optimized_p50_ns} optimized_p95_ns={optimized_p95_ns} legacy_ns={} optimized_ns={}",
        HIT_COUNT * SUMMARIES * 3,
        HIT_COUNT * SUMMARIES,
        join_samples(&legacy_ns),
        join_samples(&optimized_ns),
    );
}

fn hit(owner: u64, pickable: Pickable) -> HitRecord {
    HitRecord::new(
        HitTarget::renderable(owner),
        HitData::new(0, owner as Real, None, None),
    )
    .with_pickable(pickable)
}

fn legacy_summary(sorted_hits: &[HitRecord]) -> usize {
    let blocking_index = sorted_hits
        .iter()
        .position(|hit| hit.pickable.should_block_lower);
    let resolved_hits = blocking_index
        .map(|index| &sorted_hits[..=index])
        .unwrap_or(sorted_hits);
    let hovered_hit_count = resolved_hits
        .iter()
        .filter(|hit| hit.pickable.is_hoverable)
        .count();
    let non_hoverable_hit_count = sorted_hits
        .iter()
        .filter(|hit| !hit.pickable.is_hoverable)
        .count();
    hovered_hit_count + non_hoverable_hit_count + usize::from(blocking_index.is_some())
}

fn measure_ns(iterations: usize, mut operation: impl FnMut() -> usize) -> u128 {
    let started = Instant::now();
    let mut checksum = 0usize;
    for _ in 0..iterations {
        checksum = checksum.wrapping_add(black_box(operation()));
    }
    black_box(checksum);
    started.elapsed().as_nanos()
}

fn bounded_source<'a>(source: &'a str, start: &str, end: &str) -> &'a str {
    source
        .split(start)
        .nth(1)
        .expect("source start")
        .split(end)
        .next()
        .expect("source end")
}

fn nearest_rank(samples: &[u128], percentile: usize) -> u128 {
    let mut sorted = samples.to_vec();
    sorted.sort_unstable();
    let rank = sorted.len().saturating_mul(percentile).div_ceil(100);
    sorted[rank.saturating_sub(1)]
}

fn join_samples(samples: &[u128]) -> String {
    samples
        .iter()
        .map(u128::to_string)
        .collect::<Vec<_>>()
        .join(",")
}
