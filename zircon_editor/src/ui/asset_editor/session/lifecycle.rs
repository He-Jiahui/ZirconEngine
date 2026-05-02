use std::collections::BTreeMap;

use crate::ui::asset_editor::{UiAssetEditorMode, UiAssetEditorRoute, UiAssetPreviewPreset};
use zircon_runtime::ui::template::{
    collect_asset_binding_report, component_contract_diagnostic, UiAssetLoader, UiDocumentCompiler,
};
use zircon_runtime_interface::ui::{
    layout::UiSize,
    template::{UiAssetDocument, UiAssetError, UiAssetKind},
};

use super::{
    binding_inspector::{reconcile_selected_binding_index, reconcile_selected_binding_payload_key},
    command::{
        UiAssetEditorDocumentReplayCommand, UiAssetEditorInverseTreeEdit, UiAssetEditorTreeEdit,
        UiAssetEditorTreeEditKind,
    },
    diagnostics::{map_binding_diagnostic, map_component_contract_diagnostic},
    inspector_semantics::{
        build_layout_semantic_group, build_slot_semantic_group, reconcile_selected_semantic_path,
    },
    preview_compile::{compile_preview, current_preview_size, preview_size_for_preset},
    preview_host::UiAssetPreviewHost,
    preview_mock::{apply_preview_mock_overrides, reconcile_preview_mock_state},
    promotion_state::reference_asset_id,
    session_state::{
        default_selection, ensure_asset_kind, reconcile_selection, UiAssetCompilerImports,
    },
    source_buffer::UiAssetSourceBuffer,
    source_sync::{source_byte_offset_for_line, source_outline_entry_for_node},
    style_inspection::{
        build_style_inspector, matched_style_rule_entries_for_selection,
        reconcile_selected_matched_style_rule_index,
        reconcile_selected_style_rule_declaration_path, reconcile_selected_style_rule_selection,
        reconcile_selected_style_token_name,
    },
    theme_summary::reconcile_selected_theme_source_key,
    ui_asset_editor_session::{
        serialize_document, UiAssetEditorSession, UiAssetEditorSessionError,
        UiAssetSourceCursorAnchor,
    },
    undo_stack::UiAssetEditorExternalEffect,
};
use crate::ui::asset_editor::palette::build_palette_entries;

impl UiAssetEditorSession {
    pub fn from_source(
        route: UiAssetEditorRoute,
        source: impl Into<String>,
        preview_size: UiSize,
    ) -> Result<Self, UiAssetEditorSessionError> {
        let source = source.into();
        let document = parse_ui_asset_source(&source)?;
        ensure_asset_kind(route.asset_kind, document.asset.kind)?;
        let selection = default_selection(&document);
        let compiler_imports = UiAssetCompilerImports::default();
        let style_inspector =
            build_style_inspector(&document, &selection, &compiler_imports, &Vec::new());
        let selected_binding_index = reconcile_selected_binding_index(&document, &selection, None);
        let selected_binding_payload_key = reconcile_selected_binding_payload_key(
            &document,
            &selection,
            selected_binding_index,
            None,
        );
        let selected_theme_source_key =
            reconcile_selected_theme_source_key(&document, &compiler_imports.styles, None);
        let source_cursor_anchor =
            selection
                .primary_node_id
                .as_ref()
                .map(|node_id| UiAssetSourceCursorAnchor {
                    node_id: node_id.clone(),
                    line_offset: 0,
                });
        let source_cursor_byte_offset = source_cursor_anchor
            .as_ref()
            .and_then(|anchor| {
                source_outline_entry_for_node(&source, &anchor.node_id)
                    .map(|entry| source_byte_offset_for_line(&source, entry.line as usize))
            })
            .unwrap_or(0);
        let promote_draft =
            super::promote_widget::default_external_widget_draft(&document, &selection);
        let theme_draft =
            super::theme_authoring::can_promote_local_theme_to_external_style_asset(&document)
                .then(|| {
                    super::theme_authoring::default_external_style_draft(
                        &route.asset_id,
                        &document.asset.display_name,
                    )
                });
        let (last_valid_compiled, preview_host, diagnostics, structured_diagnostics) =
            match compile_preview(&document, preview_size, &compiler_imports) {
                Ok((compiled, preview_host)) => (compiled, preview_host, Vec::new(), Vec::new()),
                Err(error) => {
                    let structured_diagnostics =
                        structured_compile_diagnostics(&document, &compiler_imports);
                    (None, None, vec![error.to_string()], structured_diagnostics)
                }
            };
        Ok(Self {
            route,
            source_buffer: UiAssetSourceBuffer::new(source.clone()),
            last_valid_source_text: source,
            last_valid_document: document,
            last_valid_compiled,
            preview_host,
            undo_stack: super::undo_stack::UiAssetEditorUndoStack::default(),
            diagnostics,
            structured_diagnostics,
            source_cursor_byte_offset,
            source_cursor_anchor,
            selection,
            style_inspector,
            selected_style_rule_index: None,
            selected_style_rule_id: None,
            selected_matched_style_rule_index: None,
            selected_style_rule_declaration_path: None,
            selected_style_token_name: None,
            selected_theme_source_key,
            selected_binding_index,
            selected_binding_payload_key,
            selected_slot_semantic_path: None,
            selected_layout_semantic_path: None,
            selected_palette_index: None,
            palette_target_chooser: None,
            selected_promote_source_component_name: promote_draft
                .as_ref()
                .map(|draft| draft.component_name.clone()),
            selected_promote_widget_asset_id: promote_draft
                .as_ref()
                .map(|draft| draft.asset_id.clone()),
            selected_promote_widget_component_name: promote_draft
                .as_ref()
                .map(|draft| draft.component_name.clone()),
            selected_promote_widget_document_id: promote_draft
                .as_ref()
                .map(|draft| draft.document_id.clone()),
            selected_promote_theme_asset_id: theme_draft
                .as_ref()
                .map(|draft| draft.asset_id.clone()),
            selected_promote_theme_document_id: theme_draft
                .as_ref()
                .map(|draft| draft.document_id.clone()),
            selected_promote_theme_display_name: theme_draft
                .as_ref()
                .map(|draft| draft.display_name.clone()),
            preview_mock_state: super::preview_mock::UiAssetPreviewMockState::default(),
            compiler_imports,
        })
    }

    pub fn route(&self) -> &UiAssetEditorRoute {
        &self.route
    }

    pub fn asset_display_name(&self) -> &str {
        &self.last_valid_document.asset.display_name
    }

    pub fn source_buffer(&self) -> &UiAssetSourceBuffer {
        &self.source_buffer
    }

    pub fn preview_host(&self) -> &UiAssetPreviewHost {
        self.preview_host
            .as_ref()
            .expect("preview host is only available for layout/widget assets")
    }

    pub fn preview_host_opt(&self) -> Option<&UiAssetPreviewHost> {
        self.preview_host.as_ref()
    }

    pub fn diagnostics(&self) -> &[String] {
        &self.diagnostics
    }

    pub fn structured_diagnostics(&self) -> &[crate::ui::asset_editor::UiAssetEditorDiagnostic] {
        &self.structured_diagnostics
    }

    pub fn can_undo(&self) -> bool {
        self.undo_stack.can_undo()
    }

    pub fn can_redo(&self) -> bool {
        self.undo_stack.can_redo()
    }

    pub fn next_undo_label(&self) -> Option<String> {
        self.undo_stack.next_undo_label()
    }

    pub fn next_redo_label(&self) -> Option<String> {
        self.undo_stack.next_redo_label()
    }

    pub fn next_undo_tree_edit_kind(&self) -> Option<UiAssetEditorTreeEditKind> {
        self.undo_stack.next_undo_tree_edit_kind()
    }

    pub fn next_redo_tree_edit_kind(&self) -> Option<UiAssetEditorTreeEditKind> {
        self.undo_stack.next_redo_tree_edit_kind()
    }

    pub fn next_undo_tree_edit(&self) -> Option<UiAssetEditorTreeEdit> {
        self.undo_stack.next_undo_tree_edit()
    }

    pub fn next_redo_tree_edit(&self) -> Option<UiAssetEditorTreeEdit> {
        self.undo_stack.next_redo_tree_edit()
    }

    pub fn next_undo_inverse_tree_edit(&self) -> Option<UiAssetEditorInverseTreeEdit> {
        self.undo_stack.next_undo_inverse_tree_edit()
    }

    pub fn next_redo_inverse_tree_edit(&self) -> Option<UiAssetEditorInverseTreeEdit> {
        self.undo_stack.next_redo_inverse_tree_edit()
    }

    pub fn next_undo_external_effect(&self) -> Option<UiAssetEditorExternalEffect> {
        self.undo_stack.next_undo_external_effect()
    }

    pub fn next_redo_external_effect(&self) -> Option<UiAssetEditorExternalEffect> {
        self.undo_stack.next_redo_external_effect()
    }

    pub fn next_undo_external_effects(&self) -> Vec<UiAssetEditorExternalEffect> {
        self.undo_stack.next_undo_external_effects()
    }

    pub fn next_redo_external_effects(&self) -> Vec<UiAssetEditorExternalEffect> {
        self.undo_stack.next_redo_external_effects()
    }

    pub fn next_undo_document_replay_commands(&self) -> Vec<UiAssetEditorDocumentReplayCommand> {
        self.undo_stack.next_undo_document_replay_commands()
    }

    pub fn next_redo_document_replay_commands(&self) -> Vec<UiAssetEditorDocumentReplayCommand> {
        self.undo_stack.next_redo_document_replay_commands()
    }

    pub fn set_mode(&mut self, mode: UiAssetEditorMode) -> Result<(), UiAssetEditorSessionError> {
        self.route.mode = mode;
        self.revalidate()
    }

    pub fn set_preview_preset(
        &mut self,
        preview_preset: UiAssetPreviewPreset,
    ) -> Result<bool, UiAssetEditorSessionError> {
        if self.route.preview_preset == preview_preset {
            return Ok(false);
        }

        self.route.preview_preset = preview_preset;
        self.refresh_preview_for_current_preset()?;
        Ok(true)
    }

    pub(super) fn rebuild_preview_snapshot(&mut self) -> Result<(), UiAssetEditorSessionError> {
        let preview_document = super::preview_mock::apply_preview_mock_overrides(
            &self.last_valid_document,
            &self.preview_mock_state,
        );
        let (compiled, preview_host) = compile_preview(
            &preview_document,
            current_preview_size(&self.preview_host, self.route.preview_preset),
            &self.compiler_imports,
        )?;
        self.last_valid_compiled = compiled;
        self.preview_host = preview_host;
        Ok(())
    }

    pub(super) fn refresh_preview_for_current_preset(
        &mut self,
    ) -> Result<(), UiAssetEditorSessionError> {
        let preview_size = preview_size_for_preset(self.route.preview_preset);
        if let (Some(compiled), Some(preview_host)) = (
            self.last_valid_compiled.as_ref(),
            self.preview_host.as_mut(),
        ) {
            preview_host.rebuild_with_size(
                preview_size,
                &self.last_valid_document.asset.id,
                compiled,
            )?;
            return Ok(());
        }

        if self.preview_host.is_none() {
            if let Some(compiled) = self.last_valid_compiled.as_ref() {
                self.preview_host = Some(UiAssetPreviewHost::new(
                    preview_size,
                    &self.last_valid_document.asset.id,
                    compiled,
                )?);
            }
        }

        Ok(())
    }

    pub(super) fn existing_external_widget_source(
        &self,
        asset_id: &str,
    ) -> Result<Option<String>, UiAssetEditorSessionError> {
        self.existing_external_asset_source(
            asset_id,
            self.compiler_imports
                .widgets
                .iter()
                .find_map(|(reference, document)| {
                    (reference_asset_id(reference) == asset_id).then_some(document)
                }),
        )
    }

    pub(super) fn existing_external_style_source(
        &self,
        asset_id: &str,
    ) -> Result<Option<String>, UiAssetEditorSessionError> {
        self.existing_external_asset_source(asset_id, self.compiler_imports.styles.get(asset_id))
    }

    pub(super) fn existing_external_asset_source(
        &self,
        asset_id: &str,
        imported_document: Option<&UiAssetDocument>,
    ) -> Result<Option<String>, UiAssetEditorSessionError> {
        if self.route.asset_id == asset_id {
            return Ok(Some(self.last_valid_source_text.clone()));
        }

        imported_document.map(serialize_document).transpose()
    }

    pub fn register_widget_import(
        &mut self,
        reference: impl Into<String>,
        document: UiAssetDocument,
    ) -> Result<(), UiAssetEditorSessionError> {
        ensure_asset_kind(UiAssetKind::Widget, document.asset.kind)?;
        let _ = self
            .compiler_imports
            .widgets
            .insert(reference.into(), document);
        self.revalidate()?;
        Ok(())
    }

    pub fn register_style_import(
        &mut self,
        reference: impl Into<String>,
        document: UiAssetDocument,
    ) -> Result<(), UiAssetEditorSessionError> {
        ensure_asset_kind(UiAssetKind::Style, document.asset.kind)?;
        let _ = self
            .compiler_imports
            .styles
            .insert(reference.into(), document);
        self.revalidate()?;
        Ok(())
    }

    pub fn replace_imports(
        &mut self,
        widgets: BTreeMap<String, UiAssetDocument>,
        styles: BTreeMap<String, UiAssetDocument>,
    ) -> Result<(), UiAssetEditorSessionError> {
        self.compiler_imports.widgets = widgets;
        self.compiler_imports.styles = styles;
        self.revalidate()
    }

    pub(super) fn revalidate(&mut self) -> Result<(), UiAssetEditorSessionError> {
        match parse_ui_asset_source(self.source_buffer.text()) {
            Ok(document) => self.apply_valid_document(document),
            Err(error) => {
                self.diagnostics = vec![error.to_string()];
                self.structured_diagnostics.clear();
                Ok(())
            }
        }
    }

    pub(super) fn apply_valid_document(
        &mut self,
        document: UiAssetDocument,
    ) -> Result<(), UiAssetEditorSessionError> {
        ensure_asset_kind(self.route.asset_kind, document.asset.kind)?;
        self.last_valid_document = document;
        self.last_valid_source_text = self.source_buffer.text().to_string();
        self.selection = reconcile_selection(&self.last_valid_document, &self.selection);
        self.reconcile_promote_widget_draft();
        self.reconcile_promote_theme_draft();
        let (selected_style_rule_index, selected_style_rule_id) =
            reconcile_selected_style_rule_selection(
                &self.last_valid_document,
                self.selected_style_rule_index,
                self.selected_style_rule_id.as_deref(),
            );
        self.selected_style_rule_index = selected_style_rule_index;
        self.selected_style_rule_id = selected_style_rule_id;
        self.selected_style_rule_declaration_path = reconcile_selected_style_rule_declaration_path(
            &self.last_valid_document,
            self.selected_style_rule_index,
            self.selected_style_rule_declaration_path.as_deref(),
        );
        self.selected_style_token_name = reconcile_selected_style_token_name(
            &self.last_valid_document,
            self.selected_style_token_name.as_deref(),
        );
        self.selected_theme_source_key = reconcile_selected_theme_source_key(
            &self.last_valid_document,
            &self.compiler_imports.styles,
            self.selected_theme_source_key.as_deref(),
        );
        self.selected_binding_index = reconcile_selected_binding_index(
            &self.last_valid_document,
            &self.selection,
            self.selected_binding_index,
        );
        self.selected_binding_payload_key = reconcile_selected_binding_payload_key(
            &self.last_valid_document,
            &self.selection,
            self.selected_binding_index,
            self.selected_binding_payload_key.as_deref(),
        );
        self.selected_slot_semantic_path = reconcile_selected_semantic_path(
            &build_slot_semantic_group(&self.last_valid_document, &self.selection).entries,
            self.selected_slot_semantic_path.as_deref(),
        );
        self.selected_layout_semantic_path = reconcile_selected_semantic_path(
            &build_layout_semantic_group(&self.last_valid_document, &self.selection).entries,
            self.selected_layout_semantic_path.as_deref(),
        );
        self.selected_palette_index = reconcile_selected_palette_index(
            &build_palette_entries(&self.last_valid_document, &self.compiler_imports.widgets),
            self.selected_palette_index,
        );
        self.clear_palette_drag_state();
        self.reconcile_source_cursor_state();
        reconcile_preview_mock_state(
            &self.last_valid_document,
            &self.selection,
            &mut self.preview_mock_state,
        );
        let active_states = self.style_inspector.active_pseudo_states.clone();
        self.style_inspector = build_style_inspector(
            &self.last_valid_document,
            &self.selection,
            &self.compiler_imports,
            &active_states,
        );
        let matched_entries = matched_style_rule_entries_for_selection(
            &self.last_valid_document,
            &self.selection,
            &self.compiler_imports,
            &active_states,
        );
        self.selected_matched_style_rule_index = reconcile_selected_matched_style_rule_index(
            &matched_entries,
            self.selected_matched_style_rule_index,
        );
        let preview_document =
            apply_preview_mock_overrides(&self.last_valid_document, &self.preview_mock_state);
        match compile_preview(
            &preview_document,
            current_preview_size(&self.preview_host, self.route.preview_preset),
            &self.compiler_imports,
        ) {
            Ok((compiled, preview_host)) => {
                self.last_valid_compiled = compiled;
                self.preview_host = preview_host;
                self.diagnostics.clear();
                self.structured_diagnostics.clear();
            }
            Err(error) => {
                self.diagnostics = vec![error.to_string()];
                self.structured_diagnostics = structured_compile_diagnostics(
                    &self.last_valid_document,
                    &self.compiler_imports,
                );
            }
        }
        Ok(())
    }

    pub fn import_references(&self) -> (Vec<String>, Vec<String>) {
        (
            self.last_valid_document.imports.widgets.clone(),
            self.last_valid_document.imports.styles.clone(),
        )
    }

    pub fn canonical_source(&self) -> Result<String, UiAssetEditorSessionError> {
        if !self.diagnostics.is_empty() {
            return Err(UiAssetEditorSessionError::InvalidSourceBuffer);
        }
        Ok(
            toml::to_string_pretty(&self.last_valid_document).map_err(|error| {
                super::ui_asset_editor_session::UiAssetEditorSessionError::Asset(
                    UiAssetError::ParseToml(error.to_string()),
                )
            })?,
        )
    }

    pub fn save_to_canonical_source(&mut self) -> Result<String, UiAssetEditorSessionError> {
        let canonical = self.canonical_source()?;
        self.source_buffer.replace(canonical.clone());
        self.source_buffer.mark_saved();
        Ok(canonical)
    }
}

pub(super) fn parse_ui_asset_source(source: &str) -> Result<UiAssetDocument, UiAssetError> {
    UiAssetLoader::load_toml_str(source).or_else(|error| {
        #[cfg(test)]
        {
            crate::tests::support::load_test_ui_asset(source).or(Err(error))
        }
        #[cfg(not(test))]
        {
            Err(error)
        }
    })
}

fn reconcile_selected_palette_index<T>(items: &[T], current: Option<usize>) -> Option<usize> {
    match (current, items.len()) {
        (_, 0) => None,
        (Some(index), count) => Some(index.min(count - 1)),
        (None, _) => None,
    }
}

fn structured_compile_diagnostics(
    document: &UiAssetDocument,
    imports: &UiAssetCompilerImports,
) -> Vec<crate::ui::asset_editor::UiAssetEditorDiagnostic> {
    let mut diagnostics = structured_contract_diagnostics(document, imports);
    diagnostics.extend(
        collect_asset_binding_report(document, UiDocumentCompiler::default().component_registry())
            .diagnostics
            .into_iter()
            .map(map_binding_diagnostic),
    );
    diagnostics
}

fn structured_contract_diagnostics(
    document: &UiAssetDocument,
    imports: &UiAssetCompilerImports,
) -> Vec<crate::ui::asset_editor::UiAssetEditorDiagnostic> {
    match component_contract_diagnostic(document, &imports.widgets, &imports.styles) {
        Ok(diagnostic) => diagnostic
            .into_iter()
            .map(map_component_contract_diagnostic)
            .collect(),
        Err(_) => Vec::new(),
    }
}
