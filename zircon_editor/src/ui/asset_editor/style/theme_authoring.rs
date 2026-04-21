use std::collections::{BTreeMap, BTreeSet};

use toml::Value;
use zircon_runtime::ui::template::{
    UiAssetDocument, UiAssetHeader, UiAssetImports, UiAssetKind, UiStyleDeclarationBlock,
    UiStyleRule, UiStyleSheet,
};

#[derive(Clone, Debug, PartialEq, Eq)]
pub(crate) struct UiAssetExternalStyleDraft {
    pub(crate) asset_id: String,
    pub(crate) document_id: String,
    pub(crate) display_name: String,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub(crate) enum UiAssetThemeRuleHelperAction {
    PromoteLocalTheme,
    AdoptActiveCascadeTokens {
        count: usize,
    },
    AdoptActiveCascadeRules {
        count: usize,
    },
    AdoptActiveCascadeChanges {
        token_count: usize,
        rule_count: usize,
    },
    AdoptActiveCascadeToken {
        reference: String,
        token_name: String,
        value_literal: String,
    },
    AdoptActiveCascadeRule {
        reference: String,
        stylesheet_id: String,
        selector: String,
    },
    DetachImportedThemeToLocal {
        reference: String,
    },
    CloneImportedThemeToLocal {
        reference: String,
    },
    AdoptComparedImportedDiffs {
        reference: String,
        count: usize,
    },
    PruneSharedComparedEntries {
        reference: String,
        count: usize,
    },
    AdoptAllImportedTokens {
        reference: String,
        count: usize,
    },
    AdoptAllImportedRules {
        reference: String,
        count: usize,
    },
    AdoptAllImportedChanges {
        reference: String,
        token_count: usize,
        rule_count: usize,
    },
    AdoptImportedToken {
        reference: String,
        token_name: String,
        value_literal: String,
    },
    AdoptImportedRule {
        reference: String,
        stylesheet_id: String,
        selector: String,
    },
    ApplyAllThemeRefactors {
        count: usize,
    },
    PruneDuplicateLocalOverrides,
}

impl UiAssetThemeRuleHelperAction {
    fn label(&self) -> String {
        match self {
            Self::PromoteLocalTheme => "Promote local theme to shared style asset".to_string(),
            Self::AdoptActiveCascadeTokens { count } => {
                format!("Adopt active cascade tokens into local layer ({count})")
            }
            Self::AdoptActiveCascadeRules { count } => {
                format!("Adopt active cascade rules into local layer ({count})")
            }
            Self::AdoptActiveCascadeChanges {
                token_count,
                rule_count,
            } => format!(
                "Adopt active cascade changes into local layer ({})",
                token_count + rule_count
            ),
            Self::AdoptActiveCascadeToken {
                token_name,
                value_literal,
                ..
            } => format!("Adopt active cascade token • {token_name} = {value_literal}"),
            Self::AdoptActiveCascadeRule {
                stylesheet_id,
                selector,
                ..
            } => format!("Adopt active cascade rule • {stylesheet_id} • {selector}"),
            Self::DetachImportedThemeToLocal { reference } => {
                format!("Detach {reference} into local theme layer")
            }
            Self::CloneImportedThemeToLocal { reference } => {
                format!("Clone {reference} into local theme layer")
            }
            Self::AdoptComparedImportedDiffs { count, .. } => {
                format!("Adopt compare diffs from selected theme ({count})")
            }
            Self::PruneSharedComparedEntries { count, .. } => {
                format!("Prune compare duplicates shared with selected theme ({count})")
            }
            Self::AdoptAllImportedTokens { count, .. } => {
                format!("Adopt all imported tokens ({count})")
            }
            Self::AdoptAllImportedRules { count, .. } => {
                format!("Adopt all imported rules ({count})")
            }
            Self::AdoptAllImportedChanges {
                token_count,
                rule_count,
                ..
            } => format!("Adopt all imported changes ({})", token_count + rule_count),
            Self::AdoptImportedToken {
                token_name,
                value_literal,
                ..
            } => format!("Adopt imported token • {token_name} = {value_literal}"),
            Self::AdoptImportedRule {
                stylesheet_id,
                selector,
                ..
            } => format!("Adopt imported rule • {stylesheet_id} • {selector}"),
            Self::ApplyAllThemeRefactors { count } => {
                format!("Apply all theme refactors ({count})")
            }
            Self::PruneDuplicateLocalOverrides => {
                "Prune duplicate local tokens and rules shadowed by imported themes".to_string()
            }
        }
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub(crate) enum UiAssetThemeRefactorAction {
    RemoveDuplicateLocalToken {
        token_name: String,
    },
    RemoveDuplicateLocalRule {
        stylesheet_id: String,
        selector: String,
    },
    RemoveRedundantImportedThemeReference {
        reference: String,
    },
}

impl UiAssetThemeRefactorAction {
    fn label(
        &self,
        document: &UiAssetDocument,
        imported_styles: &BTreeMap<String, UiAssetDocument>,
    ) -> String {
        match self {
            Self::RemoveDuplicateLocalToken { token_name } => format!(
                "duplicate local token • {token_name} • inherited = {}",
                imported_theme_tokens(document, imported_styles)
                    .get(token_name)
                    .map(Value::to_string)
                    .unwrap_or_default()
            ),
            Self::RemoveDuplicateLocalRule {
                stylesheet_id,
                selector,
            } => format!("duplicate local rule • {stylesheet_id} • {selector}"),
            Self::RemoveRedundantImportedThemeReference { reference } => {
                format!("redundant imported theme • {reference}")
            }
        }
    }
}

pub(crate) fn can_promote_local_theme_to_external_style_asset(document: &UiAssetDocument) -> bool {
    !document.tokens.is_empty() || !document.stylesheets.is_empty()
}

pub(crate) fn default_external_style_draft(
    source_asset_id: &str,
    source_display_name: &str,
) -> UiAssetExternalStyleDraft {
    let base_name = theme_base_name(source_asset_id);
    UiAssetExternalStyleDraft {
        asset_id: format!("res://ui/themes/{base_name}_theme.ui.toml"),
        document_id: format!("ui.theme.{base_name}_theme"),
        display_name: theme_display_name(source_display_name, &base_name),
    }
}

pub(crate) fn promote_local_theme_to_external_style_asset(
    document: &mut UiAssetDocument,
    style_asset_id: &str,
    style_document_id: &str,
    display_name: &str,
) -> Option<UiAssetDocument> {
    if !can_promote_local_theme_to_external_style_asset(document) {
        return None;
    }

    let promoted_theme = UiAssetDocument {
        asset: UiAssetHeader {
            kind: UiAssetKind::Style,
            id: style_document_id.to_string(),
            version: 1,
            display_name: display_name.to_string(),
        },
        imports: UiAssetImports {
            widgets: Vec::new(),
            styles: document.imports.styles.clone(),
        },
        tokens: std::mem::take(&mut document.tokens),
        root: None,
        components: Default::default(),
        stylesheets: std::mem::take(&mut document.stylesheets),
    };

    document.imports.styles.clear();
    document.imports.styles.push(style_asset_id.to_string());

    Some(promoted_theme)
}

pub(crate) fn detach_imported_theme_to_local_theme_layer(
    document: &mut UiAssetDocument,
    imported_reference: &str,
    imported_style_document: &UiAssetDocument,
) -> bool {
    merge_imported_theme_into_local_theme_layer(
        document,
        imported_reference,
        imported_style_document,
        false,
    )
}

pub(crate) fn clone_imported_theme_to_local_theme_layer(
    document: &mut UiAssetDocument,
    imported_reference: &str,
    imported_style_document: &UiAssetDocument,
) -> bool {
    merge_imported_theme_into_local_theme_layer(
        document,
        imported_reference,
        imported_style_document,
        true,
    )
}

pub(crate) fn build_imported_theme_local_merge_preview(
    document: &UiAssetDocument,
    imported_reference: &str,
    imported_style_document: &UiAssetDocument,
) -> Vec<String> {
    let mut preview_items = Vec::new();
    append_imported_theme_merge_preview(
        &mut preview_items,
        "Detach",
        document,
        imported_reference,
        imported_style_document,
        detach_imported_theme_to_local_theme_layer,
    );
    append_imported_theme_merge_preview(
        &mut preview_items,
        "Clone",
        document,
        imported_reference,
        imported_style_document,
        clone_imported_theme_to_local_theme_layer,
    );
    preview_items
}

pub(crate) fn build_theme_rule_helper_items(
    document: &UiAssetDocument,
    imported_styles: &BTreeMap<String, UiAssetDocument>,
    selected_key: Option<&str>,
) -> Vec<String> {
    theme_rule_helper_actions(document, imported_styles, selected_key)
        .into_iter()
        .map(|action| action.label())
        .collect()
}

pub(crate) fn build_theme_refactor_items(
    document: &UiAssetDocument,
    imported_styles: &BTreeMap<String, UiAssetDocument>,
) -> Vec<String> {
    theme_refactor_actions(document, imported_styles)
        .into_iter()
        .map(|action| action.label(document, imported_styles))
        .collect()
}

pub(crate) fn can_prune_duplicate_local_theme_overrides(
    document: &UiAssetDocument,
    imported_styles: &BTreeMap<String, UiAssetDocument>,
) -> bool {
    !build_theme_refactor_items(document, imported_styles).is_empty()
}

pub(crate) fn prune_duplicate_local_theme_overrides(
    document: &mut UiAssetDocument,
    imported_styles: &BTreeMap<String, UiAssetDocument>,
) -> bool {
    let imported_tokens = imported_theme_tokens(document, imported_styles);
    let imported_rules = imported_theme_rules(document, imported_styles);
    let mut changed = false;

    let duplicate_tokens = document
        .tokens
        .iter()
        .filter_map(|(token_name, local_value)| {
            (imported_tokens.get(token_name) == Some(local_value)).then(|| token_name.clone())
        })
        .collect::<Vec<_>>();
    for token_name in duplicate_tokens {
        changed |= document.tokens.remove(&token_name).is_some();
    }

    for stylesheet in &mut document.stylesheets {
        let before_len = stylesheet.rules.len();
        stylesheet
            .rules
            .retain(|rule| !imported_rules.contains(&rule_signature(rule)));
        changed |= stylesheet.rules.len() != before_len;
    }

    let before_stylesheet_len = document.stylesheets.len();
    document
        .stylesheets
        .retain(|stylesheet| !stylesheet.rules.is_empty());
    changed |= document.stylesheets.len() != before_stylesheet_len;
    changed
}

pub(crate) fn theme_rule_helper_actions(
    document: &UiAssetDocument,
    imported_styles: &BTreeMap<String, UiAssetDocument>,
    selected_key: Option<&str>,
) -> Vec<UiAssetThemeRuleHelperAction> {
    let mut actions = Vec::new();
    let refactor_count = theme_refactor_actions(document, imported_styles).len();
    let selected_key = selected_key
        .or_else(|| can_promote_local_theme_to_external_style_asset(document).then_some("local"));
    if can_promote_local_theme_to_external_style_asset(document) && selected_key == Some("local") {
        actions.push(UiAssetThemeRuleHelperAction::PromoteLocalTheme);
        let token_actions = build_active_cascade_token_actions(document, imported_styles);
        let rule_actions = build_active_cascade_rule_actions(document, imported_styles);
        if !token_actions.is_empty() {
            actions.push(UiAssetThemeRuleHelperAction::AdoptActiveCascadeTokens {
                count: token_actions.len(),
            });
        }
        if !rule_actions.is_empty() {
            actions.push(UiAssetThemeRuleHelperAction::AdoptActiveCascadeRules {
                count: rule_actions.len(),
            });
        }
        if token_actions.len() + rule_actions.len() > 1 {
            actions.push(UiAssetThemeRuleHelperAction::AdoptActiveCascadeChanges {
                token_count: token_actions.len(),
                rule_count: rule_actions.len(),
            });
        }
        actions.extend(token_actions);
        actions.extend(rule_actions);
    }
    if let Some(reference) = selected_key.filter(|key| *key != "local") {
        if let Some(imported_style_document) = imported_styles.get(reference) {
            let token_actions = build_adopt_imported_theme_token_actions(
                document,
                reference,
                imported_style_document,
            );
            let rule_actions = build_adopt_imported_theme_rule_actions(
                document,
                reference,
                imported_style_document,
            );
            let compare_diff_count =
                compare_adoptable_imported_theme_entry_count(document, imported_style_document);
            let compare_prune_count =
                compare_prunable_local_theme_entry_count(document, imported_style_document);
            actions.push(UiAssetThemeRuleHelperAction::DetachImportedThemeToLocal {
                reference: reference.to_string(),
            });
            actions.push(UiAssetThemeRuleHelperAction::CloneImportedThemeToLocal {
                reference: reference.to_string(),
            });
            if compare_diff_count > 0 {
                actions.push(UiAssetThemeRuleHelperAction::AdoptComparedImportedDiffs {
                    reference: reference.to_string(),
                    count: compare_diff_count,
                });
            }
            if compare_prune_count > 0 {
                actions.push(UiAssetThemeRuleHelperAction::PruneSharedComparedEntries {
                    reference: reference.to_string(),
                    count: compare_prune_count,
                });
            }
            if !token_actions.is_empty() {
                actions.push(UiAssetThemeRuleHelperAction::AdoptAllImportedTokens {
                    reference: reference.to_string(),
                    count: token_actions.len(),
                });
            }
            if !rule_actions.is_empty() {
                actions.push(UiAssetThemeRuleHelperAction::AdoptAllImportedRules {
                    reference: reference.to_string(),
                    count: rule_actions.len(),
                });
            }
            if token_actions.len() + rule_actions.len() > 1 {
                actions.push(UiAssetThemeRuleHelperAction::AdoptAllImportedChanges {
                    reference: reference.to_string(),
                    token_count: token_actions.len(),
                    rule_count: rule_actions.len(),
                });
            }
            actions.extend(token_actions);
            actions.extend(rule_actions);
        }
    }
    if refactor_count > 0 {
        actions.push(UiAssetThemeRuleHelperAction::ApplyAllThemeRefactors {
            count: refactor_count,
        });
    }
    if can_prune_duplicate_local_theme_overrides(document, imported_styles) {
        actions.push(UiAssetThemeRuleHelperAction::PruneDuplicateLocalOverrides);
    }
    actions
}

pub(crate) fn adopt_imported_theme_token(
    document: &mut UiAssetDocument,
    imported_styles: &BTreeMap<String, UiAssetDocument>,
    reference: &str,
    token_name: &str,
) -> bool {
    let Some(imported_style_document) = imported_styles.get(reference) else {
        return false;
    };
    let Some(imported_value) = imported_style_document.tokens.get(token_name) else {
        return false;
    };
    if document.tokens.get(token_name) == Some(imported_value) {
        return false;
    }
    document
        .tokens
        .insert(token_name.to_string(), imported_value.clone());
    true
}

pub(crate) fn adopt_imported_theme_rule(
    document: &mut UiAssetDocument,
    imported_styles: &BTreeMap<String, UiAssetDocument>,
    reference: &str,
    stylesheet_id: &str,
    selector: &str,
) -> bool {
    let Some(imported_style_document) = imported_styles.get(reference) else {
        return false;
    };
    let Some(imported_stylesheet) = imported_style_document
        .stylesheets
        .iter()
        .find(|stylesheet| stylesheet_label(stylesheet) == stylesheet_id)
    else {
        return false;
    };
    let Some(imported_rule) = imported_stylesheet
        .rules
        .iter()
        .find(|rule| rule.selector == selector)
    else {
        return false;
    };

    if let Some(stylesheet) = document
        .stylesheets
        .iter_mut()
        .find(|stylesheet| stylesheet.id == imported_stylesheet.id)
    {
        if let Some(rule) = stylesheet
            .rules
            .iter_mut()
            .find(|rule| rule.selector == selector)
        {
            if rule.set == imported_rule.set {
                return false;
            }
            rule.set = imported_rule.set.clone();
            return true;
        }
        stylesheet.rules.push(imported_rule.clone());
        return true;
    }

    document.stylesheets.push(UiStyleSheet {
        id: imported_stylesheet.id.clone(),
        rules: vec![imported_rule.clone()],
    });
    true
}

pub(crate) fn adopt_imported_theme_tokens(
    document: &mut UiAssetDocument,
    imported_styles: &BTreeMap<String, UiAssetDocument>,
    reference: &str,
) -> usize {
    let Some(imported_style_document) = imported_styles.get(reference) else {
        return 0;
    };

    let mut adopted = 0usize;
    for (token_name, imported_value) in &imported_style_document.tokens {
        if document.tokens.get(token_name) == Some(imported_value) {
            continue;
        }
        document
            .tokens
            .insert(token_name.clone(), imported_value.clone());
        adopted += 1;
    }
    adopted
}

pub(crate) fn adopt_imported_theme_rules(
    document: &mut UiAssetDocument,
    imported_styles: &BTreeMap<String, UiAssetDocument>,
    reference: &str,
) -> usize {
    let Some(imported_style_document) = imported_styles.get(reference) else {
        return 0;
    };

    let mut adopted = 0usize;
    for stylesheet in &imported_style_document.stylesheets {
        let stylesheet_id = stylesheet_label(stylesheet);
        for rule in &stylesheet.rules {
            adopted += usize::from(adopt_imported_theme_rule(
                document,
                imported_styles,
                reference,
                &stylesheet_id,
                &rule.selector,
            ));
        }
    }
    adopted
}

pub(crate) fn adopt_all_imported_theme_changes(
    document: &mut UiAssetDocument,
    imported_styles: &BTreeMap<String, UiAssetDocument>,
    reference: &str,
) -> usize {
    adopt_imported_theme_tokens(document, imported_styles, reference)
        + adopt_imported_theme_rules(document, imported_styles, reference)
}

pub(crate) fn adopt_imported_theme_compare_diffs(
    document: &mut UiAssetDocument,
    imported_styles: &BTreeMap<String, UiAssetDocument>,
    reference: &str,
) -> usize {
    let Some(imported_style_document) = imported_styles.get(reference) else {
        return 0;
    };

    let local_rule_blocks = local_rule_blocks(document);
    let mut adopted = 0usize;
    for (token_name, imported_value) in &imported_style_document.tokens {
        if document.tokens.get(token_name) == Some(imported_value) {
            continue;
        }
        adopted += usize::from(adopt_imported_theme_token(
            document,
            imported_styles,
            reference,
            token_name,
        ));
    }
    for stylesheet in &imported_style_document.stylesheets {
        let stylesheet_id = stylesheet_label(stylesheet);
        for rule in &stylesheet.rules {
            let differs = local_rule_blocks
                .get(&rule.selector)
                .map(|local_block| local_block != &rule.set)
                .unwrap_or(true);
            if differs {
                adopted += usize::from(adopt_imported_theme_rule(
                    document,
                    imported_styles,
                    reference,
                    &stylesheet_id,
                    &rule.selector,
                ));
            }
        }
    }
    adopted
}

pub(crate) fn prune_imported_theme_compare_duplicates(
    document: &mut UiAssetDocument,
    imported_styles: &BTreeMap<String, UiAssetDocument>,
    reference: &str,
) -> usize {
    let Some(imported_style_document) = imported_styles.get(reference) else {
        return 0;
    };

    let actions = imported_theme_compare_duplicate_refactors(document, imported_style_document);
    let count = actions.len();
    for action in actions {
        let _ = apply_theme_refactor_action(document, &action);
    }
    count
}

pub(crate) fn adopt_active_cascade_token(
    document: &mut UiAssetDocument,
    imported_styles: &BTreeMap<String, UiAssetDocument>,
    token_name: &str,
) -> bool {
    let Some(entry) = active_cascade_tokens(document, imported_styles)
        .into_iter()
        .find(|entry| entry.token_name == token_name)
    else {
        return false;
    };
    adopt_imported_theme_token(document, imported_styles, &entry.reference, token_name)
}

pub(crate) fn adopt_active_cascade_tokens(
    document: &mut UiAssetDocument,
    imported_styles: &BTreeMap<String, UiAssetDocument>,
) -> usize {
    active_cascade_tokens(document, imported_styles)
        .into_iter()
        .map(|entry| {
            usize::from(adopt_imported_theme_token(
                document,
                imported_styles,
                &entry.reference,
                &entry.token_name,
            ))
        })
        .sum()
}

pub(crate) fn adopt_active_cascade_rule(
    document: &mut UiAssetDocument,
    imported_styles: &BTreeMap<String, UiAssetDocument>,
    stylesheet_id: &str,
    selector: &str,
) -> bool {
    let Some(entry) = active_cascade_rules(document, imported_styles)
        .into_iter()
        .find(|entry| entry.stylesheet_id == stylesheet_id && entry.selector == selector)
    else {
        return false;
    };
    adopt_imported_theme_rule(
        document,
        imported_styles,
        &entry.reference,
        stylesheet_id,
        selector,
    )
}

pub(crate) fn adopt_active_cascade_rules(
    document: &mut UiAssetDocument,
    imported_styles: &BTreeMap<String, UiAssetDocument>,
) -> usize {
    active_cascade_rules(document, imported_styles)
        .into_iter()
        .map(|entry| {
            usize::from(adopt_imported_theme_rule(
                document,
                imported_styles,
                &entry.reference,
                &entry.stylesheet_id,
                &entry.selector,
            ))
        })
        .sum()
}

pub(crate) fn adopt_all_active_cascade_changes(
    document: &mut UiAssetDocument,
    imported_styles: &BTreeMap<String, UiAssetDocument>,
) -> usize {
    adopt_active_cascade_tokens(document, imported_styles)
        + adopt_active_cascade_rules(document, imported_styles)
}

pub(crate) fn theme_refactor_actions(
    document: &UiAssetDocument,
    imported_styles: &BTreeMap<String, UiAssetDocument>,
) -> Vec<UiAssetThemeRefactorAction> {
    let imported_tokens = imported_theme_tokens(document, imported_styles);
    let imported_rules = imported_theme_rules(document, imported_styles);
    let mut actions = Vec::new();

    for (token_name, local_value) in &document.tokens {
        if imported_tokens.get(token_name) == Some(local_value) {
            actions.push(UiAssetThemeRefactorAction::RemoveDuplicateLocalToken {
                token_name: token_name.clone(),
            });
        }
    }

    for stylesheet in &document.stylesheets {
        let stylesheet_label = stylesheet_label(stylesheet);
        for rule in &stylesheet.rules {
            if imported_rules.contains(&rule_signature(rule)) {
                actions.push(UiAssetThemeRefactorAction::RemoveDuplicateLocalRule {
                    stylesheet_id: stylesheet_label.clone(),
                    selector: rule.selector.clone(),
                });
            }
        }
    }

    for reference in &document.imports.styles {
        if imported_styles.contains_key(reference)
            && imported_theme_is_fully_cloned_locally(document, imported_styles, reference)
        {
            actions.push(
                UiAssetThemeRefactorAction::RemoveRedundantImportedThemeReference {
                    reference: reference.clone(),
                },
            );
        }
    }

    actions
}

pub(crate) fn apply_theme_refactor_action(
    document: &mut UiAssetDocument,
    action: &UiAssetThemeRefactorAction,
) -> bool {
    match action {
        UiAssetThemeRefactorAction::RemoveDuplicateLocalToken { token_name } => {
            document.tokens.remove(token_name).is_some()
        }
        UiAssetThemeRefactorAction::RemoveDuplicateLocalRule {
            stylesheet_id,
            selector,
        } => {
            let mut changed = false;
            for stylesheet in &mut document.stylesheets {
                if stylesheet_label(stylesheet) != *stylesheet_id {
                    continue;
                }
                let before_len = stylesheet.rules.len();
                stylesheet.rules.retain(|rule| rule.selector != *selector);
                changed |= stylesheet.rules.len() != before_len;
            }
            if !changed {
                return false;
            }
            let before_stylesheet_len = document.stylesheets.len();
            document
                .stylesheets
                .retain(|stylesheet| !stylesheet.rules.is_empty());
            changed || document.stylesheets.len() != before_stylesheet_len
        }
        UiAssetThemeRefactorAction::RemoveRedundantImportedThemeReference { reference } => {
            let before_len = document.imports.styles.len();
            document
                .imports
                .styles
                .retain(|candidate| candidate != reference);
            document.imports.styles.len() != before_len
        }
    }
}

fn imported_theme_is_fully_cloned_locally(
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

fn build_adopt_imported_theme_token_actions(
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

fn compare_adoptable_imported_theme_entry_count(
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

fn compare_prunable_local_theme_entry_count(
    document: &UiAssetDocument,
    imported_style_document: &UiAssetDocument,
) -> usize {
    imported_theme_compare_duplicate_refactors(document, imported_style_document).len()
}

fn build_active_cascade_token_actions(
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

fn build_active_cascade_rule_actions(
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

fn build_adopt_imported_theme_rule_actions(
    document: &UiAssetDocument,
    reference: &str,
    imported_style_document: &UiAssetDocument,
) -> Vec<UiAssetThemeRuleHelperAction> {
    let mut actions = Vec::new();

    for stylesheet in &imported_style_document.stylesheets {
        let stylesheet_id = stylesheet_label(stylesheet);
        for rule in &stylesheet.rules {
            let local_rule = document
                .stylesheets
                .iter()
                .find(|candidate| candidate.id == stylesheet.id)
                .and_then(|candidate| {
                    candidate
                        .rules
                        .iter()
                        .find(|local_rule| local_rule.selector == rule.selector)
                });
            if local_rule.is_some_and(|local_rule| local_rule.set == rule.set) {
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

fn local_rule_blocks(document: &UiAssetDocument) -> BTreeMap<String, UiStyleDeclarationBlock> {
    let mut rules = BTreeMap::new();
    for stylesheet in &document.stylesheets {
        for rule in &stylesheet.rules {
            rules.insert(rule.selector.clone(), rule.set.clone());
        }
    }
    rules
}

fn imported_theme_compare_duplicate_refactors(
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
struct UiAssetActiveCascadeTokenEntry {
    reference: String,
    token_name: String,
    value_literal: String,
}

#[derive(Clone, Debug)]
struct UiAssetActiveCascadeRuleEntry {
    reference: String,
    stylesheet_id: String,
    selector: String,
}

fn active_cascade_tokens(
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

fn active_cascade_rules(
    document: &UiAssetDocument,
    imported_styles: &BTreeMap<String, UiAssetDocument>,
) -> Vec<UiAssetActiveCascadeRuleEntry> {
    let mut entries = BTreeMap::<String, UiAssetActiveCascadeRuleEntry>::new();
    for reference in &document.imports.styles {
        let Some(imported_style_document) = imported_styles.get(reference) else {
            continue;
        };
        for stylesheet in &imported_style_document.stylesheets {
            let stylesheet_id = stylesheet_label(stylesheet);
            for rule in &stylesheet.rules {
                let local_rule = document
                    .stylesheets
                    .iter()
                    .find(|candidate| stylesheet_label(candidate) == stylesheet_id)
                    .and_then(|candidate| {
                        candidate
                            .rules
                            .iter()
                            .find(|local_rule| local_rule.selector == rule.selector)
                    });
                if local_rule.is_some_and(|local_rule| local_rule.set == rule.set) {
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

fn resolve_local_clone_token_renames(
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

fn find_local_cloned_stylesheet<'a>(
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

fn merge_imported_theme_into_local_theme_layer(
    document: &mut UiAssetDocument,
    imported_reference: &str,
    imported_style_document: &UiAssetDocument,
    keep_import_reference: bool,
) -> bool {
    let Some(import_index) = document
        .imports
        .styles
        .iter()
        .position(|reference| reference == imported_reference)
    else {
        return false;
    };

    let source_prefix = theme_base_name(imported_reference);
    let token_renames = build_imported_token_rename_map(
        &document.tokens,
        &imported_style_document.tokens,
        &source_prefix,
    );
    merge_detached_style_imports(
        document,
        imported_reference,
        imported_style_document,
        import_index,
        keep_import_reference,
    );
    merge_detached_tokens(document, imported_style_document, &token_renames);
    merge_detached_stylesheets(
        &mut document.stylesheets,
        &imported_style_document.stylesheets,
        &token_renames,
        &source_prefix,
    );
    true
}

fn imported_theme_tokens(
    document: &UiAssetDocument,
    imported_styles: &BTreeMap<String, UiAssetDocument>,
) -> BTreeMap<String, Value> {
    let mut tokens = BTreeMap::new();
    for reference in &document.imports.styles {
        let Some(imported) = imported_styles.get(reference) else {
            continue;
        };
        for (token_name, value) in &imported.tokens {
            tokens
                .entry(token_name.clone())
                .or_insert_with(|| value.clone());
        }
    }
    tokens
}

fn imported_theme_rules(
    document: &UiAssetDocument,
    imported_styles: &BTreeMap<String, UiAssetDocument>,
) -> BTreeSet<String> {
    let mut rules = BTreeSet::new();
    for reference in &document.imports.styles {
        let Some(imported) = imported_styles.get(reference) else {
            continue;
        };
        for stylesheet in &imported.stylesheets {
            for rule in &stylesheet.rules {
                let _ = rules.insert(rule_signature(rule));
            }
        }
    }
    rules
}

fn stylesheet_label(stylesheet: &UiStyleSheet) -> String {
    if stylesheet.id.is_empty() {
        "<inline>".to_string()
    } else {
        stylesheet.id.clone()
    }
}

fn rule_signature(rule: &UiStyleRule) -> String {
    format!(
        "{}|{}",
        rule.selector,
        toml::to_string(&rule.set).unwrap_or_default()
    )
}

fn append_imported_theme_merge_preview(
    preview_items: &mut Vec<String>,
    mode_label: &str,
    document: &UiAssetDocument,
    imported_reference: &str,
    imported_style_document: &UiAssetDocument,
    apply_merge: fn(&mut UiAssetDocument, &str, &UiAssetDocument) -> bool,
) {
    let mut merged_document = document.clone();
    if !apply_merge(
        &mut merged_document,
        imported_reference,
        imported_style_document,
    ) {
        return;
    }

    preview_items.push(format!(
        "{mode_label} • imports • {}",
        if merged_document.imports.styles.is_empty() {
            "none".to_string()
        } else {
            merged_document.imports.styles.join(", ")
        }
    ));
    for (token_name, token_value) in &merged_document.tokens {
        preview_items.push(format!(
            "{mode_label} • token • {token_name} = {token_value}"
        ));
    }
    for stylesheet in &merged_document.stylesheets {
        for rule in &stylesheet.rules {
            let rule_label = if stylesheet.id.is_empty() {
                rule.selector.clone()
            } else {
                format!("{} • {}", stylesheet.id, rule.selector)
            };
            preview_items.push(format!("{mode_label} • rule • {rule_label}"));
        }
    }
}

fn merge_detached_style_imports(
    document: &mut UiAssetDocument,
    imported_reference: &str,
    imported_style_document: &UiAssetDocument,
    import_index: usize,
    keep_import_reference: bool,
) {
    if !keep_import_reference {
        document.imports.styles.remove(import_index);
    }
    let mut seen = document
        .imports
        .styles
        .iter()
        .cloned()
        .collect::<BTreeSet<_>>();
    let mut insert_index = if keep_import_reference {
        (import_index + 1).min(document.imports.styles.len())
    } else {
        import_index.min(document.imports.styles.len())
    };
    for nested_reference in &imported_style_document.imports.styles {
        if nested_reference == imported_reference || !seen.insert(nested_reference.clone()) {
            continue;
        }
        document
            .imports
            .styles
            .insert(insert_index, nested_reference.clone());
        insert_index += 1;
    }
}

fn merge_detached_tokens(
    document: &mut UiAssetDocument,
    imported_style_document: &UiAssetDocument,
    token_renames: &BTreeMap<String, String>,
) {
    for (token_name, token_value) in &imported_style_document.tokens {
        let mut rewritten_value = token_value.clone();
        rewrite_token_references_in_value(&mut rewritten_value, token_renames);
        let merged_name = token_renames
            .get(token_name)
            .cloned()
            .unwrap_or_else(|| token_name.clone());
        document.tokens.insert(merged_name, rewritten_value);
    }
}

fn merge_detached_stylesheets(
    local_stylesheets: &mut Vec<UiStyleSheet>,
    imported_stylesheets: &[UiStyleSheet],
    token_renames: &BTreeMap<String, String>,
    source_prefix: &str,
) {
    let mut used_ids = local_stylesheets
        .iter()
        .filter_map(|stylesheet| (!stylesheet.id.is_empty()).then_some(stylesheet.id.clone()))
        .collect::<BTreeSet<_>>();
    let mut merged_stylesheets =
        Vec::with_capacity(imported_stylesheets.len() + local_stylesheets.len());
    for stylesheet in imported_stylesheets.iter().cloned() {
        let mut stylesheet = stylesheet;
        rewrite_stylesheet_token_references(&mut stylesheet, token_renames);
        if !stylesheet.id.is_empty() {
            let original_id = stylesheet.id.clone();
            if used_ids.contains(&original_id) {
                stylesheet.id =
                    unique_identifier(&used_ids, &format!("{source_prefix}_{original_id}"));
            }
            used_ids.insert(stylesheet.id.clone());
        }
        merged_stylesheets.push(stylesheet);
    }
    merged_stylesheets.extend(std::mem::take(local_stylesheets));
    *local_stylesheets = merged_stylesheets;
}

fn build_imported_token_rename_map(
    existing_tokens: &BTreeMap<String, Value>,
    imported_tokens: &BTreeMap<String, Value>,
    source_prefix: &str,
) -> BTreeMap<String, String> {
    let mut used_names = existing_tokens.keys().cloned().collect::<BTreeSet<_>>();
    let mut rename_map = BTreeMap::new();
    for token_name in imported_tokens.keys() {
        let merged_name = if used_names.contains(token_name) {
            unique_identifier(&used_names, &format!("{source_prefix}_{token_name}"))
        } else {
            token_name.clone()
        };
        used_names.insert(merged_name.clone());
        rename_map.insert(token_name.clone(), merged_name);
    }
    rename_map
}

fn rewrite_stylesheet_token_references(
    stylesheet: &mut UiStyleSheet,
    token_renames: &BTreeMap<String, String>,
) {
    for rule in &mut stylesheet.rules {
        rewrite_declaration_block_token_references(&mut rule.set, token_renames);
    }
}

fn rewrite_declaration_block_token_references(
    declarations: &mut UiStyleDeclarationBlock,
    token_renames: &BTreeMap<String, String>,
) {
    for value in declarations.self_values.values_mut() {
        rewrite_token_references_in_value(value, token_renames);
    }
    for value in declarations.slot.values_mut() {
        rewrite_token_references_in_value(value, token_renames);
    }
}

fn rewrite_token_references_in_value(value: &mut Value, token_renames: &BTreeMap<String, String>) {
    match value {
        Value::String(text) => {
            let Some(token_name) = text.strip_prefix('$') else {
                return;
            };
            let Some(renamed) = token_renames.get(token_name) else {
                return;
            };
            *text = format!("${renamed}");
        }
        Value::Array(items) => {
            for item in items {
                rewrite_token_references_in_value(item, token_renames);
            }
        }
        Value::Table(table) => {
            for (_key, nested_value) in table.iter_mut() {
                rewrite_token_references_in_value(nested_value, token_renames);
            }
        }
        _ => {}
    }
}

fn unique_identifier(used_names: &BTreeSet<String>, base_name: &str) -> String {
    if !used_names.contains(base_name) {
        return base_name.to_string();
    }
    let mut suffix = 2usize;
    loop {
        let candidate = format!("{base_name}_{suffix}");
        if !used_names.contains(&candidate) {
            return candidate;
        }
        suffix += 1;
    }
}

fn theme_base_name(source_asset_id: &str) -> String {
    let normalized = source_asset_id.replace('\\', "/");
    let file_name = normalized.rsplit('/').next().unwrap_or("theme");
    let stem = file_name
        .strip_suffix(".ui.toml")
        .or_else(|| file_name.strip_suffix(".toml"))
        .unwrap_or(file_name);
    let stem = stem.strip_suffix(".ui").unwrap_or(stem);
    let mut normalized_name = String::new();
    let mut previous_was_separator = false;
    for ch in stem.chars() {
        if ch.is_ascii_alphanumeric() {
            normalized_name.push(ch.to_ascii_lowercase());
            previous_was_separator = false;
        } else if !previous_was_separator && !normalized_name.is_empty() {
            normalized_name.push('_');
            previous_was_separator = true;
        }
    }
    let normalized = normalized_name.trim_matches('_');
    if normalized.is_empty() {
        "theme".to_string()
    } else {
        normalized.to_string()
    }
}

fn theme_display_name(source_display_name: &str, base_name: &str) -> String {
    let trimmed = source_display_name.trim();
    if trimmed.is_empty() {
        let mut title = String::new();
        let mut capitalize_next = true;
        for ch in base_name.chars() {
            if ch == '_' || ch == '-' || ch == '.' {
                if !title.ends_with(' ') {
                    title.push(' ');
                }
                capitalize_next = true;
            } else if capitalize_next {
                title.push(ch.to_ascii_uppercase());
                capitalize_next = false;
            } else {
                title.push(ch);
            }
        }
        let title = title.trim().to_string();
        if title.is_empty() {
            "Theme".to_string()
        } else {
            format!("{title} Theme")
        }
    } else if trimmed.ends_with("Theme") {
        trimmed.to_string()
    } else {
        format!("{trimmed} Theme")
    }
}
