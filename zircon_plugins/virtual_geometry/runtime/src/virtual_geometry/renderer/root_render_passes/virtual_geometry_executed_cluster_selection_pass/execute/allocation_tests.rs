use std::collections::HashSet;
use std::hint::black_box;
use std::time::Instant;

use zircon_runtime::core::framework::render::{
    RenderVirtualGeometryExecutionDraw, RenderVirtualGeometryExecutionSegment,
    RenderVirtualGeometryExecutionState,
};

use super::collect_execution_submission_keys;

const BENCH_DRAW_COUNT: usize = 4_096;
const CHECKS_PER_SAMPLE: usize = 32;
const SAMPLE_PAIRS: usize = 21;

#[test]
fn execution_submission_keys_skip_missing_and_deduplicate_in_place() {
    let draws = [
        draw(1, Some((42, 7))),
        draw(2, None),
        draw(3, Some((42, 7))),
    ];
    let draw_refs = draws.iter().collect::<Vec<_>>();

    assert_eq!(
        collect_execution_submission_keys(&draw_refs),
        HashSet::from([(42, 7)])
    );
}

#[test]
#[ignore = "release-only execution submission key benchmark"]
fn execution_submission_keys_release_benchmark_evidence() {
    let draws = (0..BENCH_DRAW_COUNT)
        .map(|index| {
            let shuffled = (index * 1_549) % BENCH_DRAW_COUNT;
            draw(index as u32, Some((42, shuffled as u32)))
        })
        .collect::<Vec<_>>();
    let draw_refs = draws.iter().collect::<Vec<_>>();
    assert_eq!(
        legacy_keys(&draw_refs),
        collect_execution_submission_keys(&draw_refs)
    );

    for _ in 0..4 {
        black_box(measure_legacy(&draw_refs));
        black_box(measure_optimized(&draw_refs));
    }

    let mut legacy_samples = Vec::with_capacity(SAMPLE_PAIRS);
    let mut optimized_samples = Vec::with_capacity(SAMPLE_PAIRS);
    for pair in 0..SAMPLE_PAIRS {
        if pair % 2 == 0 {
            legacy_samples.push(measure_legacy(&draw_refs));
            optimized_samples.push(measure_optimized(&draw_refs));
        } else {
            optimized_samples.push(measure_optimized(&draw_refs));
            legacy_samples.push(measure_legacy(&draw_refs));
        }
    }

    let legacy_p50_ns = percentile(&legacy_samples, 50);
    let optimized_p50_ns = percentile(&optimized_samples, 50);
    let legacy_p95_ns = percentile(&legacy_samples, 95);
    let optimized_p95_ns = percentile(&optimized_samples, 95);

    println!(
        "PERF_RESULT plan=Plugins17 task=execution_submission_key_capacity \
sample_pairs={SAMPLE_PAIRS} checks_per_sample={CHECKS_PER_SAMPLE} \
draw_count={BENCH_DRAW_COUNT} unique_key_count={BENCH_DRAW_COUNT} \
pair_order=alternating_legacy_even legacy_first_pairs=11 optimized_first_pairs=10 \
legacy_preallocated_keys=0 optimized_preallocated_keys={BENCH_DRAW_COUNT} \
legacy_p50_ns={legacy_p50_ns} optimized_p50_ns={optimized_p50_ns} \
legacy_p95_ns={legacy_p95_ns} optimized_p95_ns={optimized_p95_ns} \
legacy_raw_ns={} optimized_raw_ns={}",
        raw(&legacy_samples),
        raw(&optimized_samples),
    );

    assert!(
        optimized_p95_ns.saturating_mul(10) <= legacy_p95_ns.saturating_mul(9),
        "preallocated execution submission keys must reduce P95 by at least 10%: \
legacy={legacy_p95_ns}ns optimized={optimized_p95_ns}ns"
    );
}

fn measure_legacy(draws: &[&RenderVirtualGeometryExecutionDraw]) -> u128 {
    let started = Instant::now();
    for _ in 0..CHECKS_PER_SAMPLE {
        black_box(legacy_keys(black_box(draws)));
    }
    started.elapsed().as_nanos().max(1)
}

fn measure_optimized(draws: &[&RenderVirtualGeometryExecutionDraw]) -> u128 {
    let started = Instant::now();
    for _ in 0..CHECKS_PER_SAMPLE {
        black_box(collect_execution_submission_keys(black_box(draws)));
    }
    started.elapsed().as_nanos().max(1)
}

fn legacy_keys(draws: &[&RenderVirtualGeometryExecutionDraw]) -> HashSet<(u64, u32)> {
    draws
        .iter()
        .filter_map(|draw| draw.execution_selection_key)
        .collect()
}

fn draw(index: u32, key: Option<(u64, u32)>) -> RenderVirtualGeometryExecutionDraw {
    RenderVirtualGeometryExecutionDraw {
        indirect_args_buffer_available: true,
        indirect_args_offset: u64::from(index) * 16,
        uses_indirect_draw: true,
        execution_selection_key: key,
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
        submission_order_record: None,
        draw_submission_record: None,
        draw_submission_token_record: None,
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
