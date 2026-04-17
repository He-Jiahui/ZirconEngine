mod hydration;
mod imports;
mod lifecycle;
mod open;
mod preview_refresh;
mod save;
mod sync;

use std::fs;
use std::path::PathBuf;

use crate::default_constraints_for_content;
use crate::view::{PreferredHost, ViewDescriptor, ViewDescriptorId, ViewInstanceId, ViewKind};
use crate::{
    EditorError, EditorManager, UiAssetEditorCommand, UiAssetEditorMode, UiAssetEditorSession,
    UiAssetPreviewPreset, UiSize, ViewContentKind,
};
use zircon_asset::UiWidgetAsset;

use super::project_access::normalize_ui_asset_asset_id;
use super::ui_asset_promotion::resolve_external_widget_target;

pub(super) const UI_ASSET_EDITOR_DESCRIPTOR_ID: &str = "editor.ui_asset";

pub(super) struct UiAssetWorkspaceEntry {
    pub(super) source_path: PathBuf,
    pub(super) session: UiAssetEditorSession,
}

pub(super) fn ui_asset_editor_view_descriptor() -> ViewDescriptor {
    ViewDescriptor::new(
        ViewDescriptorId::new(UI_ASSET_EDITOR_DESCRIPTOR_ID),
        ViewKind::ActivityWindow,
        "UI Asset Editor",
    )
    .with_multi_instance(true)
    .with_preferred_host(PreferredHost::DocumentCenter)
    .with_default_constraints(default_constraints_for_content(
        ViewContentKind::UiAssetEditor,
    ))
    .with_icon_key("ui-asset")
}

impl EditorManager {
    pub fn update_ui_asset_editor_source(
        &self,
        instance_id: &ViewInstanceId,
        next_source: impl Into<String>,
    ) -> Result<(), EditorError> {
        let next_source = next_source.into();
        self.ensure_ui_asset_editor_session(instance_id)?;
        let mut sessions = self.ui_asset_sessions.lock().unwrap();
        let entry = sessions.get_mut(instance_id).ok_or_else(|| {
            EditorError::UiAsset(format!("missing ui asset session {}", instance_id.0))
        })?;
        entry
            .session
            .apply_command(UiAssetEditorCommand::edit_source(next_source))
            .map_err(|error| EditorError::UiAsset(error.to_string()))?;
        drop(sessions);
        self.hydrate_ui_asset_editor_imports(instance_id)?;
        self.sync_ui_asset_editor_instance(instance_id)
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

    pub fn set_ui_asset_editor_selected_widget_control_id(
        &self,
        instance_id: &ViewInstanceId,
        control_id: impl AsRef<str>,
    ) -> Result<bool, EditorError> {
        self.ensure_ui_asset_editor_session(instance_id)?;
        let mut sessions = self.ui_asset_sessions.lock().unwrap();
        let entry = sessions.get_mut(instance_id).ok_or_else(|| {
            EditorError::UiAsset(format!("missing ui asset session {}", instance_id.0))
        })?;
        let changed = entry
            .session
            .set_selected_widget_control_id(control_id.as_ref())
            .map_err(|error| EditorError::UiAsset(error.to_string()))?;
        drop(sessions);
        self.sync_ui_asset_editor_instance(instance_id)?;
        Ok(changed)
    }

    pub fn set_ui_asset_editor_selected_widget_text_property(
        &self,
        instance_id: &ViewInstanceId,
        text: impl AsRef<str>,
    ) -> Result<bool, EditorError> {
        self.ensure_ui_asset_editor_session(instance_id)?;
        let mut sessions = self.ui_asset_sessions.lock().unwrap();
        let entry = sessions.get_mut(instance_id).ok_or_else(|| {
            EditorError::UiAsset(format!("missing ui asset session {}", instance_id.0))
        })?;
        let changed = entry
            .session
            .set_selected_widget_text_property(text.as_ref())
            .map_err(|error| EditorError::UiAsset(error.to_string()))?;
        drop(sessions);
        self.sync_ui_asset_editor_instance(instance_id)?;
        Ok(changed)
    }

    pub fn set_ui_asset_editor_selected_promote_widget_asset_id(
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
            .set_selected_promote_widget_asset_id(asset_id.as_ref())
            .map_err(|error| EditorError::UiAsset(error.to_string()))?;
        drop(sessions);
        self.sync_ui_asset_editor_instance(instance_id)?;
        Ok(changed)
    }

    pub fn set_ui_asset_editor_selected_promote_widget_component_name(
        &self,
        instance_id: &ViewInstanceId,
        component_name: impl AsRef<str>,
    ) -> Result<bool, EditorError> {
        self.ensure_ui_asset_editor_session(instance_id)?;
        let mut sessions = self.ui_asset_sessions.lock().unwrap();
        let entry = sessions.get_mut(instance_id).ok_or_else(|| {
            EditorError::UiAsset(format!("missing ui asset session {}", instance_id.0))
        })?;
        let changed = entry
            .session
            .set_selected_promote_widget_component_name(component_name.as_ref())
            .map_err(|error| EditorError::UiAsset(error.to_string()))?;
        drop(sessions);
        self.sync_ui_asset_editor_instance(instance_id)?;
        Ok(changed)
    }

    pub fn set_ui_asset_editor_selected_promote_widget_document_id(
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
            .set_selected_promote_widget_document_id(document_id.as_ref())
            .map_err(|error| EditorError::UiAsset(error.to_string()))?;
        drop(sessions);
        self.sync_ui_asset_editor_instance(instance_id)?;
        Ok(changed)
    }

    pub fn set_ui_asset_editor_selected_slot_mount(
        &self,
        instance_id: &ViewInstanceId,
        mount: impl AsRef<str>,
    ) -> Result<bool, EditorError> {
        self.ensure_ui_asset_editor_session(instance_id)?;
        let mut sessions = self.ui_asset_sessions.lock().unwrap();
        let entry = sessions.get_mut(instance_id).ok_or_else(|| {
            EditorError::UiAsset(format!("missing ui asset session {}", instance_id.0))
        })?;

        let changed = entry
            .session
            .set_selected_slot_mount(mount.as_ref())
            .map_err(|error| EditorError::UiAsset(error.to_string()))?;
        drop(sessions);
        self.sync_ui_asset_editor_instance(instance_id)?;
        Ok(changed)
    }

    pub fn set_ui_asset_editor_selected_slot_padding(
        &self,
        instance_id: &ViewInstanceId,
        literal: impl AsRef<str>,
    ) -> Result<bool, EditorError> {
        self.ensure_ui_asset_editor_session(instance_id)?;
        let mut sessions = self.ui_asset_sessions.lock().unwrap();
        let entry = sessions.get_mut(instance_id).ok_or_else(|| {
            EditorError::UiAsset(format!("missing ui asset session {}", instance_id.0))
        })?;

        let changed = entry
            .session
            .set_selected_slot_padding(literal.as_ref())
            .map_err(|error| EditorError::UiAsset(error.to_string()))?;
        drop(sessions);
        self.sync_ui_asset_editor_instance(instance_id)?;
        Ok(changed)
    }

    pub fn set_ui_asset_editor_selected_slot_width_preferred(
        &self,
        instance_id: &ViewInstanceId,
        literal: impl AsRef<str>,
    ) -> Result<bool, EditorError> {
        self.ensure_ui_asset_editor_session(instance_id)?;
        let mut sessions = self.ui_asset_sessions.lock().unwrap();
        let entry = sessions.get_mut(instance_id).ok_or_else(|| {
            EditorError::UiAsset(format!("missing ui asset session {}", instance_id.0))
        })?;

        let changed = entry
            .session
            .set_selected_slot_width_preferred(literal.as_ref())
            .map_err(|error| EditorError::UiAsset(error.to_string()))?;
        drop(sessions);
        self.sync_ui_asset_editor_instance(instance_id)?;
        Ok(changed)
    }

    pub fn set_ui_asset_editor_selected_slot_height_preferred(
        &self,
        instance_id: &ViewInstanceId,
        literal: impl AsRef<str>,
    ) -> Result<bool, EditorError> {
        self.ensure_ui_asset_editor_session(instance_id)?;
        let mut sessions = self.ui_asset_sessions.lock().unwrap();
        let entry = sessions.get_mut(instance_id).ok_or_else(|| {
            EditorError::UiAsset(format!("missing ui asset session {}", instance_id.0))
        })?;

        let changed = entry
            .session
            .set_selected_slot_height_preferred(literal.as_ref())
            .map_err(|error| EditorError::UiAsset(error.to_string()))?;
        drop(sessions);
        self.sync_ui_asset_editor_instance(instance_id)?;
        Ok(changed)
    }

    pub fn set_ui_asset_editor_selected_layout_width_preferred(
        &self,
        instance_id: &ViewInstanceId,
        literal: impl AsRef<str>,
    ) -> Result<bool, EditorError> {
        self.ensure_ui_asset_editor_session(instance_id)?;
        let mut sessions = self.ui_asset_sessions.lock().unwrap();
        let entry = sessions.get_mut(instance_id).ok_or_else(|| {
            EditorError::UiAsset(format!("missing ui asset session {}", instance_id.0))
        })?;

        let changed = entry
            .session
            .set_selected_layout_width_preferred(literal.as_ref())
            .map_err(|error| EditorError::UiAsset(error.to_string()))?;
        drop(sessions);
        self.sync_ui_asset_editor_instance(instance_id)?;
        Ok(changed)
    }

    pub fn set_ui_asset_editor_selected_layout_height_preferred(
        &self,
        instance_id: &ViewInstanceId,
        literal: impl AsRef<str>,
    ) -> Result<bool, EditorError> {
        self.ensure_ui_asset_editor_session(instance_id)?;
        let mut sessions = self.ui_asset_sessions.lock().unwrap();
        let entry = sessions.get_mut(instance_id).ok_or_else(|| {
            EditorError::UiAsset(format!("missing ui asset session {}", instance_id.0))
        })?;

        let changed = entry
            .session
            .set_selected_layout_height_preferred(literal.as_ref())
            .map_err(|error| EditorError::UiAsset(error.to_string()))?;
        drop(sessions);
        self.sync_ui_asset_editor_instance(instance_id)?;
        Ok(changed)
    }

    pub fn select_ui_asset_editor_binding(
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
            .select_binding(index)
            .map_err(|error| EditorError::UiAsset(error.to_string()))?;
        drop(sessions);
        self.sync_ui_asset_editor_instance(instance_id)?;
        Ok(changed)
    }

    pub fn select_ui_asset_editor_slot_semantic(
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
            .select_slot_semantic(index)
            .map_err(|error| EditorError::UiAsset(error.to_string()))?;
        drop(sessions);
        self.sync_ui_asset_editor_instance(instance_id)?;
        Ok(changed)
    }

    pub fn set_ui_asset_editor_selected_slot_semantic_value(
        &self,
        instance_id: &ViewInstanceId,
        value: impl AsRef<str>,
    ) -> Result<bool, EditorError> {
        self.ensure_ui_asset_editor_session(instance_id)?;
        let mut sessions = self.ui_asset_sessions.lock().unwrap();
        let entry = sessions.get_mut(instance_id).ok_or_else(|| {
            EditorError::UiAsset(format!("missing ui asset session {}", instance_id.0))
        })?;
        let changed = entry
            .session
            .set_selected_slot_semantic_value(value.as_ref())
            .map_err(|error| EditorError::UiAsset(error.to_string()))?;
        drop(sessions);
        self.sync_ui_asset_editor_instance(instance_id)?;
        Ok(changed)
    }

    pub fn set_ui_asset_editor_selected_slot_semantic_field(
        &self,
        instance_id: &ViewInstanceId,
        path: impl AsRef<str>,
        value: impl AsRef<str>,
    ) -> Result<bool, EditorError> {
        self.ensure_ui_asset_editor_session(instance_id)?;
        let mut sessions = self.ui_asset_sessions.lock().unwrap();
        let entry = sessions.get_mut(instance_id).ok_or_else(|| {
            EditorError::UiAsset(format!("missing ui asset session {}", instance_id.0))
        })?;
        let changed = entry
            .session
            .set_selected_slot_semantic_field(path.as_ref(), value.as_ref())
            .map_err(|error| EditorError::UiAsset(error.to_string()))?;
        drop(sessions);
        self.sync_ui_asset_editor_instance(instance_id)?;
        Ok(changed)
    }

    pub fn delete_ui_asset_editor_selected_slot_semantic(
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
            .delete_selected_slot_semantic()
            .map_err(|error| EditorError::UiAsset(error.to_string()))?;
        drop(sessions);
        self.sync_ui_asset_editor_instance(instance_id)?;
        Ok(changed)
    }

    pub fn select_ui_asset_editor_layout_semantic(
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
            .select_layout_semantic(index)
            .map_err(|error| EditorError::UiAsset(error.to_string()))?;
        drop(sessions);
        self.sync_ui_asset_editor_instance(instance_id)?;
        Ok(changed)
    }

    pub fn set_ui_asset_editor_selected_layout_semantic_value(
        &self,
        instance_id: &ViewInstanceId,
        value: impl AsRef<str>,
    ) -> Result<bool, EditorError> {
        self.ensure_ui_asset_editor_session(instance_id)?;
        let mut sessions = self.ui_asset_sessions.lock().unwrap();
        let entry = sessions.get_mut(instance_id).ok_or_else(|| {
            EditorError::UiAsset(format!("missing ui asset session {}", instance_id.0))
        })?;
        let changed = entry
            .session
            .set_selected_layout_semantic_value(value.as_ref())
            .map_err(|error| EditorError::UiAsset(error.to_string()))?;
        drop(sessions);
        self.sync_ui_asset_editor_instance(instance_id)?;
        Ok(changed)
    }

    pub fn set_ui_asset_editor_selected_layout_semantic_field(
        &self,
        instance_id: &ViewInstanceId,
        path: impl AsRef<str>,
        value: impl AsRef<str>,
    ) -> Result<bool, EditorError> {
        self.ensure_ui_asset_editor_session(instance_id)?;
        let mut sessions = self.ui_asset_sessions.lock().unwrap();
        let entry = sessions.get_mut(instance_id).ok_or_else(|| {
            EditorError::UiAsset(format!("missing ui asset session {}", instance_id.0))
        })?;
        let changed = entry
            .session
            .set_selected_layout_semantic_field(path.as_ref(), value.as_ref())
            .map_err(|error| EditorError::UiAsset(error.to_string()))?;
        drop(sessions);
        self.sync_ui_asset_editor_instance(instance_id)?;
        Ok(changed)
    }

    pub fn delete_ui_asset_editor_selected_layout_semantic(
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
            .delete_selected_layout_semantic()
            .map_err(|error| EditorError::UiAsset(error.to_string()))?;
        drop(sessions);
        self.sync_ui_asset_editor_instance(instance_id)?;
        Ok(changed)
    }

    pub fn add_ui_asset_editor_binding(
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
            .add_binding()
            .map_err(|error| EditorError::UiAsset(error.to_string()))?;
        drop(sessions);
        self.sync_ui_asset_editor_instance(instance_id)?;
        Ok(changed)
    }

    pub fn select_ui_asset_editor_binding_event_option(
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
            .select_binding_event_option(index)
            .map_err(|error| EditorError::UiAsset(error.to_string()))?;
        drop(sessions);
        self.sync_ui_asset_editor_instance(instance_id)?;
        Ok(changed)
    }

    pub fn delete_ui_asset_editor_selected_binding(
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
            .delete_selected_binding()
            .map_err(|error| EditorError::UiAsset(error.to_string()))?;
        drop(sessions);
        self.sync_ui_asset_editor_instance(instance_id)?;
        Ok(changed)
    }

    pub fn set_ui_asset_editor_selected_binding_id(
        &self,
        instance_id: &ViewInstanceId,
        value: impl AsRef<str>,
    ) -> Result<bool, EditorError> {
        self.ensure_ui_asset_editor_session(instance_id)?;
        let mut sessions = self.ui_asset_sessions.lock().unwrap();
        let entry = sessions.get_mut(instance_id).ok_or_else(|| {
            EditorError::UiAsset(format!("missing ui asset session {}", instance_id.0))
        })?;
        let changed = entry
            .session
            .set_selected_binding_id(value.as_ref())
            .map_err(|error| EditorError::UiAsset(error.to_string()))?;
        drop(sessions);
        self.sync_ui_asset_editor_instance(instance_id)?;
        Ok(changed)
    }

    pub fn set_ui_asset_editor_selected_binding_event(
        &self,
        instance_id: &ViewInstanceId,
        value: impl AsRef<str>,
    ) -> Result<bool, EditorError> {
        self.ensure_ui_asset_editor_session(instance_id)?;
        let mut sessions = self.ui_asset_sessions.lock().unwrap();
        let entry = sessions.get_mut(instance_id).ok_or_else(|| {
            EditorError::UiAsset(format!("missing ui asset session {}", instance_id.0))
        })?;
        let changed = entry
            .session
            .set_selected_binding_event(value.as_ref())
            .map_err(|error| EditorError::UiAsset(error.to_string()))?;
        drop(sessions);
        self.sync_ui_asset_editor_instance(instance_id)?;
        Ok(changed)
    }

    pub fn select_ui_asset_editor_binding_action_kind(
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
            .select_binding_action_kind(index)
            .map_err(|error| EditorError::UiAsset(error.to_string()))?;
        drop(sessions);
        self.sync_ui_asset_editor_instance(instance_id)?;
        Ok(changed)
    }

    pub fn set_ui_asset_editor_selected_binding_route(
        &self,
        instance_id: &ViewInstanceId,
        value: impl AsRef<str>,
    ) -> Result<bool, EditorError> {
        self.ensure_ui_asset_editor_session(instance_id)?;
        let mut sessions = self.ui_asset_sessions.lock().unwrap();
        let entry = sessions.get_mut(instance_id).ok_or_else(|| {
            EditorError::UiAsset(format!("missing ui asset session {}", instance_id.0))
        })?;
        let changed = entry
            .session
            .set_selected_binding_route(value.as_ref())
            .map_err(|error| EditorError::UiAsset(error.to_string()))?;
        drop(sessions);
        self.sync_ui_asset_editor_instance(instance_id)?;
        Ok(changed)
    }

    pub fn set_ui_asset_editor_selected_binding_route_target(
        &self,
        instance_id: &ViewInstanceId,
        value: impl AsRef<str>,
    ) -> Result<bool, EditorError> {
        self.ensure_ui_asset_editor_session(instance_id)?;
        let mut sessions = self.ui_asset_sessions.lock().unwrap();
        let entry = sessions.get_mut(instance_id).ok_or_else(|| {
            EditorError::UiAsset(format!("missing ui asset session {}", instance_id.0))
        })?;
        let changed = entry
            .session
            .set_selected_binding_route_target(value.as_ref())
            .map_err(|error| EditorError::UiAsset(error.to_string()))?;
        drop(sessions);
        self.sync_ui_asset_editor_instance(instance_id)?;
        Ok(changed)
    }

    pub fn set_ui_asset_editor_selected_binding_action_target(
        &self,
        instance_id: &ViewInstanceId,
        value: impl AsRef<str>,
    ) -> Result<bool, EditorError> {
        self.ensure_ui_asset_editor_session(instance_id)?;
        let mut sessions = self.ui_asset_sessions.lock().unwrap();
        let entry = sessions.get_mut(instance_id).ok_or_else(|| {
            EditorError::UiAsset(format!("missing ui asset session {}", instance_id.0))
        })?;
        let changed = entry
            .session
            .set_selected_binding_action_target(value.as_ref())
            .map_err(|error| EditorError::UiAsset(error.to_string()))?;
        drop(sessions);
        self.sync_ui_asset_editor_instance(instance_id)?;
        Ok(changed)
    }

    pub fn select_ui_asset_editor_binding_payload(
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
            .select_binding_payload(index)
            .map_err(|error| EditorError::UiAsset(error.to_string()))?;
        drop(sessions);
        self.sync_ui_asset_editor_instance(instance_id)?;
        Ok(changed)
    }

    pub fn upsert_ui_asset_editor_selected_binding_payload(
        &self,
        instance_id: &ViewInstanceId,
        payload_key: impl AsRef<str>,
        value_literal: impl AsRef<str>,
    ) -> Result<bool, EditorError> {
        self.ensure_ui_asset_editor_session(instance_id)?;
        let mut sessions = self.ui_asset_sessions.lock().unwrap();
        let entry = sessions.get_mut(instance_id).ok_or_else(|| {
            EditorError::UiAsset(format!("missing ui asset session {}", instance_id.0))
        })?;
        let changed = entry
            .session
            .upsert_selected_binding_payload(payload_key.as_ref(), value_literal.as_ref())
            .map_err(|error| EditorError::UiAsset(error.to_string()))?;
        drop(sessions);
        self.sync_ui_asset_editor_instance(instance_id)?;
        Ok(changed)
    }

    pub fn delete_ui_asset_editor_selected_binding_payload(
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
            .delete_selected_binding_payload()
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

    pub fn undo_ui_asset_editor(&self, instance_id: &ViewInstanceId) -> Result<bool, EditorError> {
        self.ensure_ui_asset_editor_session(instance_id)?;
        let mut sessions = self.ui_asset_sessions.lock().unwrap();
        let entry = sessions.get_mut(instance_id).ok_or_else(|| {
            EditorError::UiAsset(format!("missing ui asset session {}", instance_id.0))
        })?;
        let changed = entry
            .session
            .undo()
            .map_err(|error| EditorError::UiAsset(error.to_string()))?;
        drop(sessions);
        self.hydrate_ui_asset_editor_imports(instance_id)?;
        self.sync_ui_asset_editor_instance(instance_id)?;
        Ok(changed)
    }

    pub fn redo_ui_asset_editor(&self, instance_id: &ViewInstanceId) -> Result<bool, EditorError> {
        self.ensure_ui_asset_editor_session(instance_id)?;
        let mut sessions = self.ui_asset_sessions.lock().unwrap();
        let entry = sessions.get_mut(instance_id).ok_or_else(|| {
            EditorError::UiAsset(format!("missing ui asset session {}", instance_id.0))
        })?;
        let changed = entry
            .session
            .redo()
            .map_err(|error| EditorError::UiAsset(error.to_string()))?;
        drop(sessions);
        self.hydrate_ui_asset_editor_imports(instance_id)?;
        self.sync_ui_asset_editor_instance(instance_id)?;
        Ok(changed)
    }

    pub fn set_ui_asset_editor_mode(
        &self,
        instance_id: &ViewInstanceId,
        mode: UiAssetEditorMode,
    ) -> Result<(), EditorError> {
        self.ensure_ui_asset_editor_session(instance_id)?;
        let mut sessions = self.ui_asset_sessions.lock().unwrap();
        let entry = sessions.get_mut(instance_id).ok_or_else(|| {
            EditorError::UiAsset(format!("missing ui asset session {}", instance_id.0))
        })?;
        entry
            .session
            .set_mode(mode)
            .map_err(|error| EditorError::UiAsset(error.to_string()))?;
        drop(sessions);
        self.hydrate_ui_asset_editor_imports(instance_id)?;
        self.sync_ui_asset_editor_instance(instance_id)
    }

    pub fn select_ui_asset_editor_hierarchy_index(
        &self,
        instance_id: &ViewInstanceId,
        index: usize,
    ) -> Result<(), EditorError> {
        self.ensure_ui_asset_editor_session(instance_id)?;
        let mut sessions = self.ui_asset_sessions.lock().unwrap();
        let entry = sessions.get_mut(instance_id).ok_or_else(|| {
            EditorError::UiAsset(format!("missing ui asset session {}", instance_id.0))
        })?;
        entry
            .session
            .select_hierarchy_index(index)
            .map_err(|error| EditorError::UiAsset(error.to_string()))?;
        drop(sessions);
        self.sync_ui_asset_editor_instance(instance_id)
    }

    pub fn activate_ui_asset_editor_hierarchy_index(
        &self,
        instance_id: &ViewInstanceId,
        index: usize,
    ) -> Result<Option<ViewInstanceId>, EditorError> {
        self.select_ui_asset_editor_hierarchy_index(instance_id, index)?;
        self.open_ui_asset_editor_selected_reference(instance_id)
    }

    pub fn select_ui_asset_editor_source_outline_index(
        &self,
        instance_id: &ViewInstanceId,
        index: usize,
    ) -> Result<(), EditorError> {
        self.ensure_ui_asset_editor_session(instance_id)?;
        let mut sessions = self.ui_asset_sessions.lock().unwrap();
        let entry = sessions.get_mut(instance_id).ok_or_else(|| {
            EditorError::UiAsset(format!("missing ui asset session {}", instance_id.0))
        })?;
        entry
            .session
            .select_source_outline_index(index)
            .map_err(|error| EditorError::UiAsset(error.to_string()))?;
        drop(sessions);
        self.sync_ui_asset_editor_instance(instance_id)
    }

    pub fn select_ui_asset_editor_palette_index(
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
            .select_palette_index(index)
            .map_err(|error| EditorError::UiAsset(error.to_string()))?;
        drop(sessions);
        self.sync_ui_asset_editor_instance(instance_id)?;
        Ok(changed)
    }

    pub fn update_ui_asset_editor_palette_drag_target(
        &self,
        instance_id: &ViewInstanceId,
        surface_x: f32,
        surface_y: f32,
    ) -> Result<bool, EditorError> {
        self.ensure_ui_asset_editor_session(instance_id)?;
        let mut sessions = self.ui_asset_sessions.lock().unwrap();
        let entry = sessions.get_mut(instance_id).ok_or_else(|| {
            EditorError::UiAsset(format!("missing ui asset session {}", instance_id.0))
        })?;
        let changed = entry
            .session
            .update_palette_drag_target(surface_x, surface_y)
            .map_err(|error| EditorError::UiAsset(error.to_string()))?;
        drop(sessions);
        self.sync_ui_asset_editor_instance(instance_id)?;
        Ok(changed)
    }

    pub fn clear_ui_asset_editor_palette_drag_target(
        &self,
        instance_id: &ViewInstanceId,
    ) -> Result<bool, EditorError> {
        self.ensure_ui_asset_editor_session(instance_id)?;
        let mut sessions = self.ui_asset_sessions.lock().unwrap();
        let entry = sessions.get_mut(instance_id).ok_or_else(|| {
            EditorError::UiAsset(format!("missing ui asset session {}", instance_id.0))
        })?;
        let changed = entry.session.clear_palette_drag_target();
        drop(sessions);
        self.sync_ui_asset_editor_instance(instance_id)?;
        Ok(changed)
    }

    pub fn cycle_ui_asset_editor_palette_drag_target_candidate_next(
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
            .cycle_palette_drag_target_candidate_next()
            .map_err(|error| EditorError::UiAsset(error.to_string()))?;
        drop(sessions);
        self.sync_ui_asset_editor_instance(instance_id)?;
        Ok(changed)
    }

    pub fn cycle_ui_asset_editor_palette_drag_target_candidate_previous(
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
            .cycle_palette_drag_target_candidate_previous()
            .map_err(|error| EditorError::UiAsset(error.to_string()))?;
        drop(sessions);
        self.sync_ui_asset_editor_instance(instance_id)?;
        Ok(changed)
    }

    pub fn select_ui_asset_editor_palette_target_candidate(
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
            .select_palette_target_candidate(index)
            .map_err(|error| EditorError::UiAsset(error.to_string()))?;
        drop(sessions);
        self.sync_ui_asset_editor_instance(instance_id)?;
        Ok(changed)
    }

    pub fn confirm_ui_asset_editor_palette_target_choice(
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
            .confirm_palette_target_choice()
            .map_err(|error| EditorError::UiAsset(error.to_string()))?;
        drop(sessions);
        self.sync_ui_asset_editor_instance(instance_id)?;
        Ok(changed)
    }

    pub fn cancel_ui_asset_editor_palette_target_choice(
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
            .cancel_palette_target_choice()
            .map_err(|error| EditorError::UiAsset(error.to_string()))?;
        drop(sessions);
        self.sync_ui_asset_editor_instance(instance_id)?;
        Ok(changed)
    }

    pub fn drop_ui_asset_editor_selected_palette_item_at_drag_target(
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
            .drop_selected_palette_item_at_palette_drag_target()
            .map_err(|error| EditorError::UiAsset(error.to_string()))?;
        drop(sessions);
        self.sync_ui_asset_editor_instance(instance_id)?;
        Ok(changed)
    }

    pub fn insert_ui_asset_editor_selected_palette_item_as_child(
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
            .insert_selected_palette_item_as_child()
            .map_err(|error| EditorError::UiAsset(error.to_string()))?;
        drop(sessions);
        self.sync_ui_asset_editor_instance(instance_id)?;
        Ok(changed)
    }

    pub fn insert_ui_asset_editor_selected_palette_item_after_selection(
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
            .insert_selected_palette_item_after_selection()
            .map_err(|error| EditorError::UiAsset(error.to_string()))?;
        drop(sessions);
        self.sync_ui_asset_editor_instance(instance_id)?;
        Ok(changed)
    }

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

fn preview_size_for_preset(preview_preset: UiAssetPreviewPreset) -> UiSize {
    match preview_preset {
        UiAssetPreviewPreset::EditorDocked => UiSize::new(1280.0, 720.0),
        UiAssetPreviewPreset::EditorFloating => UiSize::new(1100.0, 780.0),
        UiAssetPreviewPreset::GameHud => UiSize::new(1920.0, 1080.0),
        UiAssetPreviewPreset::Dialog => UiSize::new(640.0, 480.0),
    }
}
