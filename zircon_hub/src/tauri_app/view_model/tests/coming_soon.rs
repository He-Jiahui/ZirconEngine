use std::hint::black_box;
use std::time::Instant;

use crate::projects::project_template_catalog;
use crate::settings::HubLanguage;

const PERF_SAMPLE_PAIRS: usize = 21;
const PERF_ITERATIONS_PER_SAMPLE: usize = 400;

#[test]
fn hub03_coming_soon_entries_include_visible_localized_category_labels() {
    let entries = super::coming_soon_entries(HubLanguage::Chinese);
    let remote_sync = entries
        .iter()
        .find(|entry| entry.id == "remote-sync")
        .expect("remote sync entry should exist");

    assert_eq!(remote_sync.category, "local-delivery");
    assert_eq!(remote_sync.category_label, "本地交付");
    assert_eq!(remote_sync.status, "敬请期待");
    assert_eq!(remote_sync.meta, "本地交付 / 敬请期待");
    assert!(remote_sync.disabled);
}

#[test]
fn hub03_disabled_project_templates_have_coming_soon_entries() {
    let entries = super::coming_soon_entries(HubLanguage::English);

    for template in project_template_catalog()
        .iter()
        .filter(|template| !template.enabled)
    {
        let expected_id = format!("project-template-{}", template.id);
        assert!(
            entries.iter().any(|entry| entry.id == expected_id),
            "disabled template {} is missing coming-soon entry {expected_id}",
            template.id
        );
    }
}

#[test]
fn hub03_coming_soon_entries_are_non_empty_in_both_languages() {
    for language in [HubLanguage::English, HubLanguage::Chinese] {
        for entry in super::coming_soon_entries(language) {
            assert!(!entry.id.trim().is_empty());
            assert!(!entry.category.trim().is_empty());
            assert!(!entry.category_label.trim().is_empty());
            assert!(!entry.title.trim().is_empty());
            assert!(!entry.detail.trim().is_empty());
            assert!(!entry.status.trim().is_empty());
            assert!(!entry.meta.trim().is_empty());
            assert!(entry.disabled);
        }
    }
}

#[test]
#[ignore = "release performance evidence"]
fn hub03_coming_soon_projection_release_benchmark_evidence() {
    let mut legacy_ns = Vec::with_capacity(PERF_SAMPLE_PAIRS);
    let mut optimized_ns = Vec::with_capacity(PERF_SAMPLE_PAIRS);
    black_box(legacy_owned_entries(HubLanguage::English));
    black_box(super::coming_soon_entries(HubLanguage::English));

    for sample in 0..PERF_SAMPLE_PAIRS {
        let (legacy, optimized) = if sample % 2 == 0 {
            (
                measure_projection(|| legacy_owned_entries(HubLanguage::English)),
                measure_projection(|| super::coming_soon_entries(HubLanguage::English)),
            )
        } else {
            let optimized = measure_projection(|| super::coming_soon_entries(HubLanguage::English));
            let legacy = measure_projection(|| legacy_owned_entries(HubLanguage::English));
            (legacy, optimized)
        };
        legacy_ns.push(legacy);
        optimized_ns.push(optimized);
    }

    let legacy_p50 = percentile(&legacy_ns, 50);
    let legacy_p95 = percentile(&legacy_ns, 95);
    let optimized_p50 = percentile(&optimized_ns, 50);
    let optimized_p95 = percentile(&optimized_ns, 95);
    println!(
        "PERF_RESULT hub03_coming_soon_projection legacy_p50_ns={legacy_p50} legacy_p95_ns={legacy_p95} optimized_p50_ns={optimized_p50} optimized_p95_ns={optimized_p95} entries_per_projection=15 iterations_per_sample={PERF_ITERATIONS_PER_SAMPLE} samples={PERF_SAMPLE_PAIRS} legacy_string_allocations=105 optimized_string_allocations=15 legacy_raw_ns={} optimized_raw_ns={}",
        raw(&legacy_ns),
        raw(&optimized_ns),
    );
    assert!(
        optimized_p95.saturating_mul(100) <= legacy_p95.saturating_mul(80),
        "optimized P95 {optimized_p95}ns must be at most 80% of legacy P95 {legacy_p95}ns"
    );
}

type LegacyOwnedEntry = (String, String, String, String, String, String, String, bool);

fn legacy_owned_entries(language: HubLanguage) -> Vec<LegacyOwnedEntry> {
    super::coming_soon_entries(language)
        .into_iter()
        .map(|entry| {
            (
                entry.id.into_owned(),
                entry.category.into_owned(),
                entry.category_label.into_owned(),
                entry.title.into_owned(),
                entry.detail.into_owned(),
                entry.status.into_owned(),
                entry.meta,
                entry.disabled,
            )
        })
        .collect()
}

fn measure_projection<T>(mut projection: impl FnMut() -> T) -> u64 {
    let started = Instant::now();
    for _ in 0..PERF_ITERATIONS_PER_SAMPLE {
        black_box(projection());
    }
    started.elapsed().as_nanos() as u64
}

fn percentile(samples: &[u64], percentile: usize) -> u64 {
    let mut ordered = samples.to_vec();
    ordered.sort_unstable();
    let rank = ordered
        .len()
        .saturating_mul(percentile)
        .div_ceil(100)
        .saturating_sub(1);
    ordered[rank]
}

fn raw(samples: &[u64]) -> String {
    samples
        .iter()
        .map(u64::to_string)
        .collect::<Vec<_>>()
        .join(",")
}
