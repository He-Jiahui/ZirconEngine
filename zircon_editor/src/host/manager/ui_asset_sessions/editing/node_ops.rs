use super::*;

impl EditorManager {
    pub fn convert_ui_asset_editor_selected_node_to_reference(
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
        let mut sessions = self.ui_asset_sessions.lock().unwrap();
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
        let project_root = self.current_project_root()?.ok_or_else(|| {
            EditorError::UiAsset(
                "cannot promote component to an external widget without an open project"
                    .to_string(),
            )
        })?;
        let (widget_asset, target_asset_id, target_source_path) = {
            let mut sessions = self.ui_asset_sessions.lock().unwrap();
            let entry = sessions.get_mut(instance_id).ok_or_else(|| {
                EditorError::UiAsset(format!("missing ui asset session {}", instance_id.0))
            })?;
            let Some(draft) = entry.session.selected_promote_widget_draft() else {
                return Ok(false);
            };
            let target = resolve_external_widget_target(
                &project_root,
                &draft.asset_id,
                &draft.component_name,
                &draft.document_id,
            );
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
            (
                UiWidgetAsset {
                    document: widget_document,
                },
                target.asset_id,
                target.source_path,
            )
        };
        if let Some(parent) = target_source_path.parent() {
            fs::create_dir_all(parent).map_err(|error| EditorError::UiAsset(error.to_string()))?;
        }
        let widget_source = widget_asset
            .to_toml_string()
            .map_err(|error| EditorError::UiAsset(error.to_string()))?;
        fs::write(&target_source_path, widget_source)
            .map_err(|error| EditorError::UiAsset(error.to_string()))?;
        let normalized = normalize_ui_asset_asset_id(&target_asset_id).to_string();
        let _ = self.asset_manager()?.import_asset(&normalized);
        self.hydrate_ui_asset_editor_imports(instance_id)?;
        self.sync_ui_asset_editor_instance(instance_id)?;
        Ok(true)
    }

    pub fn move_ui_asset_editor_selected_node_up(
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
        let mut sessions = self.ui_asset_sessions.lock().unwrap();
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
        let mut sessions = self.ui_asset_sessions.lock().unwrap();
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
        let mut sessions = self.ui_asset_sessions.lock().unwrap();
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
        let mut sessions = self.ui_asset_sessions.lock().unwrap();
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
        let mut sessions = self.ui_asset_sessions.lock().unwrap();
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
        let mut sessions = self.ui_asset_sessions.lock().unwrap();
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
}
