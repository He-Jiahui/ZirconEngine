use std::{collections::BTreeMap, hint::black_box, time::Instant};

use super::*;
use zircon_runtime_interface::ui::template::{
    UiAssetHeader, UiAssetImports, UiAssetKind, UiStyleRule, UiStyleSheet,
};

const SAMPLE_PAIRS: usize = 17;

#[test]
fn optimization_batch_20260826ay_theme_cascade_borrowing_preserves_active_shadowed_order() {
    let imported = style_document("base", "red", ".button", "base-sheet");
    let mut local = style_document("local", "blue", ".button", "local-sheet");
    local.imports.styles.push("base".to_string());
    let imported_styles = BTreeMap::from([("base".to_string(), imported)]);

    let inspection = build_theme_cascade_inspection(&local, &imported_styles);

    assert_eq!(inspection.layer_items.len(), 2);
    assert_eq!(
        inspection.token_items[0],
        "active • accent • Local = \"blue\""
    );
    assert_eq!(
        inspection.token_items[1],
        "shadowed • accent • base = \"red\""
    );
    assert!(inspection.rule_items[0].contains("Imported • base • base-sheet • .button"));
    assert!(inspection.rule_items[1].contains("Local • local-sheet • .button"));
    assert!(inspection.rule_items[2].contains("active • rule • .button • Local • local-sheet"));
    assert!(inspection.rule_items[3].contains("shadowed • rule • .button • base • base-sheet"));
}

#[test]
fn optimization_batch_20260826ay_theme_cascade_uses_borrowed_definitions() {
    let source = include_str!("../theme_cascade_inspection.rs");
    let tokens = bounded_source(source, "fn cascade_token_items", "fn cascade_rule_items");
    let rules = bounded_source(source, "fn cascade_rule_items", "fn total_rule_count");

    assert!(source.contains("struct UiAssetThemeTokenDefinition<'a>"));
    assert!(source.contains("value: &'a Value"));
    assert!(source.contains("struct UiAssetThemeRuleDefinition<'a>"));
    assert!(source.contains("declarations: &'a UiStyleDeclarationBlock"));
    assert!(tokens.contains("BTreeMap::<&'a str"));
    assert!(!tokens.contains("name.clone()"));
    assert!(!tokens.contains("value.to_string()"));
    assert!(rules.contains("BTreeMap::<&'a str"));
    assert!(!rules.contains("rule.selector.clone()"));
    assert!(!rules.contains("format_rule_block(&rule.set)"));
}

#[test]
#[ignore = "release performance evidence"]
fn optimization_batch_20260826ay_theme_cascade_borrowed_definitions_p95() {
    const RULE_COUNT: usize = 2_048;
    const DECLARATIONS_PER_RULE: usize = 16;
    const BUILDS: usize = 2;
    let document = benchmark_document(RULE_COUNT, DECLARATIONS_PER_RULE);
    let imported_styles = BTreeMap::new();
    let layers = cascade_layers(&document, &imported_styles);
    let mut legacy_ns = Vec::with_capacity(SAMPLE_PAIRS);
    let mut optimized_ns = Vec::with_capacity(SAMPLE_PAIRS);

    for sample_index in 0..SAMPLE_PAIRS {
        if sample_index % 2 == 0 {
            legacy_ns.push(measure_ns(BUILDS, || {
                legacy_cascade_rule_items(black_box(&layers)).len()
            }));
            optimized_ns.push(measure_ns(BUILDS, || {
                cascade_rule_items(black_box(&layers)).len()
            }));
        } else {
            optimized_ns.push(measure_ns(BUILDS, || {
                cascade_rule_items(black_box(&layers)).len()
            }));
            legacy_ns.push(measure_ns(BUILDS, || {
                legacy_cascade_rule_items(black_box(&layers)).len()
            }));
        }
    }

    let legacy_p50_ns = nearest_rank(&legacy_ns, 50);
    let legacy_p95_ns = nearest_rank(&legacy_ns, 95);
    let optimized_p50_ns = nearest_rank(&optimized_ns, 50);
    let optimized_p95_ns = nearest_rank(&optimized_ns, 95);
    assert!(
        optimized_p95_ns.saturating_mul(5) <= legacy_p95_ns,
        "borrowed cascade definitions P95 must be at least 80% below eager declaration formatting: legacy={legacy_p95_ns}ns optimized={optimized_p95_ns}ns"
    );

    println!(
        "EDITOR23_THEME_CASCADE_BORROWED_DEFINITIONS_BENCH_V1 rules={RULE_COUNT} declarations_per_rule={DECLARATIONS_PER_RULE} builds_per_sample={BUILDS} sample_pairs={SAMPLE_PAIRS} pair_order=alternating_legacy_even legacy_selector_clones_per_sample={} legacy_source_clones_per_sample={} legacy_stylesheet_clones_per_sample={} legacy_declaration_leaf_formats_per_sample={} optimized_definition_clones_per_sample=0 optimized_unique_rule_declaration_formats_per_sample=0 legacy_p50_ns={legacy_p50_ns} legacy_p95_ns={legacy_p95_ns} optimized_p50_ns={optimized_p50_ns} optimized_p95_ns={optimized_p95_ns} legacy_ns={} optimized_ns={}",
        RULE_COUNT * BUILDS * 2,
        RULE_COUNT * BUILDS,
        RULE_COUNT * BUILDS,
        RULE_COUNT * DECLARATIONS_PER_RULE * BUILDS,
        join_samples(&legacy_ns),
        join_samples(&optimized_ns),
    );
}

fn legacy_cascade_rule_items(layers: &[UiAssetThemeCascadeLayer<'_>]) -> Vec<String> {
    #[derive(Clone)]
    struct Definition {
        selector: String,
        source: String,
        stylesheet_id: String,
        declarations: String,
    }

    let mut items = Vec::new();
    let mut rules_by_selector = BTreeMap::<String, Vec<Definition>>::new();
    let mut order = 1usize;
    for layer in layers {
        let Some(document) = layer.document else {
            continue;
        };
        for stylesheet in &document.stylesheets {
            let stylesheet_label = if stylesheet.id.is_empty() {
                "<inline>"
            } else {
                stylesheet.id.as_str()
            };
            for rule in &stylesheet.rules {
                items.push(match layer.kind {
                    UiAssetThemeCascadeLayerKind::Imported => format!(
                        "{order}. Imported • {} • {stylesheet_label} • {}",
                        layer.reference, rule.selector
                    ),
                    UiAssetThemeCascadeLayerKind::Local => {
                        format!("{order}. Local • {stylesheet_label} • {}", rule.selector)
                    }
                });
                rules_by_selector
                    .entry(rule.selector.clone())
                    .or_default()
                    .push(Definition {
                        selector: rule.selector.clone(),
                        source: match layer.kind {
                            UiAssetThemeCascadeLayerKind::Imported => layer.reference.to_string(),
                            UiAssetThemeCascadeLayerKind::Local => "Local".to_string(),
                        },
                        stylesheet_id: stylesheet_label.to_string(),
                        declarations: format_rule_block(&rule.set),
                    });
                order += 1;
            }
        }
    }
    for definitions in rules_by_selector.into_values() {
        let Some((active, shadowed)) = definitions.split_last() else {
            continue;
        };
        if shadowed.is_empty() {
            continue;
        }
        items.push(format!(
            "active • rule • {} • {} • {} • {}",
            active.selector, active.source, active.stylesheet_id, active.declarations
        ));
        for definition in shadowed.iter().rev() {
            items.push(format!(
                "shadowed • rule • {} • {} • {} • {}",
                definition.selector,
                definition.source,
                definition.stylesheet_id,
                definition.declarations
            ));
        }
    }
    items
}

fn benchmark_document(rule_count: usize, declarations_per_rule: usize) -> UiAssetDocument {
    let rules = (0..rule_count)
        .map(|rule_index| {
            let mut set = UiStyleDeclarationBlock::default();
            for declaration_index in 0..declarations_per_rule {
                set.self_values.insert(
                    format!("property-{declaration_index:02}"),
                    Value::String(format!(
                        "value-{rule_index:04}-{declaration_index:02}-abcdefghijklmnopqrstuvwxyz"
                    )),
                );
            }
            UiStyleRule {
                id: None,
                selector: format!(".component-{rule_index:04}"),
                set,
            }
        })
        .collect();
    document(Vec::new(), BTreeMap::new(), rules, "local-sheet")
}

fn style_document(id: &str, color: &str, selector: &str, stylesheet_id: &str) -> UiAssetDocument {
    document(
        Vec::new(),
        BTreeMap::from([("accent".to_string(), Value::String(color.to_string()))]),
        vec![UiStyleRule {
            id: None,
            selector: selector.to_string(),
            set: UiStyleDeclarationBlock {
                self_values: BTreeMap::from([(
                    "color".to_string(),
                    Value::String(color.to_string()),
                )]),
                slot: BTreeMap::new(),
            },
        }],
        stylesheet_id,
    )
    .with_asset_id(id)
}

trait WithAssetId {
    fn with_asset_id(self, id: &str) -> Self;
}

impl WithAssetId for UiAssetDocument {
    fn with_asset_id(mut self, id: &str) -> Self {
        self.asset.id = format!("style.{id}");
        self
    }
}

fn document(
    imports: Vec<String>,
    tokens: BTreeMap<String, Value>,
    rules: Vec<UiStyleRule>,
    stylesheet_id: &str,
) -> UiAssetDocument {
    UiAssetDocument {
        asset: UiAssetHeader {
            kind: UiAssetKind::Style,
            id: "style.cascade-bench".to_string(),
            version: 1,
            display_name: String::new(),
        },
        imports: UiAssetImports {
            styles: imports,
            ..UiAssetImports::default()
        },
        tokens,
        root: None,
        components: BTreeMap::new(),
        stylesheets: vec![UiStyleSheet {
            id: stylesheet_id.to_string(),
            rules,
        }],
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
