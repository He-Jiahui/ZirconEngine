use super::{
    binding_inspector::{
        add_default_binding,
        apply_selected_binding_action_suggestion as apply_selected_binding_action_suggestion_field,
        apply_selected_binding_payload_suggestion as apply_selected_binding_payload_suggestion_field,
        apply_selected_binding_route_suggestion as apply_selected_binding_route_suggestion_field,
        build_binding_fields, delete_selected_binding as delete_selected_binding_field,
        delete_selected_binding_payload as delete_selected_binding_payload_field,
        reconcile_selected_binding_payload_key,
        set_selected_binding_action_kind as set_selected_binding_action_kind_field,
        set_selected_binding_action_target as set_selected_binding_action_target_field,
        set_selected_binding_event as set_selected_binding_event_field,
        set_selected_binding_id as set_selected_binding_id_field,
        set_selected_binding_route as set_selected_binding_route_field,
        set_selected_binding_route_target as set_selected_binding_route_target_field,
        upsert_selected_binding_payload as upsert_selected_binding_payload_field,
    },
    ui_asset_editor_session::{UiAssetEditorSession, UiAssetEditorSessionError},
};

impl UiAssetEditorSession {
    pub fn select_binding(&mut self, index: usize) -> Result<bool, UiAssetEditorSessionError> {
        let binding_fields = build_binding_fields(
            &self.last_valid_document,
            &self.selection,
            &self.preview_mock_state,
            self.selected_binding_index,
            self.selected_binding_payload_key.as_deref(),
        );
        if index >= binding_fields.items.len() {
            return Err(UiAssetEditorSessionError::InvalidBindingIndex { index });
        }
        let changed = self.selected_binding_index != Some(index);
        self.selected_binding_index = Some(index);
        self.selected_binding_payload_key = reconcile_selected_binding_payload_key(
            &self.last_valid_document,
            &self.selection,
            self.selected_binding_index,
            self.selected_binding_payload_key.as_deref(),
        );
        Ok(changed)
    }

    pub fn add_binding(&mut self) -> Result<bool, UiAssetEditorSessionError> {
        self.ensure_editable_source()?;
        let mut document = self.last_valid_document.clone();
        let Some(next_index) = add_default_binding(&mut document, &self.selection) else {
            return Ok(false);
        };
        self.selected_binding_index = Some(next_index);
        self.selected_binding_payload_key = None;
        self.apply_binding_document_edit_with_label(document, "Binding Add")?;
        Ok(true)
    }

    pub fn delete_selected_binding(&mut self) -> Result<bool, UiAssetEditorSessionError> {
        self.ensure_editable_source()?;
        let mut document = self.last_valid_document.clone();
        if !delete_selected_binding_field(
            &mut document,
            &self.selection,
            self.selected_binding_index,
        ) {
            return Ok(false);
        }
        self.apply_binding_document_edit_with_label(document, "Binding Delete")?;
        Ok(true)
    }

    pub fn set_selected_binding_id(
        &mut self,
        value: impl AsRef<str>,
    ) -> Result<bool, UiAssetEditorSessionError> {
        self.ensure_editable_source()?;
        let mut document = self.last_valid_document.clone();
        if !set_selected_binding_id_field(
            &mut document,
            &self.selection,
            self.selected_binding_index,
            value.as_ref(),
        ) {
            return Ok(false);
        }
        self.apply_binding_document_edit_with_label(document, "Binding Id Edit")?;
        Ok(true)
    }

    pub fn select_binding_event_option(
        &mut self,
        index: usize,
    ) -> Result<bool, UiAssetEditorSessionError> {
        let binding_fields = build_binding_fields(
            &self.last_valid_document,
            &self.selection,
            &self.preview_mock_state,
            self.selected_binding_index,
            self.selected_binding_payload_key.as_deref(),
        );
        let Some(event_name) = binding_fields.binding_event_items.get(index) else {
            return Err(UiAssetEditorSessionError::InvalidSelectionIndex { index });
        };
        self.set_selected_binding_event(event_name)
    }

    pub fn set_selected_binding_event(
        &mut self,
        value: impl AsRef<str>,
    ) -> Result<bool, UiAssetEditorSessionError> {
        self.ensure_editable_source()?;
        let mut document = self.last_valid_document.clone();
        let value = value.as_ref();
        let changed = set_selected_binding_event_field(
            &mut document,
            &self.selection,
            self.selected_binding_index,
            value,
        )
        .map_err(|_| UiAssetEditorSessionError::InvalidBindingEvent {
            value: value.to_string(),
        })?;
        if !changed {
            return Ok(false);
        }
        self.apply_binding_document_edit_with_label(document, "Binding Event Edit")?;
        Ok(true)
    }

    pub fn select_binding_action_kind(
        &mut self,
        index: usize,
    ) -> Result<bool, UiAssetEditorSessionError> {
        let binding_fields = build_binding_fields(
            &self.last_valid_document,
            &self.selection,
            &self.preview_mock_state,
            self.selected_binding_index,
            self.selected_binding_payload_key.as_deref(),
        );
        let Some(kind_label) = binding_fields.binding_action_kind_items.get(index) else {
            return Err(UiAssetEditorSessionError::InvalidSelectionIndex { index });
        };
        self.ensure_editable_source()?;
        let mut document = self.last_valid_document.clone();
        if !set_selected_binding_action_kind_field(
            &mut document,
            &self.selection,
            self.selected_binding_index,
            kind_label,
        ) {
            return Ok(false);
        }
        self.apply_binding_document_edit_with_label(document, "Binding Action Kind Edit")?;
        Ok(true)
    }

    pub fn set_selected_binding_route(
        &mut self,
        value: impl AsRef<str>,
    ) -> Result<bool, UiAssetEditorSessionError> {
        self.ensure_editable_source()?;
        let mut document = self.last_valid_document.clone();
        if !set_selected_binding_route_field(
            &mut document,
            &self.selection,
            self.selected_binding_index,
            value.as_ref(),
        ) {
            return Ok(false);
        }
        self.apply_binding_document_edit_with_label(document, "Binding Route Edit")?;
        Ok(true)
    }

    pub fn set_selected_binding_route_target(
        &mut self,
        value: impl AsRef<str>,
    ) -> Result<bool, UiAssetEditorSessionError> {
        self.ensure_editable_source()?;
        let mut document = self.last_valid_document.clone();
        if !set_selected_binding_route_target_field(
            &mut document,
            &self.selection,
            self.selected_binding_index,
            value.as_ref(),
        ) {
            return Ok(false);
        }
        self.apply_binding_document_edit_with_label(document, "Binding Route Target Edit")?;
        Ok(true)
    }

    pub fn set_selected_binding_action_target(
        &mut self,
        value: impl AsRef<str>,
    ) -> Result<bool, UiAssetEditorSessionError> {
        self.ensure_editable_source()?;
        let mut document = self.last_valid_document.clone();
        if !set_selected_binding_action_target_field(
            &mut document,
            &self.selection,
            self.selected_binding_index,
            value.as_ref(),
        ) {
            return Ok(false);
        }
        self.apply_binding_document_edit_with_label(document, "Binding Action Target Edit")?;
        Ok(true)
    }

    pub fn apply_selected_binding_route_suggestion(
        &mut self,
        index: usize,
    ) -> Result<bool, UiAssetEditorSessionError> {
        self.ensure_editable_source()?;
        let binding_fields = build_binding_fields(
            &self.last_valid_document,
            &self.selection,
            &self.preview_mock_state,
            self.selected_binding_index,
            self.selected_binding_payload_key.as_deref(),
        );
        if index >= binding_fields.binding_route_suggestion_items.len() {
            return Err(UiAssetEditorSessionError::InvalidSelectionIndex { index });
        }
        let mut document = self.last_valid_document.clone();
        if !apply_selected_binding_route_suggestion_field(
            &mut document,
            &self.selection,
            self.selected_binding_index,
            index,
        ) {
            return Ok(false);
        }
        self.apply_binding_document_edit_with_label(document, "Binding Route Suggestion Apply")?;
        Ok(true)
    }

    pub fn apply_selected_binding_action_suggestion(
        &mut self,
        index: usize,
    ) -> Result<bool, UiAssetEditorSessionError> {
        self.ensure_editable_source()?;
        let binding_fields = build_binding_fields(
            &self.last_valid_document,
            &self.selection,
            &self.preview_mock_state,
            self.selected_binding_index,
            self.selected_binding_payload_key.as_deref(),
        );
        if index >= binding_fields.binding_action_suggestion_items.len() {
            return Err(UiAssetEditorSessionError::InvalidSelectionIndex { index });
        }
        let mut document = self.last_valid_document.clone();
        if !apply_selected_binding_action_suggestion_field(
            &mut document,
            &self.selection,
            self.selected_binding_index,
            index,
        ) {
            return Ok(false);
        }
        self.apply_binding_document_edit_with_label(document, "Binding Action Suggestion Apply")?;
        Ok(true)
    }

    pub fn select_binding_payload(
        &mut self,
        index: usize,
    ) -> Result<bool, UiAssetEditorSessionError> {
        let binding_fields = build_binding_fields(
            &self.last_valid_document,
            &self.selection,
            &self.preview_mock_state,
            self.selected_binding_index,
            self.selected_binding_payload_key.as_deref(),
        );
        let Some(item) = binding_fields.binding_payload_items.get(index) else {
            return Err(UiAssetEditorSessionError::InvalidSelectionIndex { index });
        };
        let Some((payload_key, _)) = item.split_once(" = ") else {
            return Err(UiAssetEditorSessionError::InvalidSelectionIndex { index });
        };
        let changed = self.selected_binding_payload_key.as_deref() != Some(payload_key);
        self.selected_binding_payload_key = Some(payload_key.to_string());
        Ok(changed)
    }

    pub fn upsert_selected_binding_payload(
        &mut self,
        payload_key: impl AsRef<str>,
        value_literal: impl AsRef<str>,
    ) -> Result<bool, UiAssetEditorSessionError> {
        self.ensure_editable_source()?;
        let mut document = self.last_valid_document.clone();
        let Some(resolved_payload_key) = upsert_selected_binding_payload_field(
            &mut document,
            &self.selection,
            self.selected_binding_index,
            self.selected_binding_payload_key.as_deref(),
            payload_key.as_ref(),
            value_literal.as_ref(),
        ) else {
            return Ok(false);
        };
        self.selected_binding_payload_key = Some(resolved_payload_key);
        self.apply_binding_document_edit_with_label(document, "Binding Payload Upsert")?;
        Ok(true)
    }

    pub fn delete_selected_binding_payload(&mut self) -> Result<bool, UiAssetEditorSessionError> {
        self.ensure_editable_source()?;
        let mut document = self.last_valid_document.clone();
        if !delete_selected_binding_payload_field(
            &mut document,
            &self.selection,
            self.selected_binding_index,
            self.selected_binding_payload_key.as_deref(),
        ) {
            return Ok(false);
        }
        self.apply_binding_document_edit_with_label(document, "Binding Payload Delete")?;
        Ok(true)
    }

    pub fn apply_selected_binding_payload_suggestion(
        &mut self,
        index: usize,
    ) -> Result<bool, UiAssetEditorSessionError> {
        self.ensure_editable_source()?;
        let binding_fields = build_binding_fields(
            &self.last_valid_document,
            &self.selection,
            &self.preview_mock_state,
            self.selected_binding_index,
            self.selected_binding_payload_key.as_deref(),
        );
        if index >= binding_fields.binding_payload_suggestion_items.len() {
            return Err(UiAssetEditorSessionError::InvalidSelectionIndex { index });
        }
        let mut document = self.last_valid_document.clone();
        let Some(resolved_payload_key) = apply_selected_binding_payload_suggestion_field(
            &mut document,
            &self.selection,
            self.selected_binding_index,
            self.selected_binding_payload_key.as_deref(),
            index,
        ) else {
            return Ok(false);
        };
        self.selected_binding_payload_key = Some(resolved_payload_key);
        self.apply_binding_document_edit_with_label(document, "Binding Payload Suggestion Apply")?;
        Ok(true)
    }
}
