use std::collections::{BTreeMap, BTreeSet};

use super::{
    command::{
        UiAssetEditorCommand, UiAssetEditorDocumentReplayBundle,
        UiAssetEditorDocumentReplayCommand, UiAssetEditorTreeEditKind,
    },
    promotion_state::reference_asset_id,
    theme_authoring::{
        adopt_active_cascade_rule, adopt_active_cascade_rules, adopt_active_cascade_token,
        adopt_active_cascade_tokens, adopt_all_active_cascade_changes,
        adopt_all_imported_theme_changes, adopt_imported_theme_compare_diffs,
        adopt_imported_theme_rule, adopt_imported_theme_rules, adopt_imported_theme_token,
        adopt_imported_theme_tokens, apply_theme_refactor_action,
        clone_imported_theme_to_local_theme_layer, detach_imported_theme_to_local_theme_layer,
        prune_duplicate_local_theme_overrides, prune_imported_theme_compare_duplicates,
        theme_refactor_actions, theme_rule_helper_actions, UiAssetThemeRefactorAction,
        UiAssetThemeRuleHelperAction,
    },
    theme_summary::reconcile_selected_theme_source_key,
    ui_asset_editor_session::{
        serialize_document, UiAssetEditorSession, UiAssetEditorSessionError,
    },
    undo_stack::UiAssetEditorUndoExternalEffects,
};
use zircon_runtime::ui::template::{UiAssetDocument, UiStyleRule, UiStyleSheet};

impl UiAssetEditorSession {
    pub fn select_theme_source(&mut self, index: usize) -> Result<bool, UiAssetEditorSessionError> {
        let Some(key) = super::theme_summary::select_theme_source_key(
            &self.last_valid_document,
            &self.compiler_imports.styles,
            index,
        ) else {
            return Err(UiAssetEditorSessionError::InvalidSelectionIndex { index });
        };
        let changed = self.selected_theme_source_key.as_deref() != Some(key.as_str());
        self.selected_theme_source_key = Some(key);
        Ok(changed)
    }

    pub fn selected_theme_source_asset_id(&self) -> Option<String> {
        let selected_key = reconcile_selected_theme_source_key(
            &self.last_valid_document,
            &self.compiler_imports.styles,
            self.selected_theme_source_key.as_deref(),
        )?;
        if selected_key == "local" || !self.compiler_imports.styles.contains_key(&selected_key) {
            return None;
        }
        Some(reference_asset_id(&selected_key).to_string())
    }

    pub fn detach_selected_theme_source_to_local(
        &mut self,
    ) -> Result<bool, UiAssetEditorSessionError> {
        self.ensure_editable_source()?;
        let Some(selected_key) = reconcile_selected_theme_source_key(
            &self.last_valid_document,
            &self.compiler_imports.styles,
            self.selected_theme_source_key.as_deref(),
        ) else {
            return Ok(false);
        };
        if selected_key == "local" {
            return Ok(false);
        }
        let Some(imported_style_document) =
            self.compiler_imports.styles.get(&selected_key).cloned()
        else {
            return Ok(false);
        };

        let mut document = self.last_valid_document.clone();
        if !detach_imported_theme_to_local_theme_layer(
            &mut document,
            &selected_key,
            &imported_style_document,
        ) {
            return Ok(false);
        }

        let replay = theme_document_replay_bundle(&self.last_valid_document, &document);
        self.apply_document_edit_with_label_and_replay(document, "Detach Imported Theme", replay)?;
        Ok(true)
    }

    pub fn clone_selected_theme_source_to_local(
        &mut self,
    ) -> Result<bool, UiAssetEditorSessionError> {
        self.ensure_editable_source()?;
        let Some(selected_key) = reconcile_selected_theme_source_key(
            &self.last_valid_document,
            &self.compiler_imports.styles,
            self.selected_theme_source_key.as_deref(),
        ) else {
            return Ok(false);
        };
        if selected_key == "local" {
            return Ok(false);
        }
        let Some(imported_style_document) =
            self.compiler_imports.styles.get(&selected_key).cloned()
        else {
            return Ok(false);
        };

        let mut document = self.last_valid_document.clone();
        if !clone_imported_theme_to_local_theme_layer(
            &mut document,
            &selected_key,
            &imported_style_document,
        ) {
            return Ok(false);
        }

        let replay = theme_document_replay_bundle(&self.last_valid_document, &document);
        self.apply_command_with_effects_and_theme_source(
            UiAssetEditorCommand::tree_edit(
                UiAssetEditorTreeEditKind::DocumentEdit,
                "Clone Imported Theme",
                serialize_document(&document)?,
            )
            .with_document_replay(replay),
            UiAssetEditorUndoExternalEffects::default(),
            Some("local".to_string()),
        )?;
        Ok(true)
    }

    pub(crate) fn theme_rule_helper_action(
        &self,
        index: usize,
    ) -> Option<UiAssetThemeRuleHelperAction> {
        theme_rule_helper_actions(
            &self.last_valid_document,
            &self.compiler_imports.styles,
            self.selected_theme_source_key.as_deref(),
        )
        .get(index)
        .cloned()
    }

    pub fn apply_theme_rule_helper_item(
        &mut self,
        index: usize,
    ) -> Result<bool, UiAssetEditorSessionError> {
        let Some(action) = self.theme_rule_helper_action(index) else {
            return Err(UiAssetEditorSessionError::InvalidSelectionIndex { index });
        };
        match action {
            UiAssetThemeRuleHelperAction::PromoteLocalTheme => {
                let Some(draft) = self.selected_promote_theme_draft() else {
                    return Ok(false);
                };
                self.promote_local_theme_to_external_style_asset(
                    &draft.asset_id,
                    &draft.document_id,
                    &draft.display_name,
                )
                .map(|changed| changed.is_some())
            }
            UiAssetThemeRuleHelperAction::AdoptActiveCascadeTokens { .. } => {
                self.adopt_active_cascade_tokens()
            }
            UiAssetThemeRuleHelperAction::AdoptActiveCascadeRules { .. } => {
                self.adopt_active_cascade_rules()
            }
            UiAssetThemeRuleHelperAction::AdoptActiveCascadeChanges { .. } => {
                self.adopt_all_active_cascade_changes()
            }
            UiAssetThemeRuleHelperAction::AdoptActiveCascadeToken { token_name, .. } => {
                self.adopt_active_cascade_token(&token_name)
            }
            UiAssetThemeRuleHelperAction::AdoptActiveCascadeRule {
                stylesheet_id,
                selector,
                ..
            } => self.adopt_active_cascade_rule(&stylesheet_id, &selector),
            UiAssetThemeRuleHelperAction::DetachImportedThemeToLocal { .. } => {
                self.detach_selected_theme_source_to_local()
            }
            UiAssetThemeRuleHelperAction::CloneImportedThemeToLocal { .. } => {
                self.clone_selected_theme_source_to_local()
            }
            UiAssetThemeRuleHelperAction::AdoptComparedImportedDiffs { reference, .. } => {
                self.adopt_imported_theme_compare_diffs(&reference)
            }
            UiAssetThemeRuleHelperAction::PruneSharedComparedEntries { reference, .. } => {
                self.prune_imported_theme_compare_duplicates(&reference)
            }
            UiAssetThemeRuleHelperAction::AdoptAllImportedTokens { reference, .. } => {
                self.adopt_imported_theme_tokens(&reference)
            }
            UiAssetThemeRuleHelperAction::AdoptAllImportedRules { reference, .. } => {
                self.adopt_imported_theme_rules(&reference)
            }
            UiAssetThemeRuleHelperAction::AdoptAllImportedChanges { reference, .. } => {
                self.adopt_all_imported_theme_changes(&reference)
            }
            UiAssetThemeRuleHelperAction::AdoptImportedToken {
                reference,
                token_name,
                ..
            } => self.adopt_imported_theme_token(&reference, &token_name),
            UiAssetThemeRuleHelperAction::AdoptImportedRule {
                reference,
                stylesheet_id,
                selector,
            } => self.adopt_imported_theme_rule(&reference, &stylesheet_id, &selector),
            UiAssetThemeRuleHelperAction::ApplyAllThemeRefactors { .. } => {
                self.apply_all_theme_refactors()
            }
            UiAssetThemeRuleHelperAction::PruneDuplicateLocalOverrides => {
                self.prune_duplicate_local_theme_overrides()
            }
        }
    }

    pub(crate) fn theme_refactor_action(&self, index: usize) -> Option<UiAssetThemeRefactorAction> {
        theme_refactor_actions(&self.last_valid_document, &self.compiler_imports.styles)
            .get(index)
            .cloned()
    }

    pub fn apply_theme_refactor_item(
        &mut self,
        index: usize,
    ) -> Result<bool, UiAssetEditorSessionError> {
        self.ensure_editable_source()?;
        let Some(action) = self.theme_refactor_action(index) else {
            return Err(UiAssetEditorSessionError::InvalidSelectionIndex { index });
        };
        let mut document = self.last_valid_document.clone();
        if !apply_theme_refactor_action(&mut document, &action) {
            return Ok(false);
        }
        let replay = theme_document_replay_bundle(&self.last_valid_document, &document);
        self.apply_document_edit_with_label_and_replay(document, "Apply Theme Refactor", replay)?;
        Ok(true)
    }

    pub fn apply_all_theme_refactors(&mut self) -> Result<bool, UiAssetEditorSessionError> {
        self.ensure_editable_source()?;
        let actions =
            theme_refactor_actions(&self.last_valid_document, &self.compiler_imports.styles);
        if actions.is_empty() {
            return Ok(false);
        }
        let mut document = self.last_valid_document.clone();
        let mut changed = false;
        for action in &actions {
            changed |= apply_theme_refactor_action(&mut document, action);
        }
        if !changed {
            return Ok(false);
        }
        let replay = theme_document_replay_bundle(&self.last_valid_document, &document);
        self.apply_document_edit_with_label_and_replay(
            document,
            "Apply All Theme Refactors",
            replay,
        )?;
        Ok(true)
    }

    fn adopt_imported_theme_token(
        &mut self,
        reference: &str,
        token_name: &str,
    ) -> Result<bool, UiAssetEditorSessionError> {
        self.ensure_editable_source()?;
        let mut document = self.last_valid_document.clone();
        if !adopt_imported_theme_token(
            &mut document,
            &self.compiler_imports.styles,
            reference,
            token_name,
        ) {
            return Ok(false);
        }
        let replay = theme_document_replay_bundle(&self.last_valid_document, &document);
        self.apply_document_edit_with_label_and_replay(
            document,
            "Adopt Imported Theme Token",
            replay,
        )?;
        Ok(true)
    }

    fn adopt_active_cascade_token(
        &mut self,
        token_name: &str,
    ) -> Result<bool, UiAssetEditorSessionError> {
        self.ensure_editable_source()?;
        let mut document = self.last_valid_document.clone();
        if !adopt_active_cascade_token(&mut document, &self.compiler_imports.styles, token_name) {
            return Ok(false);
        }
        let replay = theme_document_replay_bundle(&self.last_valid_document, &document);
        self.apply_document_edit_with_label_and_replay(
            document,
            "Adopt Active Cascade Theme Token",
            replay,
        )?;
        Ok(true)
    }

    fn adopt_imported_theme_tokens(
        &mut self,
        reference: &str,
    ) -> Result<bool, UiAssetEditorSessionError> {
        self.ensure_editable_source()?;
        let mut document = self.last_valid_document.clone();
        if adopt_imported_theme_tokens(&mut document, &self.compiler_imports.styles, reference) == 0
        {
            return Ok(false);
        }
        let replay = theme_document_replay_bundle(&self.last_valid_document, &document);
        self.apply_document_edit_with_label_and_replay(
            document,
            "Adopt Imported Theme Tokens",
            replay,
        )?;
        Ok(true)
    }

    fn adopt_imported_theme_compare_diffs(
        &mut self,
        reference: &str,
    ) -> Result<bool, UiAssetEditorSessionError> {
        self.ensure_editable_source()?;
        let mut document = self.last_valid_document.clone();
        if adopt_imported_theme_compare_diffs(
            &mut document,
            &self.compiler_imports.styles,
            reference,
        ) == 0
        {
            return Ok(false);
        }
        let replay = theme_document_replay_bundle(&self.last_valid_document, &document);
        self.apply_document_edit_with_label_and_replay(
            document,
            "Adopt Imported Theme Compare Diffs",
            replay,
        )?;
        Ok(true)
    }

    fn prune_imported_theme_compare_duplicates(
        &mut self,
        reference: &str,
    ) -> Result<bool, UiAssetEditorSessionError> {
        self.ensure_editable_source()?;
        let mut document = self.last_valid_document.clone();
        if prune_imported_theme_compare_duplicates(
            &mut document,
            &self.compiler_imports.styles,
            reference,
        ) == 0
        {
            return Ok(false);
        }
        let replay = theme_document_replay_bundle(&self.last_valid_document, &document);
        self.apply_document_edit_with_label_and_replay(
            document,
            "Prune Imported Theme Compare Duplicates",
            replay,
        )?;
        Ok(true)
    }

    fn adopt_active_cascade_tokens(&mut self) -> Result<bool, UiAssetEditorSessionError> {
        self.ensure_editable_source()?;
        let mut document = self.last_valid_document.clone();
        if adopt_active_cascade_tokens(&mut document, &self.compiler_imports.styles) == 0 {
            return Ok(false);
        }
        let replay = theme_document_replay_bundle(&self.last_valid_document, &document);
        self.apply_document_edit_with_label_and_replay(
            document,
            "Adopt Active Cascade Theme Tokens",
            replay,
        )?;
        Ok(true)
    }

    fn adopt_imported_theme_rule(
        &mut self,
        reference: &str,
        stylesheet_id: &str,
        selector: &str,
    ) -> Result<bool, UiAssetEditorSessionError> {
        self.ensure_editable_source()?;
        let mut document = self.last_valid_document.clone();
        if !adopt_imported_theme_rule(
            &mut document,
            &self.compiler_imports.styles,
            reference,
            stylesheet_id,
            selector,
        ) {
            return Ok(false);
        }
        let replay = theme_document_replay_bundle(&self.last_valid_document, &document);
        self.apply_document_edit_with_label_and_replay(
            document,
            "Adopt Imported Theme Rule",
            replay,
        )?;
        Ok(true)
    }

    fn adopt_active_cascade_rule(
        &mut self,
        stylesheet_id: &str,
        selector: &str,
    ) -> Result<bool, UiAssetEditorSessionError> {
        self.ensure_editable_source()?;
        let mut document = self.last_valid_document.clone();
        if !adopt_active_cascade_rule(
            &mut document,
            &self.compiler_imports.styles,
            stylesheet_id,
            selector,
        ) {
            return Ok(false);
        }
        let replay = theme_document_replay_bundle(&self.last_valid_document, &document);
        self.apply_document_edit_with_label_and_replay(
            document,
            "Adopt Active Cascade Theme Rule",
            replay,
        )?;
        Ok(true)
    }

    fn adopt_imported_theme_rules(
        &mut self,
        reference: &str,
    ) -> Result<bool, UiAssetEditorSessionError> {
        self.ensure_editable_source()?;
        let mut document = self.last_valid_document.clone();
        if adopt_imported_theme_rules(&mut document, &self.compiler_imports.styles, reference) == 0
        {
            return Ok(false);
        }
        let replay = theme_document_replay_bundle(&self.last_valid_document, &document);
        self.apply_document_edit_with_label_and_replay(
            document,
            "Adopt Imported Theme Rules",
            replay,
        )?;
        Ok(true)
    }

    fn adopt_active_cascade_rules(&mut self) -> Result<bool, UiAssetEditorSessionError> {
        self.ensure_editable_source()?;
        let mut document = self.last_valid_document.clone();
        if adopt_active_cascade_rules(&mut document, &self.compiler_imports.styles) == 0 {
            return Ok(false);
        }
        let replay = theme_document_replay_bundle(&self.last_valid_document, &document);
        self.apply_document_edit_with_label_and_replay(
            document,
            "Adopt Active Cascade Theme Rules",
            replay,
        )?;
        Ok(true)
    }

    fn adopt_all_imported_theme_changes(
        &mut self,
        reference: &str,
    ) -> Result<bool, UiAssetEditorSessionError> {
        self.ensure_editable_source()?;
        let mut document = self.last_valid_document.clone();
        if adopt_all_imported_theme_changes(&mut document, &self.compiler_imports.styles, reference)
            == 0
        {
            return Ok(false);
        }
        let replay = theme_document_replay_bundle(&self.last_valid_document, &document);
        self.apply_document_edit_with_label_and_replay(
            document,
            "Adopt Imported Theme Changes",
            replay,
        )?;
        Ok(true)
    }

    fn adopt_all_active_cascade_changes(&mut self) -> Result<bool, UiAssetEditorSessionError> {
        self.ensure_editable_source()?;
        let mut document = self.last_valid_document.clone();
        if adopt_all_active_cascade_changes(&mut document, &self.compiler_imports.styles) == 0 {
            return Ok(false);
        }
        let replay = theme_document_replay_bundle(&self.last_valid_document, &document);
        self.apply_document_edit_with_label_and_replay(
            document,
            "Adopt Active Cascade Theme Changes",
            replay,
        )?;
        Ok(true)
    }

    pub fn prune_duplicate_local_theme_overrides(
        &mut self,
    ) -> Result<bool, UiAssetEditorSessionError> {
        self.ensure_editable_source()?;
        let mut document = self.last_valid_document.clone();
        if !prune_duplicate_local_theme_overrides(&mut document, &self.compiler_imports.styles) {
            return Ok(false);
        }
        let replay = theme_document_replay_bundle(&self.last_valid_document, &document);
        self.apply_document_edit_with_label_and_replay(
            document,
            "Prune Duplicate Local Theme Overrides",
            replay,
        )?;
        Ok(true)
    }
}

pub(super) fn theme_document_replay_bundle(
    before_document: &UiAssetDocument,
    after_document: &UiAssetDocument,
) -> UiAssetEditorDocumentReplayBundle {
    UiAssetEditorDocumentReplayBundle {
        undo: theme_document_replay_commands(after_document, before_document),
        redo: theme_document_replay_commands(before_document, after_document),
    }
}

fn theme_document_replay_commands(
    current: &UiAssetDocument,
    target: &UiAssetDocument,
) -> Vec<UiAssetEditorDocumentReplayCommand> {
    let mut commands = Vec::new();
    commands.extend(build_style_import_replay_commands(
        &current.imports.styles,
        &target.imports.styles,
    ));
    commands.extend(build_style_token_replay_commands(
        &current.tokens,
        &target.tokens,
    ));
    commands.extend(build_stylesheet_replay_commands(
        &current.stylesheets,
        &target.stylesheets,
    ));
    commands
}

fn build_style_import_replay_commands(
    current: &[String],
    target: &[String],
) -> Vec<UiAssetEditorDocumentReplayCommand> {
    if current == target {
        return Vec::new();
    }
    if has_duplicate_string_entries(current) || has_duplicate_string_entries(target) {
        return vec![UiAssetEditorDocumentReplayCommand::SetStyleImports {
            references: target.to_vec(),
        }];
    }

    let target_entries = target.iter().cloned().collect::<BTreeSet<_>>();
    let mut working = current.to_vec();
    let mut commands = Vec::new();

    for index in (0..working.len()).rev() {
        if target_entries.contains(&working[index]) {
            continue;
        }
        let reference = working.remove(index);
        commands.push(UiAssetEditorDocumentReplayCommand::RemoveStyleImport { index, reference });
    }

    for (target_index, target_reference) in target.iter().enumerate() {
        match working
            .iter()
            .position(|reference| reference == target_reference)
        {
            Some(current_index) => {
                if current_index != target_index {
                    let moved = working.remove(current_index);
                    working.insert(target_index, moved);
                    commands.push(UiAssetEditorDocumentReplayCommand::MoveStyleImport {
                        from_index: current_index,
                        to_index: target_index,
                        reference: target_reference.clone(),
                    });
                }
            }
            None => {
                working.insert(target_index, target_reference.clone());
                commands.push(UiAssetEditorDocumentReplayCommand::InsertStyleImport {
                    index: target_index,
                    reference: target_reference.clone(),
                });
            }
        }
    }

    if working != target {
        return vec![UiAssetEditorDocumentReplayCommand::SetStyleImports {
            references: target.to_vec(),
        }];
    }

    commands
}

fn build_style_token_replay_commands(
    current: &BTreeMap<String, toml::Value>,
    target: &BTreeMap<String, toml::Value>,
) -> Vec<UiAssetEditorDocumentReplayCommand> {
    if current == target {
        return Vec::new();
    }

    let mut commands = Vec::new();
    for token_name in current.keys().rev() {
        if target.contains_key(token_name) {
            continue;
        }
        commands.push(UiAssetEditorDocumentReplayCommand::RemoveStyleToken {
            token_name: token_name.clone(),
        });
    }

    for (token_name, target_value) in target {
        if current.get(token_name) == Some(target_value) {
            continue;
        }
        commands.push(UiAssetEditorDocumentReplayCommand::UpsertStyleToken {
            token_name: token_name.clone(),
            value: target_value.clone(),
        });
    }

    commands
}

fn build_stylesheet_replay_commands(
    current: &[UiStyleSheet],
    target: &[UiStyleSheet],
) -> Vec<UiAssetEditorDocumentReplayCommand> {
    if current == target {
        return Vec::new();
    }
    if has_duplicate_stylesheet_ids(current) || has_duplicate_stylesheet_ids(target) {
        return vec![UiAssetEditorDocumentReplayCommand::SetStyleSheets {
            stylesheets: target.to_vec(),
        }];
    }

    let target_ids = target
        .iter()
        .map(|stylesheet| stylesheet.id.clone())
        .collect::<BTreeSet<_>>();
    let mut working = current.to_vec();
    let mut commands = Vec::new();

    for index in (0..working.len()).rev() {
        if target_ids.contains(working[index].id.as_str()) {
            continue;
        }
        let stylesheet_id = working[index].id.clone();
        let _ = working.remove(index);
        commands.push(UiAssetEditorDocumentReplayCommand::RemoveStyleSheet {
            index,
            stylesheet_id,
        });
    }

    for (target_index, target_stylesheet) in target.iter().enumerate() {
        let current_index = working
            .iter()
            .position(|stylesheet| stylesheet.id == target_stylesheet.id);
        match current_index {
            Some(current_index) => {
                if current_index != target_index {
                    let moved = working.remove(current_index);
                    working.insert(target_index, moved);
                    commands.push(UiAssetEditorDocumentReplayCommand::MoveStyleSheet {
                        from_index: current_index,
                        to_index: target_index,
                        stylesheet_id: target_stylesheet.id.clone(),
                    });
                }
                if working
                    .get(target_index)
                    .is_some_and(|stylesheet| stylesheet != target_stylesheet)
                {
                    if let Some(rule_commands) = build_style_rule_replay_commands(
                        target_index,
                        &working[target_index].rules,
                        &target_stylesheet.rules,
                    ) {
                        working[target_index].rules = target_stylesheet.rules.clone();
                        commands.extend(rule_commands);
                    } else {
                        working[target_index] = target_stylesheet.clone();
                        commands.push(UiAssetEditorDocumentReplayCommand::ReplaceStyleSheet {
                            index: target_index,
                            stylesheet_id: target_stylesheet.id.clone(),
                            stylesheet: target_stylesheet.clone(),
                        });
                    }
                }
            }
            None => {
                working.insert(target_index, target_stylesheet.clone());
                commands.push(UiAssetEditorDocumentReplayCommand::InsertStyleSheet {
                    index: target_index,
                    stylesheet_id: target_stylesheet.id.clone(),
                    stylesheet: Some(target_stylesheet.clone()),
                });
            }
        }
    }

    if working != target {
        return vec![UiAssetEditorDocumentReplayCommand::SetStyleSheets {
            stylesheets: target.to_vec(),
        }];
    }

    commands
}

fn build_style_rule_replay_commands(
    stylesheet_index: usize,
    current: &[UiStyleRule],
    target: &[UiStyleRule],
) -> Option<Vec<UiAssetEditorDocumentReplayCommand>> {
    if current == target {
        return Some(Vec::new());
    }
    if has_duplicate_rule_selectors(current) || has_duplicate_rule_selectors(target) {
        return None;
    }

    let target_selectors = target
        .iter()
        .map(|rule| rule.selector.clone())
        .collect::<BTreeSet<_>>();
    let mut working = current.to_vec();
    let mut commands = Vec::new();

    for index in (0..working.len()).rev() {
        if target_selectors.contains(working[index].selector.as_str()) {
            continue;
        }
        let selector = working[index].selector.clone();
        let _ = working.remove(index);
        commands.push(UiAssetEditorDocumentReplayCommand::RemoveStyleRule {
            stylesheet_index,
            index,
            selector,
        });
    }

    for (target_index, target_rule) in target.iter().enumerate() {
        let current_index = working
            .iter()
            .position(|rule| rule.selector == target_rule.selector);
        match current_index {
            Some(current_index) => {
                if current_index != target_index {
                    let moved = working.remove(current_index);
                    working.insert(target_index, moved);
                    commands.push(UiAssetEditorDocumentReplayCommand::MoveStyleRule {
                        stylesheet_index,
                        from_index: current_index,
                        to_index: target_index,
                    });
                }
                if working
                    .get(target_index)
                    .is_some_and(|rule| rule != target_rule)
                {
                    let selector = target_rule.selector.clone();
                    let _ = working.remove(target_index);
                    commands.push(UiAssetEditorDocumentReplayCommand::RemoveStyleRule {
                        stylesheet_index,
                        index: target_index,
                        selector: selector.clone(),
                    });
                    working.insert(target_index, target_rule.clone());
                    commands.push(UiAssetEditorDocumentReplayCommand::InsertStyleRule {
                        stylesheet_index,
                        index: target_index,
                        selector,
                        rule: Some(target_rule.clone()),
                    });
                }
            }
            None => {
                working.insert(target_index, target_rule.clone());
                commands.push(UiAssetEditorDocumentReplayCommand::InsertStyleRule {
                    stylesheet_index,
                    index: target_index,
                    selector: target_rule.selector.clone(),
                    rule: Some(target_rule.clone()),
                });
            }
        }
    }

    (working == target).then_some(commands)
}

fn has_duplicate_stylesheet_ids(stylesheets: &[UiStyleSheet]) -> bool {
    let mut seen = BTreeSet::new();
    stylesheets
        .iter()
        .map(|stylesheet| stylesheet.id.as_str())
        .any(|stylesheet_id| !seen.insert(stylesheet_id))
}

fn has_duplicate_rule_selectors(rules: &[UiStyleRule]) -> bool {
    let mut seen = BTreeSet::new();
    rules
        .iter()
        .map(|rule| rule.selector.as_str())
        .any(|selector| !seen.insert(selector))
}

fn has_duplicate_string_entries(entries: &[String]) -> bool {
    let mut seen = BTreeSet::new();
    entries
        .iter()
        .map(String::as_str)
        .any(|entry| !seen.insert(entry))
}
