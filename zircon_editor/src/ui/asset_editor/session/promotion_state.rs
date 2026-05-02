use super::{
    command::{UiAssetEditorCommand, UiAssetEditorTreeEdit, UiAssetEditorTreeEditKind},
    command_entry::tree_document_replay_bundle,
    hierarchy_projection::selection_for_node,
    palette::{
        build_palette_entries,
        convert_selected_node_to_reference as tree_convert_selected_node_to_reference,
        UiAssetPaletteEntryKind,
    },
    promote_widget::{
        can_promote_selected_component_to_external_widget, default_external_widget_draft,
        promote_selected_component_to_external_widget as tree_promote_selected_component_to_external_widget,
        selected_local_component_name, UiAssetExternalWidgetDraft,
    },
    theme_authoring::{
        can_promote_local_theme_to_external_style_asset, default_external_style_draft,
        promote_local_theme_to_external_style_asset as tree_promote_local_theme_to_external_style_asset,
        UiAssetExternalStyleDraft,
    },
    theme_state::theme_document_replay_bundle,
    tree_editing::extract_selected_node_to_component as tree_extract_selected_node_to_component,
    ui_asset_editor_session::{
        serialize_document, UiAssetEditorSession, UiAssetEditorSessionError,
    },
    undo_stack::{UiAssetEditorExternalEffect, UiAssetEditorUndoExternalEffects},
};
use zircon_runtime::ui::template::UiAssetDocumentRuntimeExt;
use zircon_runtime_interface::ui::template::{UiAssetDocument, UiAssetError, UiNodeDefinitionKind};

impl UiAssetEditorSession {
    pub fn selected_reference_asset_id(&self) -> Option<String> {
        let node_id = self.selection.primary_node_id.as_deref()?;
        let node = self.last_valid_document.node(node_id)?;
        if node.kind != UiNodeDefinitionKind::Reference {
            return None;
        }
        node.component_ref
            .as_deref()
            .map(reference_asset_id)
            .map(str::to_string)
    }

    pub fn convert_selected_node_to_reference(
        &mut self,
    ) -> Result<bool, UiAssetEditorSessionError> {
        self.ensure_editable_source()?;
        let Some(selected_palette_index) = self.selected_palette_index else {
            return Ok(false);
        };
        let palette_entries =
            build_palette_entries(&self.last_valid_document, &self.compiler_imports.widgets);
        let Some(entry) = palette_entries.get(selected_palette_index) else {
            return Ok(false);
        };
        let mut document = self.last_valid_document.clone();
        let Some(node_id) = tree_convert_selected_node_to_reference(
            &mut document,
            &self.selection,
            entry,
            &self.compiler_imports.widgets,
        ) else {
            return Ok(false);
        };
        let selection = selection_for_node(&document, &node_id);
        let component_ref = match &entry.kind {
            UiAssetPaletteEntryKind::Reference { component_ref } => component_ref.clone(),
            _ => String::new(),
        };
        self.apply_document_edit_with_tree_edit_and_selection(
            document.clone(),
            UiAssetEditorTreeEdit::ConvertToReference {
                node_id,
                component_ref,
            },
            "Convert To Reference",
            selection,
        )?;
        Ok(true)
    }

    pub fn extract_selected_node_to_component(
        &mut self,
    ) -> Result<bool, UiAssetEditorSessionError> {
        self.ensure_editable_source()?;
        let mut document = self.last_valid_document.clone();
        let Some(node_id) = tree_extract_selected_node_to_component(&mut document, &self.selection)
        else {
            return Ok(false);
        };
        let selection = selection_for_node(&document, &node_id);
        let component_name = document
            .node(&node_id)
            .and_then(|node| node.component.clone())
            .unwrap_or_default();
        let component_root_id = document
            .components
            .get(&component_name)
            .map(|component| component.root.node_id.clone())
            .unwrap_or_default();
        self.apply_document_edit_with_tree_edit_and_selection(
            document.clone(),
            UiAssetEditorTreeEdit::ExtractComponent {
                node_id,
                component_name,
                component_root_id,
            },
            "Extract Component",
            selection,
        )?;
        Ok(true)
    }

    pub fn selected_local_component_name(&self) -> Option<String> {
        selected_local_component_name(&self.last_valid_document, &self.selection)
    }

    pub(crate) fn selected_promote_widget_draft(&self) -> Option<UiAssetExternalWidgetDraft> {
        Some(UiAssetExternalWidgetDraft {
            asset_id: self.selected_promote_widget_asset_id.clone()?,
            component_name: self.selected_promote_widget_component_name.clone()?,
            document_id: self.selected_promote_widget_document_id.clone()?,
        })
    }

    pub(crate) fn selected_promote_theme_draft(&self) -> Option<UiAssetExternalStyleDraft> {
        if !can_promote_local_theme_to_external_style_asset(&self.last_valid_document) {
            return None;
        }
        Some(UiAssetExternalStyleDraft {
            asset_id: self.selected_promote_theme_asset_id.clone()?,
            document_id: self.selected_promote_theme_document_id.clone()?,
            display_name: self.selected_promote_theme_display_name.clone()?,
        })
    }

    pub fn set_selected_promote_widget_asset_id(
        &mut self,
        asset_id: impl AsRef<str>,
    ) -> Result<bool, UiAssetEditorSessionError> {
        let Some(normalized) = normalized_promote_asset_id(asset_id.as_ref()) else {
            return Ok(false);
        };
        if self.selected_promote_widget_asset_id.as_deref() == Some(normalized.as_str()) {
            return Ok(false);
        }
        if !can_promote_selected_component_to_external_widget(
            &self.last_valid_document,
            &self.selection,
        ) {
            return Ok(false);
        }
        self.selected_promote_widget_asset_id = Some(normalized);
        Ok(true)
    }

    pub fn set_selected_promote_widget_component_name(
        &mut self,
        component_name: impl AsRef<str>,
    ) -> Result<bool, UiAssetEditorSessionError> {
        let Some(normalized) = normalized_promote_component_name(component_name.as_ref()) else {
            return Ok(false);
        };
        if self.selected_promote_widget_component_name.as_deref() == Some(normalized.as_str()) {
            return Ok(false);
        }
        if !can_promote_selected_component_to_external_widget(
            &self.last_valid_document,
            &self.selection,
        ) {
            return Ok(false);
        }
        self.selected_promote_widget_component_name = Some(normalized);
        Ok(true)
    }

    pub fn set_selected_promote_widget_document_id(
        &mut self,
        document_id: impl AsRef<str>,
    ) -> Result<bool, UiAssetEditorSessionError> {
        let Some(normalized) = normalized_promote_document_id(document_id.as_ref()) else {
            return Ok(false);
        };
        if self.selected_promote_widget_document_id.as_deref() == Some(normalized.as_str()) {
            return Ok(false);
        }
        if !can_promote_selected_component_to_external_widget(
            &self.last_valid_document,
            &self.selection,
        ) {
            return Ok(false);
        }
        self.selected_promote_widget_document_id = Some(normalized);
        Ok(true)
    }

    pub fn set_promote_theme_asset_id(
        &mut self,
        asset_id: impl AsRef<str>,
    ) -> Result<bool, UiAssetEditorSessionError> {
        let Some(normalized) = normalized_promote_asset_id(asset_id.as_ref()) else {
            return Ok(false);
        };
        if self.selected_promote_theme_asset_id.as_deref() == Some(normalized.as_str()) {
            return Ok(false);
        }
        if !can_promote_local_theme_to_external_style_asset(&self.last_valid_document) {
            return Ok(false);
        }
        self.selected_promote_theme_asset_id = Some(normalized);
        Ok(true)
    }

    pub fn set_promote_theme_document_id(
        &mut self,
        document_id: impl AsRef<str>,
    ) -> Result<bool, UiAssetEditorSessionError> {
        let Some(normalized) = normalized_promote_document_id(document_id.as_ref()) else {
            return Ok(false);
        };
        if self.selected_promote_theme_document_id.as_deref() == Some(normalized.as_str()) {
            return Ok(false);
        }
        if !can_promote_local_theme_to_external_style_asset(&self.last_valid_document) {
            return Ok(false);
        }
        self.selected_promote_theme_document_id = Some(normalized);
        Ok(true)
    }

    pub fn set_promote_theme_display_name(
        &mut self,
        display_name: impl AsRef<str>,
    ) -> Result<bool, UiAssetEditorSessionError> {
        let Some(normalized) = normalized_promote_display_name(display_name.as_ref()) else {
            return Ok(false);
        };
        if self.selected_promote_theme_display_name.as_deref() == Some(normalized.as_str()) {
            return Ok(false);
        }
        if !can_promote_local_theme_to_external_style_asset(&self.last_valid_document) {
            return Ok(false);
        }
        self.selected_promote_theme_display_name = Some(normalized);
        Ok(true)
    }

    pub fn promote_selected_component_to_external_widget(
        &mut self,
        widget_asset_id: &str,
        widget_component_name: &str,
        widget_document_id: &str,
    ) -> Result<Option<UiAssetDocument>, UiAssetEditorSessionError> {
        self.ensure_editable_source()?;
        let Some(node_id) = self.selection.primary_node_id.clone() else {
            return Ok(None);
        };
        let Some(source_component_name) = self.selected_local_component_name() else {
            return Ok(None);
        };
        let mut document = self.last_valid_document.clone();
        let previous_widget_source = self.existing_external_widget_source(widget_asset_id)?;
        let Some(widget_document) = tree_promote_selected_component_to_external_widget(
            &mut document,
            &self.selection,
            widget_asset_id,
            widget_component_name,
            widget_document_id,
        ) else {
            return Ok(None);
        };
        let widget_source = toml::to_string_pretty(&widget_document)
            .map_err(|error| UiAssetError::ParseToml(error.to_string()))?;
        let widget_reference = if widget_asset_id.contains('#') {
            widget_asset_id.to_string()
        } else {
            format!("{widget_asset_id}#{widget_component_name}")
        };
        let _ = self
            .compiler_imports
            .widgets
            .insert(widget_reference, widget_document.clone());
        let selection = selection_for_node(&document, &node_id);
        let replay = tree_document_replay_bundle(&self.last_valid_document, &document);
        self.apply_command_with_effects(
            UiAssetEditorCommand::tree_edit_structured_with_selection(
                UiAssetEditorTreeEdit::PromoteToExternalWidget {
                    source_component_name,
                    asset_id: widget_asset_id.to_string(),
                    component_name: widget_component_name.to_string(),
                    document_id: widget_document_id.to_string(),
                },
                "Promote To External Widget",
                serialize_document(&document)?,
                selection,
            )
            .with_document_replay(replay),
            UiAssetEditorUndoExternalEffects {
                undo: restore_or_remove_external_asset_source(
                    widget_asset_id,
                    previous_widget_source,
                ),
                redo: vec![UiAssetEditorExternalEffect::UpsertAssetSource {
                    asset_id: widget_asset_id.to_string(),
                    source: widget_source,
                }],
            },
        )?;
        Ok(Some(widget_document))
    }

    pub fn promote_local_theme_to_external_style_asset(
        &mut self,
        style_asset_id: &str,
        style_document_id: &str,
        display_name: &str,
    ) -> Result<Option<UiAssetDocument>, UiAssetEditorSessionError> {
        self.ensure_editable_source()?;
        let mut document = self.last_valid_document.clone();
        let previous_style_source = self.existing_external_style_source(style_asset_id)?;
        let Some(style_document) = tree_promote_local_theme_to_external_style_asset(
            &mut document,
            style_asset_id,
            style_document_id,
            display_name,
        ) else {
            return Ok(None);
        };
        let style_source = toml::to_string_pretty(&style_document)
            .map_err(|error| UiAssetError::ParseToml(error.to_string()))?;
        let _ = self
            .compiler_imports
            .styles
            .insert(style_asset_id.to_string(), style_document.clone());
        let replay = theme_document_replay_bundle(&self.last_valid_document, &document);
        self.apply_command_with_effects(
            UiAssetEditorCommand::tree_edit(
                UiAssetEditorTreeEditKind::DocumentEdit,
                "Promote Local Theme",
                serialize_document(&document)?,
            )
            .with_document_replay(replay),
            UiAssetEditorUndoExternalEffects {
                undo: restore_or_remove_external_asset_source(
                    style_asset_id,
                    previous_style_source,
                ),
                redo: vec![UiAssetEditorExternalEffect::UpsertAssetSource {
                    asset_id: style_asset_id.to_string(),
                    source: style_source,
                }],
            },
        )?;
        Ok(Some(style_document))
    }

    pub(super) fn reconcile_promote_widget_draft(&mut self) {
        let selected_component =
            selected_local_component_name(&self.last_valid_document, &self.selection);
        if selected_component.is_none() {
            self.selected_promote_source_component_name = None;
            self.selected_promote_widget_asset_id = None;
            self.selected_promote_widget_component_name = None;
            self.selected_promote_widget_document_id = None;
            return;
        }
        let selected_component = selected_component.unwrap();
        if self.selected_promote_source_component_name.as_deref()
            != Some(selected_component.as_str())
        {
            self.selected_promote_source_component_name = Some(selected_component.clone());
            let draft = default_external_widget_draft(&self.last_valid_document, &self.selection);
            self.selected_promote_widget_asset_id =
                draft.as_ref().map(|draft| draft.asset_id.clone());
            self.selected_promote_widget_component_name =
                draft.as_ref().map(|draft| draft.component_name.clone());
            self.selected_promote_widget_document_id =
                draft.as_ref().map(|draft| draft.document_id.clone());
        } else {
            let draft = default_external_widget_draft(&self.last_valid_document, &self.selection);
            if self.selected_promote_widget_asset_id.is_none() {
                self.selected_promote_widget_asset_id =
                    draft.as_ref().map(|draft| draft.asset_id.clone());
            }
            if self.selected_promote_widget_component_name.is_none() {
                self.selected_promote_widget_component_name =
                    draft.as_ref().map(|draft| draft.component_name.clone());
            }
            if self.selected_promote_widget_document_id.is_none() {
                self.selected_promote_widget_document_id =
                    draft.as_ref().map(|draft| draft.document_id.clone());
            }
        }
    }

    pub(super) fn reconcile_promote_theme_draft(&mut self) {
        if !can_promote_local_theme_to_external_style_asset(&self.last_valid_document) {
            self.selected_promote_theme_asset_id = None;
            self.selected_promote_theme_document_id = None;
            self.selected_promote_theme_display_name = None;
            return;
        }

        let draft = default_external_style_draft(&self.route.asset_id, self.asset_display_name());
        if self.selected_promote_theme_asset_id.is_none() {
            self.selected_promote_theme_asset_id = Some(draft.asset_id.clone());
        }
        if self.selected_promote_theme_document_id.is_none() {
            self.selected_promote_theme_document_id = Some(draft.document_id.clone());
        }
        if self.selected_promote_theme_display_name.is_none() {
            self.selected_promote_theme_display_name = Some(draft.display_name);
        }
    }
}

pub(super) fn normalized_promote_asset_id(asset_id: &str) -> Option<String> {
    let trimmed = asset_id.trim();
    if trimmed.is_empty() {
        return None;
    }

    let normalized = trimmed.replace('\\', "/");
    let normalized = normalized
        .strip_prefix("asset://")
        .map(|relative| format!("res://{}", relative.trim_start_matches('/')))
        .unwrap_or(normalized);
    let normalized = normalized
        .split_once('#')
        .map(|(path, _)| path)
        .unwrap_or(normalized.as_str())
        .trim_end_matches('/');
    if normalized.is_empty() {
        return None;
    }

    let normalized = if normalized.starts_with("res://") {
        normalized.to_string()
    } else {
        format!("res://{}", normalized.trim_start_matches('/'))
    };
    if normalized.ends_with(".ui.toml") {
        Some(normalized)
    } else if let Some(base) = normalized.strip_suffix(".toml") {
        Some(format!("{base}.ui.toml"))
    } else {
        Some(format!("{normalized}.ui.toml"))
    }
}

pub(super) fn normalized_promote_component_name(component_name: &str) -> Option<String> {
    let trimmed = component_name.trim();
    if trimmed.is_empty() {
        return None;
    }

    let mut normalized = String::new();
    let mut capitalize_next = true;
    for ch in trimmed.chars() {
        if ch.is_ascii_alphanumeric() {
            if normalized.is_empty() && ch.is_ascii_digit() {
                normalized.push('W');
            }
            if capitalize_next && ch.is_ascii_alphabetic() {
                normalized.push(ch.to_ascii_uppercase());
            } else {
                normalized.push(ch);
            }
            capitalize_next = false;
        } else {
            capitalize_next = true;
        }
    }

    (!normalized.is_empty()).then_some(normalized)
}

pub(super) fn normalized_promote_document_id(document_id: &str) -> Option<String> {
    let trimmed = document_id.trim();
    if trimmed.is_empty() {
        return None;
    }

    let mut normalized = String::new();
    let mut previous_was_separator = true;
    for ch in trimmed.chars() {
        if ch.is_ascii_lowercase() || ch.is_ascii_digit() || ch == '_' {
            normalized.push(ch);
            previous_was_separator = false;
        } else if ch.is_ascii_uppercase() {
            normalized.push(ch.to_ascii_lowercase());
            previous_was_separator = false;
        } else if ch == '.' {
            if !previous_was_separator && !normalized.is_empty() {
                normalized.push('.');
                previous_was_separator = true;
            }
        } else if !previous_was_separator && !normalized.is_empty() {
            normalized.push('.');
            previous_was_separator = true;
        }
    }

    let normalized = normalized.trim_matches('.').to_string();
    (!normalized.is_empty()).then_some(normalized)
}

pub(super) fn normalized_promote_display_name(display_name: &str) -> Option<String> {
    let trimmed = display_name.trim();
    (!trimmed.is_empty()).then(|| trimmed.to_string())
}

pub(super) fn reference_asset_id(reference: &str) -> &str {
    reference
        .split_once('#')
        .map(|(asset_id, _)| asset_id)
        .unwrap_or(reference)
}

pub(super) fn restore_or_remove_external_asset_source(
    asset_id: &str,
    previous_source: Option<String>,
) -> Vec<UiAssetEditorExternalEffect> {
    match previous_source {
        Some(source) => vec![UiAssetEditorExternalEffect::RestoreAssetSource {
            asset_id: asset_id.to_string(),
            source,
        }],
        None => vec![UiAssetEditorExternalEffect::RemoveAssetSource {
            asset_id: asset_id.to_string(),
        }],
    }
}
