use std::hint::black_box;
use std::time::Instant;

use super::{active_main_page_differs, MainPageId};

const SAMPLE_PAIRS: usize = 31;
const COMPARISONS_PER_SAMPLE: usize = 100_000;
const ACTIVE_PAGE: &str = "exclusive.activity.material-editor.viewport.inspector.properties.animation.timeline.asset-browser";

#[test]
fn optimization_batch_20260829w_editor242_borrowed_page_comparison_preserves_equality() {
    let active = MainPageId::new(ACTIVE_PAGE);
    let same = MainPageId::new(ACTIVE_PAGE);
    let different = MainPageId::new(format!("{ACTIVE_PAGE}.secondary"));

    assert!(!active_main_page_differs(&active, &same));
    assert!(active_main_page_differs(&active, &different));
}

#[test]
fn optimization_batch_20260829w_editor242_exclusive_focus_avoids_comparison_clone() {
    let source = include_str!("../focus.rs");
    let implementation = source.split("#[cfg(test)]").next().expect("implementation");

    assert!(implementation.contains("active_main_page_differs(&layout.active_main_page, id)"));
    assert!(implementation.contains("fn active_main_page_differs"));
    assert!(!implementation.contains("layout.active_main_page != id.clone()"));
}

#[test]
#[ignore = "managed Windows release performance evidence"]
fn optimization_batch_20260829w_editor242_borrowed_exclusive_page_comparison_bench() {
    let active = MainPageId::new(ACTIVE_PAGE);
    let same = MainPageId::new(ACTIVE_PAGE);
    let different = MainPageId::new(format!("{ACTIVE_PAGE}.secondary"));
    let mut legacy_samples = Vec::with_capacity(SAMPLE_PAIRS);
    let mut optimized_samples = Vec::with_capacity(SAMPLE_PAIRS);
    for pair in 0..SAMPLE_PAIRS {
        if pair % 2 == 0 {
            legacy_samples.push(measure(false, &active, &same, &different));
            optimized_samples.push(measure(true, &active, &same, &different));
        } else {
            optimized_samples.push(measure(true, &active, &same, &different));
            legacy_samples.push(measure(false, &active, &same, &different));
        }
    }
    let legacy_p50_ns = percentile(&legacy_samples, 50);
    let optimized_p50_ns = percentile(&optimized_samples, 50);
    let legacy_p95_ns = percentile(&legacy_samples, 95);
    let optimized_p95_ns = percentile(&optimized_samples, 95);
    println!(
        "EDITOR242_BORROWED_EXCLUSIVE_PAGE_COMPARISON_BENCH_V1 sample_pairs={SAMPLE_PAIRS} \
comparisons_per_sample={COMPARISONS_PER_SAMPLE} page_id_bytes={} \
legacy_page_id_allocations_per_comparison=1 optimized_page_id_allocations_per_comparison=0 \
legacy_p50_ns={legacy_p50_ns} optimized_p50_ns={optimized_p50_ns} \
legacy_p95_ns={legacy_p95_ns} optimized_p95_ns={optimized_p95_ns} \
legacy_raw_ns={} optimized_raw_ns={}",
        ACTIVE_PAGE.len(),
        sample_csv(&legacy_samples),
        sample_csv(&optimized_samples),
    );
    assert!(optimized_p95_ns.saturating_mul(100) <= legacy_p95_ns.saturating_mul(70));
}

fn legacy_active_main_page_differs(active: &MainPageId, candidate: &MainPageId) -> bool {
    active != &(*candidate).clone()
}

fn measure(
    optimized: bool,
    active: &MainPageId,
    same: &MainPageId,
    different: &MainPageId,
) -> u128 {
    let started = Instant::now();
    let mut checksum = 0usize;
    for index in 0..COMPARISONS_PER_SAMPLE {
        let candidate = black_box(if index % 2 == 0 { same } else { different });
        checksum = checksum.wrapping_add(usize::from(if optimized {
            active_main_page_differs(active, candidate)
        } else {
            legacy_active_main_page_differs(active, candidate)
        }));
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
