use super::*;
use crate::ui::asset_editor::style::theme_authoring::UiAssetThemeRuleHelperAction;

impl EditorUiHost {
    pub fn select_ui_asset_editor_theme_source(
        &self,
        instance_id: &ViewInstanceId,
        index: usize,
    ) -> Result<bool, EditorError> {
        self.ensure_ui_asset_editor_session(instance_id)?;
        let mut sessions = self.lock_ui_asset_sessions();
        let entry = sessions.get_mut(instance_id).ok_or_else(|| {
            EditorError::UiAsset(format!("missing ui asset session {}", instance_id.0))
        })?;
        let changed = entry
            .session
            .select_theme_source(index)
            .map_err(|error| EditorError::UiAsset(error.to_string()))?;
        drop(sessions);
        self.sync_ui_asset_editor_style_projection_if_changed(instance_id, changed)?;
        Ok(changed)
    }

    pub fn detach_ui_asset_editor_selected_theme_source_to_local(
        &self,
        instance_id: &ViewInstanceId,
    ) -> Result<bool, EditorError> {
        self.ensure_ui_asset_editor_session(instance_id)?;
        let mut sessions = self.lock_ui_asset_sessions();
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
        let mut sessions = self.lock_ui_asset_sessions();
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
        let mut sessions = self.lock_ui_asset_sessions();
        let entry = sessions.get_mut(instance_id).ok_or_else(|| {
            EditorError::UiAsset(format!("missing ui asset session {}", instance_id.0))
        })?;
        let changed = entry
            .session
            .prune_duplicate_local_theme_overrides()
            .map_err(|error| EditorError::UiAsset(error.to_string()))?;
        drop(sessions);
        self.sync_ui_asset_editor_style_projection_if_changed(instance_id, changed)?;
        Ok(changed)
    }

    pub fn apply_ui_asset_editor_all_theme_refactors(
        &self,
        instance_id: &ViewInstanceId,
    ) -> Result<bool, EditorError> {
        self.ensure_ui_asset_editor_session(instance_id)?;
        let mut sessions = self.lock_ui_asset_sessions();
        let entry = sessions.get_mut(instance_id).ok_or_else(|| {
            EditorError::UiAsset(format!("missing ui asset session {}", instance_id.0))
        })?;
        let changed = entry
            .session
            .apply_all_theme_refactors()
            .map_err(|error| EditorError::UiAsset(error.to_string()))?;
        drop(sessions);
        self.sync_ui_asset_editor_style_projection_if_changed(instance_id, changed)?;
        Ok(changed)
    }

    pub fn apply_ui_asset_editor_theme_rule_helper_item(
        &self,
        instance_id: &ViewInstanceId,
        index: usize,
    ) -> Result<bool, EditorError> {
        self.ensure_ui_asset_editor_session(instance_id)?;
        let helper_action = {
            let sessions = self.lock_ui_asset_sessions();
            let entry = sessions.get(instance_id).ok_or_else(|| {
                EditorError::UiAsset(format!("missing ui asset session {}", instance_id.0))
            })?;
            entry.session.theme_rule_helper_action(index)
        }
        .ok_or_else(|| EditorError::UiAsset(format!("invalid theme helper index {index}")))?;

        let changed = match helper_action {
            UiAssetThemeRuleHelperAction::PromoteLocalTheme => {
                self.promote_ui_asset_editor_local_theme_to_external_style_asset(instance_id)?
            }
            UiAssetThemeRuleHelperAction::DetachImportedThemeToLocal { .. } => {
                self.detach_ui_asset_editor_selected_theme_source_to_local(instance_id)?
            }
            UiAssetThemeRuleHelperAction::CloneImportedThemeToLocal { .. } => {
                self.clone_ui_asset_editor_selected_theme_source_to_local(instance_id)?
            }
            UiAssetThemeRuleHelperAction::AdoptActiveCascadeTokens { .. }
            | UiAssetThemeRuleHelperAction::AdoptActiveCascadeRules { .. }
            | UiAssetThemeRuleHelperAction::AdoptActiveCascadeChanges { .. }
            | UiAssetThemeRuleHelperAction::AdoptActiveCascadeToken { .. }
            | UiAssetThemeRuleHelperAction::AdoptActiveCascadeRule { .. }
            | UiAssetThemeRuleHelperAction::AdoptComparedImportedDiffs { .. }
            | UiAssetThemeRuleHelperAction::PruneSharedComparedEntries { .. }
            | UiAssetThemeRuleHelperAction::AdoptAllImportedTokens { .. }
            | UiAssetThemeRuleHelperAction::AdoptAllImportedRules { .. }
            | UiAssetThemeRuleHelperAction::AdoptAllImportedChanges { .. }
            | UiAssetThemeRuleHelperAction::AdoptImportedToken { .. }
            | UiAssetThemeRuleHelperAction::AdoptImportedRule { .. } => {
                let mut sessions = self.lock_ui_asset_sessions();
                let entry = sessions.get_mut(instance_id).ok_or_else(|| {
                    EditorError::UiAsset(format!("missing ui asset session {}", instance_id.0))
                })?;
                let changed = entry
                    .session
                    .apply_theme_rule_helper_item(index)
                    .map_err(|error| EditorError::UiAsset(error.to_string()))?;
                drop(sessions);
                self.sync_ui_asset_editor_style_projection_if_changed(instance_id, changed)?;
                changed
            }
            UiAssetThemeRuleHelperAction::ApplyAllThemeRefactors { .. } => {
                self.apply_ui_asset_editor_all_theme_refactors(instance_id)?
            }
            UiAssetThemeRuleHelperAction::PruneDuplicateLocalOverrides => {
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
        let mut sessions = self.lock_ui_asset_sessions();
        let entry = sessions.get_mut(instance_id).ok_or_else(|| {
            EditorError::UiAsset(format!("missing ui asset session {}", instance_id.0))
        })?;
        let changed = entry
            .session
            .apply_theme_refactor_item(index)
            .map_err(|error| EditorError::UiAsset(error.to_string()))?;
        drop(sessions);
        self.sync_ui_asset_editor_style_projection_if_changed(instance_id, changed)?;
        Ok(changed)
    }

    pub fn set_ui_asset_editor_promote_theme_asset_id(
        &self,
        instance_id: &ViewInstanceId,
        asset_id: impl AsRef<str>,
    ) -> Result<bool, EditorError> {
        self.ensure_ui_asset_editor_session(instance_id)?;
        let mut sessions = self.lock_ui_asset_sessions();
        let entry = sessions.get_mut(instance_id).ok_or_else(|| {
            EditorError::UiAsset(format!("missing ui asset session {}", instance_id.0))
        })?;
        let changed = entry
            .session
            .set_promote_theme_asset_id(asset_id.as_ref())
            .map_err(|error| EditorError::UiAsset(error.to_string()))?;
        drop(sessions);
        self.sync_ui_asset_editor_style_projection_if_changed(instance_id, changed)?;
        Ok(changed)
    }

    pub fn set_ui_asset_editor_promote_theme_document_id(
        &self,
        instance_id: &ViewInstanceId,
        document_id: impl AsRef<str>,
    ) -> Result<bool, EditorError> {
        self.ensure_ui_asset_editor_session(instance_id)?;
        let mut sessions = self.lock_ui_asset_sessions();
        let entry = sessions.get_mut(instance_id).ok_or_else(|| {
            EditorError::UiAsset(format!("missing ui asset session {}", instance_id.0))
        })?;
        let changed = entry
            .session
            .set_promote_theme_document_id(document_id.as_ref())
            .map_err(|error| EditorError::UiAsset(error.to_string()))?;
        drop(sessions);
        self.sync_ui_asset_editor_style_projection_if_changed(instance_id, changed)?;
        Ok(changed)
    }

    pub fn set_ui_asset_editor_promote_theme_display_name(
        &self,
        instance_id: &ViewInstanceId,
        display_name: impl AsRef<str>,
    ) -> Result<bool, EditorError> {
        self.ensure_ui_asset_editor_session(instance_id)?;
        let mut sessions = self.lock_ui_asset_sessions();
        let entry = sessions.get_mut(instance_id).ok_or_else(|| {
            EditorError::UiAsset(format!("missing ui asset session {}", instance_id.0))
        })?;
        let changed = entry
            .session
            .set_promote_theme_display_name(display_name.as_ref())
            .map_err(|error| EditorError::UiAsset(error.to_string()))?;
        drop(sessions);
        self.sync_ui_asset_editor_style_projection_if_changed(instance_id, changed)?;
        Ok(changed)
    }

    pub fn create_ui_asset_editor_rule_from_selection(
        &self,
        instance_id: &ViewInstanceId,
    ) -> Result<bool, EditorError> {
        self.ensure_ui_asset_editor_session(instance_id)?;
        let mut sessions = self.lock_ui_asset_sessions();
        let entry = sessions.get_mut(instance_id).ok_or_else(|| {
            EditorError::UiAsset(format!("missing ui asset session {}", instance_id.0))
        })?;
        let changed = entry
            .session
            .create_rule_from_selection()
            .map_err(|error| EditorError::UiAsset(error.to_string()))?;
        drop(sessions);
        self.sync_ui_asset_editor_style_projection_if_changed(instance_id, changed)?;
        Ok(changed)
    }

    pub fn extract_ui_asset_editor_inline_overrides_to_rule(
        &self,
        instance_id: &ViewInstanceId,
    ) -> Result<bool, EditorError> {
        self.ensure_ui_asset_editor_session(instance_id)?;
        let mut sessions = self.lock_ui_asset_sessions();
        let entry = sessions.get_mut(instance_id).ok_or_else(|| {
            EditorError::UiAsset(format!("missing ui asset session {}", instance_id.0))
        })?;
        let changed = entry
            .session
            .extract_inline_overrides_to_rule()
            .map_err(|error| EditorError::UiAsset(error.to_string()))?;
        drop(sessions);
        self.sync_ui_asset_editor_style_projection_if_changed(instance_id, changed)?;
        Ok(changed)
    }

    pub fn toggle_ui_asset_editor_pseudo_state(
        &self,
        instance_id: &ViewInstanceId,
        state: impl AsRef<str>,
    ) -> Result<bool, EditorError> {
        self.ensure_ui_asset_editor_session(instance_id)?;
        let mut sessions = self.lock_ui_asset_sessions();
        let entry = sessions.get_mut(instance_id).ok_or_else(|| {
            EditorError::UiAsset(format!("missing ui asset session {}", instance_id.0))
        })?;
        let changed = entry
            .session
            .toggle_pseudo_state_preview(state.as_ref())
            .map_err(|error| EditorError::UiAsset(error.to_string()))?;
        drop(sessions);
        self.sync_ui_asset_editor_style_projection_if_changed(instance_id, changed)?;
        Ok(changed)
    }

    pub fn add_ui_asset_editor_class_to_selection(
        &self,
        instance_id: &ViewInstanceId,
        class_name: impl AsRef<str>,
    ) -> Result<bool, EditorError> {
        self.ensure_ui_asset_editor_session(instance_id)?;
        let mut sessions = self.lock_ui_asset_sessions();
        let entry = sessions.get_mut(instance_id).ok_or_else(|| {
            EditorError::UiAsset(format!("missing ui asset session {}", instance_id.0))
        })?;
        let changed = entry
            .session
            .add_class_to_selection(class_name.as_ref())
            .map_err(|error| EditorError::UiAsset(error.to_string()))?;
        drop(sessions);
        self.sync_ui_asset_editor_style_projection_if_changed(instance_id, changed)?;
        Ok(changed)
    }

    pub fn remove_ui_asset_editor_class_from_selection(
        &self,
        instance_id: &ViewInstanceId,
        class_name: impl AsRef<str>,
    ) -> Result<bool, EditorError> {
        self.ensure_ui_asset_editor_session(instance_id)?;
        let mut sessions = self.lock_ui_asset_sessions();
        let entry = sessions.get_mut(instance_id).ok_or_else(|| {
            EditorError::UiAsset(format!("missing ui asset session {}", instance_id.0))
        })?;
        let changed = entry
            .session
            .remove_class_from_selection(class_name.as_ref())
            .map_err(|error| EditorError::UiAsset(error.to_string()))?;
        drop(sessions);
        self.sync_ui_asset_editor_style_projection_if_changed(instance_id, changed)?;
        Ok(changed)
    }

    pub fn select_ui_asset_editor_style_token(
        &self,
        instance_id: &ViewInstanceId,
        index: usize,
    ) -> Result<bool, EditorError> {
        self.ensure_ui_asset_editor_session(instance_id)?;
        let mut sessions = self.lock_ui_asset_sessions();
        let entry = sessions.get_mut(instance_id).ok_or_else(|| {
            EditorError::UiAsset(format!("missing ui asset session {}", instance_id.0))
        })?;
        let changed = entry
            .session
            .select_style_token(index)
            .map_err(|error| EditorError::UiAsset(error.to_string()))?;
        drop(sessions);
        self.sync_ui_asset_editor_style_projection_if_changed(instance_id, changed)?;
        Ok(changed)
    }

    pub fn upsert_ui_asset_editor_style_token(
        &self,
        instance_id: &ViewInstanceId,
        token_name: impl AsRef<str>,
        value_literal: impl AsRef<str>,
    ) -> Result<bool, EditorError> {
        self.ensure_ui_asset_editor_session(instance_id)?;
        let mut sessions = self.lock_ui_asset_sessions();
        let entry = sessions.get_mut(instance_id).ok_or_else(|| {
            EditorError::UiAsset(format!("missing ui asset session {}", instance_id.0))
        })?;
        let changed = entry
            .session
            .upsert_style_token(token_name.as_ref(), value_literal.as_ref())
            .map_err(|error| EditorError::UiAsset(error.to_string()))?;
        drop(sessions);
        self.sync_ui_asset_editor_style_projection_if_changed(instance_id, changed)?;
        Ok(changed)
    }

    pub fn delete_ui_asset_editor_selected_style_token(
        &self,
        instance_id: &ViewInstanceId,
    ) -> Result<bool, EditorError> {
        self.ensure_ui_asset_editor_session(instance_id)?;
        let mut sessions = self.lock_ui_asset_sessions();
        let entry = sessions.get_mut(instance_id).ok_or_else(|| {
            EditorError::UiAsset(format!("missing ui asset session {}", instance_id.0))
        })?;
        let changed = entry
            .session
            .delete_selected_style_token()
            .map_err(|error| EditorError::UiAsset(error.to_string()))?;
        drop(sessions);
        self.sync_ui_asset_editor_style_projection_if_changed(instance_id, changed)?;
        Ok(changed)
    }

    pub fn select_ui_asset_editor_stylesheet_rule(
        &self,
        instance_id: &ViewInstanceId,
        index: usize,
    ) -> Result<bool, EditorError> {
        self.ensure_ui_asset_editor_session(instance_id)?;
        let mut sessions = self.lock_ui_asset_sessions();
        let entry = sessions.get_mut(instance_id).ok_or_else(|| {
            EditorError::UiAsset(format!("missing ui asset session {}", instance_id.0))
        })?;
        let changed = entry
            .session
            .select_stylesheet_rule(index)
            .map_err(|error| EditorError::UiAsset(error.to_string()))?;
        drop(sessions);
        self.sync_ui_asset_editor_style_projection_if_changed(instance_id, changed)?;
        Ok(changed)
    }

    pub fn move_ui_asset_editor_selected_stylesheet_rule_up(
        &self,
        instance_id: &ViewInstanceId,
    ) -> Result<bool, EditorError> {
        self.ensure_ui_asset_editor_session(instance_id)?;
        let mut sessions = self.lock_ui_asset_sessions();
        let entry = sessions.get_mut(instance_id).ok_or_else(|| {
            EditorError::UiAsset(format!("missing ui asset session {}", instance_id.0))
        })?;
        let changed = entry
            .session
            .move_selected_stylesheet_rule_up()
            .map_err(|error| EditorError::UiAsset(error.to_string()))?;
        drop(sessions);
        self.sync_ui_asset_editor_style_projection_if_changed(instance_id, changed)?;
        Ok(changed)
    }

    pub fn move_ui_asset_editor_selected_stylesheet_rule_down(
        &self,
        instance_id: &ViewInstanceId,
    ) -> Result<bool, EditorError> {
        self.ensure_ui_asset_editor_session(instance_id)?;
        let mut sessions = self.lock_ui_asset_sessions();
        let entry = sessions.get_mut(instance_id).ok_or_else(|| {
            EditorError::UiAsset(format!("missing ui asset session {}", instance_id.0))
        })?;
        let changed = entry
            .session
            .move_selected_stylesheet_rule_down()
            .map_err(|error| EditorError::UiAsset(error.to_string()))?;
        drop(sessions);
        self.sync_ui_asset_editor_style_projection_if_changed(instance_id, changed)?;
        Ok(changed)
    }

    pub fn select_ui_asset_editor_matched_style_rule(
        &self,
        instance_id: &ViewInstanceId,
        index: usize,
    ) -> Result<bool, EditorError> {
        self.ensure_ui_asset_editor_session(instance_id)?;
        let mut sessions = self.lock_ui_asset_sessions();
        let entry = sessions.get_mut(instance_id).ok_or_else(|| {
            EditorError::UiAsset(format!("missing ui asset session {}", instance_id.0))
        })?;
        let changed = entry
            .session
            .select_matched_style_rule(index)
            .map_err(|error| EditorError::UiAsset(error.to_string()))?;
        drop(sessions);
        self.sync_ui_asset_editor_style_projection_if_changed(instance_id, changed)?;
        Ok(changed)
    }

    pub fn rename_ui_asset_editor_selected_stylesheet_rule(
        &self,
        instance_id: &ViewInstanceId,
        selector: impl AsRef<str>,
    ) -> Result<bool, EditorError> {
        self.ensure_ui_asset_editor_session(instance_id)?;
        let mut sessions = self.lock_ui_asset_sessions();
        let entry = sessions.get_mut(instance_id).ok_or_else(|| {
            EditorError::UiAsset(format!("missing ui asset session {}", instance_id.0))
        })?;
        let changed = entry
            .session
            .rename_selected_stylesheet_rule(selector.as_ref())
            .map_err(|error| EditorError::UiAsset(error.to_string()))?;
        drop(sessions);
        self.sync_ui_asset_editor_style_projection_if_changed(instance_id, changed)?;
        Ok(changed)
    }

    pub fn select_ui_asset_editor_style_rule_declaration(
        &self,
        instance_id: &ViewInstanceId,
        index: usize,
    ) -> Result<bool, EditorError> {
        self.ensure_ui_asset_editor_session(instance_id)?;
        let mut sessions = self.lock_ui_asset_sessions();
        let entry = sessions.get_mut(instance_id).ok_or_else(|| {
            EditorError::UiAsset(format!("missing ui asset session {}", instance_id.0))
        })?;
        let changed = entry
            .session
            .select_stylesheet_rule_declaration(index)
            .map_err(|error| EditorError::UiAsset(error.to_string()))?;
        drop(sessions);
        self.sync_ui_asset_editor_style_projection_if_changed(instance_id, changed)?;
        Ok(changed)
    }

    pub fn upsert_ui_asset_editor_selected_style_rule_declaration(
        &self,
        instance_id: &ViewInstanceId,
        path: impl AsRef<str>,
        value_literal: impl AsRef<str>,
    ) -> Result<bool, EditorError> {
        self.ensure_ui_asset_editor_session(instance_id)?;
        let mut sessions = self.lock_ui_asset_sessions();
        let entry = sessions.get_mut(instance_id).ok_or_else(|| {
            EditorError::UiAsset(format!("missing ui asset session {}", instance_id.0))
        })?;
        let changed = entry
            .session
            .upsert_selected_stylesheet_rule_declaration(path.as_ref(), value_literal.as_ref())
            .map_err(|error| EditorError::UiAsset(error.to_string()))?;
        drop(sessions);
        self.sync_ui_asset_editor_style_projection_if_changed(instance_id, changed)?;
        Ok(changed)
    }

    pub fn delete_ui_asset_editor_selected_style_rule_declaration(
        &self,
        instance_id: &ViewInstanceId,
    ) -> Result<bool, EditorError> {
        self.ensure_ui_asset_editor_session(instance_id)?;
        let mut sessions = self.lock_ui_asset_sessions();
        let entry = sessions.get_mut(instance_id).ok_or_else(|| {
            EditorError::UiAsset(format!("missing ui asset session {}", instance_id.0))
        })?;
        let changed = entry
            .session
            .delete_selected_stylesheet_rule_declaration()
            .map_err(|error| EditorError::UiAsset(error.to_string()))?;
        drop(sessions);
        self.sync_ui_asset_editor_style_projection_if_changed(instance_id, changed)?;
        Ok(changed)
    }

    pub fn delete_ui_asset_editor_selected_stylesheet_rule(
        &self,
        instance_id: &ViewInstanceId,
    ) -> Result<bool, EditorError> {
        self.ensure_ui_asset_editor_session(instance_id)?;
        let mut sessions = self.lock_ui_asset_sessions();
        let entry = sessions.get_mut(instance_id).ok_or_else(|| {
            EditorError::UiAsset(format!("missing ui asset session {}", instance_id.0))
        })?;
        let changed = entry
            .session
            .delete_selected_stylesheet_rule()
            .map_err(|error| EditorError::UiAsset(error.to_string()))?;
        drop(sessions);
        self.sync_ui_asset_editor_style_projection_if_changed(instance_id, changed)?;
        Ok(changed)
    }

    fn sync_ui_asset_editor_style_projection_if_changed(
        &self,
        instance_id: &ViewInstanceId,
        changed: bool,
    ) -> Result<(), EditorError> {
        if changed {
            self.sync_ui_asset_editor_instance(instance_id)?;
        }
        Ok(())
    }
}

#[cfg(test)]
mod performance_tests {
    use std::hint::black_box;
    use std::time::Instant;

    #[test]
    fn optimization_batch_ec_unchanged_style_actions_guard_projection_sync() {
        let source = include_str!("style.rs");
        let production = source
            .split("#[cfg(test)]")
            .next()
            .expect("style production implementation");

        assert_eq!(
            production
                .matches(
                    "self.sync_ui_asset_editor_style_projection_if_changed(instance_id, changed)?;",
                )
                .count(),
            25,
            "every previously unconditional style mutation must use the no-change projection guard"
        );
        assert_eq!(
            production
                .matches("self.sync_ui_asset_editor_instance(instance_id)?;")
                .count(),
            3,
            "only two existing changed blocks and the style guard may synchronize directly"
        );
        assert_eq!(production.matches("if changed {").count(), 3);
    }

    #[test]
    #[ignore = "release-only unchanged style action benchmark"]
    fn optimization_batch_ec_unchanged_style_action_release_benchmark_evidence() {
        const SAMPLE_PAIRS: usize = 17;
        const NOOP_ACTIONS_PER_SAMPLE: usize = 64;
        const PROJECTION_ROWS: usize = 1_024;

        fn projection_checksum(rows: &[String]) -> usize {
            let projected = black_box(rows.to_vec());
            black_box(projected.iter().map(String::len).sum())
        }

        fn measure_legacy(rows: &[String]) -> u128 {
            let started = Instant::now();
            let mut checksum = 0;
            for _ in 0..NOOP_ACTIONS_PER_SAMPLE {
                checksum ^= projection_checksum(black_box(rows));
            }
            black_box(checksum);
            started.elapsed().as_nanos().max(1)
        }

        fn measure_optimized(rows: &[String]) -> u128 {
            let started = Instant::now();
            let mut checksum = 0;
            for _ in 0..NOOP_ACTIONS_PER_SAMPLE {
                if black_box(false) {
                    checksum ^= projection_checksum(black_box(rows));
                }
            }
            black_box(checksum);
            started.elapsed().as_nanos().max(1)
        }

        fn percentile(samples: &[u128], percentile: usize) -> u128 {
            let mut sorted = samples.to_vec();
            sorted.sort_unstable();
            let rank = (sorted.len() * percentile).div_ceil(100);
            sorted[rank.saturating_sub(1)]
        }

        fn raw(samples: &[u128]) -> String {
            samples
                .iter()
                .map(u128::to_string)
                .collect::<Vec<_>>()
                .join(",")
        }

        let rows = (0..PROJECTION_ROWS)
            .map(|index| {
                format!(
                    "ui.asset.style.projection.row.{index:04}.{}",
                    "x".repeat(96)
                )
            })
            .collect::<Vec<_>>();

        for _ in 0..4 {
            black_box(measure_legacy(&rows));
            black_box(measure_optimized(&rows));
        }

        let mut legacy_samples = Vec::with_capacity(SAMPLE_PAIRS);
        let mut optimized_samples = Vec::with_capacity(SAMPLE_PAIRS);
        for pair in 0..SAMPLE_PAIRS {
            if pair % 2 == 0 {
                legacy_samples.push(measure_legacy(&rows));
                optimized_samples.push(measure_optimized(&rows));
            } else {
                optimized_samples.push(measure_optimized(&rows));
                legacy_samples.push(measure_legacy(&rows));
            }
        }

        let legacy_p50_ns = percentile(&legacy_samples, 50);
        let optimized_p50_ns = percentile(&optimized_samples, 50);
        let legacy_p95_ns = percentile(&legacy_samples, 95);
        let optimized_p95_ns = percentile(&optimized_samples, 95);

        println!(
            "EDITOR365_NOOP_STYLE_PROJECTION_SYNC_BENCH_V1 sample_pairs={SAMPLE_PAIRS} \
noop_actions_per_sample={NOOP_ACTIONS_PER_SAMPLE} projection_rows={PROJECTION_ROWS} \
pair_order=alternating_legacy_even legacy_first_pairs=9 optimized_first_pairs=8 \
legacy_projection_syncs_per_sample={NOOP_ACTIONS_PER_SAMPLE} \
optimized_projection_syncs_per_sample=0 legacy_p50_ns={legacy_p50_ns} \
optimized_p50_ns={optimized_p50_ns} legacy_p95_ns={legacy_p95_ns} \
optimized_p95_ns={optimized_p95_ns} legacy_raw_ns={} optimized_raw_ns={}",
            raw(&legacy_samples),
            raw(&optimized_samples),
        );

        assert!(
            optimized_p95_ns.saturating_mul(100) <= legacy_p95_ns.saturating_mul(70),
            "unchanged style actions must reduce projection-sync P95 by at least 30%: legacy={legacy_p95_ns}ns optimized={optimized_p95_ns}ns"
        );
    }
}
