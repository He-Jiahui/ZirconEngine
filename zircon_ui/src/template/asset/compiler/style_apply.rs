use std::collections::BTreeMap;

use toml::Value;

use crate::{UiAssetError, UiSelector, UiStyleDeclarationBlock, UiTemplateNode};

use super::super::style::UiSelectorMatchNode;
use super::ui_document_compiler::ResolvedStyleSheet;
use super::value_normalizer::{merge_value_maps, merge_value_maps_resolved};

#[derive(Clone)]
pub(super) struct ParsedStyleRule {
    selector: UiSelector,
    specificity: usize,
    order: usize,
    set: UiStyleDeclarationBlock,
    tokens: BTreeMap<String, Value>,
}

pub(super) fn build_style_plan(
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
                tokens: sheet.tokens.clone(),
            });
            order += 1;
        }
    }
    Ok(rules)
}

pub(super) fn apply_styles_to_tree(
    node: &mut UiTemplateNode,
    rules: &[ParsedStyleRule],
    path: &mut Vec<StylePathEntry>,
) {
    path.push(StylePathEntry::from_node(node, path.is_empty()));

    let path_snapshot: Vec<_> = path.iter().map(StylePathEntry::as_match_node).collect();
    let mut matched: Vec<_> = rules
        .iter()
        .filter(|rule| rule.selector.matches_path(&path_snapshot))
        .cloned()
        .collect();
    matched.sort_by_key(|rule| (rule.specificity, rule.order));
    for rule in matched {
        merge_value_maps_resolved(
            &mut node.attributes,
            &rule.set.self_values,
            &rule.tokens,
            &BTreeMap::new(),
        );
        merge_value_maps_resolved(
            &mut node.slot_attributes,
            &rule.set.slot,
            &rule.tokens,
            &BTreeMap::new(),
        );
    }

    if !node.style_overrides.is_empty() {
        let inline = node.style_overrides.clone();
        merge_value_maps(&mut node.attributes, &inline);
    }

    for child in &mut node.children {
        apply_styles_to_tree(child, rules, path);
    }

    let _ = path.pop();
}

#[derive(Clone)]
pub(super) struct StylePathEntry {
    component: String,
    control_id: Option<String>,
    classes: Vec<String>,
    is_host: bool,
}

impl StylePathEntry {
    fn from_node(node: &UiTemplateNode, is_host: bool) -> Self {
        Self {
            component: node.component.clone().unwrap_or_default(),
            control_id: node.control_id.clone(),
            classes: node.classes.clone(),
            is_host,
        }
    }

    fn as_match_node(&self) -> UiSelectorMatchNode<'_> {
        UiSelectorMatchNode {
            component: &self.component,
            control_id: self.control_id.as_deref(),
            classes: &self.classes,
            is_host: self.is_host,
            states: &[],
        }
    }
}
