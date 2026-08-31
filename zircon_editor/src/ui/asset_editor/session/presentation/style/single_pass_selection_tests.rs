use std::collections::BTreeMap;
use std::hint::black_box;
use std::time::Instant;

use super::*;
use crate::ui::asset_editor::session::style_inspection::local_style_token_entries;
use toml::Value;
use zircon_runtime_interface::ui::template::{
    UiAssetDocument, UiAssetHeader, UiAssetImports, UiAssetKind,
};

const SAMPLE_PAIRS: usize = 17;

#[test]
fn optimization_batch_20260826ar_style_token_presentation_preserves_order_and_selection() {
    let document = token_document([
        ("accent", Value::String("red".to_string())),
        ("gap", Value::Integer(12)),
        ("surface", Value::String("white".to_string())),
    ]);

    let presentation = build_local_style_token_presentation(&document, Some("gap"));

    assert_eq!(
        presentation.items,
        vec![
            "accent = \"red\"".to_string(),
            "gap = 12".to_string(),
            "surface = \"white\"".to_string(),
        ]
    );
    assert_eq!(presentation.selected_index, 1);
    assert_eq!(presentation.selected_name, "gap");
    assert_eq!(presentation.selected_value, "12");

    let missing = build_local_style_token_presentation(&document, Some("missing"));
    assert_eq!(missing.selected_index, -1);
    assert!(missing.selected_name.is_empty());
    assert!(missing.selected_value.is_empty());

    let declarations = [("self.width", "10"), ("self.width", "20")];
    let (items, selected) = collect_items_and_selection(
        &declarations,
        |(path, _)| *path == "self.width",
        |(path, value)| format!("{path} = {value}"),
    );
    assert_eq!(
        items,
        vec!["self.width = 10".to_string(), "self.width = 20".to_string(),]
    );
    assert_eq!(selected.map(|(index, _)| index), Some(0));
}

#[test]
fn optimization_batch_20260826ar_style_presentation_avoids_token_entry_projection() {
    let presentation_source = include_str!("../style.rs");
    let style_pane = bounded_source(
        presentation_source,
        "pub(super) fn style_pane_presentation",
        "fn collect_items_and_selection",
    );
    let token_builder = bounded_source(
        presentation_source,
        "fn build_local_style_token_presentation",
        "impl UiAssetEditorSession",
    );

    assert!(style_pane.contains("build_local_style_token_presentation"));
    assert!(!style_pane.contains("local_style_token_entries"));
    assert!(!style_pane.contains("let selected_style_token ="));
    assert!(!style_pane.contains(".position("));
    assert_eq!(
        token_builder.matches("for (index, (name, value))").count(),
        1
    );
    assert!(!token_builder.contains(".position("));
    assert!(!token_builder.contains("LocalStyleTokenEntry"));
}

#[test]
#[ignore = "release performance evidence"]
fn optimization_batch_20260826ar_style_token_single_pass_p95() {
    const TOKENS: usize = 4_096;
    const BUILDS: usize = 64;
    let tokens = (0..TOKENS)
        .map(|index| {
            (
                format!("token_{index:05}"),
                Value::String(format!("{}-{index:05}", "value".repeat(12))),
            )
        })
        .collect::<Vec<_>>();
    let document = token_document(tokens);
    let selected = format!("token_{:05}", TOKENS - 1);

    let mut legacy_ns = Vec::with_capacity(SAMPLE_PAIRS);
    let mut optimized_ns = Vec::with_capacity(SAMPLE_PAIRS);
    for sample_index in 0..SAMPLE_PAIRS {
        if sample_index % 2 == 0 {
            legacy_ns.push(measure_ns(BUILDS, || {
                legacy_token_checksum(&document, &selected)
            }));
            optimized_ns.push(measure_ns(BUILDS, || {
                optimized_token_checksum(&document, &selected)
            }));
        } else {
            optimized_ns.push(measure_ns(BUILDS, || {
                optimized_token_checksum(&document, &selected)
            }));
            legacy_ns.push(measure_ns(BUILDS, || {
                legacy_token_checksum(&document, &selected)
            }));
        }
    }

    let legacy_p50_ns = nearest_rank(&legacy_ns, 50);
    let legacy_p95_ns = nearest_rank(&legacy_ns, 95);
    let optimized_p50_ns = nearest_rank(&optimized_ns, 50);
    let optimized_p95_ns = nearest_rank(&optimized_ns, 95);
    assert!(
        optimized_p95_ns.saturating_mul(5) <= legacy_p95_ns.saturating_mul(4),
        "direct token presentation P95 must be at least 20% below entry projection plus selection: legacy={legacy_p95_ns}ns optimized={optimized_p95_ns}ns"
    );

    println!(
        "EDITOR23_STYLE_TOKEN_SINGLE_PASS_BENCH_V1 tokens={TOKENS} builds_per_sample={BUILDS} sample_pairs={SAMPLE_PAIRS} pair_order=alternating_legacy_even legacy_intermediate_entries_per_sample={} optimized_intermediate_entries_per_sample=0 legacy_token_visits_per_sample={} optimized_token_visits_per_sample={} legacy_p50_ns={legacy_p50_ns} legacy_p95_ns={legacy_p95_ns} optimized_p50_ns={optimized_p50_ns} optimized_p95_ns={optimized_p95_ns} legacy_ns={} optimized_ns={}",
        TOKENS * BUILDS,
        TOKENS * BUILDS * 3,
        TOKENS * BUILDS,
        join_samples(&legacy_ns),
        join_samples(&optimized_ns),
    );
}

fn optimized_token_checksum(document: &UiAssetDocument, selected: &str) -> usize {
    let presentation = build_local_style_token_presentation(document, Some(selected));
    presentation.items.len()
        + presentation.selected_index.max(0) as usize
        + presentation.selected_name.len()
        + presentation.selected_value.len()
}

fn legacy_token_checksum(document: &UiAssetDocument, selected: &str) -> usize {
    let entries = local_style_token_entries(document);
    let selected_entry = entries
        .iter()
        .position(|entry| entry.name.as_str() == selected)
        .and_then(|index| entries.get(index).map(|entry| (index, entry)));
    let items = entries
        .iter()
        .map(|entry| format!("{} = {}", entry.name, entry.literal))
        .collect::<Vec<_>>();
    items.len()
        + selected_entry.map(|(index, _)| index).unwrap_or_default()
        + selected_entry
            .map(|(_, entry)| entry.name.len())
            .unwrap_or_default()
        + selected_entry
            .map(|(_, entry)| entry.literal.len())
            .unwrap_or_default()
}

fn token_document<I, S>(tokens: I) -> UiAssetDocument
where
    I: IntoIterator<Item = (S, Value)>,
    S: Into<String>,
{
    UiAssetDocument {
        asset: UiAssetHeader {
            kind: UiAssetKind::Style,
            id: "style.single-pass-test".to_string(),
            version: 1,
            display_name: String::new(),
        },
        imports: UiAssetImports::default(),
        tokens: tokens
            .into_iter()
            .map(|(name, value)| (name.into(), value))
            .collect::<BTreeMap<_, _>>(),
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
