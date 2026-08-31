use super::*;

use std::io::ErrorKind;

use zircon_runtime::core::resource::io::atomic_write_new;

impl EditorUiHost {
    pub fn convert_ui_asset_editor_selected_node_to_reference(
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
            .convert_selected_node_to_reference()
            .map_err(|error| EditorError::UiAsset(error.to_string()))?;
        drop(sessions);
        self.sync_ui_asset_editor_instance(instance_id)?;
        Ok(changed)
    }

    pub fn extract_ui_asset_editor_selected_node_to_component(
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
            .extract_selected_node_to_component()
            .map_err(|error| EditorError::UiAsset(error.to_string()))?;
        drop(sessions);
        self.sync_ui_asset_editor_instance(instance_id)?;
        Ok(changed)
    }

    pub fn promote_ui_asset_editor_selected_component_to_external_widget(
        &self,
        instance_id: &ViewInstanceId,
    ) -> Result<bool, EditorError> {
        self.ensure_ui_asset_editor_session(instance_id)?;
        let project = self.current_project_snapshot()?.ok_or_else(|| {
            EditorError::UiAsset(
                "cannot promote component to an external widget without an open project"
                    .to_string(),
            )
        })?;
        for _ in 0..PROMOTION_TARGET_ALLOCATION_RETRIES {
            let (widget_asset, target_asset_id, target_source_path) = {
                let mut sessions = self.lock_ui_asset_sessions();
                let entry = sessions.get_mut(instance_id).ok_or_else(|| {
                    EditorError::UiAsset(format!("missing ui asset session {}", instance_id.0))
                })?;
                let Some(draft) = entry.session.selected_promote_widget_draft() else {
                    return Ok(false);
                };
                let target = resolve_external_widget_target(
                    &project,
                    &draft.asset_id,
                    &draft.component_name,
                    &draft.document_id,
                )?;
                let Some(widget_document) = entry
                    .session
                    .promote_selected_component_to_external_widget(
                        &target.asset_id,
                        &draft.component_name,
                        &target.document_id,
                    )
                    .map_err(|error| EditorError::UiAsset(error.to_string()))?
                else {
                    return Ok(false);
                };
                (widget_document, target.asset_id, target.source_path)
            };
            let widget_source =
                match crate::ui::asset_editor::serialize_authoring_document_as_v2(&widget_asset) {
                    Ok(source) => source,
                    Err(error) => {
                        self.rollback_unpublished_ui_asset_promotion(instance_id)?;
                        return Err(EditorError::UiAsset(error.to_string()));
                    }
                };
            match atomic_write_new(&target_source_path, widget_source.as_bytes()) {
                Ok(()) => {
                    let normalized = normalize_ui_asset_asset_id(&target_asset_id).to_string();
                    let _ = self.asset_manager()?.import_asset(&normalized);
                    self.refresh_ui_asset_workspace_for_changes(vec![normalized])?;
                    self.hydrate_ui_asset_editor_imports(instance_id)?;
                    self.sync_ui_asset_editor_instance(instance_id)?;
                    return Ok(true);
                }
                Err(error) if error.kind() == ErrorKind::AlreadyExists => {
                    self.rollback_unpublished_ui_asset_promotion(instance_id)?;
                }
                Err(source) => {
                    self.rollback_unpublished_ui_asset_promotion(instance_id)?;
                    return Err(EditorError::UiAssetSaveIo {
                        stage: UiAssetSaveStage::AtomicCommit,
                        source_path: target_source_path,
                        source,
                    });
                }
            }
        }
        Err(EditorError::UiAsset(
            "could not allocate a collision-free external widget asset path".to_string(),
        ))
    }

    pub fn promote_ui_asset_editor_local_theme_to_external_style_asset(
        &self,
        instance_id: &ViewInstanceId,
    ) -> Result<bool, EditorError> {
        self.ensure_ui_asset_editor_session(instance_id)?;
        let project = self.current_project_snapshot()?.ok_or_else(|| {
            EditorError::UiAsset(
                "cannot promote local theme to an external style asset without an open project"
                    .to_string(),
            )
        })?;
        for _ in 0..PROMOTION_TARGET_ALLOCATION_RETRIES {
            let (style_asset, target_asset_id, target_source_path) = {
                let mut sessions = self.lock_ui_asset_sessions();
                let entry = sessions.get_mut(instance_id).ok_or_else(|| {
                    EditorError::UiAsset(format!("missing ui asset session {}", instance_id.0))
                })?;
                let Some(draft) = entry.session.selected_promote_theme_draft() else {
                    return Ok(false);
                };
                let target = resolve_external_style_target(
                    &project,
                    &draft.asset_id,
                    &draft.document_id,
                    &draft.display_name,
                )?;
                let Some(style_document) = entry
                    .session
                    .promote_local_theme_to_external_style_asset(
                        &target.asset_id,
                        &target.document_id,
                        &target.display_name,
                    )
                    .map_err(|error| EditorError::UiAsset(error.to_string()))?
                else {
                    return Ok(false);
                };
                (style_document, target.asset_id, target.source_path)
            };
            let style_source =
                match crate::ui::asset_editor::serialize_authoring_document_as_v2(&style_asset) {
                    Ok(source) => source,
                    Err(error) => {
                        self.rollback_unpublished_ui_asset_promotion(instance_id)?;
                        return Err(EditorError::UiAsset(error.to_string()));
                    }
                };
            match atomic_write_new(&target_source_path, style_source.as_bytes()) {
                Ok(()) => {
                    let normalized = normalize_ui_asset_asset_id(&target_asset_id).to_string();
                    let _ = self.asset_manager()?.import_asset(&normalized);
                    self.refresh_ui_asset_workspace_for_changes(vec![normalized])?;
                    self.hydrate_ui_asset_editor_imports(instance_id)?;
                    self.sync_ui_asset_editor_instance(instance_id)?;
                    return Ok(true);
                }
                Err(error) if error.kind() == ErrorKind::AlreadyExists => {
                    self.rollback_unpublished_ui_asset_promotion(instance_id)?;
                }
                Err(source) => {
                    self.rollback_unpublished_ui_asset_promotion(instance_id)?;
                    return Err(EditorError::UiAssetSaveIo {
                        stage: UiAssetSaveStage::AtomicCommit,
                        source_path: target_source_path,
                        source,
                    });
                }
            }
        }
        Err(EditorError::UiAsset(
            "could not allocate a collision-free external style asset path".to_string(),
        ))
    }

    pub fn move_ui_asset_editor_selected_node_up(
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
            .move_selected_node_up()
            .map_err(|error| EditorError::UiAsset(error.to_string()))?;
        drop(sessions);
        self.sync_ui_asset_editor_instance(instance_id)?;
        Ok(changed)
    }

    pub fn move_ui_asset_editor_selected_node_down(
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
            .move_selected_node_down()
            .map_err(|error| EditorError::UiAsset(error.to_string()))?;
        drop(sessions);
        self.sync_ui_asset_editor_instance(instance_id)?;
        Ok(changed)
    }

    pub fn reparent_ui_asset_editor_selected_node_into_previous(
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
            .reparent_selected_node_into_previous()
            .map_err(|error| EditorError::UiAsset(error.to_string()))?;
        drop(sessions);
        self.sync_ui_asset_editor_instance(instance_id)?;
        Ok(changed)
    }

    pub fn reparent_ui_asset_editor_selected_node_into_next(
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
            .reparent_selected_node_into_next()
            .map_err(|error| EditorError::UiAsset(error.to_string()))?;
        drop(sessions);
        self.sync_ui_asset_editor_instance(instance_id)?;
        Ok(changed)
    }

    pub fn reparent_ui_asset_editor_selected_node_outdent(
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
            .reparent_selected_node_outdent()
            .map_err(|error| EditorError::UiAsset(error.to_string()))?;
        drop(sessions);
        self.sync_ui_asset_editor_instance(instance_id)?;
        Ok(changed)
    }

    pub fn wrap_ui_asset_editor_selected_node(
        &self,
        instance_id: &ViewInstanceId,
        widget_type: impl AsRef<str>,
    ) -> Result<bool, EditorError> {
        self.ensure_ui_asset_editor_session(instance_id)?;
        let mut sessions = self.lock_ui_asset_sessions();
        let entry = sessions.get_mut(instance_id).ok_or_else(|| {
            EditorError::UiAsset(format!("missing ui asset session {}", instance_id.0))
        })?;
        let changed = entry
            .session
            .wrap_selected_node_with(widget_type.as_ref())
            .map_err(|error| EditorError::UiAsset(error.to_string()))?;
        drop(sessions);
        self.sync_ui_asset_editor_instance(instance_id)?;
        Ok(changed)
    }

    pub fn unwrap_ui_asset_editor_selected_node(
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
            .unwrap_selected_node()
            .map_err(|error| EditorError::UiAsset(error.to_string()))?;
        drop(sessions);
        self.sync_ui_asset_editor_instance(instance_id)?;
        Ok(changed)
    }

    fn rollback_unpublished_ui_asset_promotion(
        &self,
        instance_id: &ViewInstanceId,
    ) -> Result<(), EditorError> {
        let mut sessions = self.lock_ui_asset_sessions();
        let entry = sessions.get_mut(instance_id).ok_or_else(|| {
            EditorError::UiAsset(format!("missing ui asset session {}", instance_id.0))
        })?;
        let _ = entry
            .session
            .undo_replay()
            .map_err(|error| EditorError::UiAsset(error.to_string()))?;
        Ok(())
    }
}

const PROMOTION_TARGET_ALLOCATION_RETRIES: usize = 8;
