use std::collections::HashSet;
use std::hint::black_box;
use std::time::Instant;

use super::{PreparedMeshVirtualGeometryExecutionStats, VirtualGeometryExecutionSegmentKey};
use crate::core::framework::render::{
    RenderVirtualGeometryExecutionDraw, RenderVirtualGeometryExecutionSegment,
    RenderVirtualGeometryExecutionState,
};

const SAMPLE_PAIRS: usize = 31;
const BUILDS_PER_SAMPLE: usize = 20;
const DRAW_COUNT: usize = 4_096;

#[test]
fn optimization_batch_20260829ar_runtime318_preallocated_sets_preserve_execution_stats() {
    let draws = execution_draws(64);

    assert_eq!(
        PreparedMeshVirtualGeometryExecutionStats::from_execution_draws(draws.iter().cloned()),
        legacy_stats(draws.iter().cloned())
    );
}

#[test]
fn optimization_batch_20260829ar_runtime318_execution_stats_use_iterator_capacity_hint() {
    let source = include_str!("../stats.rs");
    let implementation = source
        .split("fn from_execution_draws")
        .nth(1)
        .expect("virtual geometry execution stats")
        .split("struct VirtualGeometryExecutionSegmentKey")
        .next()
        .expect("virtual geometry execution stats body");

    assert!(implementation.contains("execution_draws.size_hint()"));
    assert_eq!(
        implementation
            .matches("HashSet::with_capacity(draw_capacity)")
            .count(),
        2
    );
}

#[test]
#[ignore = "managed Windows release performance evidence"]
fn optimization_batch_20260829ar_runtime318_preallocated_virtual_geometry_stats_sets_bench() {
    let draws = execution_draws(DRAW_COUNT);
    assert_eq!(
        PreparedMeshVirtualGeometryExecutionStats::from_execution_draws(draws.iter().cloned()),
        legacy_stats(draws.iter().cloned())
    );

    let mut legacy_samples = Vec::with_capacity(SAMPLE_PAIRS);
    let mut optimized_samples = Vec::with_capacity(SAMPLE_PAIRS);
    for pair in 0..SAMPLE_PAIRS {
        if pair % 2 == 0 {
            legacy_samples.push(measure(&draws, false));
            optimized_samples.push(measure(&draws, true));
        } else {
            optimized_samples.push(measure(&draws, true));
            legacy_samples.push(measure(&draws, false));
        }
    }
    let legacy_p50_ns = percentile(&legacy_samples, 50);
    let optimized_p50_ns = percentile(&optimized_samples, 50);
    let legacy_p95_ns = percentile(&legacy_samples, 95);
    let optimized_p95_ns = percentile(&optimized_samples, 95);
    println!(
        "RUNTIME318_PREALLOCATED_VIRTUAL_GEOMETRY_STATS_SETS_BENCH_V1 sample_pairs={SAMPLE_PAIRS} \
builds_per_sample={BUILDS_PER_SAMPLE} draws_per_build={DRAW_COUNT} unique_segments={DRAW_COUNT} \
legacy_initial_segment_capacity=0 optimized_initial_segment_capacity={DRAW_COUNT} \
legacy_p50_ns={legacy_p50_ns} optimized_p50_ns={optimized_p50_ns} \
legacy_p95_ns={legacy_p95_ns} optimized_p95_ns={optimized_p95_ns} \
legacy_raw_ns={} optimized_raw_ns={}",
        sample_csv(&legacy_samples),
        sample_csv(&optimized_samples),
    );
    assert!(optimized_p95_ns.saturating_mul(100) <= legacy_p95_ns.saturating_mul(70));
}

fn execution_draws(count: usize) -> Vec<RenderVirtualGeometryExecutionDraw> {
    (0..count)
        .map(|index| execution_draw(index as u32))
        .collect()
}

fn execution_draw(index: u32) -> RenderVirtualGeometryExecutionDraw {
    RenderVirtualGeometryExecutionDraw {
        indirect_args_buffer_available: true,
        indirect_args_offset: u64::from(index) * 20,
        uses_indirect_draw: true,
        execution_selection_key: Some((u64::from(index), index)),
        execution_segment: RenderVirtualGeometryExecutionSegment {
            original_index: index,
            instance_index: Some(index),
            entity: u64::from(index),
            stable_instance_key: u64::from(index) + 1,
            page_id: index,
            draw_ref_index: index,
            submission_index: Some(index),
            draw_ref_rank: Some(0),
            cluster_start_ordinal: index,
            cluster_span_count: 1,
            cluster_total_count: 1,
            submission_slot: Some(index),
            state: RenderVirtualGeometryExecutionState::Resident,
            lineage_depth: 0,
            lod_level: 0,
            frontier_rank: index,
        },
        submission_order_record: None,
        draw_submission_record: None,
        draw_submission_token_record: None,
        execution_draw_ref_index: index,
    }
}

fn legacy_stats(
    execution_draws: impl IntoIterator<Item = RenderVirtualGeometryExecutionDraw>,
) -> PreparedMeshVirtualGeometryExecutionStats {
    let mut stats = PreparedMeshVirtualGeometryExecutionStats::default();
    let mut segments = HashSet::new();
    let mut pages = HashSet::new();
    for draw in execution_draws {
        if !draw.uses_indirect_draw || draw.execution_selection_key.is_none() {
            continue;
        }
        stats.draw_count += 1;
        let segment = draw.execution_segment;
        if !segments.insert(VirtualGeometryExecutionSegmentKey::from(&segment)) {
            continue;
        }
        stats.segment_count += 1;
        pages.insert(segment.page_id);
        match segment.state {
            RenderVirtualGeometryExecutionState::Resident => stats.resident_segment_count += 1,
            RenderVirtualGeometryExecutionState::PendingUpload => stats.pending_segment_count += 1,
            RenderVirtualGeometryExecutionState::Missing => stats.missing_segment_count += 1,
        }
    }
    stats.page_count = pages.len();
    stats.repeated_draw_count = stats.draw_count.saturating_sub(stats.segment_count);
    stats
}

fn measure(draws: &[RenderVirtualGeometryExecutionDraw], optimized: bool) -> u128 {
    let started = Instant::now();
    let mut checksum = 0usize;
    for _ in 0..BUILDS_PER_SAMPLE {
        let stats = if optimized {
            PreparedMeshVirtualGeometryExecutionStats::from_execution_draws(
                black_box(draws).iter().cloned(),
            )
        } else {
            legacy_stats(black_box(draws).iter().cloned())
        };
        checksum = checksum
            .wrapping_add(stats.draw_count)
            .wrapping_add(stats.segment_count)
            .wrapping_add(stats.page_count);
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
