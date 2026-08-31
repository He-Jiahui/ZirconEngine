use super::super::{
    style_inspection::{
        build_stylesheet_items, local_style_rule_entries, matched_style_rule_entries_for_selection,
        selected_node_selector, MatchedStyleRuleEntry,
    },
    style_rule_declarations::declaration_entries,
    ui_asset_editor_session::UiAssetEditorSession,
};
use crate::ui::retained_host::ui_perf::{record_current_ui_perf_counter, UiPerfCounter};
use zircon_runtime_interface::ui::template::UiAssetDocument;

pub(super) struct UiAssetStylePaneData {
    pub(super) has_selected_node_selector: bool,
    pub(super) stylesheet_items: Vec<String>,
    pub(super) rule_items: Vec<String>,
    pub(super) rule_selected_index: i32,
    pub(super) selected_rule_id: String,
    pub(super) selected_rule_selector: String,
    pub(super) can_edit_rule: bool,
    pub(super) can_delete_rule: bool,
    pub(super) matched_rule_items: Vec<String>,
    pub(super) matched_rule_selected_index: i32,
    pub(super) selected_matched_rule_origin: String,
    pub(super) selected_matched_rule_selector: String,
    pub(super) selected_matched_rule_specificity: i32,
    pub(super) selected_matched_rule_source_order: i32,
    pub(super) selected_matched_rule_declaration_items: Vec<String>,
    pub(super) rule_declaration_items: Vec<String>,
    pub(super) rule_declaration_selected_index: i32,
    pub(super) selected_rule_declaration_path: String,
    pub(super) selected_rule_declaration_value: String,
    pub(super) can_edit_rule_declaration: bool,
    pub(super) can_delete_rule_declaration: bool,
    pub(super) token_items: Vec<String>,
    pub(super) token_selected_index: i32,
    pub(super) selected_token_name: String,
    pub(super) selected_token_value: String,
    pub(super) can_edit_token: bool,
    pub(super) can_delete_token: bool,
}

struct LocalStyleTokenPresentation {
    items: Vec<String>,
    selected_index: i32,
    selected_name: String,
    selected_value: String,
}

fn build_local_style_token_presentation(
    document: &UiAssetDocument,
    selected_name: Option<&str>,
) -> LocalStyleTokenPresentation {
    let mut items = Vec::with_capacity(document.tokens.len());
    let mut selected_index = -1;
    let mut selected_token_name = String::new();
    let mut selected_value = String::new();
    for (index, (name, value)) in document.tokens.iter().enumerate() {
        let literal = value.to_string();
        if selected_index < 0 && selected_name == Some(name.as_str()) {
            selected_index = index as i32;
            selected_token_name.clone_from(name);
            selected_value.clone_from(&literal);
        }
        items.push(format!("{name} = {literal}"));
    }
    LocalStyleTokenPresentation {
        items,
        selected_index,
        selected_name: selected_token_name,
        selected_value,
    }
}

impl UiAssetEditorSession {
    pub(super) fn style_pane_presentation(&self) -> UiAssetStylePaneData {
        zircon_runtime::profile_scope!("editor", "asset_editor.presentation", "style",);
        let selector_hint = selected_node_selector(&self.last_valid_document, &self.selection);
        let has_selected_node_selector = selector_hint.is_some();
        let stylesheet_items = build_stylesheet_items(&self.style_inspector, selector_hint);
        let style_rules = local_style_rule_entries(&self.last_valid_document);
        let matched_style_rules = matched_style_rule_entries_for_selection(
            &self.last_valid_document,
            &self.selection,
            &self.compiler_imports,
            &self.style_inspector.active_pseudo_states,
        );
        let LocalStyleTokenPresentation {
            items: token_items,
            selected_index: token_selected_index,
            selected_name: selected_token_name,
            selected_value: selected_token_value,
        } = build_local_style_token_presentation(
            &self.last_valid_document,
            self.selected_style_token_name.as_deref(),
        );
        let selected_style_rule = self
            .selected_style_rule_index
            .and_then(|index| style_rules.get(index));
        let selected_matched_style_rule = self
            .selected_matched_style_rule_index
            .and_then(|index| matched_style_rules.get(index).map(|entry| (index, entry)));
        let style_rule_declarations = selected_style_rule
            .map(|entry| {
                declaration_entries(
                    &self.last_valid_document.stylesheets[entry.stylesheet_index].rules
                        [entry.rule_index]
                        .set,
                )
            })
            .unwrap_or_default();
        let selected_declaration_path = self.selected_style_rule_declaration_path.as_deref();
        let (rule_declaration_items, selected_style_rule_declaration) = collect_items_and_selection(
            &style_rule_declarations,
            |entry| selected_declaration_path == Some(entry.path.as_str()),
            |entry| format!("{} = {}", entry.path, entry.literal),
        );
        record_current_ui_perf_counter(UiPerfCounter::AssetEditorPaneStyleBuildCount, 1.0);
        UiAssetStylePaneData {
            has_selected_node_selector,
            stylesheet_items,
            rule_items: style_rules
                .iter()
                .map(|rule| rule.selector.clone())
                .collect(),
            rule_selected_index: self
                .selected_style_rule_index
                .map(|index| index as i32)
                .unwrap_or(-1),
            selected_rule_id: selected_style_rule
                .and_then(|rule| rule.id.clone())
                .unwrap_or_default(),
            selected_rule_selector: selected_style_rule
                .map(|rule| rule.selector.clone())
                .unwrap_or_default(),
            can_edit_rule: self.diagnostics.is_empty() && selected_style_rule.is_some(),
            can_delete_rule: self.diagnostics.is_empty() && selected_style_rule.is_some(),
            matched_rule_items: matched_style_rules
                .iter()
                .map(MatchedStyleRuleEntry::label)
                .collect(),
            matched_rule_selected_index: selected_matched_style_rule
                .map(|(index, _)| index as i32)
                .unwrap_or(-1),
            selected_matched_rule_origin: selected_matched_style_rule
                .map(|(_, rule)| rule.origin_id.clone())
                .unwrap_or_default(),
            selected_matched_rule_selector: selected_matched_style_rule
                .map(|(_, rule)| rule.selector.clone())
                .unwrap_or_default(),
            selected_matched_rule_specificity: selected_matched_style_rule
                .map(|(_, rule)| rule.specificity as i32)
                .unwrap_or(-1),
            selected_matched_rule_source_order: selected_matched_style_rule
                .map(|(_, rule)| rule.source_order as i32)
                .unwrap_or(-1),
            selected_matched_rule_declaration_items: selected_matched_style_rule
                .map(|(_, rule)| rule.declaration_items())
                .unwrap_or_default(),
            rule_declaration_items,
            rule_declaration_selected_index: selected_style_rule_declaration
                .map(|(index, _)| index as i32)
                .unwrap_or(-1),
            selected_rule_declaration_path: selected_style_rule_declaration
                .map(|(_, entry)| entry.path.clone())
                .unwrap_or_default(),
            selected_rule_declaration_value: selected_style_rule_declaration
                .map(|(_, entry)| entry.literal.clone())
                .unwrap_or_default(),
            can_edit_rule_declaration: self.diagnostics.is_empty() && selected_style_rule.is_some(),
            can_delete_rule_declaration: self.diagnostics.is_empty()
                && selected_style_rule_declaration.is_some(),
            token_items,
            token_selected_index,
            selected_token_name,
            selected_token_value,
            can_edit_token: self.diagnostics.is_empty() && token_selected_index >= 0,
            can_delete_token: self.diagnostics.is_empty() && token_selected_index >= 0,
        }
    }
}

fn collect_items_and_selection<'a, T>(
    entries: &'a [T],
    mut is_selected: impl FnMut(&T) -> bool,
    mut label: impl FnMut(&T) -> String,
) -> (Vec<String>, Option<(usize, &'a T)>) {
    let mut items = Vec::with_capacity(entries.len());
    let mut selected = None;
    for (index, entry) in entries.iter().enumerate() {
        if selected.is_none() && is_selected(entry) {
            selected = Some((index, entry));
        }
        items.push(label(entry));
    }
    (items, selected)
}

#[cfg(test)]
#[path = "style/single_pass_selection_tests.rs"]
mod single_pass_selection_tests;
