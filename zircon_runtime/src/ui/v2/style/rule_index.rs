use std::collections::HashMap;

use zircon_runtime_interface::ui::template::UiSelectorToken;

use super::{ResolvedRule, SelectorPathNode};

#[derive(Clone, Debug, Default, PartialEq)]
pub(super) struct ResolvedRuleTerminalIndex {
    universal: Vec<usize>,
    by_type: HashMap<String, Vec<usize>>,
    by_id: HashMap<String, Vec<usize>>,
    by_class: HashMap<String, Vec<usize>>,
    by_state: HashMap<String, Vec<usize>>,
    host: Vec<usize>,
}

impl ResolvedRuleTerminalIndex {
    pub(super) fn from_rules(rules: &[ResolvedRule]) -> Self {
        let mut index = Self::default();
        for (rule_index, rule) in rules.iter().enumerate() {
            index.insert_rule(rule_index, rule);
        }
        index
    }

    pub(super) fn collect_candidate_indices(
        &self,
        node: &SelectorPathNode,
        candidates: &mut Vec<usize>,
    ) {
        candidates.clear();
        let required_capacity = self.candidate_capacity_upper_bound(node);
        if candidates.capacity() < required_capacity {
            candidates.reserve(required_capacity - candidates.capacity());
        }
        candidates.extend_from_slice(&self.universal);
        if node.is_host {
            candidates.extend_from_slice(&self.host);
        }
        if let Some(control_id) = node.control_id.as_ref() {
            extend_bucket(candidates, &self.by_id, control_id);
        }
        for class in &node.classes {
            extend_bucket(candidates, &self.by_class, class);
        }
        extend_bucket(candidates, &self.by_type, &node.component);
        for state in &node.states {
            extend_bucket(candidates, &self.by_state, state);
        }
        if candidates.len() > 1 {
            candidates.sort_unstable();
            candidates.dedup();
        }
    }

    fn candidate_capacity_upper_bound(&self, node: &SelectorPathNode) -> usize {
        let mut bound = self.universal.len();
        if node.is_host {
            bound = bound.saturating_add(self.host.len());
        }
        if let Some(control_id) = node.control_id.as_ref() {
            add_bucket_capacity(&mut bound, &self.by_id, control_id);
        }
        for class in &node.classes {
            add_bucket_capacity(&mut bound, &self.by_class, class);
        }
        add_bucket_capacity(&mut bound, &self.by_type, &node.component);
        for state in &node.states {
            add_bucket_capacity(&mut bound, &self.by_state, state);
        }
        bound
    }

    fn insert_rule(&mut self, rule_index: usize, rule: &ResolvedRule) {
        let Some(terminal) = rule.selector.segments.last() else {
            self.universal.push(rule_index);
            return;
        };
        if insert_first_token(
            &mut self.by_id,
            rule_index,
            &terminal.tokens,
            |token| match token {
                UiSelectorToken::Id(value) => Some(value),
                _ => None,
            },
        ) {
            return;
        }
        if insert_first_token(
            &mut self.by_class,
            rule_index,
            &terminal.tokens,
            |token| match token {
                UiSelectorToken::Class(value) => Some(value),
                _ => None,
            },
        ) {
            return;
        }
        if insert_first_token(
            &mut self.by_type,
            rule_index,
            &terminal.tokens,
            |token| match token {
                UiSelectorToken::Type(value) if value != "*" => Some(value),
                _ => None,
            },
        ) {
            return;
        }
        if terminal
            .tokens
            .iter()
            .any(|token| matches!(token, UiSelectorToken::Type(value) if value == "*"))
        {
            self.universal.push(rule_index);
            return;
        }
        if insert_first_token(
            &mut self.by_state,
            rule_index,
            &terminal.tokens,
            |token| match token {
                UiSelectorToken::State(value) => Some(value),
                _ => None,
            },
        ) {
            return;
        }
        if terminal
            .tokens
            .iter()
            .any(|token| matches!(token, UiSelectorToken::Host))
        {
            self.host.push(rule_index);
        } else {
            // Unknown or fail-closed terminal tokens still reach the full matcher.
            self.universal.push(rule_index);
        }
    }
}

fn insert_first_token<'a>(
    buckets: &mut HashMap<String, Vec<usize>>,
    rule_index: usize,
    tokens: &'a [UiSelectorToken],
    value: impl Fn(&'a UiSelectorToken) -> Option<&'a String>,
) -> bool {
    let Some(value) = tokens.iter().find_map(value) else {
        return false;
    };
    buckets.entry(value.clone()).or_default().push(rule_index);
    true
}

fn extend_bucket(candidates: &mut Vec<usize>, buckets: &HashMap<String, Vec<usize>>, key: &str) {
    if let Some(bucket) = buckets.get(key) {
        candidates.extend_from_slice(bucket);
    }
}

fn add_bucket_capacity(bound: &mut usize, buckets: &HashMap<String, Vec<usize>>, key: &str) {
    if let Some(bucket) = buckets.get(key) {
        *bound = (*bound).saturating_add(bucket.len());
    }
}

#[cfg(test)]
#[path = "tests/rule_index.rs"]
mod tests;

#[cfg(test)]
#[path = "rule_index/tests/hash_bucket_tests.rs"]
mod hash_bucket_tests;

#[cfg(test)]
#[path = "rule_index/tests/candidate_capacity_tests.rs"]
mod candidate_capacity_tests;
