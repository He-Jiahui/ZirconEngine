use std::hint::black_box;
use std::time::Instant;

use zircon_runtime::core::framework::render::{
    RenderVirtualGeometryExecutionDraw, RenderVirtualGeometryExecutionSegment,
    RenderVirtualGeometryExecutionState,
};

use super::{collect_execution_draw_projection, ExecutionDrawProjection};

const BENCH_DRAW_COUNT: usize = 4_096;
const CHECKS_PER_SAMPLE: usize = 32;
const SAMPLE_PAIRS: usize = 21;

#[test]
fn execution_draw_projection_preserves_each_source_order() {
    let draws = [
        draw(0, true, true),
        draw(1, false, true),
        draw(2, true, false),
    ];

    let optimized = collect_execution_draw_projection(&draws);
    let legacy = legacy_projection(&draws);

    assert_projection_eq(&optimized, &legacy);
    assert_eq!(
        optimized
            .indirect_execution_draws
            .iter()
            .map(|draw| draw.execution_draw_ref_index)
            .collect::<Vec<_>>(),
        vec![0, 2]
    );
    assert_eq!(optimized.execution_indirect_offsets, vec![0, 32]);
    assert!(optimized.has_execution_args_buffer);
}

#[test]
#[ignore = "release-only execution draw projection benchmark"]
fn execution_draw_projection_release_benchmark_evidence() {
    let draws = (0..BENCH_DRAW_COUNT)
        .map(|index| draw(index as u32, index % 2 == 0, index % 3 == 0))
        .collect::<Vec<_>>();
    assert_projection_eq(
        &collect_execution_draw_projection(&draws),
        &legacy_projection(&draws),
    );

    for _ in 0..4 {
        black_box(measure_legacy(&draws));
        black_box(measure_optimized(&draws));
    }

    let mut legacy_samples = Vec::with_capacity(SAMPLE_PAIRS);
    let mut optimized_samples = Vec::with_capacity(SAMPLE_PAIRS);
    for pair in 0..SAMPLE_PAIRS {
        if pair % 2 == 0 {
            legacy_samples.push(measure_legacy(&draws));
            optimized_samples.push(measure_optimized(&draws));
        } else {
            optimized_samples.push(measure_optimized(&draws));
            legacy_samples.push(measure_legacy(&draws));
        }
    }

    let legacy_p50_ns = percentile(&legacy_samples, 50);
    let optimized_p50_ns = percentile(&optimized_samples, 50);
    let legacy_p95_ns = percentile(&legacy_samples, 95);
    let optimized_p95_ns = percentile(&optimized_samples, 95);

    println!(
        "PERF_RESULT plan=Plugins17 task=execution_draw_projection \
sample_pairs={SAMPLE_PAIRS} checks_per_sample={CHECKS_PER_SAMPLE} \
draw_count={BENCH_DRAW_COUNT} indirect_draw_count={} \
pair_order=alternating_legacy_even legacy_first_pairs=11 optimized_first_pairs=10 \
legacy_source_scans=6 optimized_source_scans=1 optimized_vectors_preallocated=5 \
legacy_p50_ns={legacy_p50_ns} optimized_p50_ns={optimized_p50_ns} \
legacy_p95_ns={legacy_p95_ns} optimized_p95_ns={optimized_p95_ns} \
legacy_raw_ns={} optimized_raw_ns={}",
        BENCH_DRAW_COUNT / 2,
        raw(&legacy_samples),
        raw(&optimized_samples),
    );

    assert!(
        optimized_p95_ns.saturating_mul(4) <= legacy_p95_ns.saturating_mul(3),
        "single-pass execution draw projection must reduce P95 by at least 25%: \
legacy={legacy_p95_ns}ns optimized={optimized_p95_ns}ns"
    );
}

fn measure_legacy(draws: &[RenderVirtualGeometryExecutionDraw]) -> u128 {
    let started = Instant::now();
    for _ in 0..CHECKS_PER_SAMPLE {
        black_box(legacy_projection(black_box(draws)));
    }
    started.elapsed().as_nanos().max(1)
}

fn measure_optimized(draws: &[RenderVirtualGeometryExecutionDraw]) -> u128 {
    let started = Instant::now();
    for _ in 0..CHECKS_PER_SAMPLE {
        black_box(collect_execution_draw_projection(black_box(draws)));
    }
    started.elapsed().as_nanos().max(1)
}

fn legacy_projection(
    execution_draws: &[RenderVirtualGeometryExecutionDraw],
) -> ExecutionDrawProjection<'_> {
    let indirect_execution_draws = execution_draws
        .iter()
        .filter(|draw| draw.uses_indirect_draw)
        .collect::<Vec<_>>();
    let has_execution_args_buffer = indirect_execution_draws
        .iter()
        .any(|draw| draw.indirect_args_buffer_available);
    let draw_submission_order = execution_draws
        .iter()
        .filter_map(|draw| draw.submission_order_record)
        .collect::<Vec<_>>();
    let execution_indirect_offsets = indirect_execution_draws
        .iter()
        .map(|draw| draw.indirect_args_offset)
        .collect::<Vec<_>>();
    let draw_submission_records = execution_draws
        .iter()
        .filter_map(|draw| draw.draw_submission_record)
        .collect::<Vec<_>>();
    let draw_submission_token_records = execution_draws
        .iter()
        .filter_map(|draw| draw.draw_submission_token_record)
        .collect::<Vec<_>>();
    ExecutionDrawProjection {
        indirect_execution_draws,
        has_execution_args_buffer,
        draw_submission_order,
        execution_indirect_offsets,
        draw_submission_records,
        draw_submission_token_records,
    }
}

fn assert_projection_eq(left: &ExecutionDrawProjection<'_>, right: &ExecutionDrawProjection<'_>) {
    assert_eq!(
        left.indirect_execution_draws
            .iter()
            .map(|draw| draw.execution_draw_ref_index)
            .collect::<Vec<_>>(),
        right
            .indirect_execution_draws
            .iter()
            .map(|draw| draw.execution_draw_ref_index)
            .collect::<Vec<_>>()
    );
    assert_eq!(
        left.has_execution_args_buffer,
        right.has_execution_args_buffer
    );
    assert_eq!(left.draw_submission_order, right.draw_submission_order);
    assert_eq!(
        left.execution_indirect_offsets,
        right.execution_indirect_offsets
    );
    assert_eq!(left.draw_submission_records, right.draw_submission_records);
    assert_eq!(
        left.draw_submission_token_records,
        right.draw_submission_token_records
    );
}

fn draw(
    index: u32,
    uses_indirect_draw: bool,
    indirect_args_buffer_available: bool,
) -> RenderVirtualGeometryExecutionDraw {
    RenderVirtualGeometryExecutionDraw {
        indirect_args_buffer_available,
        indirect_args_offset: u64::from(index) * 16,
        uses_indirect_draw,
        execution_selection_key: Some((42, index)),
        execution_segment: RenderVirtualGeometryExecutionSegment {
            original_index: index,
            instance_index: Some(index),
            entity: 42,
            stable_instance_key: u64::from(index) + 1,
            page_id: index,
            draw_ref_index: index,
            submission_index: Some(index),
            draw_ref_rank: Some(index),
            cluster_start_ordinal: index,
            cluster_span_count: 1,
            cluster_total_count: BENCH_DRAW_COUNT as u32,
            submission_slot: Some(index),
            state: RenderVirtualGeometryExecutionState::Resident,
            lineage_depth: 0,
            lod_level: 0,
            frontier_rank: index,
        },
        submission_order_record: (index % 3 == 0).then_some((Some(index), 42, index)),
        draw_submission_record: (index % 5 == 0).then_some((42, index, index, index as usize)),
        draw_submission_token_record: (index % 7 == 0).then_some((
            42,
            index,
            index,
            index,
            index as usize,
        )),
        execution_draw_ref_index: index,
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
