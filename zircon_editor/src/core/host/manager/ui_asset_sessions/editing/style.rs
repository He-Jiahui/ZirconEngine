use super::*;

impl EditorManager {
    pub fn select_ui_asset_editor_theme_source(
        &self,
        instance_id: &ViewInstanceId,
        index: usize,
    ) -> Result<bool, EditorError> {
        self.ensure_ui_asset_editor_session(instance_id)?;
        let mut sessions = self.ui_asset_sessions.lock().unwrap();
        let entry = sessions.get_mut(instance_id).ok_or_else(|| {
            EditorError::UiAsset(format!("missing ui asset session {}", instance_id.0))
        })?;
        let changed = entry
            .session
            .select_theme_source(index)
            .map_err(|error| EditorError::UiAsset(error.to_string()))?;
        drop(sessions);
        self.sync_ui_asset_editor_instance(instance_id)?;
        Ok(changed)
    }

    pub fn detach_ui_asset_editor_selected_theme_source_to_local(
        &self,
        instance_id: &ViewInstanceId,
    ) -> Result<bool, EditorError> {
        self.ensure_ui_asset_editor_session(instance_id)?;
        let mut sessions = self.ui_asset_sessions.lock().unwrap();
        let entry = sessions.get_mut(instance_id).ok_or_else(|| {
            EditorError::UiAsset(format!("missing ui asset session {}", instance_id.0))
        })?;
        let changed = entry
            .session
            .detach_selected_theme_source_to_local()
            .map_err(|error| EditorError::UiAsset(error.to_string()))?;
        drop(sessions);
        if changed {
            self.hydrate_ui_asset_editor_imports(instance_id)?;
            self.sync_ui_asset_editor_instance(instance_id)?;
        }
        Ok(changed)
    }

    pub fn clone_ui_asset_editor_selected_theme_source_to_local(
        &self,
        instance_id: &ViewInstanceId,
    ) -> Result<bool, EditorError> {
        self.ensure_ui_asset_editor_session(instance_id)?;
        let mut sessions = self.ui_asset_sessions.lock().unwrap();
        let entry = sessions.get_mut(instance_id).ok_or_else(|| {
            EditorError::UiAsset(format!("missing ui asset session {}", instance_id.0))
        })?;
        let changed = entry
            .session
            .clone_selected_theme_source_to_local()
            .map_err(|error| EditorError::UiAsset(error.to_string()))?;
        drop(sessions);
        if changed {
            self.hydrate_ui_asset_editor_imports(instance_id)?;
            self.sync_ui_asset_editor_instance(instance_id)?;
        }
        Ok(changed)
    }

    pub fn prune_ui_asset_editor_duplicate_local_theme_overrides(
        &self,
        instance_id: &ViewInstanceId,
    ) -> Result<bool, EditorError> {
        self.ensure_ui_asset_editor_session(instance_id)?;
        let mut sessions = self.ui_asset_sessions.lock().unwrap();
        let entry = sessions.get_mut(instance_id).ok_or_else(|| {
            EditorError::UiAsset(format!("missing ui asset session {}", instance_id.0))
        })?;
        let changed = entry
            .session
            .prune_duplicate_local_theme_overrides()
            .map_err(|error| EditorError::UiAsset(error.to_string()))?;
        drop(sessions);
        self.sync_ui_asset_editor_instance(instance_id)?;
        Ok(changed)
    }

    pub fn apply_ui_asset_editor_all_theme_refactors(
        &self,
        instance_id: &ViewInstanceId,
    ) -> Result<bool, EditorError> {
        self.ensure_ui_asset_editor_session(instance_id)?;
        let mut sessions = self.ui_asset_sessions.lock().unwrap();
        let entry = sessions.get_mut(instance_id).ok_or_else(|| {
            EditorError::UiAsset(format!("missing ui asset session {}", instance_id.0))
        })?;
        let changed = entry
            .session
            .apply_all_theme_refactors()
            .map_err(|error| EditorError::UiAsset(error.to_string()))?;
        drop(sessions);
        self.sync_ui_asset_editor_instance(instance_id)?;
        Ok(changed)
    }

    pub fn apply_ui_asset_editor_theme_rule_helper_item(
        &self,
        instance_id: &ViewInstanceId,
        index: usize,
    ) -> Result<bool, EditorError> {
        self.ensure_ui_asset_editor_session(instance_id)?;
        let helper_action = {
            let sessions = self.ui_asset_sessions.lock().unwrap();
            let entry = sessions.get(instance_id).ok_or_else(|| {
                EditorError::UiAsset(format!("missing ui asset session {}", instance_id.0))
            })?;
            entry.session.theme_rule_helper_action(index)
        }
        .ok_or_else(|| EditorError::UiAsset(format!("invalid theme helper index {index}")))?;

        let changed = match helper_action {
            crate::core::editing::ui_asset::UiAssetThemeRuleHelperAction::PromoteLocalTheme => {
                self.promote_ui_asset_editor_local_theme_to_external_style_asset(instance_id)?
            }
            crate::core::editing::ui_asset::UiAssetThemeRuleHelperAction::DetachImportedThemeToLocal { .. } => {
                self.detach_ui_asset_editor_selected_theme_source_to_local(instance_id)?
            }
            crate::core::editing::ui_asset::UiAssetThemeRuleHelperAction::CloneImportedThemeToLocal { .. } => {
                self.clone_ui_asset_editor_selected_theme_source_to_local(instance_id)?
            }
            crate::core::editing::ui_asset::UiAssetThemeRuleHelperAction::AdoptActiveCascadeTokens { .. }
            | crate::core::editing::ui_asset::UiAssetThemeRuleHelperAction::AdoptActiveCascadeRules { .. }
            | crate::core::editing::ui_asset::UiAssetThemeRuleHelperAction::AdoptActiveCascadeChanges { .. }
            | crate::core::editing::ui_asset::UiAssetThemeRuleHelperAction::AdoptActiveCascadeToken { .. }
            | crate::core::editing::ui_asset::UiAssetThemeRuleHelperAction::AdoptActiveCascadeRule { .. }
            | crate::core::editing::ui_asset::UiAssetThemeRuleHelperAction::AdoptComparedImportedDiffs { .. }
            | crate::core::editing::ui_asset::UiAssetThemeRuleHelperAction::PruneSharedComparedEntries { .. }
            | crate::core::editing::ui_asset::UiAssetThemeRuleHelperAction::AdoptAllImportedTokens { .. }
            | crate::core::editing::ui_asset::UiAssetThemeRuleHelperAction::AdoptAllImportedRules { .. }
            | crate::core::editing::ui_asset::UiAssetThemeRuleHelperAction::AdoptAllImportedChanges { .. }
            | crate::core::editing::ui_asset::UiAssetThemeRuleHelperAction::AdoptImportedToken { .. }
            | crate::core::editing::ui_asset::UiAssetThemeRuleHelperAction::AdoptImportedRule { .. } => {
                let mut sessions = self.ui_asset_sessions.lock().unwrap();
                let entry = sessions.get_mut(instance_id).ok_or_else(|| {
                    EditorError::UiAsset(format!("missing ui asset session {}", instance_id.0))
                })?;
                let changed = entry
                    .session
                    .apply_theme_rule_helper_item(index)
                    .map_err(|error| EditorError::UiAsset(error.to_string()))?;
                drop(sessions);
                self.sync_ui_asset_editor_instance(instance_id)?;
                changed
            }
            crate::core::editing::ui_asset::UiAssetThemeRuleHelperAction::ApplyAllThemeRefactors { .. } => {
                self.apply_ui_asset_editor_all_theme_refactors(instance_id)?
            }
            crate::core::editing::ui_asset::UiAssetThemeRuleHelperAction::PruneDuplicateLocalOverrides => {
                self.prune_ui_asset_editor_duplicate_local_theme_overrides(instance_id)?
            }
        };
        Ok(changed)
    }

    pub fn apply_ui_asset_editor_theme_refactor_item(
        &self,
        instance_id: &ViewInstanceId,
        index: usize,
    ) -> Result<bool, EditorError> {
        self.ensure_ui_asset_editor_session(instance_id)?;
        let mut sessions = self.ui_asset_sessions.lock().unwrap();
        let entry = sessions.get_mut(instance_id).ok_or_else(|| {
            EditorError::UiAsset(format!("missing ui asset session {}", instance_id.0))
        })?;
        let changed = entry
            .session
            .apply_theme_refactor_item(index)
            .map_err(|error| EditorError::UiAsset(error.to_string()))?;
        drop(sessions);
        self.sync_ui_asset_editor_instance(instance_id)?;
        Ok(changed)
    }

    pub fn set_ui_asset_editor_promote_theme_asset_id(
        &self,
        instance_id: &ViewInstanceId,
        asset_id: impl AsRef<str>,
    ) -> Result<bool, EditorError> {
        self.ensure_ui_asset_editor_session(instance_id)?;
        let mut sessions = self.ui_asset_sessions.lock().unwrap();
        let entry = sessions.get_mut(instance_id).ok_or_else(|| {
            EditorError::UiAsset(format!("missing ui asset session {}", instance_id.0))
        })?;
        let changed = entry
            .session
            .set_promote_theme_asset_id(asset_id.as_ref())
            .map_err(|error| EditorError::UiAsset(error.to_string()))?;
        drop(sessions);
        self.sync_ui_asset_editor_instance(instance_id)?;
        Ok(changed)
    }

    pub fn set_ui_asset_editor_promote_theme_document_id(
        &self,
        instance_id: &ViewInstanceId,
        document_id: impl AsRef<str>,
    ) -> Result<bool, EditorError> {
        self.ensure_ui_asset_editor_session(instance_id)?;
        let mut sessions = self.ui_asset_sessions.lock().unwrap();
        let entry = sessions.get_mut(instance_id).ok_or_else(|| {
            EditorError::UiAsset(format!("missing ui asset session {}", instance_id.0))
        })?;
        let changed = entry
            .session
            .set_promote_theme_document_id(document_id.as_ref())
            .map_err(|error| EditorError::UiAsset(error.to_string()))?;
        drop(sessions);
        self.sync_ui_asset_editor_instance(instance_id)?;
        Ok(changed)
    }

    pub fn set_ui_asset_editor_promote_theme_display_name(
        &self,
        instance_id: &ViewInstanceId,
        display_name: impl AsRef<str>,
    ) -> Result<bool, EditorError> {
        self.ensure_ui_asset_editor_session(instance_id)?;
        let mut sessions = self.ui_asset_sessions.lock().unwrap();
        let entry = sessions.get_mut(instance_id).ok_or_else(|| {
            EditorError::UiAsset(format!("missing ui asset session {}", instance_id.0))
        })?;
        let changed = entry
            .session
            .set_promote_theme_display_name(display_name.as_ref())
            .map_err(|error| EditorError::UiAsset(error.to_string()))?;
        drop(sessions);
        self.sync_ui_asset_editor_instance(instance_id)?;
        Ok(changed)
    }

    pub fn create_ui_asset_editor_rule_from_selection(
        &self,
        instance_id: &ViewInstanceId,
    ) -> Result<bool, EditorError> {
        self.ensure_ui_asset_editor_session(instance_id)?;
        let mut sessions = self.ui_asset_sessions.lock().unwrap();
        let entry = sessions.get_mut(instance_id).ok_or_else(|| {
            EditorError::UiAsset(format!("missing ui asset session {}", instance_id.0))
        })?;
        let changed = entry
            .session
            .create_rule_from_selection()
            .map_err(|error| EditorError::UiAsset(error.to_string()))?;
        drop(sessions);
        self.sync_ui_asset_editor_instance(instance_id)?;
        Ok(changed)
    }

    pub fn extract_ui_asset_editor_inline_overrides_to_rule(
        &self,
        instance_id: &ViewInstanceId,
    ) -> Result<bool, EditorError> {
        self.ensure_ui_asset_editor_session(instance_id)?;
        let mut sessions = self.ui_asset_sessions.lock().unwrap();
        let entry = sessions.get_mut(instance_id).ok_or_else(|| {
            EditorError::UiAsset(format!("missing ui asset session {}", instance_id.0))
        })?;
        let changed = entry
            .session
            .extract_inline_overrides_to_rule()
            .map_err(|error| EditorError::UiAsset(error.to_string()))?;
        drop(sessions);
        self.sync_ui_asset_editor_instance(instance_id)?;
        Ok(changed)
    }

    pub fn toggle_ui_asset_editor_pseudo_state(
        &self,
        instance_id: &ViewInstanceId,
        state: impl AsRef<str>,
    ) -> Result<bool, EditorError> {
        self.ensure_ui_asset_editor_session(instance_id)?;
        let mut sessions = self.ui_asset_sessions.lock().unwrap();
        let entry = sessions.get_mut(instance_id).ok_or_else(|| {
            EditorError::UiAsset(format!("missing ui asset session {}", instance_id.0))
        })?;
        let changed = entry
            .session
            .toggle_pseudo_state_preview(state.as_ref())
            .map_err(|error| EditorError::UiAsset(error.to_string()))?;
        drop(sessions);
        self.sync_ui_asset_editor_instance(instance_id)?;
        Ok(changed)
    }

    pub fn add_ui_asset_editor_class_to_selection(
        &self,
        instance_id: &ViewInstanceId,
        class_name: impl AsRef<str>,
    ) -> Result<bool, EditorError> {
        self.ensure_ui_asset_editor_session(instance_id)?;
        let mut sessions = self.ui_asset_sessions.lock().unwrap();
        let entry = sessions.get_mut(instance_id).ok_or_else(|| {
            EditorError::UiAsset(format!("missing ui asset session {}", instance_id.0))
        })?;
        let changed = entry
            .session
            .add_class_to_selection(class_name.as_ref())
            .map_err(|error| EditorError::UiAsset(error.to_string()))?;
        drop(sessions);
        self.sync_ui_asset_editor_instance(instance_id)?;
        Ok(changed)
    }

    pub fn remove_ui_asset_editor_class_from_selection(
        &self,
        instance_id: &ViewInstanceId,
        class_name: impl AsRef<str>,
    ) -> Result<bool, EditorError> {
        self.ensure_ui_asset_editor_session(instance_id)?;
        let mut sessions = self.ui_asset_sessions.lock().unwrap();
        let entry = sessions.get_mut(instance_id).ok_or_else(|| {
            EditorError::UiAsset(format!("missing ui asset session {}", instance_id.0))
        })?;
        let changed = entry
            .session
            .remove_class_from_selection(class_name.as_ref())
            .map_err(|error| EditorError::UiAsset(error.to_string()))?;
        drop(sessions);
        self.sync_ui_asset_editor_instance(instance_id)?;
        Ok(changed)
    }

    pub fn select_ui_asset_editor_style_token(
        &self,
        instance_id: &ViewInstanceId,
        index: usize,
    ) -> Result<bool, EditorError> {
        self.ensure_ui_asset_editor_session(instance_id)?;
        let mut sessions = self.ui_asset_sessions.lock().unwrap();
        let entry = sessions.get_mut(instance_id).ok_or_else(|| {
            EditorError::UiAsset(format!("missing ui asset session {}", instance_id.0))
        })?;
        let changed = entry
            .session
            .select_style_token(index)
            .map_err(|error| EditorError::UiAsset(error.to_string()))?;
        drop(sessions);
        self.sync_ui_asset_editor_instance(instance_id)?;
        Ok(changed)
    }

    pub fn upsert_ui_asset_editor_style_token(
        &self,
        instance_id: &ViewInstanceId,
        token_name: impl AsRef<str>,
        value_literal: impl AsRef<str>,
    ) -> Result<bool, EditorError> {
        self.ensure_ui_asset_editor_session(instance_id)?;
        let mut sessions = self.ui_asset_sessions.lock().unwrap();
        let entry = sessions.get_mut(instance_id).ok_or_else(|| {
            EditorError::UiAsset(format!("missing ui asset session {}", instance_id.0))
        })?;
        let changed = entry
            .session
            .upsert_style_token(token_name.as_ref(), value_literal.as_ref())
            .map_err(|error| EditorError::UiAsset(error.to_string()))?;
        drop(sessions);
        self.sync_ui_asset_editor_instance(instance_id)?;
        Ok(changed)
    }

    pub fn delete_ui_asset_editor_selected_style_token(
        &self,
        instance_id: &ViewInstanceId,
    ) -> Result<bool, EditorError> {
        self.ensure_ui_asset_editor_session(instance_id)?;
        let mut sessions = self.ui_asset_sessions.lock().unwrap();
        let entry = sessions.get_mut(instance_id).ok_or_else(|| {
            EditorError::UiAsset(format!("missing ui asset session {}", instance_id.0))
        })?;
        let changed = entry
            .session
            .delete_selected_style_token()
            .map_err(|error| EditorError::UiAsset(error.to_string()))?;
        drop(sessions);
        self.sync_ui_asset_editor_instance(instance_id)?;
        Ok(changed)
    }

    pub fn select_ui_asset_editor_stylesheet_rule(
        &self,
        instance_id: &ViewInstanceId,
        index: usize,
    ) -> Result<bool, EditorError> {
        self.ensure_ui_asset_editor_session(instance_id)?;
        let mut sessions = self.ui_asset_sessions.lock().unwrap();
        let entry = sessions.get_mut(instance_id).ok_or_else(|| {
            EditorError::UiAsset(format!("missing ui asset session {}", instance_id.0))
        })?;
        let changed = entry
            .session
            .select_stylesheet_rule(index)
            .map_err(|error| EditorError::UiAsset(error.to_string()))?;
        drop(sessions);
        self.sync_ui_asset_editor_instance(instance_id)?;
        Ok(changed)
    }

    pub fn move_ui_asset_editor_selected_stylesheet_rule_up(
        &self,
        instance_id: &ViewInstanceId,
    ) -> Result<bool, EditorError> {
        self.ensure_ui_asset_editor_session(instance_id)?;
        let mut sessions = self.ui_asset_sessions.lock().unwrap();
        let entry = sessions.get_mut(instance_id).ok_or_else(|| {
            EditorError::UiAsset(format!("missing ui asset session {}", instance_id.0))
        })?;
        let changed = entry
            .session
            .move_selected_stylesheet_rule_up()
            .map_err(|error| EditorError::UiAsset(error.to_string()))?;
        drop(sessions);
        self.sync_ui_asset_editor_instance(instance_id)?;
        Ok(changed)
    }

    pub fn move_ui_asset_editor_selected_stylesheet_rule_down(
        &self,
        instance_id: &ViewInstanceId,
    ) -> Result<bool, EditorError> {
        self.ensure_ui_asset_editor_session(instance_id)?;
        let mut sessions = self.ui_asset_sessions.lock().unwrap();
        let entry = sessions.get_mut(instance_id).ok_or_else(|| {
            EditorError::UiAsset(format!("missing ui asset session {}", instance_id.0))
        })?;
        let changed = entry
            .session
            .move_selected_stylesheet_rule_down()
            .map_err(|error| EditorError::UiAsset(error.to_string()))?;
        drop(sessions);
        self.sync_ui_asset_editor_instance(instance_id)?;
        Ok(changed)
    }

    pub fn select_ui_asset_editor_matched_style_rule(
        &self,
        instance_id: &ViewInstanceId,
        index: usize,
    ) -> Result<bool, EditorError> {
        self.ensure_ui_asset_editor_session(instance_id)?;
        let mut sessions = self.ui_asset_sessions.lock().unwrap();
        let entry = sessions.get_mut(instance_id).ok_or_else(|| {
            EditorError::UiAsset(format!("missing ui asset session {}", instance_id.0))
        })?;
        let changed = entry
            .session
            .select_matched_style_rule(index)
            .map_err(|error| EditorError::UiAsset(error.to_string()))?;
        drop(sessions);
        self.sync_ui_asset_editor_instance(instance_id)?;
        Ok(changed)
    }

    pub fn rename_ui_asset_editor_selected_stylesheet_rule(
        &self,
        instance_id: &ViewInstanceId,
        selector: impl AsRef<str>,
    ) -> Result<bool, EditorError> {
        self.ensure_ui_asset_editor_session(instance_id)?;
        let mut sessions = self.ui_asset_sessions.lock().unwrap();
        let entry = sessions.get_mut(instance_id).ok_or_else(|| {
            EditorError::UiAsset(format!("missing ui asset session {}", instance_id.0))
        })?;
        let changed = entry
            .session
            .rename_selected_stylesheet_rule(selector.as_ref())
            .map_err(|error| EditorError::UiAsset(error.to_string()))?;
        drop(sessions);
        self.sync_ui_asset_editor_instance(instance_id)?;
        Ok(changed)
    }

    pub fn select_ui_asset_editor_style_rule_declaration(
        &self,
        instance_id: &ViewInstanceId,
        index: usize,
    ) -> Result<bool, EditorError> {
        self.ensure_ui_asset_editor_session(instance_id)?;
        let mut sessions = self.ui_asset_sessions.lock().unwrap();
        let entry = sessions.get_mut(instance_id).ok_or_else(|| {
            EditorError::UiAsset(format!("missing ui asset session {}", instance_id.0))
        })?;
        let changed = entry
            .session
            .select_stylesheet_rule_declaration(index)
            .map_err(|error| EditorError::UiAsset(error.to_string()))?;
        drop(sessions);
        self.sync_ui_asset_editor_instance(instance_id)?;
        Ok(changed)
    }

    pub fn upsert_ui_asset_editor_selected_style_rule_declaration(
        &self,
        instance_id: &ViewInstanceId,
        path: impl AsRef<str>,
        value_literal: impl AsRef<str>,
    ) -> Result<bool, EditorError> {
        self.ensure_ui_asset_editor_session(instance_id)?;
        let mut sessions = self.ui_asset_sessions.lock().unwrap();
        let entry = sessions.get_mut(instance_id).ok_or_else(|| {
            EditorError::UiAsset(format!("missing ui asset session {}", instance_id.0))
        })?;
        let changed = entry
            .session
            .upsert_selected_stylesheet_rule_declaration(path.as_ref(), value_literal.as_ref())
            .map_err(|error| EditorError::UiAsset(error.to_string()))?;
        drop(sessions);
        self.sync_ui_asset_editor_instance(instance_id)?;
        Ok(changed)
    }

    pub fn delete_ui_asset_editor_selected_style_rule_declaration(
        &self,
        instance_id: &ViewInstanceId,
    ) -> Result<bool, EditorError> {
        self.ensure_ui_asset_editor_session(instance_id)?;
        let mut sessions = self.ui_asset_sessions.lock().unwrap();
        let entry = sessions.get_mut(instance_id).ok_or_else(|| {
            EditorError::UiAsset(format!("missing ui asset session {}", instance_id.0))
        })?;
        let changed = entry
            .session
            .delete_selected_stylesheet_rule_declaration()
            .map_err(|error| EditorError::UiAsset(error.to_string()))?;
        drop(sessions);
        self.sync_ui_asset_editor_instance(instance_id)?;
        Ok(changed)
    }

    pub fn delete_ui_asset_editor_selected_stylesheet_rule(
        &self,
        instance_id: &ViewInstanceId,
    ) -> Result<bool, EditorError> {
        self.ensure_ui_asset_editor_session(instance_id)?;
        let mut sessions = self.ui_asset_sessions.lock().unwrap();
        let entry = sessions.get_mut(instance_id).ok_or_else(|| {
            EditorError::UiAsset(format!("missing ui asset session {}", instance_id.0))
        })?;
        let changed = entry
            .session
            .delete_selected_stylesheet_rule()
            .map_err(|error| EditorError::UiAsset(error.to_string()))?;
        drop(sessions);
        self.sync_ui_asset_editor_instance(instance_id)?;
        Ok(changed)
    }
}
