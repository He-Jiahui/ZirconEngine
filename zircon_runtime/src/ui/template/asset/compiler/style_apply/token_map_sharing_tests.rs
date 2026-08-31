use std::hint::black_box;
use std::sync::Arc;
use std::time::Instant;

use super::*;
use zircon_runtime_interface::ui::template::{UiStyleDeclarationBlock, UiStyleRule, UiStyleSheet};

const SAMPLE_PAIRS: usize = 17;

#[test]
fn optimization_batch_20260826at_style_plan_shares_tokens_per_sheet() {
    let sheets = vec![
        resolved_sheet("base", 2, 3),
        resolved_sheet("overlay", 2, 2),
    ];

    let rules = build_style_plan(&sheets).expect("style plan");

    assert_eq!(rules.len(), 4);
    assert!(Arc::ptr_eq(&rules[0].tokens, &rules[1].tokens));
    assert!(Arc::ptr_eq(&rules[2].tokens, &rules[3].tokens));
    assert!(!Arc::ptr_eq(&rules[1].tokens, &rules[2].tokens));
    assert_eq!(rules[0].tokens.len(), 3);
    assert_eq!(rules[2].tokens.len(), 2);
}

#[test]
fn optimization_batch_20260826at_style_plan_clones_token_map_once_per_sheet() {
    let source = include_str!("../style_apply.rs");
    let build = bounded_source(
        source,
        "pub(super) fn build_style_plan",
        "pub(super) fn apply_styles_to_tree",
    );

    assert!(build.contains("let tokens = Arc::new(sheet.tokens.clone())"));
    assert!(build.contains("tokens: Arc::clone(&tokens)"));
    assert!(!build.contains("tokens: sheet.tokens.clone()"));
    assert_eq!(build.matches("sheet.tokens.clone()").count(), 1);
}

#[test]
#[ignore = "release performance evidence"]
fn optimization_batch_20260826at_style_plan_token_map_sharing_p95() {
    const RULES: usize = 256;
    const TOKENS: usize = 512;
    const BUILDS: usize = 1;
    let sheets = vec![resolved_sheet("large", RULES, TOKENS)];

    let mut legacy_ns = Vec::with_capacity(SAMPLE_PAIRS);
    let mut optimized_ns = Vec::with_capacity(SAMPLE_PAIRS);
    for sample_index in 0..SAMPLE_PAIRS {
        if sample_index % 2 == 0 {
            legacy_ns.push(measure_ns(BUILDS, || legacy_plan_checksum(&sheets)));
            optimized_ns.push(measure_ns(BUILDS, || optimized_plan_checksum(&sheets)));
        } else {
            optimized_ns.push(measure_ns(BUILDS, || optimized_plan_checksum(&sheets)));
            legacy_ns.push(measure_ns(BUILDS, || legacy_plan_checksum(&sheets)));
        }
    }

    let legacy_p50_ns = nearest_rank(&legacy_ns, 50);
    let legacy_p95_ns = nearest_rank(&legacy_ns, 95);
    let optimized_p50_ns = nearest_rank(&optimized_ns, 50);
    let optimized_p95_ns = nearest_rank(&optimized_ns, 95);
    assert!(
        optimized_p95_ns.saturating_mul(2) <= legacy_p95_ns,
        "shared style token maps P95 must be at least 50% below per-rule deep clones: legacy={legacy_p95_ns}ns optimized={optimized_p95_ns}ns"
    );

    println!(
        "RUNTIME03_STYLE_PLAN_TOKEN_MAP_SHARING_BENCH_V1 rules={RULES} tokens={TOKENS} builds_per_sample={BUILDS} sample_pairs={SAMPLE_PAIRS} pair_order=alternating_legacy_even legacy_token_map_instances_per_sample={} optimized_token_map_instances_per_sample={} legacy_token_entry_clones_per_sample={} optimized_token_entry_clones_per_sample={} legacy_p50_ns={legacy_p50_ns} legacy_p95_ns={legacy_p95_ns} optimized_p50_ns={optimized_p50_ns} optimized_p95_ns={optimized_p95_ns} legacy_ns={} optimized_ns={}",
        RULES * BUILDS,
        BUILDS,
        RULES * TOKENS * BUILDS,
        TOKENS * BUILDS,
        join_samples(&legacy_ns),
        join_samples(&optimized_ns),
    );
}

fn optimized_plan_checksum(sheets: &[ResolvedStyleSheet]) -> usize {
    build_style_plan(black_box(sheets))
        .expect("optimized style plan")
        .iter()
        .map(|rule| rule.tokens.len())
        .sum()
}

fn legacy_plan_checksum(sheets: &[ResolvedStyleSheet]) -> usize {
    legacy_build_style_plan(black_box(sheets))
        .expect("legacy style plan")
        .iter()
        .map(|rule| rule.tokens.len())
        .sum()
}

fn legacy_build_style_plan(
    sheets: &[ResolvedStyleSheet],
) -> Result<Vec<ParsedStyleRule>, UiAssetError> {
    let mut rules = Vec::new();
    let mut order = 0;
    for sheet in sheets {
        for rule in &sheet.stylesheet.rules {
            let selector = UiSelector::parse(&rule.selector)?;
            rules.push(ParsedStyleRule {
                specificity: selector.specificity(),
                selector,
                order,
                set: rule.set.clone(),
                tokens: Arc::new(sheet.tokens.clone()),
            });
            order += 1;
        }
    }
    Ok(rules)
}

fn resolved_sheet(id: &str, rule_count: usize, token_count: usize) -> ResolvedStyleSheet {
    ResolvedStyleSheet {
        stylesheet: UiStyleSheet {
            id: id.to_string(),
            rules: (0..rule_count)
                .map(|index| UiStyleRule {
                    id: Some(format!("{id}.rule.{index:04}")),
                    selector: format!(".{id}-rule-{index:04}"),
                    set: UiStyleDeclarationBlock::default(),
                })
                .collect(),
        },
        tokens: (0..token_count)
            .map(|index| {
                (
                    format!("{id}.token.{index:04}"),
                    Value::String(format!("{}-{index:04}", "value".repeat(8))),
                )
            })
            .collect(),
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
