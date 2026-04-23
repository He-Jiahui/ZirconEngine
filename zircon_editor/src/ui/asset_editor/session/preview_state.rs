use super::{
    preview_mock::{
        apply_selected_preview_mock_suggestion as apply_selected_preview_mock_suggestion_field,
        clear_selected_preview_mock_value,
        delete_selected_preview_mock_nested_entry as delete_selected_preview_mock_nested_entry_field,
        select_preview_mock_nested_entry as select_preview_mock_nested_entry_field,
        select_preview_mock_property, select_preview_mock_subject,
        select_preview_mock_subject_node,
        set_selected_preview_mock_nested_value as set_selected_preview_mock_nested_value_field,
        set_selected_preview_mock_value,
        upsert_selected_preview_mock_nested_entry as upsert_selected_preview_mock_nested_entry_field,
    },
    ui_asset_editor_session::{UiAssetEditorSession, UiAssetEditorSessionError},
};

impl UiAssetEditorSession {
    pub fn select_preview_mock_property(
        &mut self,
        index: usize,
    ) -> Result<bool, UiAssetEditorSessionError> {
        let Some(changed) = select_preview_mock_property(
            &self.last_valid_document,
            &self.selection,
            &mut self.preview_mock_state,
            index,
        ) else {
            return Err(UiAssetEditorSessionError::InvalidSelectionIndex { index });
        };
        Ok(changed)
    }

    pub fn select_preview_mock_nested_entry(
        &mut self,
        index: usize,
    ) -> Result<bool, UiAssetEditorSessionError> {
        let Some(changed) = select_preview_mock_nested_entry_field(
            &self.last_valid_document,
            &self.selection,
            &mut self.preview_mock_state,
            index,
        ) else {
            return Err(UiAssetEditorSessionError::InvalidSelectionIndex { index });
        };
        Ok(changed)
    }

    pub fn select_preview_mock_subject_node(
        &mut self,
        node_id: impl AsRef<str>,
    ) -> Result<bool, UiAssetEditorSessionError> {
        Ok(select_preview_mock_subject_node(
            &self.last_valid_document,
            &self.selection,
            &mut self.preview_mock_state,
            node_id.as_ref(),
        ))
    }

    pub fn select_preview_mock_subject(
        &mut self,
        index: usize,
    ) -> Result<bool, UiAssetEditorSessionError> {
        let Some(changed) = select_preview_mock_subject(
            &self.last_valid_document,
            &self.selection,
            &mut self.preview_mock_state,
            index,
        ) else {
            return Err(UiAssetEditorSessionError::InvalidSelectionIndex { index });
        };
        Ok(changed)
    }

    pub fn set_selected_preview_mock_value(
        &mut self,
        value: impl AsRef<str>,
    ) -> Result<bool, UiAssetEditorSessionError> {
        let changed = set_selected_preview_mock_value(
            &self.last_valid_document,
            &self.selection,
            &mut self.preview_mock_state,
            value.as_ref(),
        )
        .map_err(|message| UiAssetEditorSessionError::InvalidPreviewMockValue { message })?;
        if !changed {
            return Ok(false);
        }
        self.rebuild_preview_snapshot()?;
        Ok(true)
    }

    pub fn set_selected_preview_mock_nested_value(
        &mut self,
        value: impl AsRef<str>,
    ) -> Result<bool, UiAssetEditorSessionError> {
        let changed = set_selected_preview_mock_nested_value_field(
            &self.last_valid_document,
            &self.selection,
            &mut self.preview_mock_state,
            value.as_ref(),
        )
        .map_err(|message| UiAssetEditorSessionError::InvalidPreviewMockValue { message })?;
        if !changed {
            return Ok(false);
        }
        self.rebuild_preview_snapshot()?;
        Ok(true)
    }

    pub fn upsert_selected_preview_mock_nested_entry(
        &mut self,
        key: impl AsRef<str>,
        value_literal: impl AsRef<str>,
    ) -> Result<bool, UiAssetEditorSessionError> {
        let changed = upsert_selected_preview_mock_nested_entry_field(
            &self.last_valid_document,
            &self.selection,
            &mut self.preview_mock_state,
            key.as_ref(),
            value_literal.as_ref(),
        )
        .map_err(|message| UiAssetEditorSessionError::InvalidPreviewMockValue { message })?;
        if !changed {
            return Ok(false);
        }
        self.rebuild_preview_snapshot()?;
        Ok(true)
    }

    pub fn apply_selected_preview_mock_suggestion(
        &mut self,
        index: usize,
    ) -> Result<bool, UiAssetEditorSessionError> {
        let before_state = self.preview_mock_state.clone();
        let Some(_) = apply_selected_preview_mock_suggestion_field(
            &self.last_valid_document,
            &self.selection,
            &mut self.preview_mock_state,
            index,
        )
        .map_err(|message| UiAssetEditorSessionError::InvalidPreviewMockValue { message })?
        else {
            return Err(UiAssetEditorSessionError::InvalidSelectionIndex { index });
        };
        if before_state == self.preview_mock_state {
            return Ok(false);
        }
        self.rebuild_preview_snapshot()?;
        Ok(true)
    }

    pub fn delete_selected_preview_mock_nested_entry(
        &mut self,
    ) -> Result<bool, UiAssetEditorSessionError> {
        let changed = delete_selected_preview_mock_nested_entry_field(
            &self.last_valid_document,
            &self.selection,
            &mut self.preview_mock_state,
        )
        .map_err(|message| UiAssetEditorSessionError::InvalidPreviewMockValue { message })?;
        if !changed {
            return Ok(false);
        }
        self.rebuild_preview_snapshot()?;
        Ok(true)
    }

    pub fn clear_selected_preview_mock_value(&mut self) -> Result<bool, UiAssetEditorSessionError> {
        if !clear_selected_preview_mock_value(
            &self.last_valid_document,
            &self.selection,
            &mut self.preview_mock_state,
        ) {
            return Ok(false);
        }
        self.rebuild_preview_snapshot()?;
        Ok(true)
    }
}
