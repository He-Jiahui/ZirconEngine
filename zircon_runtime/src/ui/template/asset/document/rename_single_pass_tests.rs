use std::collections::BTreeMap;
use std::hint::black_box;
use std::time::Instant;

use super::*;
use zircon_runtime_interface::ui::template::{
    UiAssetHeader, UiAssetImports, UiAssetKind, UiStyleDeclarationBlock,
};

const SAMPLE_PAIRS: usize = 17;

#[test]
fn optimization_batch_20260826as_ui_asset_rename_preserves_results_and_duplicates() {
    let mut document = style_document(vec![
        stylesheet("base", ["rule.base", "rule.shared"]),
        stylesheet("overlay", ["rule.overlay"]),
    ]);

    assert!(document
        .rename_style_rule("rule.overlay", "rule.renamed")
        .expect("rename rule"));
    assert!(document.style_rule("rule.overlay").is_none());
    assert!(document.style_rule("rule.renamed").is_some());
    assert!(!document
        .rename_style_rule("rule.missing", "rule.new")
        .expect("missing rule"));
    assert!(matches!(
        document.rename_style_rule("rule.renamed", "rule.shared"),
        Err(UiAssetError::InvalidDocument { .. })
    ));
    assert!(document
        .rename_style_rule("rule.renamed", "rule.renamed")
        .expect("same rule id"));

    assert!(document
        .rename_style_sheet("overlay", "floating")
        .expect("rename stylesheet"));
    assert!(document.style_sheet("overlay").is_none());
    assert!(document.style_sheet("floating").is_some());
    assert!(!document
        .rename_style_sheet("missing", "new-sheet")
        .expect("missing stylesheet"));
    assert!(matches!(
        document.rename_style_sheet("floating", "base"),
        Err(UiAssetError::InvalidDocument { .. })
    ));
    assert!(document
        .rename_style_sheet("floating", "floating")
        .expect("same stylesheet id"));
}

#[test]
fn optimization_batch_20260826as_ui_asset_rename_uses_single_scan() {
    let source = include_str!("../document.rs");
    let implementation = source
        .split("impl UiAssetDocumentRuntimeExt for UiAssetDocument")
        .nth(1)
        .expect("runtime extension implementation");
    let rename_rule = bounded_source(
        implementation,
        "fn rename_style_rule(",
        "fn remove_style_rule(",
    );
    let rename_sheet = bounded_source(
        implementation,
        "fn rename_style_sheet(",
        "fn remove_style_sheet(",
    );
    let rule_scan = bounded_source(
        source,
        "fn scan_style_rule_rename(",
        "fn scan_style_sheet_rename(",
    );
    let sheet_scan = bounded_source(
        source,
        "fn scan_style_sheet_rename(",
        "pub struct UiAssetNodeIter",
    );

    assert!(rename_rule.contains("scan_style_rule_rename"));
    assert!(rename_sheet.contains("scan_style_sheet_rename"));
    assert!(!rename_rule.contains(".position("));
    assert!(!rename_sheet.contains(".position("));
    assert!(!rule_scan.contains(".position("));
    assert_eq!(
        rule_scan
            .matches("for (stylesheet_index, stylesheet)")
            .count(),
        1
    );
    assert_eq!(sheet_scan.matches("for (index, stylesheet)").count(), 1);
}

#[test]
#[ignore = "release performance evidence"]
fn optimization_batch_20260826as_ui_asset_rule_rename_single_scan_p95() {
    const STYLE_SHEETS: usize = 16;
    const RULES_PER_SHEET: usize = 1_024;
    const SCANS: usize = 128;
    let stylesheets = (0..STYLE_SHEETS)
        .map(|sheet_index| UiStyleSheet {
            id: format!("sheet.{sheet_index:02}"),
            rules: (0..RULES_PER_SHEET)
                .map(|rule_index| style_rule(&format!("rule.{sheet_index:02}.{rule_index:04}")))
                .collect(),
        })
        .collect::<Vec<_>>();
    let current_id = format!("rule.{:02}.{:04}", STYLE_SHEETS - 1, RULES_PER_SHEET - 1);
    let new_id = current_id.as_str();

    let mut legacy_ns = Vec::with_capacity(SAMPLE_PAIRS);
    let mut optimized_ns = Vec::with_capacity(SAMPLE_PAIRS);
    for sample_index in 0..SAMPLE_PAIRS {
        if sample_index % 2 == 0 {
            legacy_ns.push(measure_ns(SCANS, || {
                legacy_rule_scan_checksum(&stylesheets, &current_id, new_id)
            }));
            optimized_ns.push(measure_ns(SCANS, || {
                optimized_rule_scan_checksum(&stylesheets, &current_id, new_id)
            }));
        } else {
            optimized_ns.push(measure_ns(SCANS, || {
                optimized_rule_scan_checksum(&stylesheets, &current_id, new_id)
            }));
            legacy_ns.push(measure_ns(SCANS, || {
                legacy_rule_scan_checksum(&stylesheets, &current_id, new_id)
            }));
        }
    }

    let legacy_p50_ns = nearest_rank(&legacy_ns, 50);
    let legacy_p95_ns = nearest_rank(&legacy_ns, 95);
    let optimized_p50_ns = nearest_rank(&optimized_ns, 50);
    let optimized_p95_ns = nearest_rank(&optimized_ns, 95);
    assert!(
        optimized_p95_ns.saturating_mul(5) <= legacy_p95_ns.saturating_mul(4),
        "single-pass rule rename P95 must be at least 20% below locate-plus-duplicate scans: legacy={legacy_p95_ns}ns optimized={optimized_p95_ns}ns"
    );

    let rules = STYLE_SHEETS * RULES_PER_SHEET;
    println!(
        "RUNTIME03_UI_ASSET_RENAME_SINGLE_PASS_BENCH_V1 stylesheets={STYLE_SHEETS} rules_per_stylesheet={RULES_PER_SHEET} scans_per_sample={SCANS} sample_pairs={SAMPLE_PAIRS} pair_order=alternating_legacy_even legacy_rule_visits_per_sample={} optimized_rule_visits_per_sample={} legacy_p50_ns={legacy_p50_ns} legacy_p95_ns={legacy_p95_ns} optimized_p50_ns={optimized_p50_ns} optimized_p95_ns={optimized_p95_ns} legacy_ns={} optimized_ns={}",
        rules * SCANS * 2,
        rules * SCANS,
        join_samples(&legacy_ns),
        join_samples(&optimized_ns),
    );
}

fn optimized_rule_scan_checksum(
    stylesheets: &[UiStyleSheet],
    current_id: &str,
    new_id: &str,
) -> usize {
    let scan = scan_style_rule_rename(stylesheets, current_id, new_id);
    scan.current
        .map(|(stylesheet_index, rule_index)| stylesheet_index + rule_index)
        .unwrap_or_default()
        + usize::from(scan.duplicate)
}

fn legacy_rule_scan_checksum(
    stylesheets: &[UiStyleSheet],
    current_id: &str,
    new_id: &str,
) -> usize {
    let current =
        black_box(stylesheets)
            .iter()
            .enumerate()
            .find_map(|(stylesheet_index, stylesheet)| {
                stylesheet
                    .rules
                    .iter()
                    .position(|rule| rule.id.as_deref() == Some(black_box(current_id)))
                    .map(|rule_index| (stylesheet_index, rule_index))
            });
    let duplicate = stylesheets
        .iter()
        .enumerate()
        .flat_map(|(stylesheet_index, stylesheet)| {
            stylesheet
                .rules
                .iter()
                .enumerate()
                .map(move |(rule_index, rule)| (stylesheet_index, rule_index, rule))
        })
        .any(|(stylesheet_index, rule_index, rule)| {
            Some((stylesheet_index, rule_index)) != current
                && rule.id.as_deref() == Some(black_box(new_id))
        });
    current
        .map(|(stylesheet_index, rule_index)| stylesheet_index + rule_index)
        .unwrap_or_default()
        + usize::from(duplicate)
}

fn style_document(stylesheets: Vec<UiStyleSheet>) -> UiAssetDocument {
    UiAssetDocument {
        asset: UiAssetHeader {
            kind: UiAssetKind::Style,
            id: "style.rename-test".to_string(),
            version: 1,
            display_name: String::new(),
        },
        imports: UiAssetImports::default(),
        tokens: BTreeMap::new(),
        root: None,
        components: BTreeMap::new(),
        stylesheets,
    }
}

fn stylesheet<const N: usize>(id: &str, rule_ids: [&str; N]) -> UiStyleSheet {
    UiStyleSheet {
        id: id.to_string(),
        rules: rule_ids.into_iter().map(style_rule).collect(),
    }
}

fn style_rule(id: &str) -> UiStyleRule {
    UiStyleRule {
        id: Some(id.to_string()),
        selector: "Button".to_string(),
        set: UiStyleDeclarationBlock::default(),
    }
}

fn measure_ns(iterations: usize, mut operation: impl FnMut() -> usize) -> u128 {
    let started = Instant::now();
    let mut checksum = 0usize;
    for _ in 0..iterations {
        checksum = checksum.wrapping_add(black_box(operation()));
    }
    black_box(checksum);
    started.elapsed().as_nanos()
}

fn bounded_source<'a>(source: &'a str, start: &str, end: &str) -> &'a str {
    source
        .split(start)
        .nth(1)
        .expect("source start")
        .split(end)
        .next()
        .expect("source end")
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
