use std::hint::black_box;
use std::time::Instant;

use super::{extend_incremental_stages, full_rebuild_stages};
use zircon_runtime_interface::ui::template::UiInvalidationStage;

const PERF_MARKER: &str = "RUNTIME362_UI_INVALIDATION_FULL_REBUILD_BENCH_V1";

const INCREMENTAL_STAGES: [UiInvalidationStage; 6] = [
    UiInvalidationStage::ImportGraph,
    UiInvalidationStage::ComponentContract,
    UiInvalidationStage::SelectorMatch,
    UiInvalidationStage::StyleValue,
    UiInvalidationStage::Layout,
    UiInvalidationStage::Render,
];

#[test]
fn optimization_batch_20260830bj_runtime_full_rebuild_preserves_complete_stage_set() {
    let stages = full_rebuild_stages();
    assert_eq!(stages.len(), 8);
    let mut guarded = stages.clone();
    extend_incremental_stages(&mut guarded, true, INCREMENTAL_STAGES);
    assert_eq!(guarded, stages);
}

#[test]
fn optimization_batch_20260830bj_runtime_full_rebuild_source_contract() {
    let source = include_str!("../graph.rs");
    assert!(source.contains("let full_rebuild = previous.document != next.document"));
    assert!(source.contains("extend_incremental_stages("));
    assert!(source.contains("if !full_rebuild"));
}

#[test]
#[ignore = "release performance evidence"]
fn optimization_batch_20260830bj_runtime_full_rebuild_p95() {
    const REBUILDS: usize = 2_000_000;
    const SAMPLES: usize = 17;
    let mut baseline = Vec::with_capacity(SAMPLES);
    let mut candidate = Vec::with_capacity(SAMPLES);
    let full_rebuild = black_box(true);
    for sample in 0..SAMPLES {
        let order = if sample % 2 == 0 { [0, 1] } else { [1, 0] };
        for pass in order {
            let started = Instant::now();
            let mut checksum = 0usize;
            for _ in 0..REBUILDS {
                let mut stages = full_rebuild_stages();
                if pass == 0 {
                    stages.extend(INCREMENTAL_STAGES);
                } else {
                    extend_incremental_stages(&mut stages, full_rebuild, INCREMENTAL_STAGES);
                }
                checksum += stages.len();
                black_box(&mut stages);
            }
            black_box(checksum);
            let elapsed = started.elapsed().as_nanos();
            if pass == 0 {
                baseline.push(elapsed);
            } else {
                candidate.push(elapsed);
            }
        }
    }
    baseline.sort_unstable();
    candidate.sort_unstable();
    let baseline_p95 = baseline[(SAMPLES * 95).div_ceil(100) - 1];
    let candidate_p95 = candidate[(SAMPLES * 95).div_ceil(100) - 1];
    let reduction =
        100.0 * baseline_p95.saturating_sub(candidate_p95) as f64 / baseline_p95.max(1) as f64;
    println!(
        "{PERF_MARKER} rebuilds={REBUILDS} samples={SAMPLES} baseline_p95_ns={baseline_p95} candidate_p95_ns={candidate_p95} p95_reduction_percent={reduction:.2}"
    );
    assert!(candidate_p95.saturating_mul(10) <= baseline_p95.saturating_mul(7));
}
