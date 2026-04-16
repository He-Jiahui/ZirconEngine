use std::collections::{BTreeMap, BTreeSet};
use std::fs;
use std::path::{Path, PathBuf};

use crate::default_constraints_for_content;
use crate::view::{
    PreferredHost, ViewDescriptor, ViewDescriptorId, ViewInstance, ViewInstanceId, ViewKind,
};
use crate::{
    EditorError, EditorManager, UiAssetEditorCommand, UiAssetEditorMode,
    UiAssetEditorPanePresentation, UiAssetEditorReflectionModel, UiAssetEditorRoute,
    UiAssetEditorSession, UiSize, ViewContentKind,
};
use zircon_ui::{UiAssetDocument, UiAssetLoader};

use super::project_access::normalize_ui_asset_asset_id;

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
    pub fn open_ui_asset_editor(
        &self,
        path: impl AsRef<Path>,
        mode: Option<UiAssetEditorMode>,
    ) -> Result<ViewInstanceId, EditorError> {
        self.open_ui_asset_editor_by_id(path.as_ref().to_string_lossy(), mode)
    }

    pub fn open_ui_asset_editor_by_id(
        &self,
        asset_id: impl AsRef<str>,
        mode: Option<UiAssetEditorMode>,
    ) -> Result<ViewInstanceId, EditorError> {
        let asset_id = normalize_ui_asset_asset_id(asset_id.as_ref()).to_string();
        let source_path = self.resolve_ui_asset_path(&asset_id)?;
        let source = fs::read_to_string(&source_path)
            .map_err(|error| EditorError::UiAsset(error.to_string()))?;
        let document = UiAssetLoader::load_toml_str(&source)
            .map_err(|error| EditorError::UiAsset(error.to_string()))?;
        let route =
            UiAssetEditorRoute::new(asset_id, document.asset.kind, mode.unwrap_or_default());
        let session = UiAssetEditorSession::from_source(route, source, default_preview_size())
            .map_err(|error| EditorError::UiAsset(error.to_string()))?;
        let instance_id =
            self.open_view(ViewDescriptorId::new(UI_ASSET_EDITOR_DESCRIPTOR_ID), None)?;
        self.ui_asset_sessions.lock().unwrap().insert(
            instance_id.clone(),
            UiAssetWorkspaceEntry {
                source_path,
                session,
            },
        );
        self.hydrate_ui_asset_editor_imports(&instance_id)?;
        self.sync_ui_asset_editor_instance(&instance_id)?;
        let _ = self.focus_view(&instance_id);
        Ok(instance_id)
    }

    pub fn ui_asset_editor_reflection(
        &self,
        instance_id: &ViewInstanceId,
    ) -> Result<UiAssetEditorReflectionModel, EditorError> {
        self.ensure_ui_asset_editor_session(instance_id)?;
        let sessions = self.ui_asset_sessions.lock().unwrap();
        let entry = sessions.get(instance_id).ok_or_else(|| {
            EditorError::UiAsset(format!("missing ui asset session {}", instance_id.0))
        })?;
        Ok(entry.session.reflection_model())
    }

    pub fn ui_asset_editor_pane_presentation(
        &self,
        instance_id: &ViewInstanceId,
    ) -> Result<UiAssetEditorPanePresentation, EditorError> {
        self.ensure_ui_asset_editor_session(instance_id)?;
        let sessions = self.ui_asset_sessions.lock().unwrap();
        let entry = sessions.get(instance_id).ok_or_else(|| {
            EditorError::UiAsset(format!("missing ui asset session {}", instance_id.0))
        })?;
        Ok(entry.session.pane_presentation())
    }

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

    pub fn save_ui_asset_editor(
        &self,
        instance_id: &ViewInstanceId,
    ) -> Result<String, EditorError> {
        self.ensure_ui_asset_editor_session(instance_id)?;
        let (saved, asset_id, source_path) = {
            let mut sessions = self.ui_asset_sessions.lock().unwrap();
            let entry = sessions.get_mut(instance_id).ok_or_else(|| {
                EditorError::UiAsset(format!("missing ui asset session {}", instance_id.0))
            })?;
            let saved = entry
                .session
                .save_to_canonical_source()
                .map_err(|error| EditorError::UiAsset(error.to_string()))?;
            (
                saved,
                entry.session.route().asset_id.clone(),
                entry.source_path.clone(),
            )
        };
        fs::write(&source_path, &saved).map_err(|error| EditorError::UiAsset(error.to_string()))?;
        if asset_id.starts_with("res://") {
            let normalized = normalize_ui_asset_asset_id(&asset_id).to_string();
            let _ = self.asset_manager()?.import_asset(&normalized);
        }
        self.hydrate_ui_asset_editor_imports(instance_id)?;
        self.sync_ui_asset_editor_instance(instance_id)?;
        Ok(saved)
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

    pub fn select_ui_asset_editor_preview_index(
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
            .select_preview_index(index)
            .map_err(|error| EditorError::UiAsset(error.to_string()))?;
        drop(sessions);
        self.sync_ui_asset_editor_instance(instance_id)
    }

    pub(super) fn restore_ui_asset_editor_instance(
        &self,
        instance: &ViewInstance,
    ) -> Result<(), EditorError> {
        let route: UiAssetEditorRoute =
            if let Ok(route) = serde_json::from_value(instance.serializable_payload.clone()) {
                route
            } else if let Some(asset_id) = instance
                .serializable_payload
                .get("path")
                .and_then(|value| value.as_str())
            {
                let source_path = self.resolve_ui_asset_path(asset_id)?;
                let source = fs::read_to_string(&source_path)
                    .map_err(|error| EditorError::UiAsset(error.to_string()))?;
                let document = UiAssetLoader::load_toml_str(&source)
                    .map_err(|error| EditorError::UiAsset(error.to_string()))?;
                UiAssetEditorRoute::new(asset_id, document.asset.kind, UiAssetEditorMode::Design)
            } else {
                return Err(EditorError::UiAsset(format!(
                    "invalid ui asset route for {}",
                    instance.instance_id.0
                )));
            };
        let source_path = self.resolve_ui_asset_path(&route.asset_id)?;
        let source = fs::read_to_string(&source_path)
            .map_err(|error| EditorError::UiAsset(error.to_string()))?;
        let session = UiAssetEditorSession::from_source(route, source, default_preview_size())
            .map_err(|error| EditorError::UiAsset(error.to_string()))?;
        self.ui_asset_sessions.lock().unwrap().insert(
            instance.instance_id.clone(),
            UiAssetWorkspaceEntry {
                source_path,
                session,
            },
        );
        self.hydrate_ui_asset_editor_imports(&instance.instance_id)?;
        self.sync_ui_asset_editor_instance(&instance.instance_id)
    }

    fn ensure_ui_asset_editor_session(
        &self,
        instance_id: &ViewInstanceId,
    ) -> Result<(), EditorError> {
        if self
            .ui_asset_sessions
            .lock()
            .unwrap()
            .contains_key(instance_id)
        {
            return Ok(());
        }
        let instance = self
            .session
            .lock()
            .unwrap()
            .open_view_instances
            .get(instance_id)
            .cloned()
            .ok_or_else(|| {
                EditorError::UiAsset(format!("missing ui asset view {}", instance_id.0))
            })?;
        self.restore_ui_asset_editor_instance(&instance)
    }

    fn sync_ui_asset_editor_instance(
        &self,
        instance_id: &ViewInstanceId,
    ) -> Result<(), EditorError> {
        let (title, dirty, payload) = {
            let sessions = self.ui_asset_sessions.lock().unwrap();
            let entry = sessions.get(instance_id).ok_or_else(|| {
                EditorError::UiAsset(format!("missing ui asset session {}", instance_id.0))
            })?;
            (
                entry.session.reflection_model().display_name,
                entry.session.reflection_model().source_dirty,
                serde_json::to_value(entry.session.route())
                    .map_err(|error| EditorError::UiAsset(error.to_string()))?,
            )
        };
        let mut session = self.session.lock().unwrap();
        let instance = session
            .open_view_instances
            .get_mut(instance_id)
            .ok_or_else(|| {
                EditorError::UiAsset(format!("missing ui asset view {}", instance_id.0))
            })?;
        instance.title = title;
        instance.dirty = dirty;
        instance.serializable_payload = payload;
        Ok(())
    }

    fn hydrate_ui_asset_editor_imports(
        &self,
        instance_id: &ViewInstanceId,
    ) -> Result<(), EditorError> {
        let (widget_refs, style_refs) = {
            let sessions = self.ui_asset_sessions.lock().unwrap();
            let entry = sessions.get(instance_id).ok_or_else(|| {
                EditorError::UiAsset(format!("missing ui asset session {}", instance_id.0))
            })?;
            entry.session.import_references()
        };
        let mut widget_docs = BTreeMap::new();
        let mut style_docs = BTreeMap::new();
        let mut visited = BTreeSet::new();
        for reference in widget_refs {
            self.collect_ui_asset_import_document(
                &reference,
                zircon_ui::UiAssetKind::Widget,
                &mut widget_docs,
                &mut style_docs,
                &mut visited,
            )?;
        }
        for reference in style_refs {
            self.collect_ui_asset_import_document(
                &reference,
                zircon_ui::UiAssetKind::Style,
                &mut widget_docs,
                &mut style_docs,
                &mut visited,
            )?;
        }

        let mut sessions = self.ui_asset_sessions.lock().unwrap();
        let entry = sessions.get_mut(instance_id).ok_or_else(|| {
            EditorError::UiAsset(format!("missing ui asset session {}", instance_id.0))
        })?;
        entry
            .session
            .replace_imports(widget_docs, style_docs)
            .map_err(|error| EditorError::UiAsset(error.to_string()))
    }

    fn collect_ui_asset_import_document(
        &self,
        reference: &str,
        expected_kind: zircon_ui::UiAssetKind,
        widget_docs: &mut BTreeMap<String, UiAssetDocument>,
        style_docs: &mut BTreeMap<String, UiAssetDocument>,
        visited: &mut BTreeSet<String>,
    ) -> Result<(), EditorError> {
        let source_path = self.resolve_ui_asset_path(reference)?;
        let source = fs::read_to_string(&source_path)
            .map_err(|error| EditorError::UiAsset(error.to_string()))?;
        let document = UiAssetLoader::load_toml_str(&source)
            .map_err(|error| EditorError::UiAsset(error.to_string()))?;
        if document.asset.kind != expected_kind {
            return Err(EditorError::UiAsset(format!(
                "ui import {reference} expected {:?} but parsed {:?}",
                expected_kind, document.asset.kind
            )));
        }

        match expected_kind {
            zircon_ui::UiAssetKind::Widget => {
                widget_docs.insert(reference.to_string(), document.clone());
            }
            zircon_ui::UiAssetKind::Style => {
                style_docs.insert(reference.to_string(), document.clone());
            }
            zircon_ui::UiAssetKind::Layout => {}
        }

        let visited_key = normalize_ui_asset_asset_id(reference).to_string();
        if !visited.insert(visited_key) {
            return Ok(());
        }

        for nested in &document.imports.widgets {
            self.collect_ui_asset_import_document(
                nested,
                zircon_ui::UiAssetKind::Widget,
                widget_docs,
                style_docs,
                visited,
            )?;
        }
        for nested in &document.imports.styles {
            self.collect_ui_asset_import_document(
                nested,
                zircon_ui::UiAssetKind::Style,
                widget_docs,
                style_docs,
                visited,
            )?;
        }
        Ok(())
    }
}

fn default_preview_size() -> UiSize {
    UiSize::new(1280.0, 720.0)
}
