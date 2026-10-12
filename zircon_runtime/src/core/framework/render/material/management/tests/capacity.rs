use std::hint::black_box;
use std::time::Instant;

use super::{
    RenderMaterialManagementIssueKind, RenderMaterialManagementQuery,
    RenderMaterialManagementQueryFilter, RenderMaterialManagementQueryFilterKind,
    RenderMaterialReadinessStatus, ACTIVE_QUERY_FILTER_MAX_COUNT,
};

const SAMPLE_PAIRS: usize = 21;
const BUILDS_PER_SAMPLE: usize = 174_763;

#[test]
fn optimization_batch_20260826ev_runtime191_capacity_preserves_active_filter_order() {
    let query = RenderMaterialManagementQuery::new()
        .with_status(RenderMaterialReadinessStatus::Invalid)
        .with_issue_kind(RenderMaterialManagementIssueKind::Diagnostic)
        .with_text_filter(" roughness ");

    let filters = query.active_filters();

    assert_eq!(filters.len(), ACTIVE_QUERY_FILTER_MAX_COUNT);
    assert!(filters.capacity() >= ACTIVE_QUERY_FILTER_MAX_COUNT);
    assert_eq!(
        filters[0].kind,
        RenderMaterialManagementQueryFilterKind::Status
    );
    assert_eq!(
        filters[1].kind,
        RenderMaterialManagementQueryFilterKind::IssueKind
    );
    assert_eq!(
        filters[2].kind,
        RenderMaterialManagementQueryFilterKind::Text
    );
    assert_eq!(filters[2].text.as_deref(), Some("roughness"));
    assert!(filters[0].remove_query.status.is_none());
    assert!(filters[1].remove_query.issue_kind.is_none());
    assert!(filters[2].remove_query.text_filter.is_none());
}

#[test]
fn optimization_batch_20260826ev_runtime191_active_filters_reserve_fixed_maximum() {
    let source = include_str!("../query_filters.rs");
    assert!(source.contains("const ACTIVE_QUERY_FILTER_MAX_COUNT: usize = 3;"));
    assert!(source.contains("Vec::with_capacity(ACTIVE_QUERY_FILTER_MAX_COUNT)"));
}

#[test]
#[ignore = "managed Windows release performance evidence"]
fn optimization_batch_20260826ev_runtime191_material_query_filter_capacity_bench() {
    let filter = RenderMaterialManagementQueryFilter::status(
        RenderMaterialReadinessStatus::Invalid,
        RenderMaterialManagementQuery::default(),
    );
    let mut legacy_samples = Vec::with_capacity(SAMPLE_PAIRS);
    let mut optimized_samples = Vec::with_capacity(SAMPLE_PAIRS);
    for pair in 0..SAMPLE_PAIRS {
        if pair % 2 == 0 {
            legacy_samples.push(measure(&filter, false));
            optimized_samples.push(measure(&filter, true));
        } else {
            optimized_samples.push(measure(&filter, true));
            legacy_samples.push(measure(&filter, false));
        }
    }
    let legacy_p50_ns = percentile(&legacy_samples, 50);
    let optimized_p50_ns = percentile(&optimized_samples, 50);
    let legacy_p95_ns = percentile(&legacy_samples, 95);
    let optimized_p95_ns = percentile(&optimized_samples, 95);
    println!(
        "RUNTIME191_MATERIAL_QUERY_FILTER_CAPACITY_BENCH_V1 sample_pairs={SAMPLE_PAIRS} \
builds_per_sample={BUILDS_PER_SAMPLE} filters_per_build={ACTIVE_QUERY_FILTER_MAX_COUNT} \
legacy_reservations_per_build=0 optimized_reservations_per_build=1 \
legacy_p50_ns={legacy_p50_ns} optimized_p50_ns={optimized_p50_ns} \
legacy_p95_ns={legacy_p95_ns} optimized_p95_ns={optimized_p95_ns} \
legacy_raw_ns={} optimized_raw_ns={}",
        sample_csv(&legacy_samples),
        sample_csv(&optimized_samples),
    );
    assert!(optimized_p95_ns.saturating_mul(100) <= legacy_p95_ns.saturating_mul(70));
}

fn measure(filter: &RenderMaterialManagementQueryFilter, reserve: bool) -> u128 {
    let started = Instant::now();
    let mut checksum = 0usize;
    for _ in 0..BUILDS_PER_SAMPLE {
        let mut filters = if reserve {
            Vec::with_capacity(ACTIVE_QUERY_FILTER_MAX_COUNT)
        } else {
            Vec::new()
        };
        for _ in 0..ACTIVE_QUERY_FILTER_MAX_COUNT {
            filters.push(black_box(filter.clone()));
        }
        checksum ^= black_box(filters.len() ^ filters.capacity());
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
