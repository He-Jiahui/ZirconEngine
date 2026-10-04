use std::collections::BTreeMap;
use std::hint::black_box;
use std::time::Instant;

use zircon_runtime_interface::ui::template::UiSelector;
use zircon_runtime_interface::ui::v2::UiV2StyleDeclarationBlock;

use super::*;
use crate::ui::v2::style::UiV2SelectorMatchExt;

const SAMPLE_PAIRS: usize = 21;

#[test]
fn terminal_rule_index_preserves_full_match_results_and_cascade_order() {
    let rules = [
        "Button#save:hover",
        ".primary:hover",
        "Button:hover",
        ":hover",
        ":host:hover",
        ".ancestor:hover Text",
        "Label:hover",
    ]
    .into_iter()
    .enumerate()
    .map(|(order, selector)| resolved_rule(selector, order))
    .collect::<Vec<_>>();
    let path = vec![
        selector_node("Panel", None, &["ancestor"], &["hover"], true),
        selector_node("Button", Some("save"), &["primary"], &["hover"], false),
    ];

    let expected = rules
        .iter()
        .enumerate()
        .filter_map(|(index, rule)| rule.selector.matches_path(&path).then_some(index))
        .collect::<Vec<_>>();
    let index = ResolvedRuleTerminalIndex::from_rules(&rules);
    let mut candidates = Vec::new();
    index.collect_candidate_indices(path.last().unwrap(), &mut candidates);
    let actual = candidates
        .into_iter()
        .filter(|candidate| rules[*candidate].selector.matches_path(&path))
        .collect::<Vec<_>>();

    assert_eq!(actual, expected);
    assert_eq!(actual, vec![0, 1, 2, 3]);

    let host_path = vec![selector_node(
        "Button",
        Some("save"),
        &["primary"],
        &["hover"],
        true,
    )];
    assert_index_matches_full_scan(&rules, &host_path);
}

#[test]
fn universal_terminal_rule_matches_every_runtime_node() {
    let rules = [resolved_rule("*", 0), resolved_rule("Label", 1)];
    let path = vec![selector_node("Button", None, &[], &[], false)];

    let expected = rules
        .iter()
        .enumerate()
        .filter_map(|(index, rule)| rule.selector.matches_path(&path).then_some(index))
        .collect::<Vec<_>>();
    let index = ResolvedRuleTerminalIndex::from_rules(&rules);
    let mut candidates = Vec::new();
    index.collect_candidate_indices(path.last().unwrap(), &mut candidates);
    candidates.retain(|candidate| rules[*candidate].selector.matches_path(&path));

    assert_eq!(expected, vec![0]);
    assert_eq!(candidates, expected);
}

#[test]
#[ignore = "release performance evidence"]
fn terminal_rule_index_release_benchmark_evidence() {
    const RULES: usize = 16_384;
    const TARGET_RULE: usize = RULES / 2;

    let rules = (0..RULES)
        .map(|index| resolved_rule(&format!(".rule-{index}:hover"), index))
        .collect::<Vec<_>>();
    let target_class = format!("rule-{TARGET_RULE}");
    let path = vec![selector_node(
        "Button",
        None,
        &[&target_class],
        &["hover"],
        false,
    )];
    let index = ResolvedRuleTerminalIndex::from_rules(&rules);
    let mut candidates = Vec::new();
    index.collect_candidate_indices(path.last().unwrap(), &mut candidates);
    assert_eq!(candidates, vec![TARGET_RULE]);

    let mut legacy = || {
        rules
            .iter()
            .filter(|rule| rule.selector.matches_path(black_box(&path)))
            .count()
    };
    let mut optimized_candidates = candidates;
    let mut optimized = || {
        index.collect_candidate_indices(black_box(path.last().unwrap()), &mut optimized_candidates);
        optimized_candidates
            .iter()
            .filter(|candidate| rules[**candidate].selector.matches_path(black_box(&path)))
            .count()
    };

    assert_eq!(black_box(legacy()), 1);
    assert_eq!(black_box(optimized()), 1);

    let mut legacy_ns = Vec::with_capacity(SAMPLE_PAIRS);
    let mut optimized_ns = Vec::with_capacity(SAMPLE_PAIRS);
    for sample_index in 0..SAMPLE_PAIRS {
        if sample_index % 2 == 0 {
            legacy_ns.push(measure_ns(&mut legacy));
            optimized_ns.push(measure_ns(&mut optimized));
        } else {
            optimized_ns.push(measure_ns(&mut optimized));
            legacy_ns.push(measure_ns(&mut legacy));
        }
    }

    let legacy_p50_ns = nearest_rank(&legacy_ns, 50);
    let legacy_p95_ns = nearest_rank(&legacy_ns, 95);
    let optimized_p50_ns = nearest_rank(&optimized_ns, 50);
    let optimized_p95_ns = nearest_rank(&optimized_ns, 95);
    assert!(
        optimized_p95_ns.saturating_mul(4) <= legacy_p95_ns,
        "terminal selector index P95 must be at least 75% below the full rule scan: legacy={legacy_p95_ns}ns optimized={optimized_p95_ns}ns"
    );

    println!(
        "RUNTIME73_SELECTOR_TERMINAL_INDEX_BENCH_V1 rules={RULES} candidate_rules=1 sample_pairs={SAMPLE_PAIRS} pair_order=alternating_legacy_even legacy_first_pairs=11 optimized_first_pairs=10 legacy_rule_visits={RULES} optimized_candidate_visits=1 legacy_p50_ns={legacy_p50_ns} legacy_p95_ns={legacy_p95_ns} optimized_p50_ns={optimized_p50_ns} optimized_p95_ns={optimized_p95_ns} legacy_ns={} optimized_ns={}",
        join_samples(&legacy_ns),
        join_samples(&optimized_ns),
    );
}

fn resolved_rule(selector: &str, order: usize) -> ResolvedRule {
    let selector = UiSelector::parse(selector).expect("test selector must parse");
    ResolvedRule {
        specificity: selector.specificity(),
        order,
        selector,
        set: UiV2StyleDeclarationBlock::default(),
        style_tokens: BTreeMap::new(),
    }
}

fn selector_node(
    component: &str,
    control_id: Option<&str>,
    classes: &[&str],
    states: &[&str],
    is_host: bool,
) -> SelectorPathNode {
    SelectorPathNode {
        component: component.to_owned(),
        control_id: control_id.map(str::to_owned),
        classes: classes.iter().map(|value| (*value).to_owned()).collect(),
        states: states.iter().map(|value| (*value).to_owned()).collect(),
        is_host,
    }
}

fn assert_index_matches_full_scan(rules: &[ResolvedRule], path: &[SelectorPathNode]) {
    let expected = rules
        .iter()
        .enumerate()
        .filter_map(|(index, rule)| rule.selector.matches_path(path).then_some(index))
        .collect::<Vec<_>>();
    let index = ResolvedRuleTerminalIndex::from_rules(rules);
    let mut candidates = Vec::new();
    index.collect_candidate_indices(path.last().unwrap(), &mut candidates);
    candidates.retain(|candidate| rules[*candidate].selector.matches_path(path));
    assert_eq!(candidates, expected);
}

fn measure_ns(operation: &mut impl FnMut() -> usize) -> u128 {
    let started = Instant::now();
    assert_eq!(black_box(operation()), 1);
    started.elapsed().as_nanos()
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
