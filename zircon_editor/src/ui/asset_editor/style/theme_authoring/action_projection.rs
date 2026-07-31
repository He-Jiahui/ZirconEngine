use std::collections::{BTreeMap, BTreeSet};

use toml::Value;
use zircon_runtime_interface::ui::template::{
    UiAssetDocument, UiStyleDeclarationBlock, UiStyleSheet,
};

use super::merge::{
    rewrite_stylesheet_token_references, rule_signature, stylesheet_label, theme_base_name,
};
use super::{UiAssetThemeRefactorAction, UiAssetThemeRuleHelperAction};

pub(super) fn imported_theme_is_fully_cloned_locally(
    document: &UiAssetDocument,
    imported_styles: &BTreeMap<String, UiAssetDocument>,
    reference: &str,
) -> bool {
    let Some(imported_style_document) = imported_styles.get(reference) else {
        return false;
    };
    let source_prefix = theme_base_name(reference);
    let Some(token_renames) = resolve_local_clone_token_renames(
        &document.tokens,
        &imported_style_document.tokens,
        &source_prefix,
    ) else {
        return false;
    };

    for nested_reference in &imported_style_document.imports.styles {
        if nested_reference != reference && !document.imports.styles.contains(nested_reference) {
            return false;
        }
    }

    imported_style_document
        .stylesheets
        .iter()
        .all(|imported_stylesheet| {
            let Some(local_stylesheet) = find_local_cloned_stylesheet(
                &document.stylesheets,
                imported_stylesheet,
                &source_prefix,
            ) else {
                return false;
            };
            let mut rewritten_stylesheet = imported_stylesheet.clone();
            rewrite_stylesheet_token_references(&mut rewritten_stylesheet, &token_renames);
            rewritten_stylesheet.rules == local_stylesheet.rules
        })
}

pub(super) fn build_adopt_imported_theme_token_actions(
    document: &UiAssetDocument,
    reference: &str,
    imported_style_document: &UiAssetDocument,
) -> Vec<UiAssetThemeRuleHelperAction> {
    let mut actions = Vec::new();

    for (token_name, imported_value) in &imported_style_document.tokens {
        if document.tokens.get(token_name) != Some(imported_value) {
            actions.push(UiAssetThemeRuleHelperAction::AdoptImportedToken {
                reference: reference.to_string(),
                token_name: token_name.clone(),
                value_literal: imported_value.to_string(),
            });
        }
    }

    actions
}

pub(super) fn compare_adoptable_imported_theme_entry_count(
    document: &UiAssetDocument,
    imported_style_document: &UiAssetDocument,
) -> usize {
    let local_rule_blocks = local_rule_blocks(document);
    let token_count = imported_style_document
        .tokens
        .iter()
        .filter(|(token_name, imported_value)| {
            document.tokens.get(*token_name) != Some(*imported_value)
        })
        .count();
    let rule_count = imported_style_document
        .stylesheets
        .iter()
        .flat_map(|stylesheet| stylesheet.rules.iter())
        .filter(|rule| {
            local_rule_blocks
                .get(&rule.selector)
                .map(|local_block| local_block != &rule.set)
                .unwrap_or(true)
        })
        .count();
    token_count + rule_count
}

pub(super) fn compare_prunable_local_theme_entry_count(
    document: &UiAssetDocument,
    imported_style_document: &UiAssetDocument,
) -> usize {
    imported_theme_compare_duplicate_refactors(document, imported_style_document).len()
}

pub(super) fn build_active_cascade_token_actions(
    document: &UiAssetDocument,
    imported_styles: &BTreeMap<String, UiAssetDocument>,
) -> Vec<UiAssetThemeRuleHelperAction> {
    active_cascade_tokens(document, imported_styles)
        .into_iter()
        .map(
            |entry| UiAssetThemeRuleHelperAction::AdoptActiveCascadeToken {
                reference: entry.reference,
                token_name: entry.token_name,
                value_literal: entry.value_literal,
            },
        )
        .collect()
}

pub(super) fn build_active_cascade_rule_actions(
    document: &UiAssetDocument,
    imported_styles: &BTreeMap<String, UiAssetDocument>,
) -> Vec<UiAssetThemeRuleHelperAction> {
    active_cascade_rules(document, imported_styles)
        .into_iter()
        .map(
            |entry| UiAssetThemeRuleHelperAction::AdoptActiveCascadeRule {
                reference: entry.reference,
                stylesheet_id: entry.stylesheet_id,
                selector: entry.selector,
            },
        )
        .collect()
}

pub(super) fn build_adopt_imported_theme_rule_actions(
    document: &UiAssetDocument,
    reference: &str,
    imported_style_document: &UiAssetDocument,
) -> Vec<UiAssetThemeRuleHelperAction> {
    let mut actions = Vec::new();
    let local_rules = local_rule_index(document);

    for stylesheet in &imported_style_document.stylesheets {
        let stylesheet_id = stylesheet_label(stylesheet);
        for rule in &stylesheet.rules {
            if local_rules
                .get(&(stylesheet.id.as_str(), rule.selector.as_str()))
                .is_some_and(|local_block| *local_block == &rule.set)
            {
                continue;
            }
            actions.push(UiAssetThemeRuleHelperAction::AdoptImportedRule {
                reference: reference.to_string(),
                stylesheet_id: stylesheet_id.clone(),
                selector: rule.selector.clone(),
            });
        }
    }

    actions
}

fn local_rule_index<'a>(
    document: &'a UiAssetDocument,
) -> BTreeMap<(&'a str, &'a str), &'a UiStyleDeclarationBlock> {
    let mut rules = BTreeMap::new();
    for stylesheet in &document.stylesheets {
        for rule in &stylesheet.rules {
            rules
                .entry((stylesheet.id.as_str(), rule.selector.as_str()))
                .or_insert(&rule.set);
        }
    }
    rules
}

pub(super) fn local_rule_blocks(
    document: &UiAssetDocument,
) -> BTreeMap<String, UiStyleDeclarationBlock> {
    let mut rules = BTreeMap::new();
    for stylesheet in &document.stylesheets {
        for rule in &stylesheet.rules {
            rules.insert(rule.selector.clone(), rule.set.clone());
        }
    }
    rules
}

pub(super) fn imported_theme_compare_duplicate_refactors(
    document: &UiAssetDocument,
    imported_style_document: &UiAssetDocument,
) -> Vec<UiAssetThemeRefactorAction> {
    let mut actions = Vec::new();
    for (token_name, local_value) in &document.tokens {
        if imported_style_document.tokens.get(token_name) == Some(local_value) {
            actions.push(UiAssetThemeRefactorAction::RemoveDuplicateLocalToken {
                token_name: token_name.clone(),
            });
        }
    }

    let imported_rules = imported_style_document
        .stylesheets
        .iter()
        .flat_map(|stylesheet| stylesheet.rules.iter())
        .map(rule_signature)
        .collect::<BTreeSet<_>>();
    for stylesheet in &document.stylesheets {
        let stylesheet_id = stylesheet_label(stylesheet);
        for rule in &stylesheet.rules {
            if imported_rules.contains(&rule_signature(rule)) {
                actions.push(UiAssetThemeRefactorAction::RemoveDuplicateLocalRule {
                    stylesheet_id: stylesheet_id.clone(),
                    selector: rule.selector.clone(),
                });
            }
        }
    }

    actions
}

#[derive(Clone, Debug)]
pub(super) struct UiAssetActiveCascadeTokenEntry {
    pub(super) reference: String,
    pub(super) token_name: String,
    pub(super) value_literal: String,
}

#[derive(Clone, Debug)]
pub(super) struct UiAssetActiveCascadeRuleEntry {
    pub(super) reference: String,
    pub(super) stylesheet_id: String,
    pub(super) selector: String,
}

pub(super) fn active_cascade_tokens(
    document: &UiAssetDocument,
    imported_styles: &BTreeMap<String, UiAssetDocument>,
) -> Vec<UiAssetActiveCascadeTokenEntry> {
    let mut entries = BTreeMap::<String, UiAssetActiveCascadeTokenEntry>::new();
    for reference in &document.imports.styles {
        let Some(imported_style_document) = imported_styles.get(reference) else {
            continue;
        };
        for (token_name, imported_value) in &imported_style_document.tokens {
            if document.tokens.get(token_name) == Some(imported_value) {
                continue;
            }
            entries.insert(
                token_name.clone(),
                UiAssetActiveCascadeTokenEntry {
                    reference: reference.clone(),
                    token_name: token_name.clone(),
                    value_literal: imported_value.to_string(),
                },
            );
        }
    }
    entries.into_values().collect()
}

pub(super) fn active_cascade_rules(
    document: &UiAssetDocument,
    imported_styles: &BTreeMap<String, UiAssetDocument>,
) -> Vec<UiAssetActiveCascadeRuleEntry> {
    let mut entries = BTreeMap::<String, UiAssetActiveCascadeRuleEntry>::new();
    let local_rules = local_rule_index(document);
    for reference in &document.imports.styles {
        let Some(imported_style_document) = imported_styles.get(reference) else {
            continue;
        };
        for stylesheet in &imported_style_document.stylesheets {
            let stylesheet_id = stylesheet_label(stylesheet);
            for rule in &stylesheet.rules {
                if local_rules
                    .get(&(stylesheet.id.as_str(), rule.selector.as_str()))
                    .is_some_and(|local_block| *local_block == &rule.set)
                {
                    continue;
                }
                entries.insert(
                    format!("{stylesheet_id}|{}", rule.selector),
                    UiAssetActiveCascadeRuleEntry {
                        reference: reference.clone(),
                        stylesheet_id: stylesheet_id.clone(),
                        selector: rule.selector.clone(),
                    },
                );
            }
        }
    }
    entries.into_values().collect()
}

pub(super) fn resolve_local_clone_token_renames(
    local_tokens: &BTreeMap<String, Value>,
    imported_tokens: &BTreeMap<String, Value>,
    source_prefix: &str,
) -> Option<BTreeMap<String, String>> {
    let mut renames = BTreeMap::new();
    for token_name in imported_tokens.keys() {
        if local_tokens.contains_key(token_name) {
            renames.insert(token_name.clone(), token_name.clone());
            continue;
        }

        let prefixed_base = format!("{source_prefix}_{token_name}");
        let prefixed = local_tokens
            .keys()
            .find(|candidate| {
                candidate.as_str() == prefixed_base
                    || candidate.starts_with(&(prefixed_base.clone() + "_"))
            })
            .cloned()?;
        renames.insert(token_name.clone(), prefixed);
    }
    Some(renames)
}

pub(super) fn find_local_cloned_stylesheet<'a>(
    local_stylesheets: &'a [UiStyleSheet],
    imported_stylesheet: &UiStyleSheet,
    source_prefix: &str,
) -> Option<&'a UiStyleSheet> {
    let preferred_id = (!imported_stylesheet.id.is_empty()).then(|| imported_stylesheet.id.clone());
    let prefixed_id = (!imported_stylesheet.id.is_empty())
        .then(|| format!("{source_prefix}_{}", imported_stylesheet.id));

    local_stylesheets.iter().find(|stylesheet| {
        if let Some(preferred_id) = preferred_id.as_deref() {
            if stylesheet.id == preferred_id {
                return true;
            }
        }
        if let Some(prefixed_id) = prefixed_id.as_deref() {
            stylesheet.id == prefixed_id
                || stylesheet.id.starts_with(&(prefixed_id.to_string() + "_"))
        } else {
            false
        }
    })
}
