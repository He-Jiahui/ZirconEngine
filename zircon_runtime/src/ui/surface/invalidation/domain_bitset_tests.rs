use std::collections::BTreeSet;
use std::hint::black_box;
use std::time::Instant;

use super::*;

const SAMPLE_PAIRS: usize = 21;
const CHANGE_COUNT: usize = 65_536;

#[test]
fn optimization_batch_20260826cl_runtime_invalidation_domain_bitset_preserves_commit_contract() {
    let mut state = UiSurfaceInvalidationState::default();
    let mut transaction = state.begin_transaction();
    transaction.record_reason(UiNodeId::new(9), UiInvalidationReason::Render);
    transaction.record_reason(UiNodeId::new(3), UiInvalidationReason::Layout);
    transaction.record_reason(UiNodeId::new(9), UiInvalidationReason::Resource);
    transaction.record_reason(UiNodeId::new(3), UiInvalidationReason::HitTest);

    let commit = state
        .apply_transaction(transaction)
        .expect("current transaction")
        .expect("non-empty commit");

    assert_eq!(
        commit
            .changed_nodes
            .iter()
            .map(|change| change.node_id)
            .collect::<Vec<_>>(),
        vec![UiNodeId::new(3), UiNodeId::new(9)]
    );
    assert_eq!(commit.generations.generation, 1);
    assert_eq!(commit.generations.layout, 1);
    assert_eq!(commit.generations.hit_test, 1);
    assert_eq!(commit.generations.render, 1);
    assert_eq!(commit.generations.resource, 1);
    assert_eq!(commit.generations.structure, 0);
    assert_eq!(commit.generations.text, 0);
    assert_eq!(commit.generations.interaction, 0);
}

#[test]
fn optimization_batch_20260826cl_runtime_invalidation_touched_domains_use_fixed_bitset() {
    let source = include_str!("../invalidation.rs");
    let apply = source
        .split("pub fn apply_transaction")
        .nth(1)
        .and_then(|body| {
            body.split("#[derive(Clone, Copy, Debug, PartialEq, Eq)]")
                .next()
        })
        .expect("invalidation apply implementation");

    assert!(apply.contains("let mut touched_domains = 0u8"));
    assert!(apply.contains("reason.domain_bit()"));
    assert!(apply.contains("UiInvalidationReason::ALL"));
    assert!(!apply.contains("let mut touched_domains = BTreeSet::new()"));
}

fn reason_rows() -> Vec<[UiInvalidationReason; 3]> {
    (0..CHANGE_COUNT)
        .map(|index| {
            if index % 2 == 0 {
                [
                    UiInvalidationReason::Layout,
                    UiInvalidationReason::HitTest,
                    UiInvalidationReason::Render,
                ]
            } else {
                [
                    UiInvalidationReason::Text,
                    UiInvalidationReason::Interaction,
                    UiInvalidationReason::Resource,
                ]
            }
        })
        .collect()
}

fn measure_legacy(rows: &[[UiInvalidationReason; 3]]) -> u128 {
    let started = Instant::now();
    let mut touched = BTreeSet::new();
    for reasons in rows {
        touched.extend(reasons.iter().copied());
    }
    black_box(touched.len());
    started.elapsed().as_nanos().max(1)
}

fn measure_optimized(rows: &[[UiInvalidationReason; 3]]) -> u128 {
    let started = Instant::now();
    let mut touched = 0u8;
    for reasons in rows {
        for reason in reasons {
            touched |= reason.domain_bit();
        }
    }
    let touched_count = UiInvalidationReason::ALL
        .into_iter()
        .filter(|reason| touched & reason.domain_bit() != 0)
        .count();
    black_box(touched_count);
    started.elapsed().as_nanos().max(1)
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

#[test]
#[ignore = "release-only invalidation domain accumulation benchmark"]
fn optimization_batch_20260826cl_runtime_invalidation_domain_bitset_release_benchmark() {
    let rows = reason_rows();
    for _ in 0..4 {
        black_box(measure_legacy(&rows));
        black_box(measure_optimized(&rows));
    }

    let mut legacy_samples = Vec::with_capacity(SAMPLE_PAIRS);
    let mut optimized_samples = Vec::with_capacity(SAMPLE_PAIRS);
    for pair in 0..SAMPLE_PAIRS {
        if pair % 2 == 0 {
            legacy_samples.push(measure_legacy(&rows));
            optimized_samples.push(measure_optimized(&rows));
        } else {
            optimized_samples.push(measure_optimized(&rows));
            legacy_samples.push(measure_legacy(&rows));
        }
    }

    let legacy_p50_ns = percentile(&legacy_samples, 50);
    let optimized_p50_ns = percentile(&optimized_samples, 50);
    let legacy_p95_ns = percentile(&legacy_samples, 95);
    let optimized_p95_ns = percentile(&optimized_samples, 95);
    println!(
        "RUNTIME73_INVALIDATION_DOMAIN_BITSET_BENCH_V1 sample_pairs={SAMPLE_PAIRS} \
change_count={CHANGE_COUNT} reasons_per_change=3 \
pair_order=alternating_legacy_even legacy_first_pairs=11 optimized_first_pairs=10 \
legacy_p50_ns={legacy_p50_ns} optimized_p50_ns={optimized_p50_ns} \
legacy_p95_ns={legacy_p95_ns} optimized_p95_ns={optimized_p95_ns} \
legacy_raw_ns={} optimized_raw_ns={}",
        raw(&legacy_samples),
        raw(&optimized_samples),
    );

    assert!(
        optimized_p95_ns.saturating_mul(10) <= legacy_p95_ns.saturating_mul(7),
        "domain bitset must reduce P95 by at least 30%: \
legacy={legacy_p95_ns}ns optimized={optimized_p95_ns}ns"
    );
}
