use std::collections::BTreeMap;

use toml::Value;
use zircon_runtime_interface::ui::template::{
    UiAssetDocument, UiAssetHeader, UiAssetImports, UiAssetKind, UiStyleSheet,
};

mod action_projection;
mod merge;

use action_projection::{
    active_cascade_rules, active_cascade_tokens, build_active_cascade_rule_actions,
    build_active_cascade_token_actions, build_adopt_imported_theme_rule_actions,
    build_adopt_imported_theme_token_actions, compare_adoptable_imported_theme_entry_count,
    compare_prunable_local_theme_entry_count, imported_theme_compare_duplicate_refactors,
    imported_theme_is_fully_cloned_locally, local_rule_blocks,
};
use merge::{
    append_imported_theme_merge_preview, imported_theme_rules, imported_theme_tokens,
    merge_imported_theme_into_local_theme_layer, rule_signature, stylesheet_label, theme_base_name,
    theme_display_name,
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
            resources: Vec::new(),
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
