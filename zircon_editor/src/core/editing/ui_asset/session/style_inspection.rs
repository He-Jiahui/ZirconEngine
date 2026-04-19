use crate::ui::{UiDesignerSelectionModel, UiStyleInspectorReflectionModel};
use serde_json::Value as JsonValue;
use toml::Value;
use zircon_runtime::ui::template::UiAssetDocument;

use super::super::style::{
    matched_rule_inspection::{
        matched_style_rule_entries, selector_component_name, selector_is_valid,
    },
    style_rule_declarations::{declaration_entries, UiStyleRuleDeclarationEntry},
};
use super::session_state::UiAssetCompilerImports;
use super::UiAssetEditorSessionError;

pub(super) use super::super::style::matched_rule_inspection::MatchedStyleRuleEntry;

pub(super) const SUPPORTED_PSEUDO_STATES: &[&str] = &[
    "hover", "focus", "pressed", "checked", "selected", "disabled",
];

#[derive(Clone, Debug)]
pub(super) struct LocalStyleRuleEntry {
    pub(super) stylesheet_index: usize,
    pub(super) rule_index: usize,
    pub(super) selector: String,
}

#[derive(Clone, Debug)]
pub(super) struct LocalStyleTokenEntry {
    pub(super) name: String,
    pub(super) literal: String,
}

pub(super) fn build_style_inspector(
    document: &UiAssetDocument,
    selection: &UiDesignerSelectionModel,
    imports: &UiAssetCompilerImports,
    active_states: &[String],
) -> UiStyleInspectorReflectionModel {
    let Some(node_id) = selection.primary_node_id.as_deref() else {
        return UiStyleInspectorReflectionModel::default();
    };
    let Some(node) = document.nodes.get(node_id) else {
        return UiStyleInspectorReflectionModel::default();
    };

    let mut inspector = UiStyleInspectorReflectionModel::for_node(node_id.to_string());
    for class_name in &node.classes {
        inspector = inspector.with_class(class_name.clone());
    }
    for state in active_states {
        inspector = inspector.with_active_pseudo_state(state.clone());
    }
    for (path, value) in &node.style_overrides.self_values {
        inspector =
            inspector.with_inline_override(format!("self.{path}"), toml_value_to_json(value));
    }
    for (path, value) in &node.style_overrides.slot {
        inspector =
            inspector.with_inline_override(format!("slot.{path}"), toml_value_to_json(value));
    }
    for rule in matched_style_rule_entries(document, &imports.styles, node_id, active_states) {
        inspector = inspector.with_matched_rule(rule.reflection());
    }
    inspector
}

pub(super) fn local_style_rule_entries(document: &UiAssetDocument) -> Vec<LocalStyleRuleEntry> {
    let mut entries = Vec::new();
    for (stylesheet_index, stylesheet) in document.stylesheets.iter().enumerate() {
        for (rule_index, rule) in stylesheet.rules.iter().enumerate() {
            entries.push(LocalStyleRuleEntry {
                stylesheet_index,
                rule_index,
                selector: rule.selector.clone(),
            });
        }
    }
    entries
}

pub(super) fn selected_style_rule_declaration_entries(
    document: &UiAssetDocument,
    selected_rule_index: Option<usize>,
) -> Vec<UiStyleRuleDeclarationEntry> {
    selected_rule_index
        .and_then(|index| local_style_rule_entries(document).get(index).cloned())
        .map(|entry| {
            declaration_entries(
                &document.stylesheets[entry.stylesheet_index].rules[entry.rule_index].set,
            )
        })
        .unwrap_or_default()
}

pub(super) fn matched_style_rule_entries_for_selection(
    document: &UiAssetDocument,
    selection: &UiDesignerSelectionModel,
    imports: &UiAssetCompilerImports,
    active_states: &[String],
) -> Vec<MatchedStyleRuleEntry> {
    selection
        .primary_node_id
        .as_deref()
        .map(|node_id| {
            matched_style_rule_entries(document, &imports.styles, node_id, active_states)
        })
        .unwrap_or_default()
}

pub(super) fn local_style_token_entries(document: &UiAssetDocument) -> Vec<LocalStyleTokenEntry> {
    document
        .tokens
        .iter()
        .map(|(name, value)| LocalStyleTokenEntry {
            name: name.clone(),
            literal: toml_value_literal(value),
        })
        .collect()
}

pub(super) fn reconcile_selected_style_rule_index(
    document: &UiAssetDocument,
    current: Option<usize>,
) -> Option<usize> {
    let count = local_style_rule_entries(document).len();
    match (current, count) {
        (_, 0) => None,
        (Some(index), count) => Some(index.min(count - 1)),
        (None, _) => None,
    }
}

pub(super) fn reconcile_selected_style_rule_declaration_path(
    document: &UiAssetDocument,
    selected_rule_index: Option<usize>,
    current: Option<&str>,
) -> Option<String> {
    let entries = selected_style_rule_declaration_entries(document, selected_rule_index);
    current
        .filter(|path| entries.iter().any(|entry| entry.path.as_str() == *path))
        .map(str::to_string)
}

pub(super) fn reconcile_selected_matched_style_rule_index(
    entries: &[MatchedStyleRuleEntry],
    current: Option<usize>,
) -> Option<usize> {
    match (current, entries.len()) {
        (_, 0) => None,
        (Some(index), count) => Some(index.min(count - 1)),
        (None, _) => None,
    }
}

pub(super) fn reconcile_selected_style_token_name(
    document: &UiAssetDocument,
    current: Option<&str>,
) -> Option<String> {
    current
        .filter(|name| document.tokens.contains_key(*name))
        .map(str::to_string)
}

pub(super) fn normalized_selector(selector: &str) -> Result<String, UiAssetEditorSessionError> {
    let trimmed = selector.trim();
    if trimmed.is_empty() || !selector_is_valid(trimmed) {
        return Err(UiAssetEditorSessionError::InvalidStyleSelector {
            selector: trimmed.to_string(),
        });
    }
    Ok(trimmed.to_string())
}

pub(super) fn normalized_class_name(class_name: &str) -> Option<String> {
    let trimmed = class_name.trim();
    (!trimmed.is_empty() && !trimmed.chars().any(char::is_whitespace)).then(|| trimmed.to_string())
}

pub(super) fn normalized_token_name(token_name: &str) -> Option<String> {
    let trimmed = token_name.trim();
    (!trimmed.is_empty() && !trimmed.chars().any(char::is_whitespace)).then(|| trimmed.to_string())
}

pub(super) fn parse_token_literal(value_literal: &str) -> Option<Value> {
    let trimmed = value_literal.trim();
    if trimmed.is_empty() {
        return None;
    }

    let wrapped = format!("value = {trimmed}");
    toml::from_str::<toml::Table>(&wrapped)
        .ok()
        .and_then(|table| table.get("value").cloned())
        .or_else(|| Some(Value::String(trimmed.to_string())))
}

pub(super) fn toml_value_literal(value: &Value) -> String {
    value.to_string()
}

pub(super) fn selected_node_selector(
    document: &UiAssetDocument,
    selection: &UiDesignerSelectionModel,
) -> Option<String> {
    selection
        .primary_node_id
        .as_deref()
        .and_then(|node_id| document.nodes.get(node_id))
        .map(selector_for_node)
}

pub(super) fn selected_node_has_inline_overrides(
    document: &UiAssetDocument,
    selection: &UiDesignerSelectionModel,
) -> bool {
    selection
        .primary_node_id
        .as_deref()
        .and_then(|node_id| document.nodes.get(node_id))
        .map(|node| {
            !node.style_overrides.self_values.is_empty() || !node.style_overrides.slot.is_empty()
        })
        .unwrap_or(false)
}

pub(super) fn build_stylesheet_items(
    inspector: &UiStyleInspectorReflectionModel,
    selector_hint: Option<String>,
) -> Vec<String> {
    let mut items = Vec::new();
    if let Some(selector_hint) = selector_hint {
        items.push(format!("selection selector: {selector_hint}"));
    }
    if !inspector.active_pseudo_states.is_empty() {
        items.push(format!(
            "states: {}",
            inspector.active_pseudo_states.join(", ")
        ));
    }
    for (path, value) in &inspector.inline_overrides {
        items.push(format!("override {path} = {value}"));
    }
    for rule in &inspector.matched_rules {
        items.push(format!(
            "{} (spec {}, order {})",
            rule.selector, rule.specificity, rule.source_order
        ));
    }
    if items.is_empty() {
        items.push("no matched stylesheet rules".to_string());
    }
    items
}

pub(super) fn pseudo_state_active(
    inspector: &UiStyleInspectorReflectionModel,
    state: &str,
) -> bool {
    inspector
        .active_pseudo_states
        .iter()
        .any(|entry| entry == state)
}

fn selector_for_node(node: &zircon_runtime::ui::template::UiNodeDefinition) -> String {
    if let Some(control_id) = node.control_id.as_deref() {
        return format!("#{control_id}");
    }

    let mut selector = selector_component_name(node).to_string();
    for class_name in &node.classes {
        selector.push('.');
        selector.push_str(class_name);
    }
    selector
}

fn toml_value_to_json(value: &Value) -> JsonValue {
    serde_json::to_value(value).unwrap_or(JsonValue::Null)
}
