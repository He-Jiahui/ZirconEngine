use std::collections::{BTreeMap, BTreeSet};

use crate::ui::{
    UiAssetEditorMode, UiAssetEditorReflectionModel, UiAssetEditorRoute, UiAssetPreviewPreset,
    UiDesignerSelectionModel, UiStyleInspectorReflectionModel,
};
use thiserror::Error;
use zircon_runtime::ui::template::{
    UiAssetError, UiAssetLoader, UiCompiledDocument, UiStyleDeclarationBlock, UiStyleRule,
    UiStyleSheet, UiTemplateBuildError,
};
use zircon_runtime::ui::tree::UiTreeError;
use zircon_runtime::ui::{layout::UiSize, template::UiAssetDocument, template::UiAssetKind};

use super::{
    binding_inspector::{
        add_default_binding,
        apply_selected_binding_action_suggestion as apply_selected_binding_action_suggestion_field,
        apply_selected_binding_payload_suggestion as apply_selected_binding_payload_suggestion_field,
        apply_selected_binding_route_suggestion as apply_selected_binding_route_suggestion_field,
        build_binding_fields, delete_selected_binding as delete_selected_binding_field,
        delete_selected_binding_payload as delete_selected_binding_payload_field,
        reconcile_selected_binding_index, reconcile_selected_binding_payload_key,
        set_selected_binding_action_kind as set_selected_binding_action_kind_field,
        set_selected_binding_action_target as set_selected_binding_action_target_field,
        set_selected_binding_event as set_selected_binding_event_field,
        set_selected_binding_id as set_selected_binding_id_field,
        set_selected_binding_route as set_selected_binding_route_field,
        set_selected_binding_route_target as set_selected_binding_route_target_field,
        upsert_selected_binding_payload as upsert_selected_binding_payload_field,
    },
    command::{
        UiAssetEditorCommand, UiAssetEditorDocumentReplayBundle,
        UiAssetEditorDocumentReplayCommand, UiAssetEditorInverseTreeEdit, UiAssetEditorTreeEdit,
        UiAssetEditorTreeEditKind,
    },
    hierarchy_projection::{
        build_hierarchy_items, build_inspector_items, hierarchy_node_ids, selected_hierarchy_index,
        selection_for_node, selection_summary,
    },
    inspector_fields::{
        build_inspector_fields, set_selected_node_control_id,
        set_selected_node_layout_height_preferred, set_selected_node_layout_width_preferred,
        set_selected_node_mount, set_selected_node_slot_height_preferred,
        set_selected_node_slot_padding, set_selected_node_slot_width_preferred,
        set_selected_node_text_property,
    },
    inspector_semantics::{
        build_layout_semantic_group, build_slot_semantic_group,
        build_structured_layout_semantic_fields, build_structured_slot_semantic_fields,
        delete_selected_layout_semantic as delete_selected_layout_semantic_field,
        delete_selected_slot_semantic as delete_selected_slot_semantic_field,
        reconcile_selected_semantic_path,
        set_selected_layout_semantic_value as set_selected_layout_semantic_value_field,
        set_selected_slot_semantic_value as set_selected_slot_semantic_value_field,
    },
    palette_drop::{
        build_palette_drag_slot_target_overlays, build_palette_insert_plan,
        can_insert_palette_item_for_node as can_insert_palette_item_at_node,
        resolve_palette_drag_target as resolve_palette_drag_target_for_preview,
        UiAssetPaletteDragResolution, UiAssetPaletteDragTarget, UiAssetPaletteInsertPlan,
    },
    palette_target_chooser::{reconcile_palette_target_chooser, UiAssetPaletteTargetChooser},
    presentation::{
        UiAssetEditorPanePresentation, UiAssetEditorPreviewCanvasNode,
        UiAssetEditorPreviewCanvasSlotTarget,
    },
    preview_compile::{compile_preview, current_preview_size, preview_size_for_preset},
    preview_host::UiAssetPreviewHost,
    preview_mock::{
        apply_preview_mock_overrides,
        apply_selected_preview_mock_suggestion as apply_selected_preview_mock_suggestion_field,
        build_preview_mock_fields, build_preview_state_graph_items,
        clear_selected_preview_mock_value,
        delete_selected_preview_mock_nested_entry as delete_selected_preview_mock_nested_entry_field,
        reconcile_preview_mock_state,
        select_preview_mock_nested_entry as select_preview_mock_nested_entry_field,
        select_preview_mock_property, select_preview_mock_subject,
        select_preview_mock_subject_node,
        set_selected_preview_mock_nested_value as set_selected_preview_mock_nested_value_field,
        set_selected_preview_mock_value,
        upsert_selected_preview_mock_nested_entry as upsert_selected_preview_mock_nested_entry_field,
        UiAssetPreviewMockState,
    },
    preview_projection::{build_preview_projection, preview_node_id_for_index},
    promote_widget::{
        can_promote_selected_component_to_external_widget, default_external_widget_draft,
        promote_selected_component_to_external_widget as tree_promote_selected_component_to_external_widget,
        selected_local_component_name, UiAssetExternalWidgetDraft,
    },
    session_state::{
        default_selection, ensure_asset_kind, reconcile_selection, UiAssetCompilerImports,
    },
    source_buffer::UiAssetSourceBuffer,
    source_sync::{
        build_source_outline, build_source_selection_summary, source_byte_offset_for_line,
        source_line_for_byte_offset, source_outline_entry_for_node,
        source_outline_node_id_for_line,
    },
    style_inspection::{
        build_style_inspector, build_stylesheet_items, local_style_rule_entries,
        local_style_token_entries, matched_style_rule_entries_for_selection, normalized_class_name,
        normalized_selector, normalized_token_name, parse_token_literal, pseudo_state_active,
        reconcile_selected_matched_style_rule_index,
        reconcile_selected_style_rule_declaration_path, reconcile_selected_style_rule_index,
        reconcile_selected_style_token_name, selected_node_has_inline_overrides,
        selected_node_selector, selected_style_rule_declaration_entries, MatchedStyleRuleEntry,
        SUPPORTED_PSEUDO_STATES,
    },
    style_rule_declarations::{
        declaration_entries, parse_declaration_literal, remove_declaration, set_declaration,
        UiStyleRuleDeclarationPath,
    },
    theme_authoring::{
        adopt_active_cascade_rule, adopt_active_cascade_rules, adopt_active_cascade_token,
        adopt_active_cascade_tokens, adopt_all_active_cascade_changes,
        adopt_all_imported_theme_changes, adopt_imported_theme_compare_diffs,
        adopt_imported_theme_rule, adopt_imported_theme_rules, adopt_imported_theme_token,
        adopt_imported_theme_tokens, apply_theme_refactor_action,
        build_imported_theme_local_merge_preview, build_theme_refactor_items,
        build_theme_rule_helper_items, can_promote_local_theme_to_external_style_asset,
        can_prune_duplicate_local_theme_overrides, clone_imported_theme_to_local_theme_layer,
        default_external_style_draft, detach_imported_theme_to_local_theme_layer,
        promote_local_theme_to_external_style_asset as tree_promote_local_theme_to_external_style_asset,
        prune_duplicate_local_theme_overrides, prune_imported_theme_compare_duplicates,
        theme_refactor_actions, theme_rule_helper_actions, UiAssetExternalStyleDraft,
        UiAssetThemeRefactorAction, UiAssetThemeRuleHelperAction,
    },
    theme_cascade_inspection::build_theme_cascade_inspection,
    theme_compare::build_theme_compare_items,
    theme_summary::{
        build_theme_source_details, build_theme_summary, reconcile_selected_theme_source_key,
        select_theme_source_key,
    },
    tree_editing::{
        build_palette_entries, can_convert_selected_node_to_reference,
        can_extract_selected_node_to_component,
        convert_selected_node_to_reference as tree_convert_selected_node_to_reference,
        extract_selected_node_to_component as tree_extract_selected_node_to_component,
        insert_palette_item_with_placement, move_selected_node,
        reparent_selected_node as tree_reparent_selected_node, unwrap_selected_node,
        wrap_selected_node, PaletteInsertMode, UiTreeMoveDirection, UiTreeReparentDirection,
    },
    undo_stack::{
        UiAssetEditorExternalEffect, UiAssetEditorSourceCursorSnapshot,
        UiAssetEditorUndoExternalEffects, UiAssetEditorUndoStack,
    },
};

#[derive(Debug, Error)]
pub enum UiAssetEditorSessionError {
    #[error(transparent)]
    Asset(#[from] UiAssetError),
    #[error(transparent)]
    Build(#[from] UiTemplateBuildError),
    #[error(transparent)]
    Layout(#[from] UiTreeError),
    #[error("expected ui asset kind {expected:?} but document was {actual:?}")]
    UnexpectedKind {
        expected: UiAssetKind,
        actual: UiAssetKind,
    },
    #[error("cannot serialize an invalid ui asset source buffer")]
    InvalidSourceBuffer,
    #[error("ui asset selection index {index} is out of range")]
    InvalidSelectionIndex { index: usize },
    #[error("ui asset preview index {index} did not map to a selectable node")]
    InvalidPreviewIndex { index: usize },
    #[error("ui asset stylesheet rule index {index} is out of range")]
    InvalidStyleRuleIndex { index: usize },
    #[error("ui asset matched style rule index {index} is out of range")]
    InvalidMatchedStyleRuleIndex { index: usize },
    #[error("ui asset stylesheet rule declaration index {index} is out of range")]
    InvalidStyleRuleDeclarationIndex { index: usize },
    #[error("ui asset style token index {index} is out of range")]
    InvalidStyleTokenIndex { index: usize },
    #[error("ui asset binding index {index} is out of range")]
    InvalidBindingIndex { index: usize },
    #[error("ui asset palette index {index} is out of range")]
    InvalidPaletteIndex { index: usize },
    #[error("ui asset stylesheet selector is invalid: {selector}")]
    InvalidStyleSelector { selector: String },
    #[error("ui asset stylesheet declaration path is invalid: {path}")]
    InvalidStyleDeclarationPath { path: String },
    #[error("ui asset inspector field {field} expects a numeric literal, received: {value}")]
    InvalidInspectorNumericLiteral { field: &'static str, value: String },
    #[error("ui asset binding event is invalid: {value}")]
    InvalidBindingEvent { value: String },
    #[error("ui asset preview mock value is invalid: {message}")]
    InvalidPreviewMockValue { message: String },
}

#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct UiAssetEditorReplayResult {
    pub changed: bool,
    pub label: String,
    pub external_effects: Vec<UiAssetEditorExternalEffect>,
}

pub struct UiAssetEditorSession {
    route: UiAssetEditorRoute,
    source_buffer: UiAssetSourceBuffer,
    last_valid_source_text: String,
    last_valid_document: UiAssetDocument,
    last_valid_compiled: Option<UiCompiledDocument>,
    preview_host: Option<UiAssetPreviewHost>,
    undo_stack: UiAssetEditorUndoStack,
    diagnostics: Vec<String>,
    source_cursor_byte_offset: usize,
    source_cursor_anchor: Option<UiAssetSourceCursorAnchor>,
    selection: UiDesignerSelectionModel,
    style_inspector: UiStyleInspectorReflectionModel,
    selected_style_rule_index: Option<usize>,
    selected_matched_style_rule_index: Option<usize>,
    selected_style_rule_declaration_path: Option<String>,
    selected_style_token_name: Option<String>,
    selected_theme_source_key: Option<String>,
    selected_binding_index: Option<usize>,
    selected_binding_payload_key: Option<String>,
    selected_slot_semantic_path: Option<String>,
    selected_layout_semantic_path: Option<String>,
    selected_palette_index: Option<usize>,
    palette_target_chooser: Option<UiAssetPaletteTargetChooser>,
    selected_promote_source_component_name: Option<String>,
    selected_promote_widget_asset_id: Option<String>,
    selected_promote_widget_component_name: Option<String>,
    selected_promote_widget_document_id: Option<String>,
    selected_promote_theme_asset_id: Option<String>,
    selected_promote_theme_document_id: Option<String>,
    selected_promote_theme_display_name: Option<String>,
    preview_mock_state: UiAssetPreviewMockState,
    compiler_imports: UiAssetCompilerImports,
}

#[derive(Clone, Debug, PartialEq, Eq)]
struct UiAssetSourceCursorAnchor {
    node_id: String,
    line_offset: usize,
}

impl UiAssetEditorSession {
    pub fn from_source(
        route: UiAssetEditorRoute,
        source: impl Into<String>,
        preview_size: UiSize,
    ) -> Result<Self, UiAssetEditorSessionError> {
        let source = source.into();
        let document = UiAssetLoader::load_toml_str(&source)?;
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
        let promote_draft = default_external_widget_draft(&document, &selection);
        let theme_draft = can_promote_local_theme_to_external_style_asset(&document)
            .then(|| default_external_style_draft(&route.asset_id, &document.asset.display_name));
        let (last_valid_compiled, preview_host, diagnostics) =
            match compile_preview(&document, preview_size, &compiler_imports) {
                Ok((compiled, preview_host)) => (compiled, preview_host, Vec::new()),
                Err(error) => (None, None, vec![error.to_string()]),
            };
        Ok(Self {
            route,
            source_buffer: UiAssetSourceBuffer::new(source.clone()),
            last_valid_source_text: source,
            last_valid_document: document,
            last_valid_compiled,
            preview_host,
            undo_stack: UiAssetEditorUndoStack::default(),
            diagnostics,
            source_cursor_byte_offset,
            source_cursor_anchor,
            selection,
            style_inspector,
            selected_style_rule_index: None,
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
            preview_mock_state: UiAssetPreviewMockState::default(),
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

    pub fn reflection_model(&self) -> UiAssetEditorReflectionModel {
        let mut model = UiAssetEditorReflectionModel::new(
            self.route.clone(),
            self.last_valid_document.asset.display_name.clone(),
        )
        .with_source_dirty(self.source_buffer.is_dirty())
        .with_undo_state(self.can_undo(), self.can_redo())
        .with_preview_available(self.preview_host.is_some())
        .with_selection(self.selection.clone())
        .with_style_inspector(self.style_inspector.clone());
        if let Some(error) = self.diagnostics.first() {
            model = model.with_last_error(error.clone());
        }
        model
    }

    pub fn pane_presentation(&self) -> UiAssetEditorPanePresentation {
        let reflection = self.reflection_model();
        let preview_summary = preview_summary(self.preview_host.as_ref());
        let preview_projection = build_preview_projection(
            &self.last_valid_document,
            self.preview_host.as_ref(),
            &self.selection,
        );
        let selected_palette_drag_target = self.selected_palette_drag_target();
        let palette_drag_slot_target_items = self
            .selected_palette_drag_target()
            .map(|drag_target| {
                build_palette_drag_slot_target_overlays(
                    &self.last_valid_document,
                    drag_target,
                    &self.compiler_imports.widgets,
                    &preview_projection,
                )
                .into_iter()
                .map(|item| UiAssetEditorPreviewCanvasSlotTarget {
                    label: item.label,
                    detail: item.detail,
                    x: item.x,
                    y: item.y,
                    width: item.width,
                    height: item.height,
                    selected: item.selected,
                })
                .collect::<Vec<_>>()
            })
            .unwrap_or_default();
        let palette_drag_candidate_items = self
            .palette_target_chooser
            .as_ref()
            .map(|chooser| {
                chooser
                    .resolution()
                    .candidates
                    .iter()
                    .map(|candidate| {
                        if candidate.detail.is_empty() {
                            candidate.key.clone()
                        } else {
                            format!("{} • {}", candidate.key, candidate.detail)
                        }
                    })
                    .collect::<Vec<_>>()
            })
            .unwrap_or_default();
        let selector_hint = selected_node_selector(&self.last_valid_document, &self.selection);
        let palette_entries =
            build_palette_entries(&self.last_valid_document, &self.compiler_imports.widgets);
        let roundtrip_source = self.roundtrip_source_text();
        let source_summary = build_source_selection_summary(
            &self.last_valid_document,
            &self.selection,
            roundtrip_source,
            &self.diagnostics,
            self.selected_source_line_offset(),
        );
        let source_outline = build_source_outline(&self.last_valid_document, roundtrip_source);
        let source_outline_selected_index = self
            .selection
            .primary_node_id
            .as_deref()
            .and_then(|node_id| {
                source_outline
                    .iter()
                    .position(|entry| entry.node_id.as_str() == node_id)
            })
            .map(|index| index as i32)
            .unwrap_or(-1);
        let preview_mock_fields = build_preview_mock_fields(
            &self.last_valid_document,
            &self.selection,
            &self.preview_mock_state,
        );
        let preview_state_graph_items =
            build_preview_state_graph_items(&self.last_valid_document, &self.preview_mock_state);
        let inspector_fields = build_inspector_fields(&self.last_valid_document, &self.selection);
        let binding_fields = build_binding_fields(
            &self.last_valid_document,
            &self.selection,
            &self.preview_mock_state,
            self.selected_binding_index,
            self.selected_binding_payload_key.as_deref(),
        );
        let slot_semantic_group =
            build_slot_semantic_group(&self.last_valid_document, &self.selection);
        let structured_slot_semantic =
            build_structured_slot_semantic_fields(&self.last_valid_document, &self.selection);
        let selected_slot_semantic = self
            .selected_slot_semantic_path
            .as_deref()
            .and_then(|path| {
                slot_semantic_group
                    .entries
                    .iter()
                    .position(|entry| entry.path.as_str() == path)
            })
            .and_then(|index| {
                slot_semantic_group
                    .entries
                    .get(index)
                    .map(|entry| (index, entry))
            });
        let layout_semantic_group =
            build_layout_semantic_group(&self.last_valid_document, &self.selection);
        let structured_layout_semantic =
            build_structured_layout_semantic_fields(&self.last_valid_document, &self.selection);
        let selected_layout_semantic = self
            .selected_layout_semantic_path
            .as_deref()
            .and_then(|path| {
                layout_semantic_group
                    .entries
                    .iter()
                    .position(|entry| entry.path.as_str() == path)
            })
            .and_then(|index| {
                layout_semantic_group
                    .entries
                    .get(index)
                    .map(|entry| (index, entry))
            });
        let style_rules = local_style_rule_entries(&self.last_valid_document);
        let matched_style_rules = matched_style_rule_entries_for_selection(
            &self.last_valid_document,
            &self.selection,
            &self.compiler_imports,
            &self.style_inspector.active_pseudo_states,
        );
        let style_tokens = local_style_token_entries(&self.last_valid_document);
        let selected_style_rule = self
            .selected_style_rule_index
            .and_then(|index| style_rules.get(index));
        let selected_matched_style_rule = self
            .selected_matched_style_rule_index
            .and_then(|index| matched_style_rules.get(index).map(|entry| (index, entry)));
        let style_rule_declarations = selected_style_rule
            .map(|entry| {
                declaration_entries(
                    &self.last_valid_document.stylesheets[entry.stylesheet_index].rules
                        [entry.rule_index]
                        .set,
                )
            })
            .unwrap_or_default();
        let selected_style_rule_declaration = self
            .selected_style_rule_declaration_path
            .as_deref()
            .and_then(|path| {
                style_rule_declarations
                    .iter()
                    .position(|entry| entry.path.as_str() == path)
            })
            .and_then(|index| {
                style_rule_declarations
                    .get(index)
                    .map(|entry| (index, entry))
            });
        let selected_style_token = self
            .selected_style_token_name
            .as_deref()
            .and_then(|name| {
                style_tokens
                    .iter()
                    .position(|entry| entry.name.as_str() == name)
            })
            .and_then(|index| style_tokens.get(index).map(|entry| (index, entry)));
        let theme_summary = build_theme_summary(
            &self.last_valid_document,
            &self.compiler_imports.styles,
            self.selected_theme_source_key.as_deref(),
        );
        let theme_source_details = build_theme_source_details(
            &self.last_valid_document,
            &self.compiler_imports.styles,
            self.selected_theme_source_key.as_deref(),
        );
        let theme_cascade = build_theme_cascade_inspection(
            &self.last_valid_document,
            &self.compiler_imports.styles,
        );
        let theme_compare_items = build_theme_compare_items(
            &self.last_valid_document,
            &self.compiler_imports.styles,
            self.selected_theme_source_key.as_deref(),
        );
        let theme_rule_helper_items = build_theme_rule_helper_items(
            &self.last_valid_document,
            &self.compiler_imports.styles,
            self.selected_theme_source_key.as_deref(),
        );
        let theme_refactor_items =
            build_theme_refactor_items(&self.last_valid_document, &self.compiler_imports.styles);
        let theme_merge_preview_items = self
            .selected_theme_source_key
            .as_deref()
            .filter(|key| *key != "local")
            .and_then(|reference| {
                self.compiler_imports
                    .styles
                    .get(reference)
                    .map(|imported_style| {
                        build_imported_theme_local_merge_preview(
                            &self.last_valid_document,
                            reference,
                            imported_style,
                        )
                    })
            })
            .unwrap_or_default();
        let can_create_rule =
            self.diagnostics.is_empty() && selector_hint.is_some() && self.preview_host.is_some();
        let can_extract_rule = can_create_rule
            && selected_node_has_inline_overrides(&self.last_valid_document, &self.selection);
        let can_insert_child = self.diagnostics.is_empty()
            && self
                .selected_insert_target_node_id()
                .and_then(|node_id| {
                    self.selected_palette_index
                        .and_then(|index| palette_entries.get(index).map(|entry| (node_id, entry)))
                })
                .is_some_and(|(node_id, entry)| {
                    can_insert_palette_item_at_node(
                        &self.last_valid_document,
                        entry,
                        node_id,
                        PaletteInsertMode::Child,
                        &self.compiler_imports.widgets,
                    )
                });
        let can_insert_after = self.diagnostics.is_empty()
            && self
                .selected_insert_target_node_id()
                .and_then(|node_id| {
                    self.selected_palette_index
                        .and_then(|index| palette_entries.get(index).map(|entry| (node_id, entry)))
                })
                .is_some_and(|(node_id, entry)| {
                    can_insert_palette_item_at_node(
                        &self.last_valid_document,
                        entry,
                        node_id,
                        PaletteInsertMode::After,
                        &self.compiler_imports.widgets,
                    )
                });
        let can_move_up = self.diagnostics.is_empty()
            && can_apply_tree_document_edit(&self.last_valid_document, |document| {
                move_selected_node(document, &self.selection, UiTreeMoveDirection::Up)
            });
        let can_move_down = self.diagnostics.is_empty()
            && can_apply_tree_document_edit(&self.last_valid_document, |document| {
                move_selected_node(document, &self.selection, UiTreeMoveDirection::Down)
            });
        let can_reparent_into_previous = self.diagnostics.is_empty()
            && can_apply_tree_document_edit(&self.last_valid_document, |document| {
                tree_reparent_selected_node(
                    document,
                    &self.selection,
                    UiTreeReparentDirection::IntoPrevious,
                )
                .is_some()
            });
        let can_reparent_into_next = self.diagnostics.is_empty()
            && can_apply_tree_document_edit(&self.last_valid_document, |document| {
                tree_reparent_selected_node(
                    document,
                    &self.selection,
                    UiTreeReparentDirection::IntoNext,
                )
                .is_some()
            });
        let can_reparent_outdent = self.diagnostics.is_empty()
            && can_apply_tree_document_edit(&self.last_valid_document, |document| {
                tree_reparent_selected_node(
                    document,
                    &self.selection,
                    UiTreeReparentDirection::Outdent,
                )
                .is_some()
            });
        let can_open_reference = self.selected_reference_asset_id().is_some();
        let can_convert_to_reference = self
            .selected_palette_index
            .and_then(|index| palette_entries.get(index))
            .is_some_and(|entry| {
                can_convert_selected_node_to_reference(
                    &self.last_valid_document,
                    &self.selection,
                    entry,
                    &self.compiler_imports.widgets,
                )
            });
        let can_extract_component = self.diagnostics.is_empty()
            && can_extract_selected_node_to_component(&self.last_valid_document, &self.selection);
        let can_promote_to_external_widget = self.diagnostics.is_empty()
            && can_promote_selected_component_to_external_widget(
                &self.last_valid_document,
                &self.selection,
            );
        let can_wrap_in_vertical_box = self.diagnostics.is_empty()
            && can_apply_tree_document_edit(&self.last_valid_document, |document| {
                wrap_selected_node(document, &self.selection, "VerticalBox").is_some()
            });
        let can_unwrap = self.diagnostics.is_empty()
            && can_apply_tree_document_edit(&self.last_valid_document, |document| {
                unwrap_selected_node(document, &self.selection).is_some()
            });
        let promote_draft = self.selected_promote_widget_draft();
        let theme_promote_draft = self.selected_promote_theme_draft();
        let can_edit_theme_promote_draft =
            theme_summary.selected_kind == "Local" && theme_summary.can_promote_local;
        UiAssetEditorPanePresentation {
            asset_id: reflection.route.asset_id.clone(),
            mode: format!("{:?}", reflection.route.mode),
            source_dirty: reflection.source_dirty,
            can_save: reflection.source_dirty && reflection.last_error.is_none(),
            can_undo: reflection.can_undo,
            can_redo: reflection.can_redo,
            can_insert_child,
            can_insert_after,
            can_move_up,
            can_move_down,
            can_reparent_into_previous,
            can_reparent_into_next,
            can_reparent_outdent,
            can_open_reference,
            can_convert_to_reference,
            can_extract_component,
            can_promote_to_external_widget,
            can_wrap_in_vertical_box,
            can_unwrap,
            can_create_rule,
            can_extract_rule,
            preview_available: reflection.preview_available,
            style_state_hover: pseudo_state_active(&reflection.style_inspector, "hover"),
            style_state_focus: pseudo_state_active(&reflection.style_inspector, "focus"),
            style_state_pressed: pseudo_state_active(&reflection.style_inspector, "pressed"),
            style_state_disabled: pseudo_state_active(&reflection.style_inspector, "disabled"),
            style_state_selected: pseudo_state_active(&reflection.style_inspector, "selected"),
            style_class_items: reflection.style_inspector.classes.clone(),
            style_rule_items: style_rules
                .iter()
                .map(|rule| rule.selector.clone())
                .collect(),
            style_rule_selected_index: self
                .selected_style_rule_index
                .map(|index| index as i32)
                .unwrap_or(-1),
            style_selected_rule_selector: selected_style_rule
                .map(|rule| rule.selector.clone())
                .unwrap_or_default(),
            style_can_edit_rule: self.diagnostics.is_empty() && selected_style_rule.is_some(),
            style_can_delete_rule: self.diagnostics.is_empty() && selected_style_rule.is_some(),
            style_matched_rule_items: matched_style_rules
                .iter()
                .map(MatchedStyleRuleEntry::label)
                .collect(),
            style_matched_rule_selected_index: selected_matched_style_rule
                .map(|(index, _)| index as i32)
                .unwrap_or(-1),
            style_selected_matched_rule_origin: selected_matched_style_rule
                .map(|(_, rule)| rule.origin_id.clone())
                .unwrap_or_default(),
            style_selected_matched_rule_selector: selected_matched_style_rule
                .map(|(_, rule)| rule.selector.clone())
                .unwrap_or_default(),
            style_selected_matched_rule_specificity: selected_matched_style_rule
                .map(|(_, rule)| rule.specificity as i32)
                .unwrap_or(-1),
            style_selected_matched_rule_source_order: selected_matched_style_rule
                .map(|(_, rule)| rule.source_order as i32)
                .unwrap_or(-1),
            style_selected_matched_rule_declaration_items: selected_matched_style_rule
                .map(|(_, rule)| rule.declaration_items())
                .unwrap_or_default(),
            style_rule_declaration_items: style_rule_declarations
                .iter()
                .map(|entry| format!("{} = {}", entry.path, entry.literal))
                .collect(),
            style_rule_declaration_selected_index: selected_style_rule_declaration
                .map(|(index, _)| index as i32)
                .unwrap_or(-1),
            style_selected_rule_declaration_path: selected_style_rule_declaration
                .map(|(_, entry)| entry.path.clone())
                .unwrap_or_default(),
            style_selected_rule_declaration_value: selected_style_rule_declaration
                .map(|(_, entry)| entry.literal.clone())
                .unwrap_or_default(),
            style_can_edit_rule_declaration: self.diagnostics.is_empty()
                && selected_style_rule.is_some(),
            style_can_delete_rule_declaration: self.diagnostics.is_empty()
                && selected_style_rule_declaration.is_some(),
            style_token_items: style_tokens
                .iter()
                .map(|entry| format!("{} = {}", entry.name, entry.literal))
                .collect(),
            style_token_selected_index: selected_style_token
                .map(|(index, _)| index as i32)
                .unwrap_or(-1),
            style_selected_token_name: selected_style_token
                .map(|(_, entry)| entry.name.clone())
                .unwrap_or_default(),
            style_selected_token_value: selected_style_token
                .map(|(_, entry)| entry.literal.clone())
                .unwrap_or_default(),
            style_can_edit_token: self.diagnostics.is_empty() && selected_style_token.is_some(),
            style_can_delete_token: self.diagnostics.is_empty() && selected_style_token.is_some(),
            theme_source_items: theme_summary.items,
            theme_source_selected_index: theme_summary.selected_index,
            theme_selected_source_reference: theme_summary.selected_reference,
            theme_selected_source_kind: theme_summary.selected_kind,
            theme_selected_source_token_count: theme_summary.selected_token_count,
            theme_selected_source_rule_count: theme_summary.selected_rule_count,
            theme_selected_source_available: theme_summary.selected_available,
            theme_can_promote_local: theme_summary.can_promote_local,
            theme_selected_source_token_items: theme_source_details.token_items,
            theme_selected_source_rule_items: theme_source_details.rule_items,
            theme_cascade_layer_items: theme_cascade.layer_items,
            theme_cascade_token_items: theme_cascade.token_items,
            theme_cascade_rule_items: theme_cascade.rule_items,
            theme_compare_items,
            theme_merge_preview_items,
            theme_rule_helper_items,
            theme_refactor_items,
            theme_promote_asset_id: theme_promote_draft
                .as_ref()
                .map(|draft| draft.asset_id.clone())
                .unwrap_or_default(),
            theme_promote_document_id: theme_promote_draft
                .as_ref()
                .map(|draft| draft.document_id.clone())
                .unwrap_or_default(),
            theme_promote_display_name: theme_promote_draft
                .as_ref()
                .map(|draft| draft.display_name.clone())
                .unwrap_or_default(),
            theme_can_edit_promote_draft: can_edit_theme_promote_draft,
            theme_can_prune_duplicate_local_overrides: can_prune_duplicate_local_theme_overrides(
                &self.last_valid_document,
                &self.compiler_imports.styles,
            ),
            last_error: reflection.last_error.clone().unwrap_or_default(),
            selection_summary: selection_summary(&reflection.selection),
            source_text: self.source_buffer.text().to_string(),
            preview_preset: reflection.route.preview_preset.label().to_string(),
            source_selected_block_label: source_summary.block_label,
            source_selected_line: source_summary.line,
            source_cursor_byte_offset: self.source_cursor_byte_offset.min(i32::MAX as usize) as i32,
            source_selected_excerpt: source_summary.excerpt,
            source_roundtrip_status: source_summary.roundtrip_status,
            source_outline_items: source_outline
                .iter()
                .map(|entry| format!("line {} • {}", entry.line, entry.block_label))
                .collect(),
            source_outline_selected_index,
            preview_surface_width: preview_projection.surface_width,
            preview_surface_height: preview_projection.surface_height,
            preview_canvas_items: preview_projection
                .canvas_nodes
                .into_iter()
                .map(|item| UiAssetEditorPreviewCanvasNode {
                    node_id: item.node_id,
                    label: item.label,
                    kind: item.kind,
                    x: item.x,
                    y: item.y,
                    width: item.width,
                    height: item.height,
                    depth: item.depth,
                    z_index: item.z_index,
                    selected: item.selected,
                })
                .collect(),
            preview_mock_subject_items: preview_mock_fields.subject_items,
            preview_mock_subject_selected_index: preview_mock_fields.subject_selected_index,
            preview_mock_subject_node_id: preview_mock_fields.subject_node_id,
            preview_mock_items: preview_mock_fields.items,
            preview_mock_selected_index: preview_mock_fields.selected_index,
            preview_mock_property: preview_mock_fields.property,
            preview_mock_kind: preview_mock_fields.kind,
            preview_mock_value: preview_mock_fields.value,
            preview_mock_expression_result: preview_mock_fields.expression_result,
            preview_mock_nested_items: preview_mock_fields.nested_items,
            preview_mock_nested_selected_index: preview_mock_fields.nested_selected_index,
            preview_mock_nested_key: preview_mock_fields.nested_key,
            preview_mock_nested_kind: preview_mock_fields.nested_kind,
            preview_mock_nested_value: preview_mock_fields.nested_value,
            preview_mock_suggestion_items: preview_mock_fields.suggestion_items,
            preview_mock_schema_items: preview_mock_fields.schema_items,
            preview_state_graph_items,
            preview_mock_can_edit: preview_mock_fields.can_edit,
            preview_mock_can_clear: preview_mock_fields.can_clear,
            preview_mock_nested_can_edit: preview_mock_fields.nested_can_edit,
            preview_mock_nested_can_add: preview_mock_fields.nested_can_add,
            preview_mock_nested_can_delete: preview_mock_fields.nested_can_delete,
            preview_summary,
            palette_selected_index: self
                .selected_palette_index
                .map(|index| index as i32)
                .unwrap_or(-1),
            palette_drag_target_preview_index: selected_palette_drag_target
                .and_then(|target| target.preview_index.map(|index| index as i32))
                .unwrap_or(-1),
            palette_drag_target_action: selected_palette_drag_target
                .map(|target| palette_insert_mode_action(target.plan.mode).to_string())
                .unwrap_or_default(),
            palette_drag_target_label: selected_palette_drag_target
                .map(|target| target.plan.label.clone())
                .unwrap_or_default(),
            palette_drag_slot_target_items,
            palette_drag_candidate_items,
            palette_drag_candidate_selected_index: self
                .palette_target_chooser
                .as_ref()
                .map(|chooser| chooser.resolution().selected_index as i32)
                .unwrap_or(-1),
            palette_target_chooser_active: self
                .palette_target_chooser
                .as_ref()
                .map(UiAssetPaletteTargetChooser::sticky)
                .unwrap_or(false),
            hierarchy_selected_index: selected_hierarchy_index(
                &self.last_valid_document,
                &self.selection,
            ),
            preview_selected_index: preview_projection.selected_index,
            inspector_selected_node_id: inspector_fields.selected_node_id,
            inspector_parent_node_id: inspector_fields.parent_node_id,
            inspector_mount: inspector_fields.mount,
            inspector_slot_padding: inspector_fields.slot_padding,
            inspector_slot_width_preferred: inspector_fields.slot_width_preferred,
            inspector_slot_height_preferred: inspector_fields.slot_height_preferred,
            inspector_slot_semantic_title: slot_semantic_group.title,
            inspector_slot_semantic_items: slot_semantic_group
                .entries
                .iter()
                .map(|entry| entry.label())
                .collect(),
            inspector_slot_semantic_selected_index: selected_slot_semantic
                .map(|(index, _)| index as i32)
                .unwrap_or(-1),
            inspector_slot_semantic_path: selected_slot_semantic
                .map(|(_, entry)| entry.path.clone())
                .unwrap_or_default(),
            inspector_slot_semantic_value: selected_slot_semantic
                .map(|(_, entry)| entry.literal.clone())
                .unwrap_or_default(),
            inspector_slot_kind: structured_slot_semantic.kind,
            inspector_slot_linear_main_weight: structured_slot_semantic.linear_main_weight,
            inspector_slot_linear_main_stretch: structured_slot_semantic.linear_main_stretch,
            inspector_slot_linear_cross_weight: structured_slot_semantic.linear_cross_weight,
            inspector_slot_linear_cross_stretch: structured_slot_semantic.linear_cross_stretch,
            inspector_slot_overlay_anchor_x: structured_slot_semantic.overlay_anchor_x,
            inspector_slot_overlay_anchor_y: structured_slot_semantic.overlay_anchor_y,
            inspector_slot_overlay_pivot_x: structured_slot_semantic.overlay_pivot_x,
            inspector_slot_overlay_pivot_y: structured_slot_semantic.overlay_pivot_y,
            inspector_slot_overlay_position_x: structured_slot_semantic.overlay_position_x,
            inspector_slot_overlay_position_y: structured_slot_semantic.overlay_position_y,
            inspector_slot_overlay_z_index: structured_slot_semantic.overlay_z_index,
            inspector_slot_grid_row: structured_slot_semantic.grid_row,
            inspector_slot_grid_column: structured_slot_semantic.grid_column,
            inspector_slot_grid_row_span: structured_slot_semantic.grid_row_span,
            inspector_slot_grid_column_span: structured_slot_semantic.grid_column_span,
            inspector_slot_flow_break_before: structured_slot_semantic.flow_break_before,
            inspector_slot_flow_alignment: structured_slot_semantic.flow_alignment,
            inspector_layout_width_preferred: inspector_fields.layout_width_preferred,
            inspector_layout_height_preferred: inspector_fields.layout_height_preferred,
            inspector_layout_semantic_title: layout_semantic_group.title,
            inspector_layout_semantic_items: layout_semantic_group
                .entries
                .iter()
                .map(|entry| entry.label())
                .collect(),
            inspector_layout_semantic_selected_index: selected_layout_semantic
                .map(|(index, _)| index as i32)
                .unwrap_or(-1),
            inspector_layout_semantic_path: selected_layout_semantic
                .map(|(_, entry)| entry.path.clone())
                .unwrap_or_default(),
            inspector_layout_semantic_value: selected_layout_semantic
                .map(|(_, entry)| entry.literal.clone())
                .unwrap_or_default(),
            inspector_layout_kind: structured_layout_semantic.kind,
            inspector_layout_box_gap: structured_layout_semantic.box_gap,
            inspector_layout_scroll_axis: structured_layout_semantic.scroll_axis,
            inspector_layout_scroll_gap: structured_layout_semantic.scroll_gap,
            inspector_layout_scrollbar_visibility: structured_layout_semantic.scrollbar_visibility,
            inspector_layout_virtualization_item_extent: structured_layout_semantic
                .virtualization_item_extent,
            inspector_layout_virtualization_overscan: structured_layout_semantic
                .virtualization_overscan,
            inspector_layout_clip: structured_layout_semantic.clip,
            inspector_binding_items: binding_fields.items,
            inspector_binding_selected_index: binding_fields.selected_index,
            inspector_binding_id: binding_fields.binding_id,
            inspector_binding_event: binding_fields.binding_event,
            inspector_binding_event_items: binding_fields.binding_event_items,
            inspector_binding_event_selected_index: binding_fields.binding_event_selected_index,
            inspector_binding_route: binding_fields.binding_route,
            inspector_binding_route_target: binding_fields.binding_route_target,
            inspector_binding_action_target: binding_fields.binding_action_target,
            inspector_binding_route_suggestion_items: binding_fields.binding_route_suggestion_items,
            inspector_binding_action_suggestion_items: binding_fields
                .binding_action_suggestion_items,
            inspector_binding_action_kind_items: binding_fields.binding_action_kind_items,
            inspector_binding_action_kind_selected_index: binding_fields
                .binding_action_kind_selected_index,
            inspector_binding_payload_items: binding_fields.binding_payload_items,
            inspector_binding_payload_selected_index: binding_fields.binding_payload_selected_index,
            inspector_binding_payload_key: binding_fields.binding_payload_key,
            inspector_binding_payload_value: binding_fields.binding_payload_value,
            inspector_binding_payload_suggestion_items: binding_fields
                .binding_payload_suggestion_items,
            inspector_binding_schema_items: binding_fields.binding_schema_items,
            inspector_can_edit_binding: self.diagnostics.is_empty() && binding_fields.can_edit,
            inspector_can_delete_binding: self.diagnostics.is_empty() && binding_fields.can_delete,
            inspector_widget_kind: inspector_fields.widget_kind,
            inspector_widget_label: inspector_fields.widget_label,
            inspector_control_id: inspector_fields.control_id,
            inspector_text_prop: inspector_fields.text_prop,
            inspector_can_edit_control_id: inspector_fields.can_edit_control_id,
            inspector_can_edit_text_prop: inspector_fields.can_edit_text_prop,
            inspector_promote_asset_id: promote_draft
                .as_ref()
                .map(|draft| draft.asset_id.clone())
                .unwrap_or_default(),
            inspector_promote_component_name: promote_draft
                .as_ref()
                .map(|draft| draft.component_name.clone())
                .unwrap_or_default(),
            inspector_promote_document_id: promote_draft
                .as_ref()
                .map(|draft| draft.document_id.clone())
                .unwrap_or_default(),
            inspector_can_edit_promote_draft: can_promote_to_external_widget,
            palette_items: palette_entries
                .into_iter()
                .map(|entry| entry.label)
                .collect(),
            hierarchy_items: build_hierarchy_items(
                &self.last_valid_document,
                reflection.selection.primary_node_id.as_deref(),
            ),
            inspector_items: build_inspector_items(&reflection),
            stylesheet_items: build_stylesheet_items(&reflection.style_inspector, selector_hint),
            preview_items: preview_projection.items,
        }
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

    pub fn select_hierarchy_index(
        &mut self,
        index: usize,
    ) -> Result<(), UiAssetEditorSessionError> {
        let node_id = hierarchy_node_ids(&self.last_valid_document)
            .into_iter()
            .nth(index)
            .ok_or(UiAssetEditorSessionError::InvalidSelectionIndex { index })?;
        self.select_node_id(&node_id);
        self.set_source_cursor_to_selected_node_start();
        Ok(())
    }

    pub fn select_preview_index(&mut self, index: usize) -> Result<(), UiAssetEditorSessionError> {
        let Some(preview_host) = self.preview_host.as_ref() else {
            return Err(UiAssetEditorSessionError::InvalidPreviewIndex { index });
        };
        let Some(node_id) =
            preview_node_id_for_index(&self.last_valid_document, preview_host, index)
        else {
            return Err(UiAssetEditorSessionError::InvalidPreviewIndex { index });
        };
        self.select_node_id(&node_id);
        self.set_source_cursor_to_selected_node_start();
        Ok(())
    }

    pub fn select_source_outline_index(
        &mut self,
        index: usize,
    ) -> Result<(), UiAssetEditorSessionError> {
        let node_id = build_source_outline(&self.last_valid_document, self.source_buffer.text())
            .into_iter()
            .nth(index)
            .map(|entry| entry.node_id)
            .ok_or(UiAssetEditorSessionError::InvalidSelectionIndex { index })?;
        self.select_node_id(&node_id);
        self.set_source_cursor_to_selected_node_start();
        Ok(())
    }

    pub fn select_source_line(&mut self, line: usize) -> Result<(), UiAssetEditorSessionError> {
        let node_id = source_outline_node_id_for_line(
            &self.last_valid_document,
            self.source_buffer.text(),
            line,
        )
        .ok_or(UiAssetEditorSessionError::InvalidSelectionIndex { index: line })?;
        let line_offset = source_outline_entry_for_node(self.source_buffer.text(), &node_id)
            .map(|entry| line.saturating_sub(entry.line as usize))
            .unwrap_or_default();
        self.select_node_id(&node_id);
        self.set_source_cursor_for_selected_node_line(
            line_offset,
            source_byte_offset_for_line(self.source_buffer.text(), line),
        );
        Ok(())
    }

    pub fn select_source_byte_offset(
        &mut self,
        byte_offset: usize,
    ) -> Result<bool, UiAssetEditorSessionError> {
        let clamped = byte_offset.min(self.source_buffer.text().len());
        let line = source_line_for_byte_offset(self.source_buffer.text(), clamped);
        let Some(node_id) = source_outline_node_id_for_line(
            &self.last_valid_document,
            self.source_buffer.text(),
            line,
        ) else {
            return Ok(false);
        };
        let line_offset = source_outline_entry_for_node(self.source_buffer.text(), &node_id)
            .map(|entry| line.saturating_sub(entry.line as usize))
            .unwrap_or_default();
        let selection_changed = self.selection.primary_node_id.as_deref() != Some(node_id.as_str());
        let cursor_changed = self.source_cursor_byte_offset != clamped
            || self
                .source_cursor_anchor
                .as_ref()
                .map(|anchor| {
                    anchor.node_id.as_str() != node_id.as_str() || anchor.line_offset != line_offset
                })
                .unwrap_or(true);
        if !selection_changed && !cursor_changed {
            return Ok(false);
        }
        if selection_changed {
            self.select_node_id(&node_id);
        }
        self.set_source_cursor_for_selected_node_line(line_offset, clamped);
        Ok(true)
    }

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

    pub fn select_theme_source(&mut self, index: usize) -> Result<bool, UiAssetEditorSessionError> {
        let Some(key) = select_theme_source_key(
            &self.last_valid_document,
            &self.compiler_imports.styles,
            index,
        ) else {
            return Err(UiAssetEditorSessionError::InvalidSelectionIndex { index });
        };
        let changed = self.selected_theme_source_key.as_deref() != Some(key.as_str());
        self.selected_theme_source_key = Some(key);
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

    pub fn select_palette_index(
        &mut self,
        index: usize,
    ) -> Result<bool, UiAssetEditorSessionError> {
        let palette_entries =
            build_palette_entries(&self.last_valid_document, &self.compiler_imports.widgets);
        if index >= palette_entries.len() {
            return Err(UiAssetEditorSessionError::InvalidPaletteIndex { index });
        }
        let changed = self.selected_palette_index != Some(index);
        self.selected_palette_index = Some(index);
        if changed {
            self.clear_palette_drag_state();
        }
        Ok(changed)
    }

    pub fn insert_selected_palette_item_as_child(
        &mut self,
    ) -> Result<bool, UiAssetEditorSessionError> {
        self.insert_selected_palette_item(PaletteInsertMode::Child)
    }

    pub fn insert_selected_palette_item_after_selection(
        &mut self,
    ) -> Result<bool, UiAssetEditorSessionError> {
        self.insert_selected_palette_item(PaletteInsertMode::After)
    }

    pub fn update_palette_drag_target(
        &mut self,
        surface_x: f32,
        surface_y: f32,
    ) -> Result<bool, UiAssetEditorSessionError> {
        let next = reconcile_palette_target_chooser(
            self.palette_target_chooser.as_ref(),
            self.resolve_palette_drag_target(surface_x, surface_y),
        );
        if self.palette_target_chooser == next {
            return Ok(false);
        }
        self.palette_target_chooser = next;
        Ok(true)
    }

    pub fn clear_palette_drag_target(&mut self) -> bool {
        let changed = self.palette_target_chooser.is_some();
        self.clear_palette_drag_state();
        changed
    }

    pub fn cycle_palette_drag_target_candidate_next(
        &mut self,
    ) -> Result<bool, UiAssetEditorSessionError> {
        self.cycle_palette_drag_target_candidate(1)
    }

    pub fn cycle_palette_drag_target_candidate_previous(
        &mut self,
    ) -> Result<bool, UiAssetEditorSessionError> {
        self.cycle_palette_drag_target_candidate(-1)
    }

    fn cycle_palette_drag_target_candidate(
        &mut self,
        direction: isize,
    ) -> Result<bool, UiAssetEditorSessionError> {
        let Some(chooser) = self.palette_target_chooser.as_mut() else {
            return Ok(false);
        };
        let resolution = chooser.resolution_mut();
        if resolution.candidates.len() <= 1 {
            return Ok(false);
        }

        let candidate_count = resolution.candidates.len() as isize;
        let current = resolution.selected_index as isize;
        let next = (current + direction).rem_euclid(candidate_count) as usize;
        if next == resolution.selected_index {
            return Ok(false);
        }
        resolution.selected_index = next;
        chooser.set_manual_selection(true);
        Ok(true)
    }

    pub fn select_palette_target_candidate(
        &mut self,
        index: usize,
    ) -> Result<bool, UiAssetEditorSessionError> {
        let Some(chooser) = self.palette_target_chooser.as_mut() else {
            return Ok(false);
        };
        if index >= chooser.resolution().candidates.len() {
            return Err(UiAssetEditorSessionError::InvalidSelectionIndex { index });
        }
        Ok(chooser.select_candidate(index))
    }

    fn selected_insert_target_node_id(&self) -> Option<&str> {
        self.selection.primary_node_id.as_deref().or_else(|| {
            self.last_valid_document
                .root
                .as_ref()
                .map(|root| root.node.as_str())
        })
    }

    pub fn drop_selected_palette_item_at_palette_drag_target(
        &mut self,
    ) -> Result<bool, UiAssetEditorSessionError> {
        if let Some(chooser) = self.palette_target_chooser.as_mut() {
            if chooser.arm_sticky() {
                return Ok(true);
            }
        }
        self.confirm_palette_target_choice()
    }

    pub fn confirm_palette_target_choice(&mut self) -> Result<bool, UiAssetEditorSessionError> {
        let Some(target) = self.selected_palette_drag_target().cloned() else {
            return Ok(false);
        };
        let changed = self.insert_selected_palette_item_with_plan(&target.plan)?;
        self.clear_palette_drag_state();
        Ok(changed)
    }

    pub fn cancel_palette_target_choice(&mut self) -> Result<bool, UiAssetEditorSessionError> {
        let changed = self.palette_target_chooser.is_some();
        self.clear_palette_drag_state();
        Ok(changed)
    }

    fn resolve_palette_drag_target(
        &self,
        surface_x: f32,
        surface_y: f32,
    ) -> Option<UiAssetPaletteDragResolution> {
        if !self.diagnostics.is_empty() {
            return None;
        }
        let Some(preview_host) = self.preview_host.as_ref() else {
            return None;
        };
        let Some(selected_palette_index) = self.selected_palette_index else {
            return None;
        };
        let palette_entries =
            build_palette_entries(&self.last_valid_document, &self.compiler_imports.widgets);
        let entry = palette_entries.get(selected_palette_index)?;
        let projection = build_preview_projection(
            &self.last_valid_document,
            Some(preview_host),
            &self.selection,
        );
        resolve_palette_drag_target_for_preview(
            &self.last_valid_document,
            entry,
            &self.compiler_imports.widgets,
            &projection,
            surface_x,
            surface_y,
        )
    }

    fn selected_palette_drag_target(&self) -> Option<&UiAssetPaletteDragTarget> {
        self.palette_target_chooser
            .as_ref()
            .and_then(UiAssetPaletteTargetChooser::selected_target)
    }

    fn clear_palette_drag_state(&mut self) {
        self.palette_target_chooser = None;
    }

    pub fn selected_reference_asset_id(&self) -> Option<String> {
        let node_id = self.selection.primary_node_id.as_deref()?;
        let node = self.last_valid_document.nodes.get(node_id)?;
        if node.kind != zircon_runtime::ui::template::UiNodeDefinitionKind::Reference {
            return None;
        }
        node.component_ref
            .as_deref()
            .map(reference_asset_id)
            .map(str::to_string)
    }

    pub fn selected_theme_source_asset_id(&self) -> Option<String> {
        let selected_key = reconcile_selected_theme_source_key(
            &self.last_valid_document,
            &self.compiler_imports.styles,
            self.selected_theme_source_key.as_deref(),
        )?;
        if selected_key == "local" || !self.compiler_imports.styles.contains_key(&selected_key) {
            return None;
        }
        Some(reference_asset_id(&selected_key).to_string())
    }

    pub fn detach_selected_theme_source_to_local(
        &mut self,
    ) -> Result<bool, UiAssetEditorSessionError> {
        self.ensure_editable_source()?;
        let Some(selected_key) = reconcile_selected_theme_source_key(
            &self.last_valid_document,
            &self.compiler_imports.styles,
            self.selected_theme_source_key.as_deref(),
        ) else {
            return Ok(false);
        };
        if selected_key == "local" {
            return Ok(false);
        }
        let Some(imported_style_document) =
            self.compiler_imports.styles.get(&selected_key).cloned()
        else {
            return Ok(false);
        };

        let mut document = self.last_valid_document.clone();
        if !detach_imported_theme_to_local_theme_layer(
            &mut document,
            &selected_key,
            &imported_style_document,
        ) {
            return Ok(false);
        }

        let replay = theme_document_replay_bundle(&self.last_valid_document, &document);
        self.apply_document_edit_with_label_and_replay(document, "Detach Imported Theme", replay)?;
        Ok(true)
    }

    pub fn clone_selected_theme_source_to_local(
        &mut self,
    ) -> Result<bool, UiAssetEditorSessionError> {
        self.ensure_editable_source()?;
        let Some(selected_key) = reconcile_selected_theme_source_key(
            &self.last_valid_document,
            &self.compiler_imports.styles,
            self.selected_theme_source_key.as_deref(),
        ) else {
            return Ok(false);
        };
        if selected_key == "local" {
            return Ok(false);
        }
        let Some(imported_style_document) =
            self.compiler_imports.styles.get(&selected_key).cloned()
        else {
            return Ok(false);
        };

        let mut document = self.last_valid_document.clone();
        if !clone_imported_theme_to_local_theme_layer(
            &mut document,
            &selected_key,
            &imported_style_document,
        ) {
            return Ok(false);
        }

        let replay = theme_document_replay_bundle(&self.last_valid_document, &document);
        self.apply_command_with_effects_and_theme_source(
            UiAssetEditorCommand::tree_edit(
                UiAssetEditorTreeEditKind::DocumentEdit,
                "Clone Imported Theme",
                serialize_document(&document)?,
            )
            .with_document_replay(replay),
            UiAssetEditorUndoExternalEffects::default(),
            Some("local".to_string()),
        )?;
        Ok(true)
    }

    pub(crate) fn theme_rule_helper_action(
        &self,
        index: usize,
    ) -> Option<UiAssetThemeRuleHelperAction> {
        theme_rule_helper_actions(
            &self.last_valid_document,
            &self.compiler_imports.styles,
            self.selected_theme_source_key.as_deref(),
        )
        .get(index)
        .cloned()
    }

    pub fn apply_theme_rule_helper_item(
        &mut self,
        index: usize,
    ) -> Result<bool, UiAssetEditorSessionError> {
        let Some(action) = self.theme_rule_helper_action(index) else {
            return Err(UiAssetEditorSessionError::InvalidSelectionIndex { index });
        };
        match action {
            UiAssetThemeRuleHelperAction::PromoteLocalTheme => {
                let Some(draft) = self.selected_promote_theme_draft() else {
                    return Ok(false);
                };
                self.promote_local_theme_to_external_style_asset(
                    &draft.asset_id,
                    &draft.document_id,
                    &draft.display_name,
                )
                .map(|changed| changed.is_some())
            }
            UiAssetThemeRuleHelperAction::AdoptActiveCascadeTokens { .. } => {
                self.adopt_active_cascade_tokens()
            }
            UiAssetThemeRuleHelperAction::AdoptActiveCascadeRules { .. } => {
                self.adopt_active_cascade_rules()
            }
            UiAssetThemeRuleHelperAction::AdoptActiveCascadeChanges { .. } => {
                self.adopt_all_active_cascade_changes()
            }
            UiAssetThemeRuleHelperAction::AdoptActiveCascadeToken { token_name, .. } => {
                self.adopt_active_cascade_token(&token_name)
            }
            UiAssetThemeRuleHelperAction::AdoptActiveCascadeRule {
                stylesheet_id,
                selector,
                ..
            } => self.adopt_active_cascade_rule(&stylesheet_id, &selector),
            UiAssetThemeRuleHelperAction::DetachImportedThemeToLocal { .. } => {
                self.detach_selected_theme_source_to_local()
            }
            UiAssetThemeRuleHelperAction::CloneImportedThemeToLocal { .. } => {
                self.clone_selected_theme_source_to_local()
            }
            UiAssetThemeRuleHelperAction::AdoptComparedImportedDiffs { reference, .. } => {
                self.adopt_imported_theme_compare_diffs(&reference)
            }
            UiAssetThemeRuleHelperAction::PruneSharedComparedEntries { reference, .. } => {
                self.prune_imported_theme_compare_duplicates(&reference)
            }
            UiAssetThemeRuleHelperAction::AdoptAllImportedTokens { reference, .. } => {
                self.adopt_imported_theme_tokens(&reference)
            }
            UiAssetThemeRuleHelperAction::AdoptAllImportedRules { reference, .. } => {
                self.adopt_imported_theme_rules(&reference)
            }
            UiAssetThemeRuleHelperAction::AdoptAllImportedChanges { reference, .. } => {
                self.adopt_all_imported_theme_changes(&reference)
            }
            UiAssetThemeRuleHelperAction::AdoptImportedToken {
                reference,
                token_name,
                ..
            } => self.adopt_imported_theme_token(&reference, &token_name),
            UiAssetThemeRuleHelperAction::AdoptImportedRule {
                reference,
                stylesheet_id,
                selector,
            } => self.adopt_imported_theme_rule(&reference, &stylesheet_id, &selector),
            UiAssetThemeRuleHelperAction::ApplyAllThemeRefactors { .. } => {
                self.apply_all_theme_refactors()
            }
            UiAssetThemeRuleHelperAction::PruneDuplicateLocalOverrides => {
                self.prune_duplicate_local_theme_overrides()
            }
        }
    }

    pub(crate) fn theme_refactor_action(&self, index: usize) -> Option<UiAssetThemeRefactorAction> {
        theme_refactor_actions(&self.last_valid_document, &self.compiler_imports.styles)
            .get(index)
            .cloned()
    }

    pub fn apply_theme_refactor_item(
        &mut self,
        index: usize,
    ) -> Result<bool, UiAssetEditorSessionError> {
        self.ensure_editable_source()?;
        let Some(action) = self.theme_refactor_action(index) else {
            return Err(UiAssetEditorSessionError::InvalidSelectionIndex { index });
        };
        let mut document = self.last_valid_document.clone();
        if !apply_theme_refactor_action(&mut document, &action) {
            return Ok(false);
        }
        let replay = theme_document_replay_bundle(&self.last_valid_document, &document);
        self.apply_document_edit_with_label_and_replay(document, "Apply Theme Refactor", replay)?;
        Ok(true)
    }

    pub fn apply_all_theme_refactors(&mut self) -> Result<bool, UiAssetEditorSessionError> {
        self.ensure_editable_source()?;
        let actions =
            theme_refactor_actions(&self.last_valid_document, &self.compiler_imports.styles);
        if actions.is_empty() {
            return Ok(false);
        }
        let mut document = self.last_valid_document.clone();
        let mut changed = false;
        for action in &actions {
            changed |= apply_theme_refactor_action(&mut document, action);
        }
        if !changed {
            return Ok(false);
        }
        let replay = theme_document_replay_bundle(&self.last_valid_document, &document);
        self.apply_document_edit_with_label_and_replay(
            document,
            "Apply All Theme Refactors",
            replay,
        )?;
        Ok(true)
    }

    fn adopt_imported_theme_token(
        &mut self,
        reference: &str,
        token_name: &str,
    ) -> Result<bool, UiAssetEditorSessionError> {
        self.ensure_editable_source()?;
        let mut document = self.last_valid_document.clone();
        if !adopt_imported_theme_token(
            &mut document,
            &self.compiler_imports.styles,
            reference,
            token_name,
        ) {
            return Ok(false);
        }
        let replay = theme_document_replay_bundle(&self.last_valid_document, &document);
        self.apply_document_edit_with_label_and_replay(
            document,
            "Adopt Imported Theme Token",
            replay,
        )?;
        Ok(true)
    }

    fn adopt_active_cascade_token(
        &mut self,
        token_name: &str,
    ) -> Result<bool, UiAssetEditorSessionError> {
        self.ensure_editable_source()?;
        let mut document = self.last_valid_document.clone();
        if !adopt_active_cascade_token(&mut document, &self.compiler_imports.styles, token_name) {
            return Ok(false);
        }
        let replay = theme_document_replay_bundle(&self.last_valid_document, &document);
        self.apply_document_edit_with_label_and_replay(
            document,
            "Adopt Active Cascade Theme Token",
            replay,
        )?;
        Ok(true)
    }

    fn adopt_imported_theme_tokens(
        &mut self,
        reference: &str,
    ) -> Result<bool, UiAssetEditorSessionError> {
        self.ensure_editable_source()?;
        let mut document = self.last_valid_document.clone();
        if adopt_imported_theme_tokens(&mut document, &self.compiler_imports.styles, reference) == 0
        {
            return Ok(false);
        }
        let replay = theme_document_replay_bundle(&self.last_valid_document, &document);
        self.apply_document_edit_with_label_and_replay(
            document,
            "Adopt Imported Theme Tokens",
            replay,
        )?;
        Ok(true)
    }

    fn adopt_imported_theme_compare_diffs(
        &mut self,
        reference: &str,
    ) -> Result<bool, UiAssetEditorSessionError> {
        self.ensure_editable_source()?;
        let mut document = self.last_valid_document.clone();
        if adopt_imported_theme_compare_diffs(
            &mut document,
            &self.compiler_imports.styles,
            reference,
        ) == 0
        {
            return Ok(false);
        }
        let replay = theme_document_replay_bundle(&self.last_valid_document, &document);
        self.apply_document_edit_with_label_and_replay(
            document,
            "Adopt Imported Theme Compare Diffs",
            replay,
        )?;
        Ok(true)
    }

    fn prune_imported_theme_compare_duplicates(
        &mut self,
        reference: &str,
    ) -> Result<bool, UiAssetEditorSessionError> {
        self.ensure_editable_source()?;
        let mut document = self.last_valid_document.clone();
        if prune_imported_theme_compare_duplicates(
            &mut document,
            &self.compiler_imports.styles,
            reference,
        ) == 0
        {
            return Ok(false);
        }
        let replay = theme_document_replay_bundle(&self.last_valid_document, &document);
        self.apply_document_edit_with_label_and_replay(
            document,
            "Prune Imported Theme Compare Duplicates",
            replay,
        )?;
        Ok(true)
    }

    fn adopt_active_cascade_tokens(&mut self) -> Result<bool, UiAssetEditorSessionError> {
        self.ensure_editable_source()?;
        let mut document = self.last_valid_document.clone();
        if adopt_active_cascade_tokens(&mut document, &self.compiler_imports.styles) == 0 {
            return Ok(false);
        }
        let replay = theme_document_replay_bundle(&self.last_valid_document, &document);
        self.apply_document_edit_with_label_and_replay(
            document,
            "Adopt Active Cascade Theme Tokens",
            replay,
        )?;
        Ok(true)
    }

    fn adopt_imported_theme_rule(
        &mut self,
        reference: &str,
        stylesheet_id: &str,
        selector: &str,
    ) -> Result<bool, UiAssetEditorSessionError> {
        self.ensure_editable_source()?;
        let mut document = self.last_valid_document.clone();
        if !adopt_imported_theme_rule(
            &mut document,
            &self.compiler_imports.styles,
            reference,
            stylesheet_id,
            selector,
        ) {
            return Ok(false);
        }
        let replay = theme_document_replay_bundle(&self.last_valid_document, &document);
        self.apply_document_edit_with_label_and_replay(
            document,
            "Adopt Imported Theme Rule",
            replay,
        )?;
        Ok(true)
    }

    fn adopt_active_cascade_rule(
        &mut self,
        stylesheet_id: &str,
        selector: &str,
    ) -> Result<bool, UiAssetEditorSessionError> {
        self.ensure_editable_source()?;
        let mut document = self.last_valid_document.clone();
        if !adopt_active_cascade_rule(
            &mut document,
            &self.compiler_imports.styles,
            stylesheet_id,
            selector,
        ) {
            return Ok(false);
        }
        let replay = theme_document_replay_bundle(&self.last_valid_document, &document);
        self.apply_document_edit_with_label_and_replay(
            document,
            "Adopt Active Cascade Theme Rule",
            replay,
        )?;
        Ok(true)
    }

    fn adopt_imported_theme_rules(
        &mut self,
        reference: &str,
    ) -> Result<bool, UiAssetEditorSessionError> {
        self.ensure_editable_source()?;
        let mut document = self.last_valid_document.clone();
        if adopt_imported_theme_rules(&mut document, &self.compiler_imports.styles, reference) == 0
        {
            return Ok(false);
        }
        let replay = theme_document_replay_bundle(&self.last_valid_document, &document);
        self.apply_document_edit_with_label_and_replay(
            document,
            "Adopt Imported Theme Rules",
            replay,
        )?;
        Ok(true)
    }

    fn adopt_active_cascade_rules(&mut self) -> Result<bool, UiAssetEditorSessionError> {
        self.ensure_editable_source()?;
        let mut document = self.last_valid_document.clone();
        if adopt_active_cascade_rules(&mut document, &self.compiler_imports.styles) == 0 {
            return Ok(false);
        }
        let replay = theme_document_replay_bundle(&self.last_valid_document, &document);
        self.apply_document_edit_with_label_and_replay(
            document,
            "Adopt Active Cascade Theme Rules",
            replay,
        )?;
        Ok(true)
    }

    fn adopt_all_imported_theme_changes(
        &mut self,
        reference: &str,
    ) -> Result<bool, UiAssetEditorSessionError> {
        self.ensure_editable_source()?;
        let mut document = self.last_valid_document.clone();
        if adopt_all_imported_theme_changes(&mut document, &self.compiler_imports.styles, reference)
            == 0
        {
            return Ok(false);
        }
        let replay = theme_document_replay_bundle(&self.last_valid_document, &document);
        self.apply_document_edit_with_label_and_replay(
            document,
            "Adopt Imported Theme Changes",
            replay,
        )?;
        Ok(true)
    }

    fn adopt_all_active_cascade_changes(&mut self) -> Result<bool, UiAssetEditorSessionError> {
        self.ensure_editable_source()?;
        let mut document = self.last_valid_document.clone();
        if adopt_all_active_cascade_changes(&mut document, &self.compiler_imports.styles) == 0 {
            return Ok(false);
        }
        let replay = theme_document_replay_bundle(&self.last_valid_document, &document);
        self.apply_document_edit_with_label_and_replay(
            document,
            "Adopt Active Cascade Theme Changes",
            replay,
        )?;
        Ok(true)
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
            super::tree_editing::UiAssetPaletteEntryKind::Reference { component_ref } => {
                component_ref.clone()
            }
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
            .nodes
            .get(&node_id)
            .and_then(|node| node.component.clone())
            .unwrap_or_default();
        let component_root_id = document
            .components
            .get(&component_name)
            .map(|component| component.root.clone())
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

    pub fn prune_duplicate_local_theme_overrides(
        &mut self,
    ) -> Result<bool, UiAssetEditorSessionError> {
        self.ensure_editable_source()?;
        let mut document = self.last_valid_document.clone();
        if !prune_duplicate_local_theme_overrides(&mut document, &self.compiler_imports.styles) {
            return Ok(false);
        }
        let replay = theme_document_replay_bundle(&self.last_valid_document, &document);
        self.apply_document_edit_with_label_and_replay(
            document,
            "Prune Duplicate Local Theme Overrides",
            replay,
        )?;
        Ok(true)
    }

    pub fn move_selected_node_up(&mut self) -> Result<bool, UiAssetEditorSessionError> {
        self.move_selected_node(UiTreeMoveDirection::Up)
    }

    pub fn move_selected_node_down(&mut self) -> Result<bool, UiAssetEditorSessionError> {
        self.move_selected_node(UiTreeMoveDirection::Down)
    }

    pub fn reparent_selected_node_into_previous(
        &mut self,
    ) -> Result<bool, UiAssetEditorSessionError> {
        self.reparent_selected_node(UiTreeReparentDirection::IntoPrevious)
    }

    pub fn reparent_selected_node_into_next(&mut self) -> Result<bool, UiAssetEditorSessionError> {
        self.reparent_selected_node(UiTreeReparentDirection::IntoNext)
    }

    pub fn reparent_selected_node_outdent(&mut self) -> Result<bool, UiAssetEditorSessionError> {
        self.reparent_selected_node(UiTreeReparentDirection::Outdent)
    }

    pub fn wrap_selected_node_with(
        &mut self,
        widget_type: &str,
    ) -> Result<bool, UiAssetEditorSessionError> {
        self.ensure_editable_source()?;
        let Some(node_id) = self.selection.primary_node_id.clone() else {
            return Ok(false);
        };
        let mut document = self.last_valid_document.clone();
        let Some(wrapper_id) = wrap_selected_node(&mut document, &self.selection, widget_type)
        else {
            return Ok(false);
        };
        let selection = selection_for_node(&document, &wrapper_id);
        self.apply_document_edit_with_tree_edit_and_selection(
            document.clone(),
            UiAssetEditorTreeEdit::WrapNode {
                node_id,
                wrapper_node_id: wrapper_id,
                wrapper_widget_type: widget_type.to_string(),
            },
            "Wrap Node",
            selection,
        )?;
        Ok(true)
    }

    pub fn unwrap_selected_node(&mut self) -> Result<bool, UiAssetEditorSessionError> {
        self.ensure_editable_source()?;
        let Some(wrapper_node_id) = self.selection.primary_node_id.clone() else {
            return Ok(false);
        };
        let mut document = self.last_valid_document.clone();
        let Some(child_id) = unwrap_selected_node(&mut document, &self.selection) else {
            return Ok(false);
        };
        let selection = selection_for_node(&document, &child_id);
        self.apply_document_edit_with_tree_edit_and_selection(
            document.clone(),
            UiAssetEditorTreeEdit::UnwrapNode {
                wrapper_node_id,
                child_node_id: child_id,
            },
            "Unwrap Node",
            selection,
        )?;
        Ok(true)
    }

    pub fn create_rule_from_selection(&mut self) -> Result<bool, UiAssetEditorSessionError> {
        self.ensure_editable_source()?;
        let Some(selector) = selected_node_selector(&self.last_valid_document, &self.selection)
        else {
            return Ok(false);
        };
        if self
            .last_valid_document
            .stylesheets
            .iter()
            .flat_map(|sheet| sheet.rules.iter())
            .any(|rule| rule.selector == selector)
        {
            return Ok(false);
        }

        let mut document = self.last_valid_document.clone();
        let stylesheet_index = if document.stylesheets.is_empty() {
            0
        } else {
            document.stylesheets.len() - 1
        };
        let rule = UiStyleRule {
            selector,
            set: UiStyleDeclarationBlock::default(),
        };
        let rule_index = editable_stylesheet(&mut document).rules.len();
        editable_stylesheet(&mut document).rules.push(rule.clone());
        self.selected_style_rule_index = local_style_rule_entries(&document).len().checked_sub(1);
        self.selected_style_rule_declaration_path = None;
        self.apply_document_edit_with_label_and_replay(
            document,
            "Create Stylesheet Rule",
            style_rule_insert_replay_bundle(
                &self.last_valid_document,
                stylesheet_index,
                rule_index,
                rule,
            ),
        )?;
        Ok(true)
    }

    pub fn extract_inline_overrides_to_rule(&mut self) -> Result<bool, UiAssetEditorSessionError> {
        self.ensure_editable_source()?;
        let Some(node_id) = self.selection.primary_node_id.as_deref() else {
            return Ok(false);
        };
        let Some(selector) = selected_node_selector(&self.last_valid_document, &self.selection)
        else {
            return Ok(false);
        };

        let mut document = self.last_valid_document.clone();
        let Some(node) = document.nodes.get_mut(node_id) else {
            return Ok(false);
        };
        if node.style_overrides.self_values.is_empty() && node.style_overrides.slot.is_empty() {
            return Ok(false);
        }

        let overrides = std::mem::take(&mut node.style_overrides);
        let stylesheet_index = if document.stylesheets.is_empty() {
            0
        } else {
            document.stylesheets.len() - 1
        };
        let rule = UiStyleRule {
            selector,
            set: overrides,
        };
        let rule_index = editable_stylesheet(&mut document).rules.len();
        editable_stylesheet(&mut document).rules.push(rule.clone());
        self.selected_style_rule_index = local_style_rule_entries(&document).len().checked_sub(1);
        self.selected_style_rule_declaration_path = None;
        self.apply_document_edit_with_label_and_replay(
            document,
            "Extract Inline Overrides",
            style_rule_insert_replay_bundle(
                &self.last_valid_document,
                stylesheet_index,
                rule_index,
                rule,
            ),
        )?;
        Ok(true)
    }

    pub fn add_class_to_selection(
        &mut self,
        class_name: impl AsRef<str>,
    ) -> Result<bool, UiAssetEditorSessionError> {
        self.ensure_editable_source()?;
        let Some(node_id) = self.selection.primary_node_id.as_deref() else {
            return Ok(false);
        };
        let Some(class_name) = normalized_class_name(class_name.as_ref()) else {
            return Ok(false);
        };

        let mut document = self.last_valid_document.clone();
        let Some(node) = document.nodes.get_mut(node_id) else {
            return Ok(false);
        };
        if node.classes.iter().any(|existing| existing == &class_name) {
            return Ok(false);
        }
        node.classes.push(class_name);
        self.apply_document_edit(document)?;
        Ok(true)
    }

    pub fn remove_class_from_selection(
        &mut self,
        class_name: impl AsRef<str>,
    ) -> Result<bool, UiAssetEditorSessionError> {
        self.ensure_editable_source()?;
        let Some(node_id) = self.selection.primary_node_id.as_deref() else {
            return Ok(false);
        };
        let Some(class_name) = normalized_class_name(class_name.as_ref()) else {
            return Ok(false);
        };

        let mut document = self.last_valid_document.clone();
        let Some(node) = document.nodes.get_mut(node_id) else {
            return Ok(false);
        };
        let before = node.classes.len();
        node.classes.retain(|existing| existing != &class_name);
        if before == node.classes.len() {
            return Ok(false);
        }
        self.apply_document_edit(document)?;
        Ok(true)
    }

    pub fn set_selected_widget_control_id(
        &mut self,
        control_id: impl AsRef<str>,
    ) -> Result<bool, UiAssetEditorSessionError> {
        self.ensure_editable_source()?;
        let mut document = self.last_valid_document.clone();
        if !set_selected_node_control_id(&mut document, &self.selection, control_id.as_ref()) {
            return Ok(false);
        }
        self.apply_document_edit(document)?;
        Ok(true)
    }

    pub fn set_selected_widget_text_property(
        &mut self,
        text: impl AsRef<str>,
    ) -> Result<bool, UiAssetEditorSessionError> {
        self.ensure_editable_source()?;
        let mut document = self.last_valid_document.clone();
        if !set_selected_node_text_property(&mut document, &self.selection, text.as_ref()) {
            return Ok(false);
        }
        self.apply_document_edit(document)?;
        Ok(true)
    }

    pub fn set_selected_slot_mount(
        &mut self,
        mount: impl AsRef<str>,
    ) -> Result<bool, UiAssetEditorSessionError> {
        self.ensure_editable_source()?;
        let mut document = self.last_valid_document.clone();
        if !set_selected_node_mount(&mut document, &self.selection, mount.as_ref()) {
            return Ok(false);
        }
        self.apply_document_edit(document)?;
        Ok(true)
    }

    pub fn set_selected_slot_padding(
        &mut self,
        literal: impl AsRef<str>,
    ) -> Result<bool, UiAssetEditorSessionError> {
        self.ensure_editable_source()?;
        let mut document = self.last_valid_document.clone();
        let literal = literal.as_ref();
        let changed = set_selected_node_slot_padding(&mut document, &self.selection, literal)
            .map_err(
                |field| UiAssetEditorSessionError::InvalidInspectorNumericLiteral {
                    field,
                    value: literal.to_string(),
                },
            )?;
        if !changed {
            return Ok(false);
        }
        self.apply_document_edit(document)?;
        Ok(true)
    }

    pub fn set_selected_slot_width_preferred(
        &mut self,
        literal: impl AsRef<str>,
    ) -> Result<bool, UiAssetEditorSessionError> {
        self.ensure_editable_source()?;
        let mut document = self.last_valid_document.clone();
        let literal = literal.as_ref();
        let changed =
            set_selected_node_slot_width_preferred(&mut document, &self.selection, literal)
                .map_err(
                    |field| UiAssetEditorSessionError::InvalidInspectorNumericLiteral {
                        field,
                        value: literal.to_string(),
                    },
                )?;
        if !changed {
            return Ok(false);
        }
        self.apply_document_edit(document)?;
        Ok(true)
    }

    pub fn set_selected_slot_height_preferred(
        &mut self,
        literal: impl AsRef<str>,
    ) -> Result<bool, UiAssetEditorSessionError> {
        self.ensure_editable_source()?;
        let mut document = self.last_valid_document.clone();
        let literal = literal.as_ref();
        let changed =
            set_selected_node_slot_height_preferred(&mut document, &self.selection, literal)
                .map_err(
                    |field| UiAssetEditorSessionError::InvalidInspectorNumericLiteral {
                        field,
                        value: literal.to_string(),
                    },
                )?;
        if !changed {
            return Ok(false);
        }
        self.apply_document_edit(document)?;
        Ok(true)
    }

    pub fn set_selected_layout_width_preferred(
        &mut self,
        literal: impl AsRef<str>,
    ) -> Result<bool, UiAssetEditorSessionError> {
        self.ensure_editable_source()?;
        let mut document = self.last_valid_document.clone();
        let literal = literal.as_ref();
        let changed =
            set_selected_node_layout_width_preferred(&mut document, &self.selection, literal)
                .map_err(
                    |field| UiAssetEditorSessionError::InvalidInspectorNumericLiteral {
                        field,
                        value: literal.to_string(),
                    },
                )?;
        if !changed {
            return Ok(false);
        }
        self.apply_document_edit(document)?;
        Ok(true)
    }

    pub fn set_selected_layout_height_preferred(
        &mut self,
        literal: impl AsRef<str>,
    ) -> Result<bool, UiAssetEditorSessionError> {
        self.ensure_editable_source()?;
        let mut document = self.last_valid_document.clone();
        let literal = literal.as_ref();
        let changed =
            set_selected_node_layout_height_preferred(&mut document, &self.selection, literal)
                .map_err(
                    |field| UiAssetEditorSessionError::InvalidInspectorNumericLiteral {
                        field,
                        value: literal.to_string(),
                    },
                )?;
        if !changed {
            return Ok(false);
        }
        self.apply_document_edit(document)?;
        Ok(true)
    }

    pub fn select_slot_semantic(
        &mut self,
        index: usize,
    ) -> Result<bool, UiAssetEditorSessionError> {
        let semantic_group = build_slot_semantic_group(&self.last_valid_document, &self.selection);
        let Some(entry) = semantic_group.entries.get(index) else {
            return Err(UiAssetEditorSessionError::InvalidSelectionIndex { index });
        };
        let changed = self.selected_slot_semantic_path.as_deref() != Some(entry.path.as_str());
        self.selected_slot_semantic_path = Some(entry.path.clone());
        Ok(changed)
    }

    pub fn set_selected_slot_semantic_value(
        &mut self,
        literal: impl AsRef<str>,
    ) -> Result<bool, UiAssetEditorSessionError> {
        self.ensure_editable_source()?;
        let Some(path) = self.selected_slot_semantic_path.clone() else {
            return Ok(false);
        };
        let mut document = self.last_valid_document.clone();
        if !set_selected_slot_semantic_value_field(
            &mut document,
            &self.selection,
            &path,
            literal.as_ref(),
        ) {
            return Ok(false);
        }
        self.apply_document_edit_with_label(document, "Slot Semantic Edit")?;
        Ok(true)
    }

    pub fn set_selected_slot_semantic_field(
        &mut self,
        path: impl AsRef<str>,
        literal: impl AsRef<str>,
    ) -> Result<bool, UiAssetEditorSessionError> {
        self.ensure_editable_source()?;
        let path = path.as_ref().trim();
        if path.is_empty() {
            return Ok(false);
        }
        let mut document = self.last_valid_document.clone();
        if !set_selected_slot_semantic_value_field(
            &mut document,
            &self.selection,
            path,
            literal.as_ref(),
        ) {
            return Ok(false);
        }
        self.selected_slot_semantic_path = Some(path.to_string());
        self.apply_document_edit_with_label(document, "Slot Semantic Edit")?;
        Ok(true)
    }

    pub fn delete_selected_slot_semantic(&mut self) -> Result<bool, UiAssetEditorSessionError> {
        self.ensure_editable_source()?;
        let Some(path) = self.selected_slot_semantic_path.clone() else {
            return Ok(false);
        };
        let mut document = self.last_valid_document.clone();
        if !delete_selected_slot_semantic_field(&mut document, &self.selection, &path) {
            return Ok(false);
        }
        self.apply_document_edit_with_label(document, "Slot Semantic Delete")?;
        Ok(true)
    }

    pub fn select_layout_semantic(
        &mut self,
        index: usize,
    ) -> Result<bool, UiAssetEditorSessionError> {
        let semantic_group =
            build_layout_semantic_group(&self.last_valid_document, &self.selection);
        let Some(entry) = semantic_group.entries.get(index) else {
            return Err(UiAssetEditorSessionError::InvalidSelectionIndex { index });
        };
        let changed = self.selected_layout_semantic_path.as_deref() != Some(entry.path.as_str());
        self.selected_layout_semantic_path = Some(entry.path.clone());
        Ok(changed)
    }

    pub fn set_selected_layout_semantic_value(
        &mut self,
        literal: impl AsRef<str>,
    ) -> Result<bool, UiAssetEditorSessionError> {
        self.ensure_editable_source()?;
        let Some(path) = self.selected_layout_semantic_path.clone() else {
            return Ok(false);
        };
        let mut document = self.last_valid_document.clone();
        if !set_selected_layout_semantic_value_field(
            &mut document,
            &self.selection,
            &path,
            literal.as_ref(),
        ) {
            return Ok(false);
        }
        self.apply_document_edit_with_label(document, "Layout Semantic Edit")?;
        Ok(true)
    }

    pub fn set_selected_layout_semantic_field(
        &mut self,
        path: impl AsRef<str>,
        literal: impl AsRef<str>,
    ) -> Result<bool, UiAssetEditorSessionError> {
        self.ensure_editable_source()?;
        let path = path.as_ref().trim();
        if path.is_empty() {
            return Ok(false);
        }
        let mut document = self.last_valid_document.clone();
        if !set_selected_layout_semantic_value_field(
            &mut document,
            &self.selection,
            path,
            literal.as_ref(),
        ) {
            return Ok(false);
        }
        self.selected_layout_semantic_path = Some(path.to_string());
        self.apply_document_edit_with_label(document, "Layout Semantic Edit")?;
        Ok(true)
    }

    pub fn delete_selected_layout_semantic(&mut self) -> Result<bool, UiAssetEditorSessionError> {
        self.ensure_editable_source()?;
        let Some(path) = self.selected_layout_semantic_path.clone() else {
            return Ok(false);
        };
        let mut document = self.last_valid_document.clone();
        if !delete_selected_layout_semantic_field(&mut document, &self.selection, &path) {
            return Ok(false);
        }
        self.apply_document_edit_with_label(document, "Layout Semantic Delete")?;
        Ok(true)
    }

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

    pub fn select_style_token(&mut self, index: usize) -> Result<bool, UiAssetEditorSessionError> {
        let entries = local_style_token_entries(&self.last_valid_document);
        let Some(entry) = entries.get(index) else {
            return Err(UiAssetEditorSessionError::InvalidStyleTokenIndex { index });
        };
        let changed = self.selected_style_token_name.as_deref() != Some(entry.name.as_str());
        self.selected_style_token_name = Some(entry.name.clone());
        Ok(changed)
    }

    pub fn upsert_style_token(
        &mut self,
        token_name: impl AsRef<str>,
        value_literal: impl AsRef<str>,
    ) -> Result<bool, UiAssetEditorSessionError> {
        self.ensure_editable_source()?;
        let Some(token_name) = normalized_token_name(token_name.as_ref()) else {
            return Ok(false);
        };
        let Some(value) = parse_token_literal(value_literal.as_ref()) else {
            return Ok(false);
        };

        let mut document = self.last_valid_document.clone();
        let current_value = document.tokens.get(&token_name).cloned();
        if let Some(selected_name) = self.selected_style_token_name.as_deref() {
            if selected_name != token_name {
                let _ = document.tokens.remove(selected_name);
            }
        }
        let _ = document.tokens.insert(token_name.clone(), value.clone());
        if self.selected_style_token_name.as_deref() == Some(token_name.as_str())
            && current_value.as_ref() == Some(&value)
        {
            return Ok(false);
        }
        if self.selected_style_token_name.is_none()
            && current_value.as_ref() == Some(&value)
            && self.last_valid_document.tokens.contains_key(&token_name)
        {
            self.selected_style_token_name = Some(token_name);
            return Ok(true);
        }
        self.selected_style_token_name = Some(token_name);
        let replay = theme_document_replay_bundle(&self.last_valid_document, &document);
        self.apply_document_edit_with_label_and_replay(document, "Upsert Style Token", replay)?;
        Ok(true)
    }

    pub fn delete_selected_style_token(&mut self) -> Result<bool, UiAssetEditorSessionError> {
        self.ensure_editable_source()?;
        let Some(selected_name) = self.selected_style_token_name.clone() else {
            return Ok(false);
        };
        let current_entries = local_style_token_entries(&self.last_valid_document);
        let Some(current_index) = current_entries
            .iter()
            .position(|entry| entry.name.as_str() == selected_name)
        else {
            self.selected_style_token_name = None;
            return Ok(false);
        };

        let mut document = self.last_valid_document.clone();
        if document.tokens.remove(&selected_name).is_none() {
            self.selected_style_token_name = None;
            return Ok(false);
        }

        let remaining_entries = local_style_token_entries(&document);
        self.selected_style_token_name = if remaining_entries.is_empty() {
            None
        } else {
            Some(
                remaining_entries[current_index.min(remaining_entries.len() - 1)]
                    .name
                    .clone(),
            )
        };
        let replay = theme_document_replay_bundle(&self.last_valid_document, &document);
        self.apply_document_edit_with_label_and_replay(document, "Delete Style Token", replay)?;
        Ok(true)
    }

    pub fn select_stylesheet_rule(
        &mut self,
        index: usize,
    ) -> Result<bool, UiAssetEditorSessionError> {
        if local_style_rule_entries(&self.last_valid_document)
            .get(index)
            .is_none()
        {
            return Err(UiAssetEditorSessionError::InvalidStyleRuleIndex { index });
        }
        let changed = self.selected_style_rule_index != Some(index);
        self.selected_style_rule_index = Some(index);
        self.selected_style_rule_declaration_path = None;
        Ok(changed)
    }

    pub fn move_selected_stylesheet_rule_up(&mut self) -> Result<bool, UiAssetEditorSessionError> {
        self.move_selected_stylesheet_rule(-1)
    }

    pub fn move_selected_stylesheet_rule_down(
        &mut self,
    ) -> Result<bool, UiAssetEditorSessionError> {
        self.move_selected_stylesheet_rule(1)
    }

    pub fn select_matched_style_rule(
        &mut self,
        index: usize,
    ) -> Result<bool, UiAssetEditorSessionError> {
        let entries = matched_style_rule_entries_for_selection(
            &self.last_valid_document,
            &self.selection,
            &self.compiler_imports,
            &self.style_inspector.active_pseudo_states,
        );
        if entries.get(index).is_none() {
            return Err(UiAssetEditorSessionError::InvalidMatchedStyleRuleIndex { index });
        }
        let changed = self.selected_matched_style_rule_index != Some(index);
        self.selected_matched_style_rule_index = Some(index);
        Ok(changed)
    }

    pub fn select_stylesheet_rule_declaration(
        &mut self,
        index: usize,
    ) -> Result<bool, UiAssetEditorSessionError> {
        let entries = selected_style_rule_declaration_entries(
            &self.last_valid_document,
            self.selected_style_rule_index,
        );
        let Some(entry) = entries.get(index) else {
            return Err(UiAssetEditorSessionError::InvalidStyleRuleDeclarationIndex { index });
        };
        let changed =
            self.selected_style_rule_declaration_path.as_deref() != Some(entry.path.as_str());
        self.selected_style_rule_declaration_path = Some(entry.path.clone());
        Ok(changed)
    }

    pub fn upsert_selected_stylesheet_rule_declaration(
        &mut self,
        path: impl AsRef<str>,
        value_literal: impl AsRef<str>,
    ) -> Result<bool, UiAssetEditorSessionError> {
        self.ensure_editable_source()?;
        let Some(rule_index) = self.selected_style_rule_index else {
            return Ok(false);
        };
        let Some(path) = UiStyleRuleDeclarationPath::parse(path.as_ref()) else {
            return Err(UiAssetEditorSessionError::InvalidStyleDeclarationPath {
                path: path.as_ref().trim().to_string(),
            });
        };
        let Some(value) = parse_declaration_literal(value_literal.as_ref()) else {
            return Ok(false);
        };

        let mut document = self.last_valid_document.clone();
        let Some(entry) = local_style_rule_entries(&document).get(rule_index).cloned() else {
            return Err(UiAssetEditorSessionError::InvalidStyleRuleIndex { index: rule_index });
        };
        let rule = &mut document.stylesheets[entry.stylesheet_index].rules[entry.rule_index];
        let next_path = path.display();
        let next_literal = value.to_string();
        let existing_literal = declaration_entries(&rule.set)
            .into_iter()
            .find(|entry| entry.path == next_path)
            .map(|entry| entry.literal);
        if self.selected_style_rule_declaration_path.as_deref() == Some(next_path.as_str())
            && existing_literal.as_deref() == Some(next_literal.as_str())
        {
            return Ok(false);
        }
        if self.selected_style_rule_declaration_path.is_none()
            && existing_literal.as_deref() == Some(next_literal.as_str())
        {
            self.selected_style_rule_declaration_path = Some(next_path);
            return Ok(true);
        }

        if let Some(selected_path) = self.selected_style_rule_declaration_path.as_deref() {
            if selected_path != next_path {
                let selected_path =
                    UiStyleRuleDeclarationPath::parse(selected_path).expect("selected path");
                let _ = remove_declaration(&mut rule.set, &selected_path);
            }
        }
        set_declaration(&mut rule.set, &path, value);
        self.selected_style_rule_declaration_path = Some(next_path);
        self.apply_document_edit(document)?;
        Ok(true)
    }

    pub fn delete_selected_stylesheet_rule_declaration(
        &mut self,
    ) -> Result<bool, UiAssetEditorSessionError> {
        self.ensure_editable_source()?;
        let Some(rule_index) = self.selected_style_rule_index else {
            self.selected_style_rule_declaration_path = None;
            return Ok(false);
        };
        let Some(selected_path) = self.selected_style_rule_declaration_path.clone() else {
            return Ok(false);
        };
        let current_entries =
            selected_style_rule_declaration_entries(&self.last_valid_document, Some(rule_index));
        let Some(current_index) = current_entries
            .iter()
            .position(|entry| entry.path.as_str() == selected_path)
        else {
            self.selected_style_rule_declaration_path = None;
            return Ok(false);
        };

        let mut document = self.last_valid_document.clone();
        let Some(entry) = local_style_rule_entries(&document).get(rule_index).cloned() else {
            return Err(UiAssetEditorSessionError::InvalidStyleRuleIndex { index: rule_index });
        };
        let rule = &mut document.stylesheets[entry.stylesheet_index].rules[entry.rule_index];
        let Some(path) = UiStyleRuleDeclarationPath::parse(&selected_path) else {
            self.selected_style_rule_declaration_path = None;
            return Ok(false);
        };
        if !remove_declaration(&mut rule.set, &path) {
            self.selected_style_rule_declaration_path = None;
            return Ok(false);
        }

        let remaining_entries = declaration_entries(&rule.set);
        self.selected_style_rule_declaration_path = if remaining_entries.is_empty() {
            None
        } else {
            Some(
                remaining_entries[current_index.min(remaining_entries.len() - 1)]
                    .path
                    .clone(),
            )
        };
        self.apply_document_edit(document)?;
        Ok(true)
    }

    pub fn rename_selected_stylesheet_rule(
        &mut self,
        selector: impl AsRef<str>,
    ) -> Result<bool, UiAssetEditorSessionError> {
        self.ensure_editable_source()?;
        let Some(index) = self.selected_style_rule_index else {
            return Ok(false);
        };
        let selector = normalized_selector(selector.as_ref())?;
        let mut document = self.last_valid_document.clone();
        let Some(entry) = local_style_rule_entries(&document).get(index).cloned() else {
            return Err(UiAssetEditorSessionError::InvalidStyleRuleIndex { index });
        };
        let rule = &mut document.stylesheets[entry.stylesheet_index].rules[entry.rule_index];
        if rule.selector == selector {
            return Ok(false);
        }
        rule.selector = selector;
        self.selected_style_rule_index = Some(index);
        self.apply_document_edit(document)?;
        Ok(true)
    }

    pub fn delete_selected_stylesheet_rule(&mut self) -> Result<bool, UiAssetEditorSessionError> {
        self.ensure_editable_source()?;
        let Some(index) = self.selected_style_rule_index else {
            return Ok(false);
        };
        let mut document = self.last_valid_document.clone();
        let Some(entry) = local_style_rule_entries(&document).get(index).cloned() else {
            return Err(UiAssetEditorSessionError::InvalidStyleRuleIndex { index });
        };
        let rules = &mut document.stylesheets[entry.stylesheet_index].rules;
        if entry.rule_index >= rules.len() {
            return Err(UiAssetEditorSessionError::InvalidStyleRuleIndex { index });
        }
        let removed_rule = rules.remove(entry.rule_index);
        let remaining = local_style_rule_entries(&document).len();
        self.selected_style_rule_index = (remaining > 0).then(|| index.min(remaining - 1));
        self.selected_style_rule_declaration_path = None;
        self.apply_document_edit_with_label_and_replay(
            document,
            "Delete Stylesheet Rule",
            style_rule_remove_replay_bundle(entry.stylesheet_index, entry.rule_index, removed_rule),
        )?;
        Ok(true)
    }

    fn move_selected_stylesheet_rule(
        &mut self,
        delta: isize,
    ) -> Result<bool, UiAssetEditorSessionError> {
        self.ensure_editable_source()?;
        let Some(index) = self.selected_style_rule_index else {
            return Ok(false);
        };
        let mut document = self.last_valid_document.clone();
        let Some(entry) = local_style_rule_entries(&document).get(index).cloned() else {
            return Err(UiAssetEditorSessionError::InvalidStyleRuleIndex { index });
        };
        let rules = &mut document.stylesheets[entry.stylesheet_index].rules;
        let next_rule_index = entry.rule_index as isize + delta;
        if next_rule_index < 0 || next_rule_index >= rules.len() as isize {
            return Ok(false);
        }
        let from_index = entry.rule_index;
        let to_index = next_rule_index as usize;
        rules.swap(entry.rule_index, next_rule_index as usize);
        self.selected_style_rule_index = Some((index as isize + delta) as usize);
        self.selected_style_rule_declaration_path = None;
        self.apply_document_edit_with_label_and_replay(
            document,
            if delta < 0 {
                "Move Stylesheet Rule Up"
            } else {
                "Move Stylesheet Rule Down"
            },
            style_rule_move_replay_bundle(entry.stylesheet_index, from_index, to_index),
        )?;
        Ok(true)
    }

    pub fn toggle_pseudo_state_preview(
        &mut self,
        state: impl AsRef<str>,
    ) -> Result<bool, UiAssetEditorSessionError> {
        let state = state.as_ref();
        if !SUPPORTED_PSEUDO_STATES
            .iter()
            .any(|candidate| candidate == &state)
        {
            return Ok(false);
        }
        if self.selection.primary_node_id.is_none() {
            return Ok(false);
        }

        let mut active_states = self.style_inspector.active_pseudo_states.clone();
        if let Some(index) = active_states
            .iter()
            .position(|candidate| candidate == state)
        {
            let _ = active_states.remove(index);
        } else {
            active_states.push(state.to_string());
        }
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
        Ok(true)
    }

    pub fn apply_command(
        &mut self,
        command: UiAssetEditorCommand,
    ) -> Result<(), UiAssetEditorSessionError> {
        self.apply_command_with_effects(command, UiAssetEditorUndoExternalEffects::default())
    }

    fn apply_command_with_effects(
        &mut self,
        command: UiAssetEditorCommand,
        external_effects: UiAssetEditorUndoExternalEffects,
    ) -> Result<(), UiAssetEditorSessionError> {
        self.apply_command_with_effects_and_theme_source(command, external_effects, None)
    }

    fn apply_command_with_effects_and_theme_source(
        &mut self,
        command: UiAssetEditorCommand,
        external_effects: UiAssetEditorUndoExternalEffects,
        next_theme_source_key: Option<String>,
    ) -> Result<(), UiAssetEditorSessionError> {
        let before_source = self.source_buffer.text().to_string();
        let before_selection = self.selection.clone();
        let before_source_cursor = self.source_cursor_snapshot();
        let before_selected_theme_source_key = self.selected_theme_source_key.clone();
        let tree_edit = command.structured_tree_edit().cloned();
        let document_replay = command.document_replay().cloned();
        let before_document = tree_edit.as_ref().map(|_| self.last_valid_document.clone());
        self.source_buffer
            .replace(command.next_source().to_string());
        self.source_cursor_byte_offset = remap_source_byte_offset(
            &before_source,
            self.source_buffer.text(),
            self.source_cursor_byte_offset,
        );
        if let Some(next_selection) = command.next_selection() {
            self.selection = next_selection.clone();
        }
        if let Some(next_theme_source_key) = next_theme_source_key {
            self.selected_theme_source_key = Some(next_theme_source_key);
        }
        self.revalidate().map(|_| {
            if command.next_selection().is_some() {
                self.set_source_cursor_to_selected_node_start();
            } else if self.diagnostics.is_empty() {
                self.reconcile_source_cursor_state();
            }
            let after_document = tree_edit.as_ref().map(|_| self.last_valid_document.clone());
            let after_source_cursor = self.source_cursor_snapshot();
            self.undo_stack.push_edit(
                command.label().to_string(),
                tree_edit,
                document_replay,
                before_source,
                before_selection,
                before_source_cursor,
                before_selected_theme_source_key,
                before_document,
                self.source_buffer.text().to_string(),
                self.selection.clone(),
                after_source_cursor,
                self.selected_theme_source_key.clone(),
                after_document,
                external_effects,
            );
        })
    }

    fn rebuild_preview_snapshot(&mut self) -> Result<(), UiAssetEditorSessionError> {
        let preview_document =
            apply_preview_mock_overrides(&self.last_valid_document, &self.preview_mock_state);
        let (compiled, preview_host) = compile_preview(
            &preview_document,
            current_preview_size(&self.preview_host, self.route.preview_preset),
            &self.compiler_imports,
        )?;
        self.last_valid_compiled = compiled;
        self.preview_host = preview_host;
        Ok(())
    }

    fn refresh_preview_for_current_preset(&mut self) -> Result<(), UiAssetEditorSessionError> {
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

    pub fn undo_replay(&mut self) -> Result<UiAssetEditorReplayResult, UiAssetEditorSessionError> {
        let Some(record) = self.undo_stack.undo_record() else {
            return Ok(UiAssetEditorReplayResult::default());
        };
        let changed = self.apply_undo_transition(record.transition.clone())?;
        Ok(UiAssetEditorReplayResult {
            changed,
            label: record.label,
            external_effects: record.transition.external_effects,
        })
    }

    pub fn undo(&mut self) -> Result<bool, UiAssetEditorSessionError> {
        self.undo_replay().map(|result| result.changed)
    }

    pub fn redo_replay(&mut self) -> Result<UiAssetEditorReplayResult, UiAssetEditorSessionError> {
        let Some(record) = self.undo_stack.redo_record() else {
            return Ok(UiAssetEditorReplayResult::default());
        };
        let changed = self.apply_undo_transition(record.transition.clone())?;
        Ok(UiAssetEditorReplayResult {
            changed,
            label: record.label,
            external_effects: record.transition.external_effects,
        })
    }

    pub fn redo(&mut self) -> Result<bool, UiAssetEditorSessionError> {
        self.redo_replay().map(|result| result.changed)
    }

    fn existing_external_widget_source(
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

    fn existing_external_style_source(
        &self,
        asset_id: &str,
    ) -> Result<Option<String>, UiAssetEditorSessionError> {
        self.existing_external_asset_source(asset_id, self.compiler_imports.styles.get(asset_id))
    }

    fn existing_external_asset_source(
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
        Ok(toml::to_string_pretty(&self.last_valid_document)
            .map_err(|error| UiAssetError::ParseToml(error.to_string()))?)
    }

    pub fn save_to_canonical_source(&mut self) -> Result<String, UiAssetEditorSessionError> {
        let canonical = self.canonical_source()?;
        self.source_buffer.replace(canonical.clone());
        self.source_buffer.mark_saved();
        Ok(canonical)
    }

    fn insert_selected_palette_item(
        &mut self,
        mode: PaletteInsertMode,
    ) -> Result<bool, UiAssetEditorSessionError> {
        let Some(node_id) = self.selected_insert_target_node_id().map(str::to_string) else {
            return Ok(false);
        };
        self.insert_selected_palette_item_at_target(mode, &node_id)
    }

    fn insert_selected_palette_item_at_target(
        &mut self,
        mode: PaletteInsertMode,
        target_node_id: &str,
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
        let Some(plan) = build_palette_insert_plan(
            &self.last_valid_document,
            entry,
            target_node_id,
            mode,
            &self.compiler_imports.widgets,
            None,
        ) else {
            return Ok(false);
        };
        self.insert_selected_palette_item_with_plan(&plan)
    }

    fn insert_selected_palette_item_with_plan(
        &mut self,
        plan: &UiAssetPaletteInsertPlan,
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
        let Some(node_id) = insert_palette_item_with_placement(
            &mut document,
            &plan.node_id,
            entry,
            plan.mode,
            &plan.placement,
        ) else {
            return Ok(false);
        };
        let selection = selection_for_node(&document, &node_id);
        self.apply_document_edit_with_tree_edit_and_selection(
            document.clone(),
            UiAssetEditorTreeEdit::InsertPaletteItem {
                node_id,
                parent_node_id: selection.parent_node_id.clone(),
                palette_item_label: entry.label.clone(),
                insert_mode: palette_insert_mode_label(plan.mode).to_string(),
            },
            "Insert Palette Item",
            selection,
        )?;
        Ok(true)
    }

    fn move_selected_node(
        &mut self,
        direction: UiTreeMoveDirection,
    ) -> Result<bool, UiAssetEditorSessionError> {
        self.ensure_editable_source()?;
        let Some(node_id) = self.selection.primary_node_id.clone() else {
            return Ok(false);
        };
        let mut document = self.last_valid_document.clone();
        if !move_selected_node(&mut document, &self.selection, direction) {
            return Ok(false);
        }
        self.apply_document_edit_with_tree_edit(
            document,
            UiAssetEditorTreeEdit::MoveNode {
                node_id,
                direction: move_direction_label(direction).to_string(),
            },
            "Move Node",
        )?;
        Ok(true)
    }

    fn reparent_selected_node(
        &mut self,
        direction: UiTreeReparentDirection,
    ) -> Result<bool, UiAssetEditorSessionError> {
        self.ensure_editable_source()?;
        let mut document = self.last_valid_document.clone();
        let Some(node_id) = tree_reparent_selected_node(&mut document, &self.selection, direction)
        else {
            return Ok(false);
        };
        let selection = selection_for_node(&document, &node_id);
        self.apply_document_edit_with_tree_edit_and_selection(
            document.clone(),
            UiAssetEditorTreeEdit::ReparentNode {
                node_id,
                parent_node_id: selection.parent_node_id.clone(),
                direction: reparent_direction_label(direction).to_string(),
            },
            "Reparent Node",
            selection,
        )?;
        Ok(true)
    }

    fn ensure_editable_source(&self) -> Result<(), UiAssetEditorSessionError> {
        if self.diagnostics.is_empty() {
            Ok(())
        } else {
            Err(UiAssetEditorSessionError::InvalidSourceBuffer)
        }
    }

    fn apply_document_edit(
        &mut self,
        document: UiAssetDocument,
    ) -> Result<(), UiAssetEditorSessionError> {
        self.apply_document_edit_with_kind(
            document,
            UiAssetEditorTreeEditKind::DocumentEdit,
            "Document Edit",
        )
    }

    fn apply_document_edit_with_kind(
        &mut self,
        document: UiAssetDocument,
        kind: UiAssetEditorTreeEditKind,
        label: &str,
    ) -> Result<(), UiAssetEditorSessionError> {
        self.apply_document_edit_with_tree_edit(
            document,
            UiAssetEditorTreeEdit::generic(kind),
            label,
        )
    }

    fn apply_document_edit_with_tree_edit(
        &mut self,
        document: UiAssetDocument,
        edit: UiAssetEditorTreeEdit,
        label: &str,
    ) -> Result<(), UiAssetEditorSessionError> {
        let replay = tree_document_replay_bundle(&self.last_valid_document, &document);
        self.apply_document_edit_with_tree_edit_and_replay(document, edit, label, replay)
    }

    fn apply_document_edit_with_tree_edit_and_replay(
        &mut self,
        document: UiAssetDocument,
        edit: UiAssetEditorTreeEdit,
        label: &str,
        replay: UiAssetEditorDocumentReplayBundle,
    ) -> Result<(), UiAssetEditorSessionError> {
        let next_source = serialize_document(&document)?;
        self.apply_command(
            UiAssetEditorCommand::tree_edit_structured(edit, label, next_source)
                .with_document_replay(replay),
        )?;
        Ok(())
    }

    fn apply_document_edit_with_label(
        &mut self,
        document: UiAssetDocument,
        label: &str,
    ) -> Result<(), UiAssetEditorSessionError> {
        self.apply_document_edit_with_kind(document, UiAssetEditorTreeEditKind::DocumentEdit, label)
    }

    fn apply_document_edit_with_label_and_replay(
        &mut self,
        document: UiAssetDocument,
        label: &str,
        replay: UiAssetEditorDocumentReplayBundle,
    ) -> Result<(), UiAssetEditorSessionError> {
        self.apply_document_edit_with_tree_edit_and_replay(
            document,
            UiAssetEditorTreeEdit::generic(UiAssetEditorTreeEditKind::DocumentEdit),
            label,
            replay,
        )
    }

    fn apply_binding_document_edit_with_label(
        &mut self,
        document: UiAssetDocument,
        label: &str,
    ) -> Result<(), UiAssetEditorSessionError> {
        let Some(node_id) = self.selection.primary_node_id.clone() else {
            return self.apply_document_edit_with_label(document, label);
        };
        let replay = binding_document_replay_bundle(&self.last_valid_document, &document, &node_id);
        self.apply_document_edit_with_label_and_replay(document, label, replay)
    }

    fn apply_document_edit_with_tree_edit_and_selection(
        &mut self,
        document: UiAssetDocument,
        edit: UiAssetEditorTreeEdit,
        label: &str,
        selection: UiDesignerSelectionModel,
    ) -> Result<(), UiAssetEditorSessionError> {
        let replay = tree_document_replay_bundle(&self.last_valid_document, &document);
        let next_source = serialize_document(&document)?;
        self.apply_command(
            UiAssetEditorCommand::tree_edit_structured_with_selection(
                edit,
                label,
                next_source,
                selection,
            )
            .with_document_replay(replay),
        )?;
        Ok(())
    }

    fn apply_undo_transition(
        &mut self,
        transition: super::undo_stack::UiAssetEditorUndoTransition,
    ) -> Result<bool, UiAssetEditorSessionError> {
        let mut source = self.source_buffer.text().to_string();
        let source_changed = transition
            .apply_to_source(&mut source)
            .map_err(|_| UiAssetEditorSessionError::InvalidSourceBuffer)?;
        let mut replay_document = self.last_valid_document.clone();
        let document_changed = transition
            .apply_to_document(&mut replay_document)
            .map_err(|_| UiAssetEditorSessionError::InvalidSourceBuffer)?;
        let super::undo_stack::UiAssetEditorUndoTransition {
            selection,
            source_cursor,
            selected_theme_source_key,
            ..
        } = transition;
        self.selection = selection;
        self.selected_theme_source_key = selected_theme_source_key;
        self.source_buffer.replace(source);
        self.restore_source_cursor_snapshot(&source_cursor);
        if document_changed {
            self.apply_valid_document(replay_document)?;
            self.clear_palette_drag_state();
            return Ok(source_changed || document_changed);
        }
        self.revalidate()?;
        self.clear_palette_drag_state();
        Ok(source_changed)
    }

    fn set_source_cursor_to_selected_node_start(&mut self) {
        let Some(node_id) = self.selection.primary_node_id.as_deref() else {
            self.source_cursor_anchor = None;
            self.source_cursor_byte_offset = 0;
            return;
        };
        self.source_cursor_anchor = Some(UiAssetSourceCursorAnchor {
            node_id: node_id.to_string(),
            line_offset: 0,
        });
        let source = self.source_buffer.text().to_string();
        if let Some(entry) = source_outline_entry_for_node(&source, node_id) {
            self.source_cursor_byte_offset =
                source_byte_offset_for_line(&source, entry.line as usize);
        } else {
            self.source_cursor_byte_offset = self.source_cursor_byte_offset.min(source.len());
        }
    }

    fn set_source_cursor_for_selected_node_line(&mut self, line_offset: usize, byte_offset: usize) {
        let Some(node_id) = self.selection.primary_node_id.as_deref() else {
            self.source_cursor_anchor = None;
            self.source_cursor_byte_offset = 0;
            return;
        };
        let source = self.source_buffer.text().to_string();
        self.source_cursor_byte_offset = byte_offset.min(source.len());
        if let Some(entry) = source_outline_entry_for_node(&source, node_id) {
            let max_offset = (entry.end_line - entry.line).max(0) as usize;
            let line_offset = line_offset.min(max_offset);
            let current_line = source_line_for_byte_offset(&source, self.source_cursor_byte_offset);
            if current_line < entry.line as usize || current_line > entry.end_line as usize {
                self.source_cursor_byte_offset =
                    source_byte_offset_for_line(&source, entry.line as usize + line_offset);
            }
            self.source_cursor_anchor = Some(UiAssetSourceCursorAnchor {
                node_id: node_id.to_string(),
                line_offset,
            });
        } else {
            self.source_cursor_anchor = Some(UiAssetSourceCursorAnchor {
                node_id: node_id.to_string(),
                line_offset,
            });
        }
    }

    fn selected_source_line_offset(&self) -> Option<usize> {
        let selected_node_id = self.selection.primary_node_id.as_deref()?;
        self.source_cursor_anchor
            .as_ref()
            .filter(|anchor| anchor.node_id.as_str() == selected_node_id)
            .map(|anchor| anchor.line_offset)
    }

    fn source_cursor_snapshot(&self) -> UiAssetEditorSourceCursorSnapshot {
        UiAssetEditorSourceCursorSnapshot {
            byte_offset: self.source_cursor_byte_offset,
            anchor_node_id: self
                .source_cursor_anchor
                .as_ref()
                .map(|anchor| anchor.node_id.clone()),
            line_offset: self
                .source_cursor_anchor
                .as_ref()
                .map(|anchor| anchor.line_offset)
                .unwrap_or_default(),
        }
    }

    fn restore_source_cursor_snapshot(&mut self, snapshot: &UiAssetEditorSourceCursorSnapshot) {
        let source_len = self.source_buffer.text().len();
        self.source_cursor_byte_offset = snapshot.byte_offset.min(source_len);
        self.source_cursor_anchor =
            snapshot
                .anchor_node_id
                .as_ref()
                .map(|node_id| UiAssetSourceCursorAnchor {
                    node_id: node_id.clone(),
                    line_offset: snapshot.line_offset,
                });
    }

    fn roundtrip_source_text(&self) -> &str {
        if self.diagnostics.is_empty() {
            self.source_buffer.text()
        } else {
            &self.last_valid_source_text
        }
    }

    fn reconcile_source_cursor_state(&mut self) {
        let Some(selected_node_id) = self.selection.primary_node_id.as_deref() else {
            self.source_cursor_anchor = None;
            self.source_cursor_byte_offset = 0;
            return;
        };
        let source = self.source_buffer.text().to_string();
        self.source_cursor_byte_offset = self.source_cursor_byte_offset.min(source.len());
        let Some(entry) = source_outline_entry_for_node(&source, selected_node_id) else {
            return;
        };
        let current_line = source_line_for_byte_offset(&source, self.source_cursor_byte_offset);
        let existing_line_offset = self
            .source_cursor_anchor
            .as_ref()
            .filter(|anchor| anchor.node_id.as_str() == selected_node_id)
            .map(|anchor| anchor.line_offset)
            .unwrap_or_default();
        let max_offset = (entry.end_line - entry.line).max(0) as usize;
        let inside_selected_block =
            current_line >= entry.line as usize && current_line <= entry.end_line as usize;
        let line_offset = if inside_selected_block {
            current_line.saturating_sub(entry.line as usize)
        } else {
            existing_line_offset.min(max_offset)
        };
        if !inside_selected_block {
            self.source_cursor_byte_offset =
                source_byte_offset_for_line(&source, entry.line as usize + line_offset);
        }
        self.source_cursor_anchor = Some(UiAssetSourceCursorAnchor {
            node_id: selected_node_id.to_string(),
            line_offset,
        });
    }

    fn select_node_id(&mut self, node_id: &str) {
        self.selection = selection_for_node(&self.last_valid_document, node_id);
        self.clear_palette_drag_state();
        self.reconcile_promote_widget_draft();
        reconcile_preview_mock_state(
            &self.last_valid_document,
            &self.selection,
            &mut self.preview_mock_state,
        );
        self.style_inspector = build_style_inspector(
            &self.last_valid_document,
            &self.selection,
            &self.compiler_imports,
            &self.style_inspector.active_pseudo_states,
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
        self.selected_matched_style_rule_index = None;
    }

    fn reconcile_promote_widget_draft(&mut self) {
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

    fn reconcile_promote_theme_draft(&mut self) {
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

    fn revalidate(&mut self) -> Result<(), UiAssetEditorSessionError> {
        match UiAssetLoader::load_toml_str(self.source_buffer.text()) {
            Ok(document) => self.apply_valid_document(document),
            Err(error) => {
                self.diagnostics = vec![error.to_string()];
                Ok(())
            }
        }
    }

    fn apply_valid_document(
        &mut self,
        document: UiAssetDocument,
    ) -> Result<(), UiAssetEditorSessionError> {
        ensure_asset_kind(self.route.asset_kind, document.asset.kind)?;
        self.last_valid_document = document;
        self.last_valid_source_text = self.source_buffer.text().to_string();
        self.selection = reconcile_selection(&self.last_valid_document, &self.selection);
        self.reconcile_promote_widget_draft();
        self.reconcile_promote_theme_draft();
        self.selected_style_rule_index = reconcile_selected_style_rule_index(
            &self.last_valid_document,
            self.selected_style_rule_index,
        );
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
            }
            Err(error) => {
                self.diagnostics = vec![error.to_string()];
            }
        }
        Ok(())
    }
}

fn serialize_document(document: &UiAssetDocument) -> Result<String, UiAssetEditorSessionError> {
    toml::to_string_pretty(document)
        .map_err(|error| UiAssetError::ParseToml(error.to_string()).into())
}

fn remap_source_byte_offset(current: &str, next: &str, byte_offset: usize) -> usize {
    let byte_offset = byte_offset.min(current.len());
    let prefix_len = common_prefix_len(current, next);
    let current_suffix = &current[prefix_len..];
    let next_suffix = &next[prefix_len..];
    let suffix_len = common_suffix_len(current_suffix, next_suffix);
    let current_replace_end = current.len().saturating_sub(suffix_len);
    let next_replace_end = next.len().saturating_sub(suffix_len);
    if byte_offset <= prefix_len {
        return byte_offset;
    }
    if byte_offset >= current_replace_end {
        return next_replace_end + byte_offset.saturating_sub(current_replace_end);
    }
    next_replace_end
}

fn common_prefix_len(left: &str, right: &str) -> usize {
    left.chars()
        .zip(right.chars())
        .take_while(|(left_char, right_char)| left_char == right_char)
        .map(|(character, _)| character.len_utf8())
        .sum()
}

fn common_suffix_len(left: &str, right: &str) -> usize {
    left.chars()
        .rev()
        .zip(right.chars().rev())
        .take_while(|(left_char, right_char)| left_char == right_char)
        .map(|(character, _)| character.len_utf8())
        .sum()
}

fn reconcile_selected_palette_index<T>(items: &[T], current: Option<usize>) -> Option<usize> {
    match (current, items.len()) {
        (_, 0) => None,
        (Some(index), count) => Some(index.min(count - 1)),
        (None, _) => None,
    }
}

fn normalized_promote_asset_id(asset_id: &str) -> Option<String> {
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

fn normalized_promote_component_name(component_name: &str) -> Option<String> {
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

fn normalized_promote_document_id(document_id: &str) -> Option<String> {
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

fn normalized_promote_display_name(display_name: &str) -> Option<String> {
    let trimmed = display_name.trim();
    (!trimmed.is_empty()).then(|| trimmed.to_string())
}

fn can_apply_tree_document_edit(
    document: &UiAssetDocument,
    edit: impl FnOnce(&mut UiAssetDocument) -> bool,
) -> bool {
    let mut document = document.clone();
    edit(&mut document)
}

fn palette_insert_mode_label(mode: PaletteInsertMode) -> &'static str {
    match mode {
        PaletteInsertMode::Child => "child",
        PaletteInsertMode::After => "after_selection",
    }
}

fn palette_insert_mode_action(mode: PaletteInsertMode) -> &'static str {
    match mode {
        PaletteInsertMode::Child => "palette.insert.child",
        PaletteInsertMode::After => "palette.insert.after",
    }
}

fn move_direction_label(direction: UiTreeMoveDirection) -> &'static str {
    match direction {
        UiTreeMoveDirection::Up => "up",
        UiTreeMoveDirection::Down => "down",
    }
}

fn reparent_direction_label(direction: UiTreeReparentDirection) -> &'static str {
    match direction {
        UiTreeReparentDirection::IntoPrevious => "into_previous",
        UiTreeReparentDirection::IntoNext => "into_next",
        UiTreeReparentDirection::Outdent => "outdent",
    }
}

fn editable_stylesheet(document: &mut UiAssetDocument) -> &mut UiStyleSheet {
    if document.stylesheets.is_empty() {
        document.stylesheets.push(UiStyleSheet {
            id: "local_editor_rules".to_string(),
            rules: Vec::new(),
        });
    }
    document
        .stylesheets
        .last_mut()
        .expect("style sheet should exist after initialization")
}

fn style_rule_insert_replay_bundle(
    before_document: &UiAssetDocument,
    stylesheet_index: usize,
    rule_index: usize,
    rule: UiStyleRule,
) -> UiAssetEditorDocumentReplayBundle {
    if before_document.stylesheets.is_empty() {
        let stylesheet = UiStyleSheet {
            id: "local_editor_rules".to_string(),
            rules: vec![rule],
        };
        return UiAssetEditorDocumentReplayBundle {
            undo: vec![UiAssetEditorDocumentReplayCommand::RemoveStyleSheet {
                index: stylesheet_index,
                stylesheet_id: stylesheet.id.clone(),
            }],
            redo: vec![UiAssetEditorDocumentReplayCommand::InsertStyleSheet {
                index: stylesheet_index,
                stylesheet_id: stylesheet.id.clone(),
                stylesheet: Some(stylesheet),
            }],
        };
    }

    UiAssetEditorDocumentReplayBundle {
        undo: vec![UiAssetEditorDocumentReplayCommand::RemoveStyleRule {
            stylesheet_index,
            index: rule_index,
            selector: rule.selector.clone(),
        }],
        redo: vec![UiAssetEditorDocumentReplayCommand::InsertStyleRule {
            stylesheet_index,
            index: rule_index,
            selector: rule.selector.clone(),
            rule: Some(rule),
        }],
    }
}

fn style_rule_remove_replay_bundle(
    stylesheet_index: usize,
    rule_index: usize,
    rule: UiStyleRule,
) -> UiAssetEditorDocumentReplayBundle {
    UiAssetEditorDocumentReplayBundle {
        undo: vec![UiAssetEditorDocumentReplayCommand::InsertStyleRule {
            stylesheet_index,
            index: rule_index,
            selector: rule.selector.clone(),
            rule: Some(rule.clone()),
        }],
        redo: vec![UiAssetEditorDocumentReplayCommand::RemoveStyleRule {
            stylesheet_index,
            index: rule_index,
            selector: rule.selector,
        }],
    }
}

fn style_rule_move_replay_bundle(
    stylesheet_index: usize,
    from_index: usize,
    to_index: usize,
) -> UiAssetEditorDocumentReplayBundle {
    UiAssetEditorDocumentReplayBundle {
        undo: vec![UiAssetEditorDocumentReplayCommand::MoveStyleRule {
            stylesheet_index,
            from_index: to_index,
            to_index: from_index,
        }],
        redo: vec![UiAssetEditorDocumentReplayCommand::MoveStyleRule {
            stylesheet_index,
            from_index,
            to_index,
        }],
    }
}

fn tree_document_replay_bundle(
    before_document: &UiAssetDocument,
    after_document: &UiAssetDocument,
) -> UiAssetEditorDocumentReplayBundle {
    UiAssetEditorDocumentReplayBundle {
        undo: tree_document_replay_commands(after_document, before_document),
        redo: tree_document_replay_commands(before_document, after_document),
    }
}

fn tree_document_replay_commands(
    current: &UiAssetDocument,
    target: &UiAssetDocument,
) -> Vec<UiAssetEditorDocumentReplayCommand> {
    let mut commands = Vec::new();
    commands.extend(build_widget_import_replay_commands(
        &current.imports.widgets,
        &target.imports.widgets,
    ));
    commands.extend(upsert_component_replay_commands(
        &current.components,
        &target.components,
    ));
    commands.extend(upsert_node_replay_commands(&current.nodes, &target.nodes));
    if current.root != target.root {
        commands.push(UiAssetEditorDocumentReplayCommand::SetRoot {
            root: target.root.clone(),
        });
    }
    commands.extend(remove_component_replay_commands(
        &current.components,
        &target.components,
    ));
    commands.extend(remove_node_replay_commands(&current.nodes, &target.nodes));
    commands
}

fn build_widget_import_replay_commands(
    current: &[String],
    target: &[String],
) -> Vec<UiAssetEditorDocumentReplayCommand> {
    if current == target {
        return Vec::new();
    }
    if has_duplicate_string_entries(current) || has_duplicate_string_entries(target) {
        return vec![UiAssetEditorDocumentReplayCommand::SetWidgetImports {
            references: target.to_vec(),
        }];
    }

    let target_entries = target.iter().cloned().collect::<BTreeSet<_>>();
    let mut working = current.to_vec();
    let mut commands = Vec::new();

    for index in (0..working.len()).rev() {
        if target_entries.contains(&working[index]) {
            continue;
        }
        let reference = working.remove(index);
        commands.push(UiAssetEditorDocumentReplayCommand::RemoveWidgetImport { index, reference });
    }

    for (target_index, target_reference) in target.iter().enumerate() {
        match working
            .iter()
            .position(|reference| reference == target_reference)
        {
            Some(current_index) => {
                if current_index != target_index {
                    let moved = working.remove(current_index);
                    working.insert(target_index, moved);
                    commands.push(UiAssetEditorDocumentReplayCommand::MoveWidgetImport {
                        from_index: current_index,
                        to_index: target_index,
                        reference: target_reference.clone(),
                    });
                }
            }
            None => {
                working.insert(target_index, target_reference.clone());
                commands.push(UiAssetEditorDocumentReplayCommand::InsertWidgetImport {
                    index: target_index,
                    reference: target_reference.clone(),
                });
            }
        }
    }

    if working != target {
        return vec![UiAssetEditorDocumentReplayCommand::SetWidgetImports {
            references: target.to_vec(),
        }];
    }

    commands
}

fn upsert_node_replay_commands(
    current: &BTreeMap<String, zircon_runtime::ui::template::UiNodeDefinition>,
    target: &BTreeMap<String, zircon_runtime::ui::template::UiNodeDefinition>,
) -> Vec<UiAssetEditorDocumentReplayCommand> {
    target
        .iter()
        .filter_map(|(node_id, node)| {
            (current.get(node_id) != Some(node)).then(|| {
                UiAssetEditorDocumentReplayCommand::UpsertNode {
                    node_id: node_id.clone(),
                    node: node.clone(),
                }
            })
        })
        .collect()
}

fn remove_node_replay_commands(
    current: &BTreeMap<String, zircon_runtime::ui::template::UiNodeDefinition>,
    target: &BTreeMap<String, zircon_runtime::ui::template::UiNodeDefinition>,
) -> Vec<UiAssetEditorDocumentReplayCommand> {
    current
        .keys()
        .filter(|node_id| !target.contains_key(*node_id))
        .map(|node_id| UiAssetEditorDocumentReplayCommand::RemoveNode {
            node_id: node_id.clone(),
        })
        .collect()
}

fn upsert_component_replay_commands(
    current: &BTreeMap<String, zircon_runtime::ui::template::UiComponentDefinition>,
    target: &BTreeMap<String, zircon_runtime::ui::template::UiComponentDefinition>,
) -> Vec<UiAssetEditorDocumentReplayCommand> {
    target
        .iter()
        .filter_map(|(component_name, component)| {
            (current.get(component_name) != Some(component)).then(|| {
                UiAssetEditorDocumentReplayCommand::UpsertComponent {
                    component_name: component_name.clone(),
                    component: component.clone(),
                }
            })
        })
        .collect()
}

fn remove_component_replay_commands(
    current: &BTreeMap<String, zircon_runtime::ui::template::UiComponentDefinition>,
    target: &BTreeMap<String, zircon_runtime::ui::template::UiComponentDefinition>,
) -> Vec<UiAssetEditorDocumentReplayCommand> {
    current
        .keys()
        .filter(|component_name| !target.contains_key(*component_name))
        .map(
            |component_name| UiAssetEditorDocumentReplayCommand::RemoveComponent {
                component_name: component_name.clone(),
            },
        )
        .collect()
}

fn binding_document_replay_bundle(
    before_document: &UiAssetDocument,
    after_document: &UiAssetDocument,
    node_id: &str,
) -> UiAssetEditorDocumentReplayBundle {
    UiAssetEditorDocumentReplayBundle {
        undo: binding_document_replay_commands(after_document, before_document, node_id),
        redo: binding_document_replay_commands(before_document, after_document, node_id),
    }
}

fn binding_document_replay_commands(
    current: &UiAssetDocument,
    target: &UiAssetDocument,
    node_id: &str,
) -> Vec<UiAssetEditorDocumentReplayCommand> {
    let current_bindings = current
        .nodes
        .get(node_id)
        .map(|node| node.bindings.clone())
        .unwrap_or_default();
    let target_bindings = target
        .nodes
        .get(node_id)
        .map(|node| node.bindings.clone())
        .unwrap_or_default();
    if current_bindings == target_bindings {
        return Vec::new();
    }
    vec![UiAssetEditorDocumentReplayCommand::SetNodeBindings {
        node_id: node_id.to_string(),
        bindings: target_bindings,
    }]
}

fn theme_document_replay_bundle(
    before_document: &UiAssetDocument,
    after_document: &UiAssetDocument,
) -> UiAssetEditorDocumentReplayBundle {
    UiAssetEditorDocumentReplayBundle {
        undo: theme_document_replay_commands(after_document, before_document),
        redo: theme_document_replay_commands(before_document, after_document),
    }
}

fn theme_document_replay_commands(
    current: &UiAssetDocument,
    target: &UiAssetDocument,
) -> Vec<UiAssetEditorDocumentReplayCommand> {
    let mut commands = Vec::new();
    commands.extend(build_style_import_replay_commands(
        &current.imports.styles,
        &target.imports.styles,
    ));
    commands.extend(build_style_token_replay_commands(
        &current.tokens,
        &target.tokens,
    ));
    commands.extend(build_stylesheet_replay_commands(
        &current.stylesheets,
        &target.stylesheets,
    ));
    commands
}

fn build_style_import_replay_commands(
    current: &[String],
    target: &[String],
) -> Vec<UiAssetEditorDocumentReplayCommand> {
    if current == target {
        return Vec::new();
    }
    if has_duplicate_string_entries(current) || has_duplicate_string_entries(target) {
        return vec![UiAssetEditorDocumentReplayCommand::SetStyleImports {
            references: target.to_vec(),
        }];
    }

    let target_entries = target.iter().cloned().collect::<BTreeSet<_>>();
    let mut working = current.to_vec();
    let mut commands = Vec::new();

    for index in (0..working.len()).rev() {
        if target_entries.contains(&working[index]) {
            continue;
        }
        let reference = working.remove(index);
        commands.push(UiAssetEditorDocumentReplayCommand::RemoveStyleImport { index, reference });
    }

    for (target_index, target_reference) in target.iter().enumerate() {
        match working
            .iter()
            .position(|reference| reference == target_reference)
        {
            Some(current_index) => {
                if current_index != target_index {
                    let moved = working.remove(current_index);
                    working.insert(target_index, moved);
                    commands.push(UiAssetEditorDocumentReplayCommand::MoveStyleImport {
                        from_index: current_index,
                        to_index: target_index,
                        reference: target_reference.clone(),
                    });
                }
            }
            None => {
                working.insert(target_index, target_reference.clone());
                commands.push(UiAssetEditorDocumentReplayCommand::InsertStyleImport {
                    index: target_index,
                    reference: target_reference.clone(),
                });
            }
        }
    }

    if working != target {
        return vec![UiAssetEditorDocumentReplayCommand::SetStyleImports {
            references: target.to_vec(),
        }];
    }

    commands
}

fn build_style_token_replay_commands(
    current: &BTreeMap<String, toml::Value>,
    target: &BTreeMap<String, toml::Value>,
) -> Vec<UiAssetEditorDocumentReplayCommand> {
    if current == target {
        return Vec::new();
    }

    let mut commands = Vec::new();
    for token_name in current.keys().rev() {
        if target.contains_key(token_name) {
            continue;
        }
        commands.push(UiAssetEditorDocumentReplayCommand::RemoveStyleToken {
            token_name: token_name.clone(),
        });
    }

    for (token_name, target_value) in target {
        if current.get(token_name) == Some(target_value) {
            continue;
        }
        commands.push(UiAssetEditorDocumentReplayCommand::UpsertStyleToken {
            token_name: token_name.clone(),
            value: target_value.clone(),
        });
    }

    commands
}

fn build_stylesheet_replay_commands(
    current: &[UiStyleSheet],
    target: &[UiStyleSheet],
) -> Vec<UiAssetEditorDocumentReplayCommand> {
    if current == target {
        return Vec::new();
    }
    if has_duplicate_stylesheet_ids(current) || has_duplicate_stylesheet_ids(target) {
        return vec![UiAssetEditorDocumentReplayCommand::SetStyleSheets {
            stylesheets: target.to_vec(),
        }];
    }

    let target_ids = target
        .iter()
        .map(|stylesheet| stylesheet.id.clone())
        .collect::<BTreeSet<_>>();
    let mut working = current.to_vec();
    let mut commands = Vec::new();

    for index in (0..working.len()).rev() {
        if target_ids.contains(working[index].id.as_str()) {
            continue;
        }
        let stylesheet_id = working[index].id.clone();
        let _ = working.remove(index);
        commands.push(UiAssetEditorDocumentReplayCommand::RemoveStyleSheet {
            index,
            stylesheet_id,
        });
    }

    for (target_index, target_stylesheet) in target.iter().enumerate() {
        let current_index = working
            .iter()
            .position(|stylesheet| stylesheet.id == target_stylesheet.id);
        match current_index {
            Some(current_index) => {
                if current_index != target_index {
                    let moved = working.remove(current_index);
                    working.insert(target_index, moved);
                    commands.push(UiAssetEditorDocumentReplayCommand::MoveStyleSheet {
                        from_index: current_index,
                        to_index: target_index,
                        stylesheet_id: target_stylesheet.id.clone(),
                    });
                }
                if working
                    .get(target_index)
                    .is_some_and(|stylesheet| stylesheet != target_stylesheet)
                {
                    if let Some(rule_commands) = build_style_rule_replay_commands(
                        target_index,
                        &working[target_index].rules,
                        &target_stylesheet.rules,
                    ) {
                        working[target_index].rules = target_stylesheet.rules.clone();
                        commands.extend(rule_commands);
                    } else {
                        working[target_index] = target_stylesheet.clone();
                        commands.push(UiAssetEditorDocumentReplayCommand::ReplaceStyleSheet {
                            index: target_index,
                            stylesheet_id: target_stylesheet.id.clone(),
                            stylesheet: target_stylesheet.clone(),
                        });
                    }
                }
            }
            None => {
                working.insert(target_index, target_stylesheet.clone());
                commands.push(UiAssetEditorDocumentReplayCommand::InsertStyleSheet {
                    index: target_index,
                    stylesheet_id: target_stylesheet.id.clone(),
                    stylesheet: Some(target_stylesheet.clone()),
                });
            }
        }
    }

    if working != target {
        return vec![UiAssetEditorDocumentReplayCommand::SetStyleSheets {
            stylesheets: target.to_vec(),
        }];
    }

    commands
}

fn build_style_rule_replay_commands(
    stylesheet_index: usize,
    current: &[UiStyleRule],
    target: &[UiStyleRule],
) -> Option<Vec<UiAssetEditorDocumentReplayCommand>> {
    if current == target {
        return Some(Vec::new());
    }
    if has_duplicate_rule_selectors(current) || has_duplicate_rule_selectors(target) {
        return None;
    }

    let target_selectors = target
        .iter()
        .map(|rule| rule.selector.clone())
        .collect::<BTreeSet<_>>();
    let mut working = current.to_vec();
    let mut commands = Vec::new();

    for index in (0..working.len()).rev() {
        if target_selectors.contains(working[index].selector.as_str()) {
            continue;
        }
        let selector = working[index].selector.clone();
        let _ = working.remove(index);
        commands.push(UiAssetEditorDocumentReplayCommand::RemoveStyleRule {
            stylesheet_index,
            index,
            selector,
        });
    }

    for (target_index, target_rule) in target.iter().enumerate() {
        let current_index = working
            .iter()
            .position(|rule| rule.selector == target_rule.selector);
        match current_index {
            Some(current_index) => {
                if current_index != target_index {
                    let moved = working.remove(current_index);
                    working.insert(target_index, moved);
                    commands.push(UiAssetEditorDocumentReplayCommand::MoveStyleRule {
                        stylesheet_index,
                        from_index: current_index,
                        to_index: target_index,
                    });
                }
                if working
                    .get(target_index)
                    .is_some_and(|rule| rule != target_rule)
                {
                    let selector = target_rule.selector.clone();
                    let _ = working.remove(target_index);
                    commands.push(UiAssetEditorDocumentReplayCommand::RemoveStyleRule {
                        stylesheet_index,
                        index: target_index,
                        selector: selector.clone(),
                    });
                    working.insert(target_index, target_rule.clone());
                    commands.push(UiAssetEditorDocumentReplayCommand::InsertStyleRule {
                        stylesheet_index,
                        index: target_index,
                        selector,
                        rule: Some(target_rule.clone()),
                    });
                }
            }
            None => {
                working.insert(target_index, target_rule.clone());
                commands.push(UiAssetEditorDocumentReplayCommand::InsertStyleRule {
                    stylesheet_index,
                    index: target_index,
                    selector: target_rule.selector.clone(),
                    rule: Some(target_rule.clone()),
                });
            }
        }
    }

    (working == target).then_some(commands)
}

fn has_duplicate_stylesheet_ids(stylesheets: &[UiStyleSheet]) -> bool {
    let mut seen = BTreeSet::new();
    stylesheets
        .iter()
        .map(|stylesheet| stylesheet.id.as_str())
        .any(|stylesheet_id| !seen.insert(stylesheet_id))
}

fn has_duplicate_rule_selectors(rules: &[UiStyleRule]) -> bool {
    let mut seen = BTreeSet::new();
    rules
        .iter()
        .map(|rule| rule.selector.as_str())
        .any(|selector| !seen.insert(selector))
}

fn has_duplicate_string_entries(entries: &[String]) -> bool {
    let mut seen = BTreeSet::new();
    entries
        .iter()
        .map(String::as_str)
        .any(|entry| !seen.insert(entry))
}

fn reference_asset_id(reference: &str) -> &str {
    reference
        .split_once('#')
        .map(|(asset_id, _)| asset_id)
        .unwrap_or(reference)
}

fn restore_or_remove_external_asset_source(
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

fn preview_summary(preview_host: Option<&UiAssetPreviewHost>) -> String {
    let Some(preview_host) = preview_host else {
        return "preview unavailable".to_string();
    };
    format!(
        "{} rendered nodes @ {:.0}x{:.0}",
        preview_host.surface().render_extract.list.commands.len(),
        preview_host.preview_size().width,
        preview_host.preview_size().height
    )
}
