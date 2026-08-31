use std::{collections::BTreeMap, hint::black_box, time::Instant};

use toml::Value;
use zircon_runtime_interface::ui::template::UiAssetDocument;

use super::{
    build_theme_summary, reconcile_selected_theme_source_key, select_theme_source_key,
    theme_source_entries,
};

const SAMPLE_PAIRS: usize = 17;

#[test]
fn optimization_batch_20260826an_theme_source_lightweight_selection_preserves_keys() {
    let mut document = UiAssetDocument::default();
    document.imports.styles = vec![
        "asset://themes/missing.zui".to_string(),
        "asset://themes/loaded.zui".to_string(),
        "asset://themes/loaded.zui".to_string(),
    ];
    let imported_styles = BTreeMap::from([(
        "asset://themes/loaded.zui".to_string(),
        UiAssetDocument::default(),
    )]);

    assert_eq!(
        reconcile_selected_theme_source_key(
            &document,
            &imported_styles,
            Some("asset://themes/loaded.zui")
        ),
        Some("asset://themes/loaded.zui".to_string())
    );
    assert_eq!(
        reconcile_selected_theme_source_key(&document, &imported_styles, Some("unknown")),
        Some("asset://themes/missing.zui".to_string())
    );
    assert_eq!(
        select_theme_source_key(&document, &imported_styles, 2),
        Some("asset://themes/loaded.zui".to_string())
    );
    assert!(select_theme_source_key(&document, &imported_styles, 3).is_none());

    document
        .tokens
        .insert("accent".to_string(), Value::String("blue".to_string()));
    assert_eq!(
        reconcile_selected_theme_source_key(&document, &imported_styles, Some("unknown")),
        Some("local".to_string())
    );
    assert_eq!(
        select_theme_source_key(&document, &imported_styles, 0),
        Some("local".to_string())
    );
    assert_eq!(
        select_theme_source_key(&document, &imported_styles, 1),
        Some("asset://themes/missing.zui".to_string())
    );
    let summary = build_theme_summary(
        &document,
        &imported_styles,
        Some("asset://themes/loaded.zui"),
    );
    assert_eq!(summary.selected_index, 2);
    assert_eq!(summary.selected_reference, "asset://themes/loaded.zui");
    assert_eq!(summary.items.len(), 4);
}

#[test]
fn optimization_batch_20260826an_theme_source_selection_avoids_entry_projection() {
    let source = include_str!("../theme_summary.rs");
    let select = bounded_function(
        source,
        "pub(crate) fn select_theme_source_key",
        "pub(crate) fn reconcile_selected_theme_source_key",
    );
    let reconcile = bounded_function(
        source,
        "pub(crate) fn reconcile_selected_theme_source_key",
        "fn theme_source_entries",
    );
    let summary = bounded_function(
        source,
        "pub(crate) fn build_theme_summary",
        "pub(crate) fn build_theme_source_details",
    );

    assert!(!select.contains("theme_source_entries"));
    assert!(!reconcile.contains("theme_source_entries"));
    assert!(!summary.contains("reconcile_selected_theme_source_key"));
    assert!(reconcile.contains("imported_style_refs"));
    assert!(reconcile.contains(".iter()"));
    assert!(reconcile.contains(".any("));
    assert!(select.contains("imported_style_refs.get("));
}

#[test]
#[ignore = "release performance evidence"]
fn optimization_batch_20260826an_theme_source_lightweight_selection_p95() {
    const IMPORTS: usize = 8_192;
    const TEXT_BYTES: usize = 96;
    let suffix = "x".repeat(TEXT_BYTES);
    let mut document = UiAssetDocument::default();
    document.imports.styles = (0..IMPORTS)
        .map(|index| format!("asset://themes/theme-{index}-{suffix}.zui"))
        .collect();
    let selected = document.imports.styles.last().cloned().unwrap();
    let imported_styles = BTreeMap::new();

    let mut legacy_ns = Vec::with_capacity(SAMPLE_PAIRS);
    let mut optimized_ns = Vec::with_capacity(SAMPLE_PAIRS);
    for sample_index in 0..SAMPLE_PAIRS {
        if sample_index % 2 == 0 {
            legacy_ns.push(measure_ns(|| {
                legacy_reconcile_selected_theme_source_key(
                    &document,
                    &imported_styles,
                    Some(&selected),
                )
            }));
            optimized_ns.push(measure_ns(|| {
                reconcile_selected_theme_source_key(&document, &imported_styles, Some(&selected))
            }));
        } else {
            optimized_ns.push(measure_ns(|| {
                reconcile_selected_theme_source_key(&document, &imported_styles, Some(&selected))
            }));
            legacy_ns.push(measure_ns(|| {
                legacy_reconcile_selected_theme_source_key(
                    &document,
                    &imported_styles,
                    Some(&selected),
                )
            }));
        }
    }

    let legacy_p50_ns = nearest_rank(&legacy_ns, 50);
    let legacy_p95_ns = nearest_rank(&legacy_ns, 95);
    let optimized_p50_ns = nearest_rank(&optimized_ns, 50);
    let optimized_p95_ns = nearest_rank(&optimized_ns, 95);
    assert!(
        optimized_p95_ns.saturating_mul(10) <= legacy_p95_ns,
        "lightweight theme selection P95 must be at least 90% below entry projection: legacy={legacy_p95_ns}ns optimized={optimized_p95_ns}ns"
    );

    println!(
        "EDITOR23_THEME_SOURCE_LIGHTWEIGHT_SELECTION_BENCH_V1 imports={IMPORTS} reference_text_bytes={TEXT_BYTES} sample_pairs={SAMPLE_PAIRS} pair_order=alternating_legacy_even legacy_first_pairs=9 optimized_first_pairs=8 legacy_entry_projections={IMPORTS} optimized_entry_projections=0 legacy_selected_string_clones=1 optimized_selected_string_clones=1 legacy_p50_ns={legacy_p50_ns} legacy_p95_ns={legacy_p95_ns} optimized_p50_ns={optimized_p50_ns} optimized_p95_ns={optimized_p95_ns} legacy_ns={} optimized_ns={}",
        join_samples(&legacy_ns),
        join_samples(&optimized_ns),
    );
}

fn bounded_function<'a>(source: &'a str, start: &str, end: &str) -> &'a str {
    source
        .split(start)
        .nth(1)
        .expect("function start")
        .split(end)
        .next()
        .expect("function end")
}

fn legacy_reconcile_selected_theme_source_key(
    document: &UiAssetDocument,
    imported_styles: &BTreeMap<String, UiAssetDocument>,
    current: Option<&str>,
) -> Option<String> {
    let entries = theme_source_entries(document, imported_styles);
    current
        .and_then(|key| entries.iter().find(|entry| entry.key == key))
        .map(|entry| entry.key.clone())
        .or_else(|| entries.first().map(|entry| entry.key.clone()))
}

fn measure_ns(operation: impl FnOnce() -> Option<String>) -> u128 {
    let started = Instant::now();
    let selected = black_box(operation)();
    let elapsed = started.elapsed().as_nanos();
    assert!(selected.is_some());
    elapsed
}

fn nearest_rank(samples: &[u128], percentile: usize) -> u128 {
    let mut sorted = samples.to_vec();
    sorted.sort_unstable();
    let rank = sorted.len().saturating_mul(percentile).div_ceil(100);
    sorted[rank.saturating_sub(1)]
}

fn join_samples(samples: &[u128]) -> String {
    samples
        .iter()
        .map(u128::to_string)
        .collect::<Vec<_>>()
        .join(",")
}
