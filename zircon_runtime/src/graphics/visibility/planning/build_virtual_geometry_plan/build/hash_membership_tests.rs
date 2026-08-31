use std::hint::black_box;
use std::time::{Duration, Instant};

use super::*;

const SAMPLE_PAIRS: usize = 17;

#[test]
fn optimization_batch_20260826aj_prioritized_pages_preserve_first_seen_order() {
    assert_eq!(
        prioritized_requested_pages(vec![7, 3, 7, 2], vec![3, 5, 7, 11], 5),
        vec![7, 3, 2, 5, 11]
    );
    assert!(prioritized_requested_pages(vec![1, 2], vec![3], 0).is_empty());
}

#[test]
fn optimization_batch_20260826aj_virtual_geometry_membership_uses_hash_indexes() {
    let source = include_str!("../build.rs");

    assert!(source.contains("HashSet::with_capacity"));
    assert!(source.contains("requested_page_ids.insert(page_id)"));
    assert!(source.contains("hot_resident_pages("));
    assert!(!source.contains("if requested_pages.contains(&page_id)"));
    assert!(!source.contains("filter(|page_id| !evictable_pages.contains(page_id))"));
}

#[test]
#[ignore = "release-only performance contract"]
fn optimization_batch_20260826aj_virtual_geometry_hash_membership_p95() {
    let cascade_requests = (0..4_096_u32).chain(0..4_096_u32).collect::<Vec<_>>();
    let ranked_requests = (2_048..6_144_u32).collect::<Vec<_>>();
    let (prioritized_baseline, prioritized_optimized) = paired_samples(
        || {
            black_box(legacy_prioritized_requested_pages(
                black_box(cascade_requests.clone()),
                black_box(ranked_requests.clone()),
                6_144,
            ));
        },
        || {
            black_box(prioritized_requested_pages(
                black_box(cascade_requests.clone()),
                black_box(ranked_requests.clone()),
                6_144,
            ));
        },
    );

    let resident_pages = (0..4_096_u32).collect::<Vec<_>>();
    let visible_page_set = (0..512_u32).collect::<BTreeSet<_>>();
    let evictable_pages = (512..3_072_u32).collect::<Vec<_>>();
    let (hot_baseline, hot_optimized) = paired_samples(
        || {
            black_box(legacy_hot_resident_pages(
                black_box(&resident_pages),
                black_box(&visible_page_set),
                black_box(&evictable_pages),
            ));
        },
        || {
            black_box(hot_resident_pages(
                black_box(&resident_pages),
                black_box(&visible_page_set),
                black_box(&evictable_pages),
            ));
        },
    );

    let prioritized_baseline_p95 = percentile_95(&prioritized_baseline);
    let prioritized_optimized_p95 = percentile_95(&prioritized_optimized);
    let hot_baseline_p95 = percentile_95(&hot_baseline);
    let hot_optimized_p95 = percentile_95(&hot_optimized);

    println!(
        "RUNTIME09B_VIRTUAL_GEOMETRY_HASH_MEMBERSHIP_BENCH_V1 \
         prioritized_baseline_p95_ns={} prioritized_optimized_p95_ns={} \
         hot_baseline_p95_ns={} hot_optimized_p95_ns={}",
        prioritized_baseline_p95.as_nanos(),
        prioritized_optimized_p95.as_nanos(),
        hot_baseline_p95.as_nanos(),
        hot_optimized_p95.as_nanos(),
    );

    assert_at_most_sixty_percent(
        "prioritized request deduplication",
        prioritized_baseline_p95,
        prioritized_optimized_p95,
    );
    assert_at_most_sixty_percent(
        "hot resident classification",
        hot_baseline_p95,
        hot_optimized_p95,
    );
}

fn legacy_prioritized_requested_pages(
    cascade_requests: Vec<u32>,
    ranked_requests: Vec<u32>,
    budget: usize,
) -> Vec<u32> {
    let mut requested_pages = Vec::with_capacity(budget);
    for page_id in cascade_requests.into_iter().chain(ranked_requests) {
        if requested_pages.contains(&page_id) {
            continue;
        }
        requested_pages.push(page_id);
        if requested_pages.len() >= budget {
            break;
        }
    }
    requested_pages
}

fn legacy_hot_resident_pages(
    resident_pages: &[u32],
    visible_page_set: &BTreeSet<u32>,
    evictable_pages: &[u32],
) -> Vec<u32> {
    resident_pages
        .iter()
        .copied()
        .filter(|page_id| !visible_page_set.contains(page_id))
        .filter(|page_id| !evictable_pages.contains(page_id))
        .collect()
}

fn paired_samples(
    mut baseline: impl FnMut(),
    mut optimized: impl FnMut(),
) -> (Vec<Duration>, Vec<Duration>) {
    let mut baseline_samples = Vec::with_capacity(SAMPLE_PAIRS);
    let mut optimized_samples = Vec::with_capacity(SAMPLE_PAIRS);
    for pair in 0..SAMPLE_PAIRS {
        if pair % 2 == 0 {
            baseline_samples.push(measure(&mut baseline));
            optimized_samples.push(measure(&mut optimized));
        } else {
            optimized_samples.push(measure(&mut optimized));
            baseline_samples.push(measure(&mut baseline));
        }
    }
    (baseline_samples, optimized_samples)
}

fn measure(operation: &mut impl FnMut()) -> Duration {
    let started = Instant::now();
    operation();
    started.elapsed()
}

fn percentile_95(samples: &[Duration]) -> Duration {
    let mut ordered = samples.to_vec();
    ordered.sort_unstable();
    ordered[(ordered.len() * 95).div_ceil(100).saturating_sub(1)]
}

fn assert_at_most_sixty_percent(label: &str, baseline: Duration, optimized: Duration) {
    assert!(
        optimized.as_nanos().saturating_mul(100) <= baseline.as_nanos().saturating_mul(60),
        "{label}: optimized P95 {optimized:?} exceeded 60% of baseline P95 {baseline:?}",
    );
}
