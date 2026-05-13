use std::collections::{BTreeMap, BTreeSet};

use crate::ui::asset_editor::{UiAssetEditorMode, UiAssetEditorRoute, UiAssetPreviewPreset};
use crate::ui::template::EditorTemplateRuntimeService;
use zircon_runtime::ui::template::component_contract_diagnostic;
use zircon_runtime::ui::v2::UiV2AssetLoader;
use zircon_runtime_interface::ui::{
    layout::UiSize,
    template::{
        UiAssetDocument, UiAssetError, UiAssetHeader, UiAssetImports, UiAssetKind, UiChildMount,
        UiComponentDefinition, UiNodeDefinition, UiNodeDefinitionKind, UiResourceDependency,
        UiResourceDiagnostic, UiStyleDeclarationBlock, UiStyleRule, UiStyleSheet,
    },
    v2::{
        UiV2AssetDocument, UiV2AssetError, UiV2AssetHeader, UiV2AssetKind, UiV2ChildMount,
        UiV2NodeDefinition, UiV2Root, UiV2StyleDeclarationBlock, UiV2StyleRule, UiV2StyleSheet,
        UI_V2_ASSET_SCHEMA_VERSION,
    },
};

use super::{
    binding_inspector::{reconcile_selected_binding_index, reconcile_selected_binding_payload_key},
    command::{
        UiAssetEditorDocumentReplayCommand, UiAssetEditorInverseTreeEdit, UiAssetEditorTreeEdit,
        UiAssetEditorTreeEditKind,
    },
    diagnostics::{
        map_binding_diagnostic, map_component_contract_diagnostic, map_localization_diagnostic,
    },
    inspector_semantics::{
        build_layout_semantic_group, build_slot_semantic_group, reconcile_selected_semantic_path,
    },
    preview_compile::{compile_preview, current_preview_size, preview_size_for_preset},
    preview_host::UiAssetPreviewHost,
    preview_mock::{apply_preview_mock_overrides, reconcile_preview_mock_state},
    promotion_state::reference_asset_id,
    runtime_report_state::DEFAULT_LOCALE_PREVIEW,
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
        UiAssetSourceCursorAnchor, UiAssetSourceSchema, UiAssetV2CompilerImports,
    },
    undo_stack::UiAssetEditorExternalEffect,
    v2_authoring::{build_v2_preview_host, ensure_v2_asset_kind},
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
        let (
            last_valid_compiled,
            preview_host,
            resource_dependencies,
            resource_diagnostics,
            diagnostics,
            structured_diagnostics,
        ) = match compile_preview(&document, preview_size, &compiler_imports) {
            Ok((compiled, preview_host)) => {
                let (resource_dependencies, resource_diagnostics) =
                    compiled_resource_report(compiled.as_ref());
                (
                    compiled,
                    preview_host,
                    resource_dependencies,
                    resource_diagnostics,
                    Vec::new(),
                    Vec::new(),
                )
            }
            Err(error) => {
                let structured_diagnostics =
                    structured_compile_diagnostics(&document, &compiler_imports);
                (
                    None,
                    None,
                    Vec::new(),
                    Vec::new(),
                    vec![error.to_string()],
                    structured_diagnostics,
                )
            }
        };
        Ok(Self {
            route,
            source_schema: UiAssetSourceSchema::Legacy,
            source_buffer: UiAssetSourceBuffer::new(source.clone()),
            last_valid_source_text: source,
            last_valid_document: document,
            last_valid_v2_document: None,
            v2_compiler_imports: UiAssetV2CompilerImports::default(),
            last_valid_compiled,
            resource_dependencies,
            resource_diagnostics,
            resource_resolver: Default::default(),
            localization_catalog: Default::default(),
            preview_host,
            undo_stack: super::undo_stack::UiAssetEditorUndoStack::default(),
            diagnostics,
            structured_diagnostics,
            source_cursor_byte_offset,
            source_cursor_anchor,
            selection,
            designer_tool_mode: Default::default(),
            last_preview_interact_dispatch: None,
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
            selected_locale_preview: DEFAULT_LOCALE_PREVIEW.to_string(),
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

    pub fn from_v2_source(
        route: UiAssetEditorRoute,
        source: impl Into<String>,
        preview_size: UiSize,
    ) -> Result<Self, UiAssetEditorSessionError> {
        let source = source.into();
        let v2_document = UiV2AssetLoader::load_toml_str(&source)?;
        let document = v2_document_to_legacy_projection_document(&v2_document)?;
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
        let v2_compiler_imports = UiAssetV2CompilerImports::default();
        let (preview_host, diagnostics) =
            match build_v2_preview_host(&v2_document, preview_size, &v2_compiler_imports) {
                Ok(preview_host) => (preview_host, Vec::new()),
                Err(error) => (None, vec![error.to_string()]),
            };

        Ok(Self {
            route,
            source_schema: UiAssetSourceSchema::V2,
            source_buffer: UiAssetSourceBuffer::new(source.clone()),
            last_valid_source_text: source,
            last_valid_document: document,
            last_valid_v2_document: Some(v2_document),
            v2_compiler_imports,
            last_valid_compiled: None,
            resource_dependencies: Vec::new(),
            resource_diagnostics: Vec::new(),
            resource_resolver: Default::default(),
            localization_catalog: Default::default(),
            preview_host,
            undo_stack: super::undo_stack::UiAssetEditorUndoStack::default(),
            diagnostics,
            structured_diagnostics: Vec::new(),
            source_cursor_byte_offset,
            source_cursor_anchor,
            selection,
            designer_tool_mode: Default::default(),
            last_preview_interact_dispatch: None,
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
            selected_locale_preview: DEFAULT_LOCALE_PREVIEW.to_string(),
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

    pub fn resource_dependencies(&self) -> &[UiResourceDependency] {
        &self.resource_dependencies
    }

    pub fn resource_diagnostics(&self) -> &[UiResourceDiagnostic] {
        &self.resource_diagnostics
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
        if self.source_schema == UiAssetSourceSchema::V2 {
            if let Some(document) = self.last_valid_v2_document.as_ref() {
                self.preview_host = build_v2_preview_host(
                    document,
                    current_preview_size(&self.preview_host, self.route.preview_preset),
                    &self.v2_compiler_imports,
                )?;
            }
            return Ok(());
        }

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
        self.refresh_resource_path_resolver();
        Ok(())
    }

    pub(super) fn refresh_preview_for_current_preset(
        &mut self,
    ) -> Result<(), UiAssetEditorSessionError> {
        let preview_size = preview_size_for_preset(self.route.preview_preset);
        if self.source_schema == UiAssetSourceSchema::V2 {
            if let Some(document) = self.last_valid_v2_document.as_ref() {
                self.preview_host =
                    build_v2_preview_host(document, preview_size, &self.v2_compiler_imports)?;
            }
            return Ok(());
        }

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

    pub fn register_v2_widget_import(
        &mut self,
        reference: impl Into<String>,
        document: UiV2AssetDocument,
    ) -> Result<(), UiAssetEditorSessionError> {
        ensure_v2_asset_kind(UiV2AssetKind::Component, document.asset.kind)?;
        let _ = self
            .v2_compiler_imports
            .widgets
            .insert(reference.into(), document);
        if self.source_schema == UiAssetSourceSchema::V2 {
            self.revalidate()?;
        }
        Ok(())
    }

    pub fn register_v2_style_import(
        &mut self,
        reference: impl Into<String>,
        document: UiV2AssetDocument,
    ) -> Result<(), UiAssetEditorSessionError> {
        ensure_v2_asset_kind(UiV2AssetKind::Style, document.asset.kind)?;
        let _ = self
            .v2_compiler_imports
            .styles
            .insert(reference.into(), document);
        if self.source_schema == UiAssetSourceSchema::V2 {
            self.revalidate()?;
        }
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
        match self.source_schema {
            UiAssetSourceSchema::Legacy => match parse_ui_asset_source(self.source_buffer.text()) {
                Ok(document) => self.apply_valid_document(document),
                Err(error) => {
                    self.mark_current_source_invalid(error.to_string());
                    Ok(())
                }
            },
            UiAssetSourceSchema::V2 => {
                match UiV2AssetLoader::load_toml_str(self.source_buffer.text()) {
                    Ok(document) => self.apply_valid_v2_document(document),
                    Err(error) => {
                        self.mark_current_source_invalid(error.to_string());
                        Ok(())
                    }
                }
            }
        }
    }

    pub(super) fn apply_valid_document(
        &mut self,
        document: UiAssetDocument,
    ) -> Result<(), UiAssetEditorSessionError> {
        self.last_valid_v2_document = None;
        self.apply_valid_projection_document(document)?;
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
                self.refresh_resource_path_resolver();
                self.refresh_structured_diagnostics_for_current_document();
            }
            Err(error) => {
                self.diagnostics = vec![error.to_string()];
                self.resource_dependencies.clear();
                self.resource_diagnostics.clear();
                self.refresh_structured_diagnostics_for_current_document();
            }
        }
        Ok(())
    }

    pub(super) fn apply_valid_v2_document(
        &mut self,
        document: UiV2AssetDocument,
    ) -> Result<(), UiAssetEditorSessionError> {
        let projection_document = v2_document_to_legacy_projection_document(&document)?;
        self.last_valid_v2_document = Some(document.clone());
        self.apply_valid_projection_document(projection_document)?;
        match build_v2_preview_host(
            &document,
            current_preview_size(&self.preview_host, self.route.preview_preset),
            &self.v2_compiler_imports,
        ) {
            Ok(preview_host) => {
                self.last_valid_compiled = None;
                self.preview_host = preview_host;
                self.diagnostics.clear();
                self.resource_dependencies.clear();
                self.resource_diagnostics.clear();
                self.refresh_resource_path_resolver();
                self.refresh_structured_diagnostics_for_current_document();
            }
            Err(error) => {
                self.diagnostics = vec![error.to_string()];
                self.resource_dependencies.clear();
                self.resource_diagnostics.clear();
                self.refresh_structured_diagnostics_for_current_document();
            }
        }
        Ok(())
    }

    fn apply_valid_projection_document(
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
        Ok(())
    }

    fn mark_current_source_invalid(&mut self, error: String) {
        self.diagnostics = vec![error];
        self.structured_diagnostics.clear();
        self.resource_dependencies.clear();
        self.resource_diagnostics.clear();
    }

    pub fn import_references(&self) -> (Vec<String>, Vec<String>) {
        (
            self.last_valid_document.imports.widgets.clone(),
            self.last_valid_document.imports.styles.clone(),
        )
    }

    pub(super) fn serialize_document_for_current_schema(
        &self,
        document: &UiAssetDocument,
    ) -> Result<String, UiAssetEditorSessionError> {
        match self.source_schema {
            UiAssetSourceSchema::Legacy => serialize_document(document),
            UiAssetSourceSchema::V2 => {
                let document = legacy_projection_document_to_v2_document(
                    document,
                    self.last_valid_v2_document.as_ref(),
                )?;
                toml::to_string_pretty(&document)
                    .map_err(|error| UiAssetError::ParseToml(error.to_string()).into())
            }
        }
    }

    pub fn canonical_source(&self) -> Result<String, UiAssetEditorSessionError> {
        if !self.diagnostics.is_empty() {
            return Err(UiAssetEditorSessionError::InvalidSourceBuffer);
        }
        self.serialize_document_for_current_schema(&self.last_valid_document)
    }

    pub fn save_to_canonical_source(&mut self) -> Result<String, UiAssetEditorSessionError> {
        let canonical = self.canonical_source()?;
        self.source_buffer.replace(canonical.clone());
        self.source_buffer.mark_saved();
        Ok(canonical)
    }
}

pub(super) fn parse_ui_asset_source(source: &str) -> Result<UiAssetDocument, UiAssetError> {
    EditorTemplateRuntimeService.parse_document_source(source)
}

fn v2_document_to_legacy_projection_document(
    document: &UiV2AssetDocument,
) -> Result<UiAssetDocument, UiAssetEditorSessionError> {
    let root = document
        .root_node_id()
        .map(|root| v2_node_tree_to_legacy_projection_node(document, root))
        .transpose()?;
    let components = document
        .components
        .iter()
        .map(|(component_name, component)| {
            let root = v2_node_tree_to_legacy_projection_node(document, &component.root)?;
            Ok((
                component_name.clone(),
                UiComponentDefinition {
                    root,
                    style_scope: component.style_scope,
                    contract: Default::default(),
                    params: component.params.clone(),
                    slots: component.slots.clone(),
                },
            ))
        })
        .collect::<Result<_, UiAssetEditorSessionError>>()?;

    Ok(UiAssetDocument {
        asset: UiAssetHeader {
            kind: legacy_asset_kind(document.asset.kind),
            id: document.asset.id.clone(),
            version: document.asset.version,
            display_name: document.asset.display_name.clone(),
        },
        imports: UiAssetImports {
            widgets: document.imports.widgets.clone(),
            styles: document.imports.styles.clone(),
            resources: document.imports.resources.clone(),
        },
        tokens: document.tokens.clone(),
        root,
        components,
        stylesheets: document
            .stylesheets
            .iter()
            .map(|stylesheet| UiStyleSheet {
                id: stylesheet.id.clone(),
                rules: stylesheet
                    .rules
                    .iter()
                    .map(|rule| UiStyleRule {
                        id: rule.id.clone(),
                        selector: rule.selector.clone(),
                        set: legacy_style_block(&rule.set),
                    })
                    .collect(),
            })
            .collect(),
    })
}

fn legacy_projection_document_to_v2_document(
    document: &UiAssetDocument,
    previous: Option<&UiV2AssetDocument>,
) -> Result<UiV2AssetDocument, UiAssetEditorSessionError> {
    let mut nodes = BTreeMap::new();
    if let Some(root) = &document.root {
        flatten_legacy_projection_nodes_into(&mut nodes, root)?;
    }
    let components = document
        .components
        .iter()
        .map(|(component_name, component)| {
            flatten_legacy_projection_nodes_into(&mut nodes, &component.root)?;
            let previous_component =
                previous.and_then(|document| document.components.get(component_name));
            Ok((
                component_name.clone(),
                zircon_runtime_interface::ui::v2::UiV2ComponentDefinition {
                    root: component.root.node_id.clone(),
                    style_scope: component.style_scope,
                    params: component.params.clone(),
                    slots: component.slots.clone(),
                    default_classes: previous_component
                        .map(|component| component.default_classes.clone())
                        .unwrap_or_default(),
                },
            ))
        })
        .collect::<Result<_, UiAssetEditorSessionError>>()?;

    Ok(UiV2AssetDocument {
        asset: UiV2AssetHeader {
            kind: v2_asset_kind(document.asset.kind),
            id: document.asset.id.clone(),
            version: UI_V2_ASSET_SCHEMA_VERSION,
            display_name: document.asset.display_name.clone(),
        },
        imports: document.imports.clone(),
        tokens: document.tokens.clone(),
        root: document.root.as_ref().map(|root| UiV2Root {
            node: root.node_id.clone(),
        }),
        nodes,
        components,
        stylesheets: document
            .stylesheets
            .iter()
            .map(|stylesheet| UiV2StyleSheet {
                id: stylesheet.id.clone(),
                rules: stylesheet
                    .rules
                    .iter()
                    .map(|rule| UiV2StyleRule {
                        id: rule.id.clone(),
                        selector: rule.selector.clone(),
                        set: v2_style_block(&rule.set),
                    })
                    .collect(),
            })
            .collect(),
    })
}

fn flatten_legacy_projection_nodes_into(
    nodes: &mut BTreeMap<String, UiV2NodeDefinition>,
    root: &UiNodeDefinition,
) -> Result<(), UiAssetEditorSessionError> {
    let mut stack = vec![root.clone()];
    while let Some(node) = stack.pop() {
        if node.node_id.trim().is_empty() {
            return Err(UiAssetEditorSessionError::InvalidSourceBuffer);
        }
        let children = node
            .children
            .iter()
            .map(|child| UiV2ChildMount {
                node: child.node.node_id.clone(),
                slot: child.slot.clone(),
            })
            .collect();
        let component = node
            .component
            .clone()
            .or_else(|| node.widget_type.clone())
            .unwrap_or_else(|| "Space".to_string());
        let next = UiV2NodeDefinition {
            component,
            control_id: node.control_id,
            classes: node.classes,
            props: node.props,
            state: node.params,
            layout: node.layout,
            style: v2_style_block(&node.style_overrides),
            slots: BTreeMap::new(),
            events: node.bindings,
            children,
        };
        if let Some(existing) = nodes.get(&node.node_id) {
            if existing != &next {
                return Err(UiAssetEditorSessionError::InvalidSourceBuffer);
            }
            continue;
        }
        for child in node.children.iter().rev() {
            stack.push(child.node.clone());
        }
        let _ = nodes.insert(node.node_id, next);
    }
    Ok(())
}

fn v2_node_tree_to_legacy_projection_node(
    document: &UiV2AssetDocument,
    root_node_id: &str,
) -> Result<UiNodeDefinition, UiAssetEditorSessionError> {
    let mut stack = vec![V2ProjectionBuildFrame::Enter {
        node_id: root_node_id.to_string(),
        slot: BTreeMap::new(),
    }];
    let mut built_nodes: Vec<(UiNodeDefinition, BTreeMap<String, toml::Value>)> = Vec::new();
    let mut active = BTreeSet::new();

    while let Some(frame) = stack.pop() {
        match frame {
            V2ProjectionBuildFrame::Enter { node_id, slot } => {
                let node = document
                    .nodes
                    .get(&node_id)
                    .ok_or_else(|| missing_v2_node(document, &node_id))?;
                if !active.insert(node_id.clone()) {
                    return Err(UiV2AssetError::InvalidDocument {
                        asset_id: document.asset.id.clone(),
                        detail: format!("ui v2 authoring projection cycle at node {node_id}"),
                    }
                    .into());
                }
                stack.push(V2ProjectionBuildFrame::Exit {
                    node_id: node_id.clone(),
                    slot,
                    child_count: node.children.len(),
                });
                for child in node.children.iter().rev() {
                    stack.push(V2ProjectionBuildFrame::Enter {
                        node_id: child.node.clone(),
                        slot: child.slot.clone(),
                    });
                }
            }
            V2ProjectionBuildFrame::Exit {
                node_id,
                slot,
                child_count,
            } => {
                let split_at = built_nodes.len().checked_sub(child_count).ok_or_else(|| {
                    UiV2AssetError::InvalidDocument {
                        asset_id: document.asset.id.clone(),
                        detail: format!(
                            "ui v2 authoring projection lost child state for node {node_id}"
                        ),
                    }
                })?;
                let children = built_nodes
                    .split_off(split_at)
                    .into_iter()
                    .map(|(node, slot)| UiChildMount {
                        mount: None,
                        slot,
                        node,
                    })
                    .collect();
                let source = document
                    .nodes
                    .get(&node_id)
                    .ok_or_else(|| missing_v2_node(document, &node_id))?;
                let node = legacy_projection_node(&node_id, source, children);
                active.remove(&node_id);
                built_nodes.push((node, slot));
            }
        }
    }

    built_nodes
        .pop()
        .map(|(node, _)| node)
        .ok_or_else(|| missing_v2_node(document, root_node_id).into())
}

fn missing_v2_node(document: &UiV2AssetDocument, node_id: &str) -> UiAssetEditorSessionError {
    UiV2AssetError::MissingNode {
        asset_id: document.asset.id.clone(),
        node_id: node_id.to_string(),
    }
    .into()
}

fn legacy_projection_node(
    node_id: &str,
    source: &UiV2NodeDefinition,
    children: Vec<UiChildMount>,
) -> UiNodeDefinition {
    UiNodeDefinition {
        node_id: node_id.to_string(),
        kind: UiNodeDefinitionKind::Native,
        widget_type: Some(source.component.clone()),
        component: Some(source.component.clone()),
        component_ref: None,
        component_api_version: None,
        slot_name: None,
        control_id: source.control_id.clone(),
        classes: source.classes.clone(),
        params: source.state.clone(),
        props: source.props.clone(),
        layout: source.layout.clone(),
        bindings: source.events.clone(),
        style_overrides: legacy_style_block(&source.style),
        focus: None,
        navigation: None,
        picking: None,
        a11y: None,
        widget: None,
        children,
    }
}

fn legacy_style_block(source: &UiV2StyleDeclarationBlock) -> UiStyleDeclarationBlock {
    UiStyleDeclarationBlock {
        self_values: source.self_values.clone(),
        slot: source.slot.clone(),
    }
}

fn v2_style_block(source: &UiStyleDeclarationBlock) -> UiV2StyleDeclarationBlock {
    UiV2StyleDeclarationBlock {
        self_values: source.self_values.clone(),
        slot: source.slot.clone(),
    }
}

fn legacy_asset_kind(kind: UiV2AssetKind) -> UiAssetKind {
    match kind {
        UiV2AssetKind::View => UiAssetKind::Layout,
        UiV2AssetKind::Component => UiAssetKind::Widget,
        UiV2AssetKind::Style => UiAssetKind::Style,
    }
}

fn v2_asset_kind(kind: UiAssetKind) -> UiV2AssetKind {
    match kind {
        UiAssetKind::Layout => UiV2AssetKind::View,
        UiAssetKind::Widget => UiV2AssetKind::Component,
        UiAssetKind::Style => UiV2AssetKind::Style,
    }
}

enum V2ProjectionBuildFrame {
    Enter {
        node_id: String,
        slot: BTreeMap<String, toml::Value>,
    },
    Exit {
        node_id: String,
        slot: BTreeMap<String, toml::Value>,
        child_count: usize,
    },
}

pub(super) fn compiled_resource_report(
    compiled: Option<&zircon_runtime::ui::template::UiCompiledDocument>,
) -> (Vec<UiResourceDependency>, Vec<UiResourceDiagnostic>) {
    compiled
        .map(|compiled| {
            (
                compiled.resource_dependencies().to_vec(),
                compiled.resource_diagnostics().to_vec(),
            )
        })
        .unwrap_or_default()
}

fn reconcile_selected_palette_index<T>(items: &[T], current: Option<usize>) -> Option<usize> {
    match (current, items.len()) {
        (_, 0) => None,
        (Some(index), count) => Some(index.min(count - 1)),
        (None, _) => None,
    }
}

pub(super) fn structured_compile_diagnostics(
    document: &UiAssetDocument,
    imports: &UiAssetCompilerImports,
) -> Vec<crate::ui::asset_editor::UiAssetEditorDiagnostic> {
    let mut diagnostics = structured_contract_diagnostics(document, imports);
    diagnostics.extend(
        EditorTemplateRuntimeService
            .collect_binding_report(document)
            .diagnostics
            .into_iter()
            .map(map_binding_diagnostic),
    );
    diagnostics.extend(
        EditorTemplateRuntimeService
            .collect_localization_report(document)
            .diagnostics
            .into_iter()
            .map(map_localization_diagnostic),
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
