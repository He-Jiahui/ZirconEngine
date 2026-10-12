use std::{collections::BTreeMap, hint::black_box, time::Instant};

use super::*;
use zircon_runtime_interface::ui::template::{
    UiAssetHeader, UiAssetImports, UiAssetKind, UiStyleRule, UiStyleSheet,
};

const SAMPLE_PAIRS: usize = 17;

#[test]
fn optimization_batch_20260826ax_theme_compare_index_preserves_last_selector_and_output() {
    let imported = document_with_rules(vec![rule(".shared", "red"), rule(".imported", "green")]);
    let local = document_with_stylesheets(vec![
        UiStyleSheet {
            id: "first".to_string(),
            rules: vec![rule(".shared", "orange")],
        },
        UiStyleSheet {
            id: "last".to_string(),
            rules: vec![rule(".shared", "blue")],
        },
    ]);
    let rules = selector_rule_blocks(&local);
    let selected = rules.get(".shared").expect("selected duplicate rule");

    assert_eq!(selected.stylesheet_label, "last");
    assert_eq!(
        selected.block.self_values.get("color"),
        Some(&Value::String("blue".to_string()))
    );

    let items = compare_imported_against_local(&imported, &local);
    assert!(items.iter().any(|item| {
        item.contains("shadowed by local")
            && item.contains("first")
            && item.contains("local = self.color = \"blue\"")
    }));
    assert!(items
        .iter()
        .any(|item| item.contains("imported-only") && item.contains(".imported")));
}

#[test]
fn optimization_batch_20260826ax_theme_compare_uses_borrowed_rule_index() {
    let source = include_str!("../theme_compare.rs");
    let selector = bounded_source(
        source,
        "fn selector_rule_blocks(",
        "fn aggregated_rule_blocks(",
    );
    let aggregate = bounded_source(
        source,
        "fn aggregated_rule_blocks(",
        "fn format_rule_block(",
    );

    assert!(source.contains("struct ThemeRuleReference<'a>"));
    assert!(selector.contains("BTreeMap<&'a str, ThemeRuleReference<'a>>"));
    assert!(selector.contains("rule.selector.as_str()"));
    assert!(selector.contains("block: &rule.set"));
    assert!(!selector.contains("rule.selector.clone()"));
    assert!(!selector.contains("rule.set.clone()"));
    assert!(aggregate.contains("selector_rule_blocks(imported)"));
}

#[test]
#[ignore = "release performance evidence"]
fn optimization_batch_20260826ax_theme_compare_borrowed_rule_index_p95() {
    const RULE_COUNT: usize = 2_048;
    const DECLARATIONS_PER_RULE: usize = 16;
    const BUILDS: usize = 4;
    let document = benchmark_document(RULE_COUNT, DECLARATIONS_PER_RULE);
    let mut legacy_ns = Vec::with_capacity(SAMPLE_PAIRS);
    let mut optimized_ns = Vec::with_capacity(SAMPLE_PAIRS);

    for sample_index in 0..SAMPLE_PAIRS {
        if sample_index % 2 == 0 {
            legacy_ns.push(measure_ns(BUILDS, || {
                legacy_selector_rule_blocks(black_box(&document))
            }));
            optimized_ns.push(measure_ns(BUILDS, || {
                selector_rule_blocks(black_box(&document)).len()
            }));
        } else {
            optimized_ns.push(measure_ns(BUILDS, || {
                selector_rule_blocks(black_box(&document)).len()
            }));
            legacy_ns.push(measure_ns(BUILDS, || {
                legacy_selector_rule_blocks(black_box(&document))
            }));
        }
    }

    let legacy_p50_ns = nearest_rank(&legacy_ns, 50);
    let legacy_p95_ns = nearest_rank(&legacy_ns, 95);
    let optimized_p50_ns = nearest_rank(&optimized_ns, 50);
    let optimized_p95_ns = nearest_rank(&optimized_ns, 95);
    assert!(
        optimized_p95_ns.saturating_mul(2) <= legacy_p95_ns,
        "borrowed theme rule index P95 must be at least 50% below cloned rule indexing: legacy={legacy_p95_ns}ns optimized={optimized_p95_ns}ns"
    );

    println!(
        "EDITOR23_THEME_COMPARE_BORROWED_RULE_INDEX_BENCH_V1 rules={RULE_COUNT} declarations_per_rule={DECLARATIONS_PER_RULE} builds_per_sample={BUILDS} sample_pairs={SAMPLE_PAIRS} pair_order=alternating_legacy_even legacy_selector_clones_per_sample={} legacy_block_clones_per_sample={} optimized_selector_clones_per_sample=0 optimized_block_clones_per_sample=0 legacy_p50_ns={legacy_p50_ns} legacy_p95_ns={legacy_p95_ns} optimized_p50_ns={optimized_p50_ns} optimized_p95_ns={optimized_p95_ns} legacy_ns={} optimized_ns={}",
        RULE_COUNT * BUILDS,
        RULE_COUNT * BUILDS,
        join_samples(&legacy_ns),
        join_samples(&optimized_ns),
    );
}

fn benchmark_document(rule_count: usize, declarations_per_rule: usize) -> UiAssetDocument {
    let rules = (0..rule_count)
        .map(|rule_index| {
            let mut block = UiStyleDeclarationBlock::default();
            for declaration_index in 0..declarations_per_rule {
                block.self_values.insert(
                    format!("property-{declaration_index:02}"),
                    Value::String(format!(
                        "value-{rule_index:04}-{declaration_index:02}-abcdefghijklmnopqrstuvwxyz"
                    )),
                );
            }
            UiStyleRule {
                id: None,
                selector: format!(".component-{rule_index:04}"),
                set: block,
            }
        })
        .collect();
    document_with_rules(rules)
}

fn legacy_selector_rule_blocks(document: &UiAssetDocument) -> usize {
    let mut rules = BTreeMap::<String, (String, UiStyleDeclarationBlock)>::new();
    for stylesheet in &document.stylesheets {
        let stylesheet_label = if stylesheet.id.is_empty() {
            "<inline>"
        } else {
            stylesheet.id.as_str()
        };
        for rule in &stylesheet.rules {
            rules.insert(
                rule.selector.clone(),
                (
                    format!("{stylesheet_label} • {}", rule.selector),
                    rule.set.clone(),
                ),
            );
        }
    }
    black_box(rules).len()
}

fn document_with_rules(rules: Vec<UiStyleRule>) -> UiAssetDocument {
    document_with_stylesheets(vec![UiStyleSheet {
        id: "first".to_string(),
        rules,
    }])
}

fn document_with_stylesheets(stylesheets: Vec<UiStyleSheet>) -> UiAssetDocument {
    UiAssetDocument {
        asset: UiAssetHeader {
            kind: UiAssetKind::Style,
            id: "style.theme-compare-bench".to_string(),
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

fn rule(selector: &str, color: &str) -> UiStyleRule {
    let mut block = UiStyleDeclarationBlock::default();
    block
        .self_values
        .insert("color".to_string(), Value::String(color.to_string()));
    UiStyleRule {
        id: None,
        selector: selector.to_string(),
        set: block,
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
