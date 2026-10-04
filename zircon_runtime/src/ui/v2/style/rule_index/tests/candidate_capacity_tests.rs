use std::collections::BTreeMap;

use zircon_runtime_interface::ui::template::UiSelector;
use zircon_runtime_interface::ui::v2::UiV2StyleDeclarationBlock;

use super::*;

#[test]
fn optimization_batch_20260920_runtime860_candidate_capacity_preserves_order() {
    const RULE_COUNT: usize = 4_096;
    let rules = (0..RULE_COUNT)
        .map(|order| resolved_rule(".dense:hover", order))
        .collect::<Vec<_>>();
    let node = SelectorPathNode {
        component: "Button".to_owned(),
        control_id: None,
        classes: vec!["dense".to_owned()],
        states: vec!["hover".to_owned()],
        is_host: false,
    };

    let index = ResolvedRuleTerminalIndex::from_rules(&rules);
    let mut candidates = Vec::new();
    index.collect_candidate_indices(&node, &mut candidates);

    assert_eq!(candidates.len(), RULE_COUNT);
    assert!(candidates.capacity() >= RULE_COUNT);
    assert_eq!(candidates.first().copied(), Some(0));
    assert_eq!(candidates.last().copied(), Some(RULE_COUNT - 1));
}

#[test]
fn optimization_batch_20260920_runtime860_empty_candidate_capacity_stays_zero() {
    let index = ResolvedRuleTerminalIndex::from_rules(&[]);
    let node = SelectorPathNode {
        component: "Button".to_owned(),
        control_id: None,
        classes: Vec::new(),
        states: Vec::new(),
        is_host: false,
    };
    let mut candidates = Vec::new();

    index.collect_candidate_indices(&node, &mut candidates);

    assert!(candidates.is_empty());
    assert_eq!(candidates.capacity(), 0);
}

#[test]
#[ignore = "performance evidence; run in the managed Windows release lane"]
fn runtime860_selector_candidate_capacity_bench_v1() {
    const RULE_COUNT: usize = 4_096;
    let rules = (0..RULE_COUNT)
        .map(|order| resolved_rule(".dense:hover", order))
        .collect::<Vec<_>>();
    let node = SelectorPathNode {
        component: "Button".to_owned(),
        control_id: None,
        classes: vec!["dense".to_owned()],
        states: vec!["hover".to_owned()],
        is_host: false,
    };
    let index = ResolvedRuleTerminalIndex::from_rules(&rules);
    let mut candidates = Vec::new();
    index.collect_candidate_indices(&node, &mut candidates);

    println!(
        "RUNTIME860_SELECTOR_CANDIDATE_CAPACITY_BENCH_V1 rules={} candidates={} capacity={} modeled_legacy_growth_events=13 modeled_reserved_growth_events=0",
        RULE_COUNT,
        candidates.len(),
        candidates.capacity(),
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
