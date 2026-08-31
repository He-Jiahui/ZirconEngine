use std::{hint::black_box, time::Instant};

use toml::Value;
use zircon_runtime_interface::ui::template::{
    UiAssetDocument, UiStyleDeclarationBlock, UiStyleRule, UiStyleSheet,
};

use super::{
    declaration_entries, local_style_rule_entries, selected_style_rule_declaration_entries,
    UiStyleRuleDeclarationEntry,
};

const SAMPLE_PAIRS: usize = 17;

#[test]
fn optimization_batch_20260826am_selected_style_rule_borrowed_lookup_preserves_flat_index() {
    let document = UiAssetDocument {
        stylesheets: vec![
            stylesheet("first", [rule("first-a", "red"), rule("first-b", "green")]),
            stylesheet("second", [rule("second-a", "blue")]),
        ],
        ..UiAssetDocument::default()
    };

    let entries = selected_style_rule_declaration_entries(&document, Some(2));
    assert_eq!(entries.len(), 1);
    assert_eq!(entries[0].path, "self.color");
    assert_eq!(entries[0].literal, "\"blue\"");
    assert!(selected_style_rule_declaration_entries(&document, Some(3)).is_empty());
    assert!(selected_style_rule_declaration_entries(&document, None).is_empty());
}

#[test]
fn optimization_batch_20260826am_selected_style_rule_uses_borrowed_nth_lookup() {
    let source = include_str!("../style_inspection.rs");
    let selected_lookup = source
        .split("pub(super) fn selected_style_rule_declaration_entries")
        .nth(1)
        .expect("selected rule lookup")
        .split("pub(super) fn matched_style_rule_entries_for_selection")
        .next()
        .expect("bounded selected rule lookup");

    assert!(selected_lookup.contains(".flat_map(|stylesheet| stylesheet.rules.iter())"));
    assert!(selected_lookup.contains(".nth(index)"));
    assert!(!selected_lookup.contains("local_style_rule_entries(document)"));
    assert!(!selected_lookup.contains(".cloned()"));
}

#[test]
#[ignore = "release performance evidence"]
fn optimization_batch_20260826am_selected_style_rule_borrowed_lookup_p95() {
    const RULES: usize = 16_384;
    const TEXT_BYTES: usize = 128;
    let suffix = "x".repeat(TEXT_BYTES);
    let mut rules = (0..RULES)
        .map(|index| UiStyleRule {
            id: Some(format!("rule-{index}-{suffix}")),
            selector: format!(".rule-{index}-{suffix}"),
            set: UiStyleDeclarationBlock::default(),
        })
        .collect::<Vec<_>>();
    rules[RULES - 1]
        .set
        .self_values
        .insert("color".to_string(), Value::String("blue".to_string()));
    let document = UiAssetDocument {
        stylesheets: vec![UiStyleSheet {
            id: "benchmark".to_string(),
            rules,
        }],
        ..UiAssetDocument::default()
    };

    let mut legacy_ns = Vec::with_capacity(SAMPLE_PAIRS);
    let mut optimized_ns = Vec::with_capacity(SAMPLE_PAIRS);
    for sample_index in 0..SAMPLE_PAIRS {
        if sample_index % 2 == 0 {
            legacy_ns.push(measure_ns(|| legacy_selected_entries(&document, RULES - 1)));
            optimized_ns.push(measure_ns(|| {
                selected_style_rule_declaration_entries(&document, Some(RULES - 1))
            }));
        } else {
            optimized_ns.push(measure_ns(|| {
                selected_style_rule_declaration_entries(&document, Some(RULES - 1))
            }));
            legacy_ns.push(measure_ns(|| legacy_selected_entries(&document, RULES - 1)));
        }
    }

    let legacy_p50_ns = nearest_rank(&legacy_ns, 50);
    let legacy_p95_ns = nearest_rank(&legacy_ns, 95);
    let optimized_p50_ns = nearest_rank(&optimized_ns, 50);
    let optimized_p95_ns = nearest_rank(&optimized_ns, 95);
    assert!(
        optimized_p95_ns.saturating_mul(5) <= legacy_p95_ns,
        "borrowed selected-rule lookup P95 must be at least 80% below full metadata projection: legacy={legacy_p95_ns}ns optimized={optimized_p95_ns}ns"
    );

    println!(
        "EDITOR23_SELECTED_STYLE_RULE_BORROWED_LOOKUP_BENCH_V1 rules={RULES} cloned_text_bytes_per_rule={} sample_pairs={SAMPLE_PAIRS} pair_order=alternating_legacy_even legacy_first_pairs=9 optimized_first_pairs=8 legacy_rule_metadata_clones={RULES} optimized_rule_metadata_clones=0 legacy_p50_ns={legacy_p50_ns} legacy_p95_ns={legacy_p95_ns} optimized_p50_ns={optimized_p50_ns} optimized_p95_ns={optimized_p95_ns} legacy_ns={} optimized_ns={}",
        TEXT_BYTES * 2,
        join_samples(&legacy_ns),
        join_samples(&optimized_ns),
    );
}

fn stylesheet<const N: usize>(id: &str, rules: [UiStyleRule; N]) -> UiStyleSheet {
    UiStyleSheet {
        id: id.to_string(),
        rules: rules.into_iter().collect(),
    }
}

fn rule(id: &str, color: &str) -> UiStyleRule {
    let mut set = UiStyleDeclarationBlock::default();
    set.self_values
        .insert("color".to_string(), Value::String(color.to_string()));
    UiStyleRule {
        id: Some(id.to_string()),
        selector: format!(".{id}"),
        set,
    }
}

fn legacy_selected_entries(
    document: &UiAssetDocument,
    selected_rule_index: usize,
) -> Vec<UiStyleRuleDeclarationEntry> {
    local_style_rule_entries(document)
        .get(selected_rule_index)
        .cloned()
        .map(|entry| {
            declaration_entries(
                &document.stylesheets[entry.stylesheet_index].rules[entry.rule_index].set,
            )
        })
        .unwrap_or_default()
}

fn measure_ns(operation: impl FnOnce() -> Vec<UiStyleRuleDeclarationEntry>) -> u128 {
    let started = Instant::now();
    let entries = black_box(operation)();
    let elapsed = started.elapsed().as_nanos();
    assert_eq!(black_box(entries.len()), 1);
    assert_eq!(entries[0].literal, "\"blue\"");
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
