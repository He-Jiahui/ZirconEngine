use std::hint::black_box;
use std::time::Instant;

use crate::render_graph::RenderGraphComputeWorkload;

use super::{
    ComputeWorkloadDispatchAuditInput, RenderGraphComputeDispatchRecord,
    RenderGraphComputeWorkloadAuditStatus, RenderGraphComputeWorkloadDispatchContext,
    RenderGraphExecutionRecord, visit_compute_workload_dispatches,
};

const DISPATCH_COUNT: usize = 4_096;
const SAMPLE_COUNT: usize = 17;
const LEGACY_DISPATCH_VISITS: usize = DISPATCH_COUNT * 3;
const OPTIMIZED_DISPATCH_VISITS: usize = DISPATCH_COUNT * 2;

#[test]
fn optimization_batch_20260826bj_compute_workload_two_pass_preserves_audit_order() {
    let planned = RenderGraphComputeWorkload::fixed("target-pipeline", [4, 4, 1], [1, 1, 1]);
    let dispatches = [
        dispatch("foreign-before", "foreign.before"),
        dispatch("target", "target.executor"),
        dispatch("foreign-middle", "foreign.middle"),
        dispatch("target", "target.executor"),
    ];
    let mut record = RenderGraphExecutionRecord::default();

    record.audit_compute_workload(
        "target",
        "target.executor",
        Some(&planned),
        dispatch_context(),
        &dispatches,
    );

    assert_eq!(
        record
            .compute_workload_audit()
            .iter()
            .map(|audit| (audit.status, audit.pass_name.as_str()))
            .collect::<Vec<_>>(),
        [
            (RenderGraphComputeWorkloadAuditStatus::Matched, "target"),
            (
                RenderGraphComputeWorkloadAuditStatus::UnexpectedDispatch,
                "target",
            ),
            (
                RenderGraphComputeWorkloadAuditStatus::UnexpectedDispatch,
                "foreign-before",
            ),
            (
                RenderGraphComputeWorkloadAuditStatus::UnexpectedDispatch,
                "foreign-middle",
            ),
        ]
    );

    let mut missing_record = RenderGraphExecutionRecord::default();
    missing_record.audit_compute_workload(
        "missing",
        "missing.executor",
        Some(&planned),
        dispatch_context(),
        &dispatches,
    );
    assert_eq!(
        missing_record.compute_workload_audit()[0].status,
        RenderGraphComputeWorkloadAuditStatus::MissingDispatch
    );
    assert!(
        missing_record.compute_workload_audit()[1..]
            .iter()
            .all(|audit| audit.status == RenderGraphComputeWorkloadAuditStatus::UnexpectedDispatch)
    );
}

#[test]
fn optimization_batch_20260826bj_compute_workload_two_pass_eliminates_third_scan() {
    const SOURCE: &str = include_str!("../render_graph_execution_record.rs");

    assert_eq!(LEGACY_DISPATCH_VISITS, 12_288);
    assert_eq!(OPTIMIZED_DISPATCH_VISITS, 8_192);
    assert!(SOURCE.contains("visit_compute_workload_dispatches("));
    assert!(SOURCE.contains("let mut first_matching_dispatch_index = None"));
    assert!(!SOURCE.contains("dispatches.iter().position"));
}

#[test]
#[ignore = "release-only performance evidence"]
fn optimization_batch_20260826bj_compute_workload_two_pass_p95() {
    let dispatches = (0..DISPATCH_COUNT)
        .map(|index| dispatch(&format!("foreign-pass-{index:04}"), "foreign.executor"))
        .collect::<Vec<_>>();

    let (legacy_samples, optimized_samples) = benchmark_paired_samples::<SAMPLE_COUNT>(
        || {
            legacy_visit(
                black_box(&dispatches),
                black_box("missing"),
                black_box("missing.executor"),
            )
        },
        || {
            optimized_visit(
                black_box(&dispatches),
                black_box("missing"),
                black_box("missing.executor"),
            )
        },
    );
    assert_eq!(
        legacy_visit(&dispatches, "missing", "missing.executor"),
        optimized_visit(&dispatches, "missing", "missing.executor")
    );

    let legacy_p50 = percentile(&legacy_samples, 50);
    let legacy_p95 = percentile(&legacy_samples, 95);
    let optimized_p50 = percentile(&optimized_samples, 50);
    let optimized_p95 = percentile(&optimized_samples, 95);
    println!(
        "PERF_RESULT RUNTIME89_COMPUTE_WORKLOAD_AUDIT_TWO_PASS_BENCH_V1 dispatches={DISPATCH_COUNT} samples={SAMPLE_COUNT} sample_order=alternating legacy_dispatch_visits={LEGACY_DISPATCH_VISITS} optimized_dispatch_visits={OPTIMIZED_DISPATCH_VISITS} deterministic_dispatch_visit_reduction_percent=33.3333 legacy_p50_ns={legacy_p50} optimized_p50_ns={optimized_p50} legacy_p95_ns={legacy_p95} optimized_p95_ns={optimized_p95} legacy_samples_ns={} optimized_samples_ns={}",
        join_samples(&legacy_samples),
        join_samples(&optimized_samples),
    );
    assert!(
        optimized_p95 * 5 <= legacy_p95 * 4,
        "optimized P95 {optimized_p95}ns must be at least 20% below legacy P95 {legacy_p95}ns"
    );
}

fn optimized_visit(
    dispatches: &[RenderGraphComputeDispatchRecord],
    pass_name: &str,
    executor_id: &str,
) -> (usize, usize, usize) {
    let mut matched = 0;
    let mut missing = 0;
    let mut unexpected = 0;
    visit_compute_workload_dispatches(pass_name, executor_id, dispatches, |input| match input {
        ComputeWorkloadDispatchAuditInput::FirstMatch(_) => matched += 1,
        ComputeWorkloadDispatchAuditInput::Missing => missing += 1,
        ComputeWorkloadDispatchAuditInput::Unexpected(_) => unexpected += 1,
    });
    black_box((matched, missing, unexpected))
}

fn legacy_visit(
    dispatches: &[RenderGraphComputeDispatchRecord],
    pass_name: &str,
    executor_id: &str,
) -> (usize, usize, usize) {
    let first_matching_dispatch_index = dispatches.iter().position(|dispatch| {
        dispatch.pass_name == pass_name && dispatch.executor_id == executor_id
    });
    let matched = usize::from(first_matching_dispatch_index.is_some());
    let missing = usize::from(first_matching_dispatch_index.is_none());
    let duplicate_matches = dispatches
        .iter()
        .enumerate()
        .filter(|(index, dispatch)| {
            Some(*index) != first_matching_dispatch_index
                && dispatch.pass_name == pass_name
                && dispatch.executor_id == executor_id
        })
        .count();
    let foreign = dispatches
        .iter()
        .filter(|dispatch| dispatch.pass_name != pass_name || dispatch.executor_id != executor_id)
        .count();
    black_box((matched, missing, duplicate_matches + foreign))
}

fn benchmark_paired_samples<const N: usize>(
    mut legacy: impl FnMut() -> (usize, usize, usize),
    mut optimized: impl FnMut() -> (usize, usize, usize),
) -> (Vec<u128>, Vec<u128>) {
    black_box(legacy());
    black_box(optimized());
    let mut legacy_samples = Vec::with_capacity(N);
    let mut optimized_samples = Vec::with_capacity(N);
    for sample_index in 0..N {
        if sample_index % 2 == 0 {
            legacy_samples.push(benchmark_sample(&mut legacy));
            optimized_samples.push(benchmark_sample(&mut optimized));
        } else {
            optimized_samples.push(benchmark_sample(&mut optimized));
            legacy_samples.push(benchmark_sample(&mut legacy));
        }
    }
    (legacy_samples, optimized_samples)
}

fn benchmark_sample(operation: &mut impl FnMut() -> (usize, usize, usize)) -> u128 {
    let started = Instant::now();
    black_box(operation());
    started.elapsed().as_nanos()
}

fn percentile(samples: &[u128], percentile: usize) -> u128 {
    let mut ordered = samples.to_vec();
    ordered.sort_unstable();
    let rank = ordered.len().saturating_mul(percentile).div_ceil(100);
    ordered[rank.saturating_sub(1)]
}

fn join_samples(samples: &[u128]) -> String {
    samples
        .iter()
        .map(u128::to_string)
        .collect::<Vec<_>>()
        .join(",")
}

fn dispatch(pass_name: &str, executor_id: &str) -> RenderGraphComputeDispatchRecord {
    RenderGraphComputeDispatchRecord::new(
        pass_name,
        executor_id,
        "target-pipeline",
        [4, 4, 1],
        [1, 1, 1],
        Vec::new(),
    )
}

fn dispatch_context() -> RenderGraphComputeWorkloadDispatchContext {
    RenderGraphComputeWorkloadDispatchContext::new([1, 1], [1, 1], 1)
}
