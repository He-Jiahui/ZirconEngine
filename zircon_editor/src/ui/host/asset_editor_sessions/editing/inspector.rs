use super::*;

impl EditorUiHost {
    pub(crate) fn begin_ui_asset_editor_canvas_anchor_popup(
        &self,
        instance_id: &ViewInstanceId,
        visible_frame: &std::sync::Arc<zircon_runtime_interface::ui::surface::UiSurfaceFrame>,
    ) -> Result<Option<crate::ui::asset_editor::UiAssetCanvasAnchorPresetTicket>, EditorError> {
        let Some(_edit) = self.begin_document_edit_if_registered(instance_id)? else {
            return Ok(None);
        };
        if !self
            .lock_session()
            .open_view_instances
            .contains_key(instance_id)
        {
            return Ok(None);
        }
        let mut sessions = self.lock_ui_asset_sessions();
        let Some(entry) = sessions.get_mut(instance_id) else {
            return Ok(None);
        };
        let matches_visible = entry.session.preview_host_opt().is_some_and(|host| {
            std::sync::Arc::ptr_eq(visible_frame, &host.surface().surface_frame())
        });
        if !matches_visible {
            return Ok(None);
        }
        entry
            .session
            .begin_canvas_anchor_preset()
            .map_err(|error| EditorError::UiAsset(error.to_string()))
    }

    pub(crate) fn ui_asset_editor_canvas_anchor_popup_is_current(
        &self,
        instance_id: &ViewInstanceId,
        ticket: &crate::ui::asset_editor::UiAssetCanvasAnchorPresetTicket,
    ) -> Result<bool, EditorError> {
        let Some(_edit) = self.begin_document_edit_if_registered(instance_id)? else {
            return Ok(false);
        };
        let sessions = self.lock_ui_asset_sessions();
        Ok(sessions
            .get(instance_id)
            .is_some_and(|entry| ticket.is_current(&entry.session)))
    }

    pub(crate) fn submit_ui_asset_editor_canvas_anchor_popup(
        &self,
        instance_id: &ViewInstanceId,
        ticket: &crate::ui::asset_editor::UiAssetCanvasAnchorPresetTicket,
        index: usize,
        shift: bool,
        control: bool,
    ) -> Result<Option<crate::ui::asset_editor::UiAssetCanvasAnchorPresetCommit>, EditorError> {
        let Some(_edit) = self.begin_document_edit_if_registered(instance_id)? else {
            return Ok(None);
        };
        let mut sessions = self.lock_ui_asset_sessions();
        let Some(entry) = sessions.get_mut(instance_id) else {
            return Ok(None);
        };
        if !ticket.is_current(&entry.session) {
            return Ok(None);
        }
        let changed = entry
            .session
            .apply_canvas_anchor_preset(ticket, index, shift, control)
            .map_err(|error| EditorError::UiAsset(error.to_string()))?;
        drop(sessions);
        // The document edit guard remains live; the Session lock is released.
        let sync_error = if changed {
            self.sync_ui_asset_editor_instance(instance_id)
                .err()
                .map(|error| error.to_string())
        } else {
            None
        };
        Ok(Some(
            crate::ui::asset_editor::UiAssetCanvasAnchorPresetCommit {
                changed,
                sync_error,
            },
        ))
    }

    pub(crate) fn cancel_ui_asset_editor_canvas_anchor_popup(
        &self,
        instance_id: &ViewInstanceId,
        ticket: &crate::ui::asset_editor::UiAssetCanvasAnchorPresetTicket,
    ) -> Result<bool, EditorError> {
        let Some(_edit) = self.begin_document_edit_if_registered(instance_id)? else {
            return Ok(false);
        };
        let mut sessions = self.lock_ui_asset_sessions();
        let Some(entry) = sessions.get_mut(instance_id) else {
            return Ok(false);
        };
        // Cancel only compares the original lease. No history, sync, focus or Stop.
        Ok(entry.session.cancel_canvas_anchor_preset(ticket))
    }

    pub fn set_ui_asset_editor_selected_widget_control_id(
        &self,
        instance_id: &ViewInstanceId,
        control_id: impl AsRef<str>,
    ) -> Result<bool, EditorError> {
        self.ensure_ui_asset_editor_session(instance_id)?;
        let _edit = self.begin_document_edit(instance_id)?;
        let mut sessions = self.lock_ui_asset_sessions();
        let entry = sessions.get_mut(instance_id).ok_or_else(|| {
            EditorError::UiAsset(format!("missing ui asset session {}", instance_id.0))
        })?;
        let changed = entry
            .session
            .set_selected_widget_control_id(control_id.as_ref())
            .map_err(|error| EditorError::UiAsset(error.to_string()))?;
        drop(sessions);
        self.sync_ui_asset_editor_instance_if_changed(instance_id, changed)?;
        Ok(changed)
    }

    pub fn set_ui_asset_editor_selected_widget_text_property(
        &self,
        instance_id: &ViewInstanceId,
        text: impl AsRef<str>,
    ) -> Result<bool, EditorError> {
        self.ensure_ui_asset_editor_session(instance_id)?;
        let _edit = self.begin_document_edit(instance_id)?;
        let mut sessions = self.lock_ui_asset_sessions();
        let entry = sessions.get_mut(instance_id).ok_or_else(|| {
            EditorError::UiAsset(format!("missing ui asset session {}", instance_id.0))
        })?;
        let changed = entry
            .session
            .set_selected_widget_text_property(text.as_ref())
            .map_err(|error| EditorError::UiAsset(error.to_string()))?;
        drop(sessions);
        self.sync_ui_asset_editor_instance_if_changed(instance_id, changed)?;
        Ok(changed)
    }

    pub fn set_ui_asset_editor_selected_widget_prop_literal(
        &self,
        instance_id: &ViewInstanceId,
        path: impl AsRef<str>,
        literal: impl AsRef<str>,
    ) -> Result<bool, EditorError> {
        self.ensure_ui_asset_editor_session(instance_id)?;
        let _edit = self.begin_document_edit(instance_id)?;
        let mut sessions = self.lock_ui_asset_sessions();
        let entry = sessions.get_mut(instance_id).ok_or_else(|| {
            EditorError::UiAsset(format!("missing ui asset session {}", instance_id.0))
        })?;
        let changed = entry
            .session
            .set_selected_widget_prop_literal(path.as_ref(), literal.as_ref())
            .map_err(|error| EditorError::UiAsset(error.to_string()))?;
        drop(sessions);
        self.sync_ui_asset_editor_instance_if_changed(instance_id, changed)?;
        Ok(changed)
    }

    pub fn set_ui_asset_editor_selected_widget_state_literal(
        &self,
        instance_id: &ViewInstanceId,
        path: impl AsRef<str>,
        literal: impl AsRef<str>,
    ) -> Result<bool, EditorError> {
        self.ensure_ui_asset_editor_session(instance_id)?;
        let _edit = self.begin_document_edit(instance_id)?;
        let mut sessions = self.lock_ui_asset_sessions();
        let entry = sessions.get_mut(instance_id).ok_or_else(|| {
            EditorError::UiAsset(format!("missing ui asset session {}", instance_id.0))
        })?;
        let changed = entry
            .session
            .set_selected_widget_state_literal(path.as_ref(), literal.as_ref())
            .map_err(|error| EditorError::UiAsset(error.to_string()))?;
        drop(sessions);
        self.sync_ui_asset_editor_instance_if_changed(instance_id, changed)?;
        Ok(changed)
    }

    pub fn set_ui_asset_editor_selected_component_root_class_policy(
        &self,
        instance_id: &ViewInstanceId,
        policy: impl AsRef<str>,
    ) -> Result<bool, EditorError> {
        self.ensure_ui_asset_editor_session(instance_id)?;
        let _edit = self.begin_document_edit(instance_id)?;
        let mut sessions = self.lock_ui_asset_sessions();
        let entry = sessions.get_mut(instance_id).ok_or_else(|| {
            EditorError::UiAsset(format!("missing ui asset session {}", instance_id.0))
        })?;
        let changed = entry
            .session
            .set_selected_component_root_class_policy(policy.as_ref())
            .map_err(|error| EditorError::UiAsset(error.to_string()))?;
        drop(sessions);
        self.sync_ui_asset_editor_instance_if_changed(instance_id, changed)?;
        Ok(changed)
    }

    pub fn set_ui_asset_editor_selected_promote_widget_asset_id(
        &self,
        instance_id: &ViewInstanceId,
        asset_id: impl AsRef<str>,
    ) -> Result<bool, EditorError> {
        self.ensure_ui_asset_editor_session(instance_id)?;
        let _edit = self.begin_document_edit(instance_id)?;
        let mut sessions = self.lock_ui_asset_sessions();
        let entry = sessions.get_mut(instance_id).ok_or_else(|| {
            EditorError::UiAsset(format!("missing ui asset session {}", instance_id.0))
        })?;
        let changed = entry
            .session
            .set_selected_promote_widget_asset_id(asset_id.as_ref())
            .map_err(|error| EditorError::UiAsset(error.to_string()))?;
        drop(sessions);
        self.sync_ui_asset_editor_instance_if_changed(instance_id, changed)?;
        Ok(changed)
    }

    pub fn set_ui_asset_editor_selected_promote_widget_component_name(
        &self,
        instance_id: &ViewInstanceId,
        component_name: impl AsRef<str>,
    ) -> Result<bool, EditorError> {
        self.ensure_ui_asset_editor_session(instance_id)?;
        let _edit = self.begin_document_edit(instance_id)?;
        let mut sessions = self.lock_ui_asset_sessions();
        let entry = sessions.get_mut(instance_id).ok_or_else(|| {
            EditorError::UiAsset(format!("missing ui asset session {}", instance_id.0))
        })?;
        let changed = entry
            .session
            .set_selected_promote_widget_component_name(component_name.as_ref())
            .map_err(|error| EditorError::UiAsset(error.to_string()))?;
        drop(sessions);
        self.sync_ui_asset_editor_instance_if_changed(instance_id, changed)?;
        Ok(changed)
    }

    pub fn set_ui_asset_editor_selected_promote_widget_document_id(
        &self,
        instance_id: &ViewInstanceId,
        document_id: impl AsRef<str>,
    ) -> Result<bool, EditorError> {
        self.ensure_ui_asset_editor_session(instance_id)?;
        let _edit = self.begin_document_edit(instance_id)?;
        let mut sessions = self.lock_ui_asset_sessions();
        let entry = sessions.get_mut(instance_id).ok_or_else(|| {
            EditorError::UiAsset(format!("missing ui asset session {}", instance_id.0))
        })?;
        let changed = entry
            .session
            .set_selected_promote_widget_document_id(document_id.as_ref())
            .map_err(|error| EditorError::UiAsset(error.to_string()))?;
        drop(sessions);
        self.sync_ui_asset_editor_instance_if_changed(instance_id, changed)?;
        Ok(changed)
    }

    pub fn set_ui_asset_editor_selected_slot_mount(
        &self,
        instance_id: &ViewInstanceId,
        mount: impl AsRef<str>,
    ) -> Result<bool, EditorError> {
        self.ensure_ui_asset_editor_session(instance_id)?;
        let _edit = self.begin_document_edit(instance_id)?;
        let mut sessions = self.lock_ui_asset_sessions();
        let entry = sessions.get_mut(instance_id).ok_or_else(|| {
            EditorError::UiAsset(format!("missing ui asset session {}", instance_id.0))
        })?;

        let changed = entry
            .session
            .set_selected_slot_mount(mount.as_ref())
            .map_err(|error| EditorError::UiAsset(error.to_string()))?;
        drop(sessions);
        self.sync_ui_asset_editor_instance_if_changed(instance_id, changed)?;
        Ok(changed)
    }

    pub fn set_ui_asset_editor_selected_slot_padding(
        &self,
        instance_id: &ViewInstanceId,
        literal: impl AsRef<str>,
    ) -> Result<bool, EditorError> {
        self.ensure_ui_asset_editor_session(instance_id)?;
        let _edit = self.begin_document_edit(instance_id)?;
        let mut sessions = self.lock_ui_asset_sessions();
        let entry = sessions.get_mut(instance_id).ok_or_else(|| {
            EditorError::UiAsset(format!("missing ui asset session {}", instance_id.0))
        })?;

        let changed = entry
            .session
            .set_selected_slot_padding(literal.as_ref())
            .map_err(|error| EditorError::UiAsset(error.to_string()))?;
        drop(sessions);
        self.sync_ui_asset_editor_instance_if_changed(instance_id, changed)?;
        Ok(changed)
    }

    pub fn set_ui_asset_editor_selected_slot_width_preferred(
        &self,
        instance_id: &ViewInstanceId,
        literal: impl AsRef<str>,
    ) -> Result<bool, EditorError> {
        self.ensure_ui_asset_editor_session(instance_id)?;
        let _edit = self.begin_document_edit(instance_id)?;
        let mut sessions = self.lock_ui_asset_sessions();
        let entry = sessions.get_mut(instance_id).ok_or_else(|| {
            EditorError::UiAsset(format!("missing ui asset session {}", instance_id.0))
        })?;

        let changed = entry
            .session
            .set_selected_slot_width_preferred(literal.as_ref())
            .map_err(|error| EditorError::UiAsset(error.to_string()))?;
        drop(sessions);
        self.sync_ui_asset_editor_instance_if_changed(instance_id, changed)?;
        Ok(changed)
    }

    pub fn set_ui_asset_editor_selected_slot_height_preferred(
        &self,
        instance_id: &ViewInstanceId,
        literal: impl AsRef<str>,
    ) -> Result<bool, EditorError> {
        self.ensure_ui_asset_editor_session(instance_id)?;
        let _edit = self.begin_document_edit(instance_id)?;
        let mut sessions = self.lock_ui_asset_sessions();
        let entry = sessions.get_mut(instance_id).ok_or_else(|| {
            EditorError::UiAsset(format!("missing ui asset session {}", instance_id.0))
        })?;

        let changed = entry
            .session
            .set_selected_slot_height_preferred(literal.as_ref())
            .map_err(|error| EditorError::UiAsset(error.to_string()))?;
        drop(sessions);
        self.sync_ui_asset_editor_instance_if_changed(instance_id, changed)?;
        Ok(changed)
    }

    pub fn set_ui_asset_editor_designer_tool_mode(
        &self,
        instance_id: &ViewInstanceId,
        mode: UiDesignerToolMode,
    ) -> Result<bool, EditorError> {
        self.ensure_ui_asset_editor_session(instance_id)?;
        let _edit = self.begin_document_edit(instance_id)?;
        let mut sessions = self.lock_ui_asset_sessions();
        let entry = sessions.get_mut(instance_id).ok_or_else(|| {
            EditorError::UiAsset(format!("missing ui asset session {}", instance_id.0))
        })?;

        let changed = entry.session.set_designer_tool_mode(mode);
        drop(sessions);
        self.sync_ui_asset_editor_instance_if_changed(instance_id, changed)?;
        Ok(changed)
    }

    pub fn set_ui_asset_editor_locale_preview(
        &self,
        instance_id: &ViewInstanceId,
        locale: impl AsRef<str>,
    ) -> Result<bool, EditorError> {
        self.ensure_ui_asset_editor_session(instance_id)?;
        let _edit = self.begin_document_edit(instance_id)?;
        let mut sessions = self.lock_ui_asset_sessions();
        let entry = sessions.get_mut(instance_id).ok_or_else(|| {
            EditorError::UiAsset(format!("missing ui asset session {}", instance_id.0))
        })?;

        let changed = entry.session.set_locale_preview(locale.as_ref());
        drop(sessions);
        self.sync_ui_asset_editor_instance_if_changed(instance_id, changed)?;
        Ok(changed)
    }

    pub fn resize_ui_asset_editor_selected_slot_preferred_size(
        &self,
        instance_id: &ViewInstanceId,
        width: f32,
        height: f32,
    ) -> Result<bool, EditorError> {
        self.ensure_ui_asset_editor_session(instance_id)?;
        let _edit = self.begin_document_edit(instance_id)?;
        let mut sessions = self.lock_ui_asset_sessions();
        let entry = sessions.get_mut(instance_id).ok_or_else(|| {
            EditorError::UiAsset(format!("missing ui asset session {}", instance_id.0))
        })?;

        let changed = entry
            .session
            .resize_selected_slot_preferred_size(width, height)
            .map_err(|error| EditorError::UiAsset(error.to_string()))?;
        drop(sessions);
        self.sync_ui_asset_editor_instance_if_changed(instance_id, changed)?;
        Ok(changed)
    }

    pub fn set_ui_asset_editor_selected_layout_width_preferred(
        &self,
        instance_id: &ViewInstanceId,
        literal: impl AsRef<str>,
    ) -> Result<bool, EditorError> {
        self.ensure_ui_asset_editor_session(instance_id)?;
        let _edit = self.begin_document_edit(instance_id)?;
        let mut sessions = self.lock_ui_asset_sessions();
        let entry = sessions.get_mut(instance_id).ok_or_else(|| {
            EditorError::UiAsset(format!("missing ui asset session {}", instance_id.0))
        })?;

        let changed = entry
            .session
            .set_selected_layout_width_preferred(literal.as_ref())
            .map_err(|error| EditorError::UiAsset(error.to_string()))?;
        drop(sessions);
        self.sync_ui_asset_editor_instance_if_changed(instance_id, changed)?;
        Ok(changed)
    }

    pub fn set_ui_asset_editor_selected_layout_height_preferred(
        &self,
        instance_id: &ViewInstanceId,
        literal: impl AsRef<str>,
    ) -> Result<bool, EditorError> {
        self.ensure_ui_asset_editor_session(instance_id)?;
        let _edit = self.begin_document_edit(instance_id)?;
        let mut sessions = self.lock_ui_asset_sessions();
        let entry = sessions.get_mut(instance_id).ok_or_else(|| {
            EditorError::UiAsset(format!("missing ui asset session {}", instance_id.0))
        })?;

        let changed = entry
            .session
            .set_selected_layout_height_preferred(literal.as_ref())
            .map_err(|error| EditorError::UiAsset(error.to_string()))?;
        drop(sessions);
        self.sync_ui_asset_editor_instance_if_changed(instance_id, changed)?;
        Ok(changed)
    }

    pub fn select_ui_asset_editor_slot_semantic(
        &self,
        instance_id: &ViewInstanceId,
        index: usize,
    ) -> Result<bool, EditorError> {
        self.ensure_ui_asset_editor_session(instance_id)?;
        let _edit = self.begin_document_edit(instance_id)?;
        let mut sessions = self.lock_ui_asset_sessions();
        let entry = sessions.get_mut(instance_id).ok_or_else(|| {
            EditorError::UiAsset(format!("missing ui asset session {}", instance_id.0))
        })?;
        let changed = entry
            .session
            .select_slot_semantic(index)
            .map_err(|error| EditorError::UiAsset(error.to_string()))?;
        drop(sessions);
        self.sync_ui_asset_editor_instance_if_changed(instance_id, changed)?;
        Ok(changed)
    }

    pub fn set_ui_asset_editor_selected_slot_semantic_value(
        &self,
        instance_id: &ViewInstanceId,
        value: impl AsRef<str>,
    ) -> Result<bool, EditorError> {
        self.ensure_ui_asset_editor_session(instance_id)?;
        let _edit = self.begin_document_edit(instance_id)?;
        let mut sessions = self.lock_ui_asset_sessions();
        let entry = sessions.get_mut(instance_id).ok_or_else(|| {
            EditorError::UiAsset(format!("missing ui asset session {}", instance_id.0))
        })?;
        let changed = entry
            .session
            .set_selected_slot_semantic_value(value.as_ref())
            .map_err(|error| EditorError::UiAsset(error.to_string()))?;
        drop(sessions);
        self.sync_ui_asset_editor_instance_if_changed(instance_id, changed)?;
        Ok(changed)
    }

    pub fn set_ui_asset_editor_selected_slot_semantic_field(
        &self,
        instance_id: &ViewInstanceId,
        path: impl AsRef<str>,
        value: impl AsRef<str>,
    ) -> Result<bool, EditorError> {
        self.ensure_ui_asset_editor_session(instance_id)?;
        let _edit = self.begin_document_edit(instance_id)?;
        let mut sessions = self.lock_ui_asset_sessions();
        let entry = sessions.get_mut(instance_id).ok_or_else(|| {
            EditorError::UiAsset(format!("missing ui asset session {}", instance_id.0))
        })?;
        let changed = entry
            .session
            .set_selected_slot_semantic_field(path.as_ref(), value.as_ref())
            .map_err(|error| EditorError::UiAsset(error.to_string()))?;
        drop(sessions);
        self.sync_ui_asset_editor_instance_if_changed(instance_id, changed)?;
        Ok(changed)
    }

    pub fn delete_ui_asset_editor_selected_slot_semantic(
        &self,
        instance_id: &ViewInstanceId,
    ) -> Result<bool, EditorError> {
        self.ensure_ui_asset_editor_session(instance_id)?;
        let _edit = self.begin_document_edit(instance_id)?;
        let mut sessions = self.lock_ui_asset_sessions();
        let entry = sessions.get_mut(instance_id).ok_or_else(|| {
            EditorError::UiAsset(format!("missing ui asset session {}", instance_id.0))
        })?;
        let changed = entry
            .session
            .delete_selected_slot_semantic()
            .map_err(|error| EditorError::UiAsset(error.to_string()))?;
        drop(sessions);
        self.sync_ui_asset_editor_instance_if_changed(instance_id, changed)?;
        Ok(changed)
    }

    pub fn select_ui_asset_editor_layout_semantic(
        &self,
        instance_id: &ViewInstanceId,
        index: usize,
    ) -> Result<bool, EditorError> {
        self.ensure_ui_asset_editor_session(instance_id)?;
        let _edit = self.begin_document_edit(instance_id)?;
        let mut sessions = self.lock_ui_asset_sessions();
        let entry = sessions.get_mut(instance_id).ok_or_else(|| {
            EditorError::UiAsset(format!("missing ui asset session {}", instance_id.0))
        })?;
        let changed = entry
            .session
            .select_layout_semantic(index)
            .map_err(|error| EditorError::UiAsset(error.to_string()))?;
        drop(sessions);
        self.sync_ui_asset_editor_instance_if_changed(instance_id, changed)?;
        Ok(changed)
    }

    pub fn set_ui_asset_editor_selected_layout_semantic_value(
        &self,
        instance_id: &ViewInstanceId,
        value: impl AsRef<str>,
    ) -> Result<bool, EditorError> {
        self.ensure_ui_asset_editor_session(instance_id)?;
        let _edit = self.begin_document_edit(instance_id)?;
        let mut sessions = self.lock_ui_asset_sessions();
        let entry = sessions.get_mut(instance_id).ok_or_else(|| {
            EditorError::UiAsset(format!("missing ui asset session {}", instance_id.0))
        })?;
        let changed = entry
            .session
            .set_selected_layout_semantic_value(value.as_ref())
            .map_err(|error| EditorError::UiAsset(error.to_string()))?;
        drop(sessions);
        self.sync_ui_asset_editor_instance_if_changed(instance_id, changed)?;
        Ok(changed)
    }

    pub fn set_ui_asset_editor_selected_layout_semantic_field(
        &self,
        instance_id: &ViewInstanceId,
        path: impl AsRef<str>,
        value: impl AsRef<str>,
    ) -> Result<bool, EditorError> {
        self.ensure_ui_asset_editor_session(instance_id)?;
        let _edit = self.begin_document_edit(instance_id)?;
        let mut sessions = self.lock_ui_asset_sessions();
        let entry = sessions.get_mut(instance_id).ok_or_else(|| {
            EditorError::UiAsset(format!("missing ui asset session {}", instance_id.0))
        })?;
        let changed = entry
            .session
            .set_selected_layout_semantic_field(path.as_ref(), value.as_ref())
            .map_err(|error| EditorError::UiAsset(error.to_string()))?;
        drop(sessions);
        self.sync_ui_asset_editor_instance_if_changed(instance_id, changed)?;
        Ok(changed)
    }

    pub fn delete_ui_asset_editor_selected_layout_semantic(
        &self,
        instance_id: &ViewInstanceId,
    ) -> Result<bool, EditorError> {
        self.ensure_ui_asset_editor_session(instance_id)?;
        let _edit = self.begin_document_edit(instance_id)?;
        let mut sessions = self.lock_ui_asset_sessions();
        let entry = sessions.get_mut(instance_id).ok_or_else(|| {
            EditorError::UiAsset(format!("missing ui asset session {}", instance_id.0))
        })?;
        let changed = entry
            .session
            .delete_selected_layout_semantic()
            .map_err(|error| EditorError::UiAsset(error.to_string()))?;
        drop(sessions);
        self.sync_ui_asset_editor_instance_if_changed(instance_id, changed)?;
        Ok(changed)
    }

    fn sync_ui_asset_editor_instance_if_changed(
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
#[path = "tests/inspector_perf.rs"]
mod performance_tests;
