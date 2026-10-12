use std::hint::black_box;
use std::time::Instant;

use super::*;
use zircon_runtime_interface::ui::template::{
    UiAssetHeader, UiAssetImports, UiAssetKind, UI_ASSET_CURRENT_SOURCE_SCHEMA_VERSION,
};

const SAMPLE_PAIRS: usize = 17;

#[test]
fn optimization_batch_20260826as_theme_refactor_labels_preserve_imported_values() {
    let (document, imported_styles) = duplicate_token_fixture(3);

    let items = build_theme_refactor_items(&document, &imported_styles);

    assert!(items.starts_with(&[
        "duplicate local token • token.0000 • inherited = \"value.0000\"".to_string(),
        "duplicate local token • token.0001 • inherited = \"value.0001\"".to_string(),
        "duplicate local token • token.0002 • inherited = \"value.0002\"".to_string(),
    ]));
}

#[test]
fn optimization_batch_20260826as_theme_refactor_labels_reuse_imported_token_map() {
    let source = include_str!("../theme_authoring.rs");
    let label = bounded_source(
        source,
        "impl UiAssetThemeRefactorAction",
        "pub(crate) fn detach_imported_theme_to_local_theme_layer",
    );
    let build = bounded_source(
        source,
        "pub(crate) fn build_theme_refactor_items",
        "pub(crate) fn can_prune_duplicate_local_theme_overrides",
    );
    let projection = bounded_source(
        source,
        "fn theme_refactor_projection",
        "pub(crate) fn apply_theme_refactor_action",
    );

    assert!(!label.contains("imported_theme_tokens("));
    assert!(label.contains("imported_tokens"));
    assert!(label.contains(".get(token_name)"));
    assert!(build.contains("theme_refactor_projection"));
    assert!(build.contains("action.label(&imported_tokens)"));
    assert_eq!(projection.matches("imported_theme_tokens(").count(), 1);
}

#[test]
#[ignore = "release performance evidence"]
fn optimization_batch_20260826as_theme_refactor_label_cache_p95() {
    const TOKENS: usize = 1_024;
    const BUILDS: usize = 1;
    let (document, imported_styles) = duplicate_token_fixture(TOKENS);

    let mut legacy_ns = Vec::with_capacity(SAMPLE_PAIRS);
    let mut optimized_ns = Vec::with_capacity(SAMPLE_PAIRS);
    for sample_index in 0..SAMPLE_PAIRS {
        if sample_index % 2 == 0 {
            legacy_ns.push(measure_ns(BUILDS, || {
                legacy_refactor_checksum(&document, &imported_styles)
            }));
            optimized_ns.push(measure_ns(BUILDS, || {
                optimized_refactor_checksum(&document, &imported_styles)
            }));
        } else {
            optimized_ns.push(measure_ns(BUILDS, || {
                optimized_refactor_checksum(&document, &imported_styles)
            }));
            legacy_ns.push(measure_ns(BUILDS, || {
                legacy_refactor_checksum(&document, &imported_styles)
            }));
        }
    }

    let legacy_p50_ns = nearest_rank(&legacy_ns, 50);
    let legacy_p95_ns = nearest_rank(&legacy_ns, 95);
    let optimized_p50_ns = nearest_rank(&optimized_ns, 50);
    let optimized_p95_ns = nearest_rank(&optimized_ns, 95);
    assert!(
        optimized_p95_ns.saturating_mul(2) <= legacy_p95_ns,
        "shared imported-token labels P95 must be at least 50% below per-action map rebuilds: legacy={legacy_p95_ns}ns optimized={optimized_p95_ns}ns"
    );

    println!(
        "EDITOR23_THEME_REFACTOR_LABEL_CACHE_BENCH_V1 duplicate_tokens={TOKENS} builds_per_sample={BUILDS} sample_pairs={SAMPLE_PAIRS} pair_order=alternating_legacy_even legacy_label_map_builds_per_sample={} optimized_label_map_builds_per_sample=0 legacy_label_map_entries_per_sample={} optimized_label_map_entries_per_sample=0 legacy_p50_ns={legacy_p50_ns} legacy_p95_ns={legacy_p95_ns} optimized_p50_ns={optimized_p50_ns} optimized_p95_ns={optimized_p95_ns} legacy_ns={} optimized_ns={}",
        TOKENS * BUILDS,
        TOKENS * TOKENS * BUILDS,
        join_samples(&legacy_ns),
        join_samples(&optimized_ns),
    );
}

fn optimized_refactor_checksum(
    document: &UiAssetDocument,
    imported_styles: &BTreeMap<String, UiAssetDocument>,
) -> usize {
    build_theme_refactor_items(document, imported_styles)
        .iter()
        .map(String::len)
        .sum()
}

fn legacy_refactor_checksum(
    document: &UiAssetDocument,
    imported_styles: &BTreeMap<String, UiAssetDocument>,
) -> usize {
    theme_refactor_actions(document, imported_styles)
        .into_iter()
        .map(|action| match action {
            UiAssetThemeRefactorAction::RemoveDuplicateLocalToken { token_name } => format!(
                "duplicate local token • {token_name} • inherited = {}",
                imported_theme_tokens(document, imported_styles)
                    .get(&token_name)
                    .map(Value::to_string)
                    .unwrap_or_default()
            ),
            UiAssetThemeRefactorAction::RemoveDuplicateLocalRule {
                stylesheet_id,
                selector,
            } => format!("duplicate local rule • {stylesheet_id} • {selector}"),
            UiAssetThemeRefactorAction::RemoveRedundantImportedThemeReference { reference } => {
                format!("redundant imported theme • {reference}")
            }
        })
        .map(|label| black_box(label).len())
        .sum()
}

fn duplicate_token_fixture(
    token_count: usize,
) -> (UiAssetDocument, BTreeMap<String, UiAssetDocument>) {
    let mut imported = style_document("theme.shared");
    for index in 0..token_count {
        imported.tokens.insert(
            format!("token.{index:04}"),
            Value::String(format!("value.{index:04}")),
        );
    }
    let mut document = style_document("theme.local");
    document.imports.styles.push("theme.shared".to_string());
    document.tokens.clone_from(&imported.tokens);
    let imported_styles = BTreeMap::from([("theme.shared".to_string(), imported)]);
    (document, imported_styles)
}

fn style_document(id: &str) -> UiAssetDocument {
    UiAssetDocument {
        asset: UiAssetHeader {
            kind: UiAssetKind::Style,
            id: id.to_string(),
            version: UI_ASSET_CURRENT_SOURCE_SCHEMA_VERSION,
            display_name: id.to_string(),
        },
        imports: UiAssetImports::default(),
        tokens: BTreeMap::new(),
        root: None,
        components: BTreeMap::new(),
        stylesheets: Vec::new(),
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
