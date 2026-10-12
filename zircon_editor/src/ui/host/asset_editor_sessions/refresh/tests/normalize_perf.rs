use std::collections::BTreeSet;
use std::hint::black_box;
use std::time::Instant;

use super::normalize_ui_asset_change_set;
use crate::ui::host::project_access::normalize_ui_asset_asset_id;

#[test]
fn changed_import_lookup_borrows_the_normalized_asset_id() {
    let source = include_str!("../imports.rs");
    let owned_lookup = [
        "contains(&normalize_ui_asset_asset_id(reference)",
        ".to_string())",
    ]
    .concat();

    assert!(!source.contains(&owned_lookup));
}

#[test]
fn optimization_batch_dw_asset_change_dedup_preserves_normalized_unique_ids() {
    let changed = [
        "res://ui/shared.widget#header",
        "res://ui/shared.widget#footer",
        "res://ui/theme.style",
        "res://ui/theme.style#dark",
    ];

    let normalized = normalize_ui_asset_change_set(changed);

    assert_eq!(
        normalized,
        BTreeSet::from([
            "res://ui/shared.widget".to_owned(),
            "res://ui/theme.style".to_owned(),
        ])
    );
}

#[test]
fn optimization_batch_dw_asset_change_dedup_checks_before_allocating() {
    let production = include_str!("../normalize.rs")
        .split("#[cfg(test)]")
        .next()
        .expect("asset change normalization production source");

    assert!(production.contains("if normalized.contains(asset_id)"));
    assert!(production.contains("normalized.insert(asset_id.to_owned())"));
    assert!(!production.contains(".map(|asset_id|"));
}

#[test]
#[ignore = "release-only alternating p95 performance gate"]
fn optimization_batch_dw_borrowed_asset_change_dedup_p95() {
    const SAMPLE_PAIRS: usize = 17;
    const NORMALIZATIONS_PER_SAMPLE: usize = 256;
    const EVENT_COUNT: usize = 2_048;
    const UNIQUE_ASSET_COUNT: usize = 32;

    let changed_asset_ids = (0..EVENT_COUNT)
        .map(|index| {
            format!(
                "res://ui/{}/asset_{:04}.widget#node_{index:04}",
                "long_asset_segment/".repeat(8),
                index % UNIQUE_ASSET_COUNT
            )
        })
        .collect::<Vec<_>>();
    let mut legacy_samples = Vec::with_capacity(SAMPLE_PAIRS);
    let mut optimized_samples = Vec::with_capacity(SAMPLE_PAIRS);
    for sample_index in 0..SAMPLE_PAIRS {
        if sample_index % 2 == 0 {
            legacy_samples.push(measure_normalization(
                &changed_asset_ids,
                NORMALIZATIONS_PER_SAMPLE,
                false,
            ));
            optimized_samples.push(measure_normalization(
                &changed_asset_ids,
                NORMALIZATIONS_PER_SAMPLE,
                true,
            ));
        } else {
            optimized_samples.push(measure_normalization(
                &changed_asset_ids,
                NORMALIZATIONS_PER_SAMPLE,
                true,
            ));
            legacy_samples.push(measure_normalization(
                &changed_asset_ids,
                NORMALIZATIONS_PER_SAMPLE,
                false,
            ));
        }
    }

    let legacy_p95 = p95(&mut legacy_samples);
    let optimized_p95 = p95(&mut optimized_samples);
    println!(
        "EDITOR359_BORROWED_UI_ASSET_CHANGE_DEDUP_BENCH_V1 normalizations_per_sample={NORMALIZATIONS_PER_SAMPLE} event_count={EVENT_COUNT} unique_asset_count={UNIQUE_ASSET_COUNT} legacy_p95_ns={legacy_p95} optimized_p95_ns={optimized_p95} ratio={:.4}",
        optimized_p95 as f64 / legacy_p95.max(1) as f64
    );
    assert!(
        optimized_p95.saturating_mul(100) <= legacy_p95.saturating_mul(70),
        "borrowed asset change dedup p95 {optimized_p95}ns exceeded 70% of legacy {legacy_p95}ns"
    );
}

#[test]
#[ignore = "release-only unique and mixed p95 non-regression gates"]
fn editor359_asset_change_unique_and_mixed_benchmark() {
    const EVENT_COUNT: usize = 128;
    let unique = (0..EVENT_COUNT)
        .map(|index| format!("res://ui/asset_{index:04}.widget#node_{index:04}"))
        .collect::<Vec<_>>();
    let mixed = (0..EVENT_COUNT)
        .map(|index| {
            format!(
                "res://ui/asset_{:04}.widget#node_{index:04}",
                index % (EVENT_COUNT / 2)
            )
        })
        .collect::<Vec<_>>();
    assert_eq!(
        normalize_ui_asset_change_set(unique.iter()).len(),
        EVENT_COUNT
    );
    assert_eq!(
        normalize_ui_asset_change_set(mixed.iter()).len(),
        EVENT_COUNT / 2
    );
    let unique_p95 = benchmark_asset_change_workload("all_unique", &unique);
    let mixed_p95 = benchmark_asset_change_workload("mixed_50_percent_unique", &mixed);
    for (workload, (legacy_p95, optimized_p95)) in [
        ("all_unique", unique_p95),
        ("mixed_50_percent_unique", mixed_p95),
    ] {
        assert!(
            optimized_p95.saturating_mul(100) <= legacy_p95.saturating_mul(110),
            "{workload} borrowed asset ID p95 {optimized_p95}ns exceeded 110% of legacy {legacy_p95}ns"
        );
    }
}

fn benchmark_asset_change_workload(workload: &str, asset_ids: &[String]) -> (u128, u128) {
    const NORMALIZATIONS_PER_SAMPLE: usize = 128;
    const SAMPLE_PAIRS: usize = 17;
    let baseline = asset_ids
        .iter()
        .map(|asset_id| normalize_ui_asset_asset_id(asset_id).to_string())
        .collect::<BTreeSet<_>>();
    assert_eq!(normalize_ui_asset_change_set(asset_ids.iter()), baseline);
    for _ in 0..2 {
        black_box(measure_normalization(
            asset_ids,
            NORMALIZATIONS_PER_SAMPLE,
            false,
        ));
        black_box(measure_normalization(
            asset_ids,
            NORMALIZATIONS_PER_SAMPLE,
            true,
        ));
    }
    let mut legacy_samples = Vec::with_capacity(SAMPLE_PAIRS);
    let mut optimized_samples = Vec::with_capacity(SAMPLE_PAIRS);
    for pair in 0..SAMPLE_PAIRS {
        if pair % 2 == 0 {
            legacy_samples.push(measure_normalization(
                asset_ids,
                NORMALIZATIONS_PER_SAMPLE,
                false,
            ));
            optimized_samples.push(measure_normalization(
                asset_ids,
                NORMALIZATIONS_PER_SAMPLE,
                true,
            ));
        } else {
            optimized_samples.push(measure_normalization(
                asset_ids,
                NORMALIZATIONS_PER_SAMPLE,
                true,
            ));
            legacy_samples.push(measure_normalization(
                asset_ids,
                NORMALIZATIONS_PER_SAMPLE,
                false,
            ));
        }
    }
    let legacy_p95 = percentile(&legacy_samples, 95);
    let optimized_p95 = percentile(&optimized_samples, 95);
    println!(
        "PERF_RESULT EDITOR359_UI_ASSET_CHANGE_UNIQUE_MIXED_BENCH_V1 workload={workload} event_count={} normalizations_per_sample={NORMALIZATIONS_PER_SAMPLE} sample_pairs={SAMPLE_PAIRS} legacy_p50_ns={} legacy_p95_ns={legacy_p95} legacy_p99_ns={} optimized_p50_ns={} optimized_p95_ns={optimized_p95} optimized_p99_ns={} threshold_p95_ratio=1.10",
        asset_ids.len(),
        percentile(&legacy_samples, 50),
        percentile(&legacy_samples, 99),
        percentile(&optimized_samples, 50),
        percentile(&optimized_samples, 99),
    );
    (legacy_p95, optimized_p95)
}

fn measure_normalization(
    changed_asset_ids: &[String],
    normalization_count: usize,
    optimized: bool,
) -> u128 {
    let started_at = Instant::now();
    let mut checksum = 0_usize;
    for _ in 0..normalization_count {
        let normalized = if optimized {
            normalize_ui_asset_change_set(changed_asset_ids.iter())
        } else {
            changed_asset_ids
                .iter()
                .map(|asset_id| normalize_ui_asset_asset_id(asset_id).to_string())
                .collect::<BTreeSet<_>>()
        };
        checksum = checksum.wrapping_add(normalized.len());
        black_box(normalized);
    }
    black_box(checksum);
    started_at.elapsed().as_nanos()
}

fn p95(samples: &mut [u128]) -> u128 {
    samples.sort_unstable();
    samples[(samples.len() * 95).div_ceil(100).saturating_sub(1)]
}

fn percentile(samples: &[u128], percentile: usize) -> u128 {
    let mut sorted = samples.to_vec();
    sorted.sort_unstable();
    sorted[(sorted.len() * percentile).div_ceil(100).saturating_sub(1)]
}
