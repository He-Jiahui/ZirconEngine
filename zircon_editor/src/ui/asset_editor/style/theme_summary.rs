use std::collections::BTreeMap;

use zircon_runtime::ui::template::UiAssetDocument;

use super::theme_authoring::can_promote_local_theme_to_external_style_asset;

#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub(crate) struct UiAssetThemeSummary {
    pub items: Vec<String>,
    pub selected_index: i32,
    pub selected_reference: String,
    pub selected_kind: String,
    pub selected_token_count: i32,
    pub selected_rule_count: i32,
    pub selected_available: bool,
    pub can_promote_local: bool,
}

#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub(crate) struct UiAssetThemeSourceDetails {
    pub token_items: Vec<String>,
    pub rule_items: Vec<String>,
}

#[derive(Clone, Debug, PartialEq, Eq)]
struct UiAssetThemeSourceEntry {
    key: String,
    label: String,
    reference: String,
    kind: &'static str,
    token_count: usize,
    rule_count: usize,
    available: bool,
}

pub(crate) fn build_theme_summary(
    document: &UiAssetDocument,
    imported_styles: &BTreeMap<String, UiAssetDocument>,
    selected_key: Option<&str>,
) -> UiAssetThemeSummary {
    let entries = theme_source_entries(document, imported_styles);
    let selected_index =
        reconcile_selected_theme_source_key(document, imported_styles, selected_key)
            .and_then(|key| entries.iter().position(|entry| entry.key == key))
            .or_else(|| (!entries.is_empty()).then_some(0));
    let Some(selected_index) = selected_index else {
        return UiAssetThemeSummary {
            can_promote_local: can_promote_local_theme_to_external_style_asset(document),
            ..UiAssetThemeSummary::default()
        };
    };
    let selected = &entries[selected_index];
    UiAssetThemeSummary {
        items: entries.iter().map(|entry| entry.label.clone()).collect(),
        selected_index: selected_index as i32,
        selected_reference: selected.reference.clone(),
        selected_kind: selected.kind.to_string(),
        selected_token_count: selected.token_count as i32,
        selected_rule_count: selected.rule_count as i32,
        selected_available: selected.available,
        can_promote_local: can_promote_local_theme_to_external_style_asset(document),
    }
}

pub(crate) fn build_theme_source_details(
    document: &UiAssetDocument,
    imported_styles: &BTreeMap<String, UiAssetDocument>,
    selected_key: Option<&str>,
) -> UiAssetThemeSourceDetails {
    selected_theme_document(document, imported_styles, selected_key)
        .map(theme_source_details_for_document)
        .unwrap_or_default()
}

pub(crate) fn select_theme_source_key(
    document: &UiAssetDocument,
    imported_styles: &BTreeMap<String, UiAssetDocument>,
    index: usize,
) -> Option<String> {
    theme_source_entries(document, imported_styles)
        .get(index)
        .map(|entry| entry.key.clone())
}

pub(crate) fn reconcile_selected_theme_source_key(
    document: &UiAssetDocument,
    imported_styles: &BTreeMap<String, UiAssetDocument>,
    current: Option<&str>,
) -> Option<String> {
    let entries = theme_source_entries(document, imported_styles);
    current
        .and_then(|key| entries.iter().find(|entry| entry.key == key))
        .map(|entry| entry.key.clone())
        .or_else(|| entries.first().map(|entry| entry.key.clone()))
}

fn theme_source_entries(
    document: &UiAssetDocument,
    imported_styles: &BTreeMap<String, UiAssetDocument>,
) -> Vec<UiAssetThemeSourceEntry> {
    let mut entries = Vec::new();
    if can_promote_local_theme_to_external_style_asset(document) {
        let token_count = document.tokens.len();
        let rule_count = total_rule_count(document);
        entries.push(UiAssetThemeSourceEntry {
            key: "local".to_string(),
            label: format!("Local Theme • {token_count} tokens • {rule_count} rules"),
            reference: "local".to_string(),
            kind: "Local",
            token_count,
            rule_count,
            available: true,
        });
    }

    for reference in &document.imports.styles {
        let (token_count, rule_count, available) = imported_styles
            .get(reference)
            .map(|style_document| {
                (
                    style_document.tokens.len(),
                    total_rule_count(style_document),
                    true,
                )
            })
            .unwrap_or((0, 0, false));
        let label = if available {
            format!("{reference} • {token_count} tokens • {rule_count} rules")
        } else {
            format!("{reference} • missing")
        };
        entries.push(UiAssetThemeSourceEntry {
            key: reference.clone(),
            label,
            reference: reference.clone(),
            kind: "Imported",
            token_count,
            rule_count,
            available,
        });
    }

    entries
}

fn selected_theme_document<'a>(
    document: &'a UiAssetDocument,
    imported_styles: &'a BTreeMap<String, UiAssetDocument>,
    selected_key: Option<&str>,
) -> Option<&'a UiAssetDocument> {
    let key = reconcile_selected_theme_source_key(document, imported_styles, selected_key)?;
    if key == "local" {
        Some(document)
    } else {
        imported_styles.get(&key)
    }
}

fn theme_source_details_for_document(document: &UiAssetDocument) -> UiAssetThemeSourceDetails {
    UiAssetThemeSourceDetails {
        token_items: document
            .tokens
            .iter()
            .map(|(name, value)| format!("{name} = {value}"))
            .collect(),
        rule_items: document
            .stylesheets
            .iter()
            .flat_map(|stylesheet| {
                stylesheet.rules.iter().map(|rule| {
                    if stylesheet.id.is_empty() {
                        rule.selector.clone()
                    } else {
                        format!("{} • {}", stylesheet.id, rule.selector)
                    }
                })
            })
            .collect(),
    }
}

fn total_rule_count(document: &UiAssetDocument) -> usize {
    document
        .stylesheets
        .iter()
        .map(|stylesheet| stylesheet.rules.len())
        .sum()
}
