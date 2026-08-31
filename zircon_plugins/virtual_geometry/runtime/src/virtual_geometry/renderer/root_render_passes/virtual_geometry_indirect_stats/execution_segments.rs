use std::collections::HashSet;

use zircon_runtime::core::framework::render::{
    RenderVirtualGeometryExecutionDraw, RenderVirtualGeometryExecutionSegment,
    RenderVirtualGeometryExecutionState,
};

pub(super) fn collect_execution_segments(
    indirect_execution_draws: &[&RenderVirtualGeometryExecutionDraw],
) -> Vec<RenderVirtualGeometryExecutionSegment> {
    indirect_execution_draws
        .iter()
        .enumerate()
        .map(|(draw_index, draw)| RenderVirtualGeometryExecutionSegment {
            original_index: draw_index as u32,
            ..draw.execution_segment.clone()
        })
        .collect()
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
struct ExecutionSegmentKey {
    instance_index: u32,
    entity: u64,
    stable_instance_key: u64,
    page_id: u32,
    cluster_start_ordinal: u32,
    cluster_span_count: u32,
    cluster_total_count: u32,
    submission_slot: u32,
    state: u32,
    lineage_depth: u32,
    lod_level: u8,
    frontier_rank: u32,
}

#[derive(Default)]
pub(super) struct ExecutionSegmentSummary {
    segment_count: u32,
    page_count: u32,
    resident_segment_count: u32,
    pending_segment_count: u32,
    missing_segment_count: u32,
    repeated_draw_count: u32,
}

impl ExecutionSegmentSummary {
    fn new(
        segment_count: u32,
        page_count: u32,
        resident_segment_count: u32,
        pending_segment_count: u32,
        missing_segment_count: u32,
        repeated_draw_count: u32,
    ) -> Self {
        Self {
            segment_count,
            page_count,
            resident_segment_count,
            pending_segment_count,
            missing_segment_count,
            repeated_draw_count,
        }
    }

    pub(super) fn segment_count(&self) -> u32 {
        self.segment_count
    }

    pub(super) fn page_count(&self) -> u32 {
        self.page_count
    }

    pub(super) fn resident_segment_count(&self) -> u32 {
        self.resident_segment_count
    }

    pub(super) fn pending_segment_count(&self) -> u32 {
        self.pending_segment_count
    }

    pub(super) fn missing_segment_count(&self) -> u32 {
        self.missing_segment_count
    }

    pub(super) fn repeated_draw_count(&self) -> u32 {
        self.repeated_draw_count
    }
}

pub(super) fn execution_segment_summary(
    execution_segments: &[RenderVirtualGeometryExecutionSegment],
    indirect_execution_draw_count: u32,
) -> ExecutionSegmentSummary {
    let mut segments = HashSet::with_capacity(execution_segments.len());
    let mut pages = HashSet::with_capacity(execution_segments.len());
    let mut resident_segment_count = 0;
    let mut pending_segment_count = 0;
    let mut missing_segment_count = 0;

    for segment in execution_segments {
        let key = ExecutionSegmentKey::from(segment);
        if segments.insert(key) {
            pages.insert(segment.page_id);
            match segment.state {
                RenderVirtualGeometryExecutionState::Resident => resident_segment_count += 1,
                RenderVirtualGeometryExecutionState::PendingUpload => pending_segment_count += 1,
                RenderVirtualGeometryExecutionState::Missing => missing_segment_count += 1,
            }
        }
    }

    let segment_count = segments.len() as u32;
    ExecutionSegmentSummary::new(
        segment_count,
        pages.len() as u32,
        resident_segment_count,
        pending_segment_count,
        missing_segment_count,
        indirect_execution_draw_count.saturating_sub(segment_count),
    )
}

impl From<&RenderVirtualGeometryExecutionSegment> for ExecutionSegmentKey {
    fn from(segment: &RenderVirtualGeometryExecutionSegment) -> Self {
        Self {
            instance_index: segment.instance_index.unwrap_or(u32::MAX),
            entity: segment.entity,
            stable_instance_key: segment.stable_instance_key_or_legacy(),
            page_id: segment.page_id,
            cluster_start_ordinal: segment.cluster_start_ordinal,
            cluster_span_count: segment.cluster_span_count,
            cluster_total_count: segment.cluster_total_count,
            submission_slot: segment.submission_slot.unwrap_or(u32::MAX),
            state: encode_execution_state(segment.state),
            lineage_depth: segment.lineage_depth,
            lod_level: segment.lod_level,
            frontier_rank: segment.frontier_rank,
        }
    }
}

fn encode_execution_state(state: RenderVirtualGeometryExecutionState) -> u32 {
    match state {
        RenderVirtualGeometryExecutionState::Resident => 0,
        RenderVirtualGeometryExecutionState::PendingUpload => 1,
        RenderVirtualGeometryExecutionState::Missing => 2,
    }
}

#[cfg(test)]
mod tests {
    use std::collections::HashSet;
    use std::hint::black_box;
    use std::time::Instant;

    use super::{
        execution_segment_summary, ExecutionSegmentKey, ExecutionSegmentSummary,
        RenderVirtualGeometryExecutionSegment,
    };
    use zircon_runtime::core::framework::render::RenderVirtualGeometryExecutionState;

    const BENCH_SEGMENT_COUNT: usize = 4_096;
    const CHECKS_PER_SAMPLE: usize = 32;
    const SAMPLE_PAIRS: usize = 21;

    #[test]
    fn execution_segment_summary_counts_unique_segments_by_execution_projection() {
        let segments = vec![
            execution_segment(0, 10, RenderVirtualGeometryExecutionState::Resident),
            execution_segment(1, 10, RenderVirtualGeometryExecutionState::Resident),
            execution_segment(2, 11, RenderVirtualGeometryExecutionState::PendingUpload),
        ];

        let summary = execution_segment_summary(&segments, segments.len() as u32);

        assert_eq!(summary.segment_count(), 2);
        assert_eq!(summary.page_count(), 2);
        assert_eq!(summary.resident_segment_count(), 1);
        assert_eq!(summary.pending_segment_count(), 1);
        assert_eq!(summary.missing_segment_count(), 0);
        assert_eq!(summary.repeated_draw_count(), 1);
    }

    #[test]
    #[ignore = "release-only execution segment summary benchmark"]
    fn execution_segment_summary_release_benchmark_evidence() {
        let segments = (0..BENCH_SEGMENT_COUNT)
            .map(|index| {
                execution_segment(
                    index as u32,
                    ((index * 1_549) % BENCH_SEGMENT_COUNT) as u32,
                    RenderVirtualGeometryExecutionState::Resident,
                )
            })
            .collect::<Vec<_>>();
        assert_eq!(
            summary_tuple(&legacy_execution_segment_summary(
                &segments,
                segments.len() as u32,
            )),
            summary_tuple(&execution_segment_summary(&segments, segments.len() as u32,))
        );

        for _ in 0..4 {
            black_box(measure_legacy(&segments));
            black_box(measure_optimized(&segments));
        }

        let mut legacy_samples = Vec::with_capacity(SAMPLE_PAIRS);
        let mut optimized_samples = Vec::with_capacity(SAMPLE_PAIRS);
        for pair in 0..SAMPLE_PAIRS {
            if pair % 2 == 0 {
                legacy_samples.push(measure_legacy(&segments));
                optimized_samples.push(measure_optimized(&segments));
            } else {
                optimized_samples.push(measure_optimized(&segments));
                legacy_samples.push(measure_legacy(&segments));
            }
        }

        let legacy_p50_ns = percentile(&legacy_samples, 50);
        let optimized_p50_ns = percentile(&optimized_samples, 50);
        let legacy_p95_ns = percentile(&legacy_samples, 95);
        let optimized_p95_ns = percentile(&optimized_samples, 95);

        println!(
            "PERF_RESULT plan=Plugins17 task=execution_segment_summary_capacity \
sample_pairs={SAMPLE_PAIRS} checks_per_sample={CHECKS_PER_SAMPLE} \
segment_count={BENCH_SEGMENT_COUNT} page_count={BENCH_SEGMENT_COUNT} \
pair_order=alternating_legacy_even legacy_first_pairs=11 optimized_first_pairs=10 \
legacy_preallocated_sets=0 optimized_preallocated_set_entries={} \
legacy_p50_ns={legacy_p50_ns} optimized_p50_ns={optimized_p50_ns} \
legacy_p95_ns={legacy_p95_ns} optimized_p95_ns={optimized_p95_ns} \
legacy_raw_ns={} optimized_raw_ns={}",
            BENCH_SEGMENT_COUNT * 2,
            raw(&legacy_samples),
            raw(&optimized_samples),
        );

        assert!(
            optimized_p95_ns.saturating_mul(10) <= legacy_p95_ns.saturating_mul(9),
            "preallocated execution segment sets must reduce P95 by at least 10%: \
legacy={legacy_p95_ns}ns optimized={optimized_p95_ns}ns"
        );
    }

    fn measure_legacy(segments: &[RenderVirtualGeometryExecutionSegment]) -> u128 {
        let started = Instant::now();
        for _ in 0..CHECKS_PER_SAMPLE {
            black_box(legacy_execution_segment_summary(
                black_box(segments),
                segments.len() as u32,
            ));
        }
        started.elapsed().as_nanos().max(1)
    }

    fn measure_optimized(segments: &[RenderVirtualGeometryExecutionSegment]) -> u128 {
        let started = Instant::now();
        for _ in 0..CHECKS_PER_SAMPLE {
            black_box(execution_segment_summary(
                black_box(segments),
                segments.len() as u32,
            ));
        }
        started.elapsed().as_nanos().max(1)
    }

    fn legacy_execution_segment_summary(
        execution_segments: &[RenderVirtualGeometryExecutionSegment],
        indirect_execution_draw_count: u32,
    ) -> ExecutionSegmentSummary {
        let mut segments = HashSet::new();
        let mut pages = HashSet::new();
        let mut resident_segment_count = 0;
        let mut pending_segment_count = 0;
        let mut missing_segment_count = 0;
        for segment in execution_segments {
            let key = ExecutionSegmentKey::from(segment);
            if segments.insert(key) {
                pages.insert(segment.page_id);
                match segment.state {
                    RenderVirtualGeometryExecutionState::Resident => resident_segment_count += 1,
                    RenderVirtualGeometryExecutionState::PendingUpload => {
                        pending_segment_count += 1
                    }
                    RenderVirtualGeometryExecutionState::Missing => missing_segment_count += 1,
                }
            }
        }
        let segment_count = segments.len() as u32;
        ExecutionSegmentSummary::new(
            segment_count,
            pages.len() as u32,
            resident_segment_count,
            pending_segment_count,
            missing_segment_count,
            indirect_execution_draw_count.saturating_sub(segment_count),
        )
    }

    fn summary_tuple(summary: &ExecutionSegmentSummary) -> (u32, u32, u32, u32, u32, u32) {
        (
            summary.segment_count(),
            summary.page_count(),
            summary.resident_segment_count(),
            summary.pending_segment_count(),
            summary.missing_segment_count(),
            summary.repeated_draw_count(),
        )
    }

    fn execution_segment(
        original_index: u32,
        page_id: u32,
        state: RenderVirtualGeometryExecutionState,
    ) -> RenderVirtualGeometryExecutionSegment {
        RenderVirtualGeometryExecutionSegment {
            original_index,
            instance_index: Some(1),
            entity: 42,
            stable_instance_key: 0,
            page_id,
            draw_ref_index: original_index,
            submission_index: Some(page_id),
            draw_ref_rank: Some(0),
            cluster_start_ordinal: 0,
            cluster_span_count: 1,
            cluster_total_count: 1,
            submission_slot: Some(page_id),
            state,
            lineage_depth: 0,
            lod_level: 0,
            frontier_rank: 0,
        }
    }

    fn percentile(samples: &[u128], percentile: usize) -> u128 {
        let mut sorted = samples.to_vec();
        sorted.sort_unstable();
        let rank = (sorted.len() * percentile).div_ceil(100);
        sorted[rank.saturating_sub(1)]
    }

    fn raw(samples: &[u128]) -> String {
        samples
            .iter()
            .map(u128::to_string)
            .collect::<Vec<_>>()
            .join(",")
    }
}
