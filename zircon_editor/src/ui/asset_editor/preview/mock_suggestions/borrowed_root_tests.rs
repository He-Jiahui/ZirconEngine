use std::hint::black_box;
use std::time::Instant;

use super::*;
use crate::ui::asset_editor::preview::preview_mock::preview_mock_nested_entries;

const SAMPLE_PAIRS: usize = 17;

#[test]
fn optimization_batch_20260826ao_preview_suggestion_borrowed_root_preserves_selection() {
    let entry = object_entry(table([
        (
            "groups",
            Value::Array(vec![
                table([("name", Value::String("first".to_string()))]),
                table([
                    ("name", Value::String("second".to_string())),
                    (
                        "options",
                        table([
                            ("primary", Value::Integer(7)),
                            ("secondary", Value::Integer(9)),
                        ]),
                    ),
                ]),
            ]),
        ),
        (
            "literal.key",
            table([("child", table([("leaf", Value::Boolean(true))]))]),
        ),
    ]));

    let selected = preview_mock_suggestions(&entry, Some("groups[1].options.primary"));
    assert_eq!(
        selected
            .iter()
            .map(|suggestion| suggestion.resolved_key.as_str())
            .collect::<Vec<_>>(),
        vec!["groups[1].options.primary", "groups[1].options.secondary"]
    );
    assert_eq!(selected[0].value, Value::Integer(7));

    let stale = preview_mock_suggestions(&entry, Some("groups[1].options.missing.deep"));
    assert_eq!(stale, selected);

    let dotted = preview_mock_suggestions(&entry, Some("literal.key.child.leaf"));
    assert_eq!(dotted.len(), 1);
    assert_eq!(dotted[0].resolved_key, "literal.key.child.leaf");
    assert_eq!(dotted[0].value, Value::Boolean(true));

    let root = preview_mock_suggestions(&entry, None);
    assert_eq!(
        root.iter()
            .map(|suggestion| suggestion.resolved_key.as_str())
            .collect::<Vec<_>>(),
        vec!["groups", "literal.key"]
    );
    assert!(preview_mock_suggestions(&text_entry(), Some("missing")).is_empty());
}

#[test]
fn optimization_batch_20260826ao_preview_suggestion_root_avoids_nested_tree_projection() {
    let source = include_str!("../mock_suggestions.rs");
    let root = bounded_function(source, "fn suggestion_root", "fn matching_nested_container");
    let matching = bounded_function(
        source,
        "fn matching_nested_container",
        "fn selected_or_descendant_path",
    );

    assert!(!source.contains("preview_mock_nested_entries"));
    assert!(!source.contains("UiAssetPreviewMockNestedEntry"));
    assert!(root.contains("&'a Value"));
    assert!(!root.contains("effective_value.clone()"));
    assert!(matching.contains("selected_or_descendant_path"));
    assert!(matching.contains("Value::Array"));
    assert!(matching.contains("Value::Table"));
}

#[test]
#[ignore = "release performance evidence"]
fn optimization_batch_20260826ao_preview_suggestion_borrowed_root_p95() {
    const BRANCHES: usize = 4_096;
    const PAYLOAD_BYTES: usize = 256;
    let payload = "x".repeat(PAYLOAD_BYTES);
    let mut root = toml::map::Map::new();
    root.insert(
        "000_target".to_string(),
        table([("choice", Value::Integer(7))]),
    );
    for index in 0..BRANCHES {
        root.insert(
            format!("branch_{index:05}"),
            table([("payload", Value::String(payload.clone()))]),
        );
    }
    let entry = object_entry(Value::Table(root));

    let mut legacy_ns = Vec::with_capacity(SAMPLE_PAIRS);
    let mut optimized_ns = Vec::with_capacity(SAMPLE_PAIRS);
    for sample_index in 0..SAMPLE_PAIRS {
        if sample_index % 2 == 0 {
            legacy_ns.push(measure_ns(|| legacy_root_width(&entry, "000_target")));
            optimized_ns.push(measure_ns(|| optimized_root_width(&entry, "000_target")));
        } else {
            optimized_ns.push(measure_ns(|| optimized_root_width(&entry, "000_target")));
            legacy_ns.push(measure_ns(|| legacy_root_width(&entry, "000_target")));
        }
    }

    let legacy_p50_ns = nearest_rank(&legacy_ns, 50);
    let legacy_p95_ns = nearest_rank(&legacy_ns, 95);
    let optimized_p50_ns = nearest_rank(&optimized_ns, 50);
    let optimized_p95_ns = nearest_rank(&optimized_ns, 95);
    assert!(
        optimized_p95_ns.saturating_mul(10) <= legacy_p95_ns,
        "borrowed suggestion root P95 must be at least 90% below full nested projection: legacy={legacy_p95_ns}ns optimized={optimized_p95_ns}ns"
    );

    println!(
        "EDITOR23_PREVIEW_SUGGESTION_BORROWED_ROOT_BENCH_V1 branches={BRANCHES} payload_bytes_per_branch={PAYLOAD_BYTES} sample_pairs={SAMPLE_PAIRS} pair_order=alternating_legacy_even legacy_first_pairs=9 optimized_first_pairs=8 legacy_nested_value_clones={} optimized_nested_value_clones=0 legacy_p50_ns={legacy_p50_ns} legacy_p95_ns={legacy_p95_ns} optimized_p50_ns={optimized_p50_ns} optimized_p95_ns={optimized_p95_ns} legacy_ns={} optimized_ns={}",
        BRANCHES * 2 + 2,
        join_samples(&legacy_ns),
        join_samples(&optimized_ns),
    );
}

fn object_entry(effective_value: Value) -> UiAssetPreviewMockEntry {
    UiAssetPreviewMockEntry {
        key: "data".to_string(),
        display_key: "data".to_string(),
        kind: UiAssetPreviewMockKind::Object,
        effective_value,
        overridden: false,
    }
}

fn text_entry() -> UiAssetPreviewMockEntry {
    UiAssetPreviewMockEntry {
        key: "label".to_string(),
        display_key: "label".to_string(),
        kind: UiAssetPreviewMockKind::Text,
        effective_value: Value::String("plain".to_string()),
        overridden: false,
    }
}

fn table<const N: usize>(entries: [(&str, Value); N]) -> Value {
    Value::Table(
        entries
            .into_iter()
            .map(|(key, value)| (key.to_string(), value))
            .collect(),
    )
}

fn legacy_root_width(entry: &UiAssetPreviewMockEntry, selected: &str) -> usize {
    let nested_entries = preview_mock_nested_entries(&entry.effective_value);
    let selected_key = nested_entries
        .iter()
        .filter(|nested| {
            nested.kind.supports_nested_entries()
                && selected_or_descendant_path(selected, &nested.key)
        })
        .max_by_key(|nested| nested.key.len())
        .map(|nested| nested.key.clone())
        .expect("legacy selected container");
    let selected = nested_entries
        .iter()
        .find(|nested| nested.key == selected_key)
        .expect("legacy selected value");
    selected.value.as_table().map_or(0, toml::map::Map::len)
}

fn optimized_root_width(entry: &UiAssetPreviewMockEntry, selected: &str) -> usize {
    suggestion_root(entry, Some(selected))
        .and_then(|(_, value)| value.as_table())
        .map_or(0, toml::map::Map::len)
}

fn measure_ns(operation: impl FnOnce() -> usize) -> u128 {
    let started = Instant::now();
    assert_eq!(black_box(operation()), 1);
    started.elapsed().as_nanos()
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
