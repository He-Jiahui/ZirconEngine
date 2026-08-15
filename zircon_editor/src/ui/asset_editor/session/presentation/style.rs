use super::super::{
    style_inspection::{
        build_stylesheet_items, local_style_rule_entries, local_style_token_entries,
        matched_style_rule_entries_for_selection, selected_node_selector, MatchedStyleRuleEntry,
    },
    style_rule_declarations::declaration_entries,
    ui_asset_editor_session::UiAssetEditorSession,
};
use crate::ui::retained_host::ui_perf::{record_current_ui_perf_counter, UiPerfCounter};

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
        let style_tokens = local_style_token_entries(&self.last_valid_document);
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
        let selected_style_rule_declaration = self
            .selected_style_rule_declaration_path
            .as_deref()
            .and_then(|path| {
                style_rule_declarations
                    .iter()
                    .position(|entry| entry.path.as_str() == path)
            })
            .and_then(|index| {
                style_rule_declarations
                    .get(index)
                    .map(|entry| (index, entry))
            });
        let selected_style_token = self
            .selected_style_token_name
            .as_deref()
            .and_then(|name| {
                style_tokens
                    .iter()
                    .position(|entry| entry.name.as_str() == name)
            })
            .and_then(|index| style_tokens.get(index).map(|entry| (index, entry)));
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
            rule_declaration_items: style_rule_declarations
                .iter()
                .map(|entry| format!("{} = {}", entry.path, entry.literal))
                .collect(),
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
            token_items: style_tokens
                .iter()
                .map(|entry| format!("{} = {}", entry.name, entry.literal))
                .collect(),
            token_selected_index: selected_style_token
                .map(|(index, _)| index as i32)
                .unwrap_or(-1),
            selected_token_name: selected_style_token
                .map(|(_, entry)| entry.name.clone())
                .unwrap_or_default(),
            selected_token_value: selected_style_token
                .map(|(_, entry)| entry.literal.clone())
                .unwrap_or_default(),
            can_edit_token: self.diagnostics.is_empty() && selected_style_token.is_some(),
            can_delete_token: self.diagnostics.is_empty() && selected_style_token.is_some(),
        }
    }
}
