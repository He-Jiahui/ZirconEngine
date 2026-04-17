use std::collections::BTreeMap;

use thiserror::Error;
use toml::Value;
use zircon_editor_ui::{
    UiAssetEditorMode, UiAssetEditorReflectionModel, UiAssetEditorRoute, UiAssetPreviewPreset,
    UiDesignerSelectionModel, UiStyleInspectorReflectionModel,
};
use zircon_ui::{
    UiAssetDocument, UiAssetError, UiAssetKind, UiAssetLoader, UiCompiledDocument,
    UiDocumentCompiler, UiSize, UiStyleDeclarationBlock, UiStyleRule, UiStyleSheet,
    UiTemplateBuildError, UiTreeError,
};

use super::{
    binding_inspector::{
        add_default_binding, build_binding_fields,
        delete_selected_binding as delete_selected_binding_field,
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
    command::{UiAssetEditorCommand, UiAssetEditorTreeEdit, UiAssetEditorTreeEditKind},
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
    matched_rule_inspection::{
        MatchedStyleRuleEntry, matched_style_rule_entries, selector_component_name,
        selector_is_valid,
    },
    palette_drop::{
        UiAssetPaletteDragResolution, UiAssetPaletteDragTarget, UiAssetPaletteInsertPlan,
        build_palette_drag_slot_target_overlays, build_palette_insert_plan,
        can_insert_palette_item_for_node as can_insert_palette_item_at_node,
        resolve_palette_drag_target as resolve_palette_drag_target_for_preview,
    },
    palette_target_chooser::{UiAssetPaletteTargetChooser, reconcile_palette_target_chooser},
    presentation::{
        UiAssetEditorPanePresentation, UiAssetEditorPreviewCanvasNode,
        UiAssetEditorPreviewCanvasSlotTarget,
    },
    preview_host::UiAssetPreviewHost,
    preview_mock::{
        UiAssetPreviewMockState, apply_preview_mock_overrides, build_preview_mock_fields,
        clear_selected_preview_mock_value, reconcile_preview_mock_state,
        select_preview_mock_property, set_selected_preview_mock_value,
    },
    preview_projection::{build_preview_projection, preview_node_id_for_index},
    promote_widget::{
        UiAssetExternalWidgetDraft, can_promote_selected_component_to_external_widget,
        default_external_widget_draft,
        promote_selected_component_to_external_widget as tree_promote_selected_component_to_external_widget,
        selected_local_component_name,
    },
    source_buffer::UiAssetSourceBuffer,
    source_sync::{build_source_outline, build_source_selection_summary},
    style_rule_declarations::{
        UiStyleRuleDeclarationEntry, UiStyleRuleDeclarationPath, declaration_entries,
        parse_declaration_literal, remove_declaration, set_declaration,
    },
    tree_editing::{
        PaletteInsertMode, UiTreeMoveDirection, UiTreeReparentDirection, build_palette_entries,
        can_convert_selected_node_to_reference, can_extract_selected_node_to_component,
        convert_selected_node_to_reference as tree_convert_selected_node_to_reference,
        extract_selected_node_to_component as tree_extract_selected_node_to_component,
        insert_palette_item_with_placement, move_selected_node,
        reparent_selected_node as tree_reparent_selected_node, unwrap_selected_node,
        wrap_selected_node,
    },
    undo_stack::UiAssetEditorUndoStack,
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

#[derive(Default)]
struct UiAssetCompilerImports {
    widgets: BTreeMap<String, UiAssetDocument>,
    styles: BTreeMap<String, UiAssetDocument>,
}

pub struct UiAssetEditorSession {
    route: UiAssetEditorRoute,
    source_buffer: UiAssetSourceBuffer,
    last_valid_document: UiAssetDocument,
    last_valid_compiled: Option<UiCompiledDocument>,
    preview_host: Option<UiAssetPreviewHost>,
    undo_stack: UiAssetEditorUndoStack,
    diagnostics: Vec<String>,
    selection: UiDesignerSelectionModel,
    style_inspector: UiStyleInspectorReflectionModel,
    selected_style_rule_index: Option<usize>,
    selected_matched_style_rule_index: Option<usize>,
    selected_style_rule_declaration_path: Option<String>,
    selected_style_token_name: Option<String>,
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
    preview_mock_state: UiAssetPreviewMockState,
    compiler_imports: UiAssetCompilerImports,
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
        let promote_draft = default_external_widget_draft(&document, &selection);
        let (last_valid_compiled, preview_host, diagnostics) =
            match compile_preview(&document, preview_size, &compiler_imports) {
                Ok((compiled, preview_host)) => (compiled, preview_host, Vec::new()),
                Err(error) => (None, None, vec![error.to_string()]),
            };
        Ok(Self {
            route,
            source_buffer: UiAssetSourceBuffer::new(source),
            last_valid_document: document,
            last_valid_compiled,
            preview_host,
            undo_stack: UiAssetEditorUndoStack::default(),
            diagnostics,
            selection,
            style_inspector,
            selected_style_rule_index: None,
            selected_matched_style_rule_index: None,
            selected_style_rule_declaration_path: None,
            selected_style_token_name: None,
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
            preview_mock_state: UiAssetPreviewMockState::default(),
            compiler_imports,
        })
    }

    pub fn route(&self) -> &UiAssetEditorRoute {
        &self.route
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
        let source_summary = build_source_selection_summary(
            &self.last_valid_document,
            &self.selection,
            self.source_buffer.text(),
            &self.diagnostics,
        );
        let source_outline =
            build_source_outline(&self.last_valid_document, self.source_buffer.text());
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
        let inspector_fields = build_inspector_fields(&self.last_valid_document, &self.selection);
        let binding_fields = build_binding_fields(
            &self.last_valid_document,
            &self.selection,
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
            last_error: reflection.last_error.clone().unwrap_or_default(),
            selection_summary: selection_summary(&reflection.selection),
            source_text: self.source_buffer.text().to_string(),
            preview_preset: reflection.route.preview_preset.label().to_string(),
            source_selected_block_label: source_summary.block_label,
            source_selected_line: source_summary.line,
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
            preview_mock_items: preview_mock_fields.items,
            preview_mock_selected_index: preview_mock_fields.selected_index,
            preview_mock_property: preview_mock_fields.property,
            preview_mock_kind: preview_mock_fields.kind,
            preview_mock_value: preview_mock_fields.value,
            preview_mock_can_edit: preview_mock_fields.can_edit,
            preview_mock_can_clear: preview_mock_fields.can_clear,
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
            inspector_binding_action_kind_items: binding_fields.binding_action_kind_items,
            inspector_binding_action_kind_selected_index: binding_fields
                .binding_action_kind_selected_index,
            inspector_binding_payload_items: binding_fields.binding_payload_items,
            inspector_binding_payload_selected_index: binding_fields.binding_payload_selected_index,
            inspector_binding_payload_key: binding_fields.binding_payload_key,
            inspector_binding_payload_value: binding_fields.binding_payload_value,
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
        Ok(())
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
        if node.kind != zircon_ui::UiNodeDefinitionKind::Reference {
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
        let Some(widget_document) = tree_promote_selected_component_to_external_widget(
            &mut document,
            &self.selection,
            widget_asset_id,
            widget_component_name,
            widget_document_id,
        ) else {
            return Ok(None);
        };
        let selection = selection_for_node(&document, &node_id);
        self.apply_document_edit_with_tree_edit_and_selection(
            document.clone(),
            UiAssetEditorTreeEdit::PromoteToExternalWidget {
                source_component_name,
                asset_id: widget_asset_id.to_string(),
                component_name: widget_component_name.to_string(),
                document_id: widget_document_id.to_string(),
            },
            "Promote To External Widget",
            selection,
        )?;
        Ok(Some(widget_document))
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
        editable_stylesheet(&mut document).rules.push(UiStyleRule {
            selector,
            set: UiStyleDeclarationBlock::default(),
        });
        self.selected_style_rule_index = local_style_rule_entries(&document).len().checked_sub(1);
        self.selected_style_rule_declaration_path = None;
        self.apply_document_edit(document)?;
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
        editable_stylesheet(&mut document).rules.push(UiStyleRule {
            selector,
            set: overrides,
        });
        self.selected_style_rule_index = local_style_rule_entries(&document).len().checked_sub(1);
        self.selected_style_rule_declaration_path = None;
        self.apply_document_edit(document)?;
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
        self.apply_document_edit(document)?;
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
        self.apply_document_edit(document)?;
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
        self.apply_document_edit(document)?;
        Ok(true)
    }

    pub fn select_binding_event_option(
        &mut self,
        index: usize,
    ) -> Result<bool, UiAssetEditorSessionError> {
        let binding_fields = build_binding_fields(
            &self.last_valid_document,
            &self.selection,
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
        self.apply_document_edit(document)?;
        Ok(true)
    }

    pub fn select_binding_action_kind(
        &mut self,
        index: usize,
    ) -> Result<bool, UiAssetEditorSessionError> {
        let binding_fields = build_binding_fields(
            &self.last_valid_document,
            &self.selection,
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
        self.apply_document_edit_with_label(document, "Binding Action Kind Edit")?;
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
        self.apply_document_edit(document)?;
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
        self.apply_document_edit_with_label(document, "Binding Route Target Edit")?;
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
        self.apply_document_edit_with_label(document, "Binding Action Target Edit")?;
        Ok(true)
    }

    pub fn select_binding_payload(
        &mut self,
        index: usize,
    ) -> Result<bool, UiAssetEditorSessionError> {
        let binding_fields = build_binding_fields(
            &self.last_valid_document,
            &self.selection,
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
        let payload_key = payload_key.as_ref();
        let mut document = self.last_valid_document.clone();
        if !upsert_selected_binding_payload_field(
            &mut document,
            &self.selection,
            self.selected_binding_index,
            payload_key,
            value_literal.as_ref(),
        ) {
            return Ok(false);
        }
        self.selected_binding_payload_key = Some(payload_key.trim().to_string());
        self.apply_document_edit_with_label(document, "Binding Payload Upsert")?;
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
        self.apply_document_edit_with_label(document, "Binding Payload Delete")?;
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
        self.apply_document_edit(document)?;
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
        self.apply_document_edit(document)?;
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
        let _ = rules.remove(entry.rule_index);
        let remaining = local_style_rule_entries(&document).len();
        self.selected_style_rule_index = (remaining > 0).then(|| index.min(remaining - 1));
        self.selected_style_rule_declaration_path = None;
        self.apply_document_edit(document)?;
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
        let before_source = self.source_buffer.text().to_string();
        let before_selection = self.selection.clone();
        let tree_edit = command.structured_tree_edit().cloned();
        let before_document = tree_edit.as_ref().map(|_| self.last_valid_document.clone());
        self.source_buffer
            .replace(command.next_source().to_string());
        if let Some(next_selection) = command.next_selection() {
            self.selection = next_selection.clone();
        }
        self.revalidate().map(|_| {
            let after_document = tree_edit.as_ref().map(|_| self.last_valid_document.clone());
            self.undo_stack.push_edit(
                command.label().to_string(),
                tree_edit,
                before_source,
                before_selection,
                before_document,
                self.source_buffer.text().to_string(),
                self.selection.clone(),
                after_document,
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

    pub fn undo(&mut self) -> Result<bool, UiAssetEditorSessionError> {
        let Some(snapshot) = self.undo_stack.undo() else {
            return Ok(false);
        };
        self.restore_snapshot(snapshot)?;
        Ok(true)
    }

    pub fn redo(&mut self) -> Result<bool, UiAssetEditorSessionError> {
        let Some(snapshot) = self.undo_stack.redo() else {
            return Ok(false);
        };
        self.restore_snapshot(snapshot)?;
        Ok(true)
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
        let next_source = serialize_document(&document)?;
        self.apply_command(UiAssetEditorCommand::tree_edit_structured(
            edit,
            label,
            next_source,
        ))?;
        Ok(())
    }

    fn apply_document_edit_with_label(
        &mut self,
        document: UiAssetDocument,
        label: &str,
    ) -> Result<(), UiAssetEditorSessionError> {
        self.apply_document_edit_with_kind(document, UiAssetEditorTreeEditKind::DocumentEdit, label)
    }

    fn apply_document_edit_with_tree_edit_and_selection(
        &mut self,
        document: UiAssetDocument,
        edit: UiAssetEditorTreeEdit,
        label: &str,
        selection: UiDesignerSelectionModel,
    ) -> Result<(), UiAssetEditorSessionError> {
        let next_source = serialize_document(&document)?;
        self.apply_command(UiAssetEditorCommand::tree_edit_structured_with_selection(
            edit,
            label,
            next_source,
            selection,
        ))?;
        Ok(())
    }

    fn restore_snapshot(
        &mut self,
        snapshot: super::undo_stack::UiAssetEditorUndoSnapshot,
    ) -> Result<(), UiAssetEditorSessionError> {
        self.selection = snapshot.selection;
        if let Some(document_replay) = snapshot.document {
            let mut document = self.last_valid_document.clone();
            let _ = document_replay
                .apply_to_document(&mut document)
                .map_err(|_| UiAssetEditorSessionError::InvalidSourceBuffer)?;
            self.source_buffer.replace(serialize_document(&document)?);
            self.apply_valid_document(document)?;
        } else {
            self.source_buffer.replace(snapshot.source);
            self.revalidate()?;
        }
        self.clear_palette_drag_state();
        Ok(())
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
        self.selection = reconcile_selection(&self.last_valid_document, &self.selection);
        self.reconcile_promote_widget_draft();
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

fn compile_preview(
    document: &UiAssetDocument,
    preview_size: UiSize,
    imports: &UiAssetCompilerImports,
) -> Result<(Option<UiCompiledDocument>, Option<UiAssetPreviewHost>), UiAssetEditorSessionError> {
    if matches!(document.asset.kind, UiAssetKind::Style) {
        return Ok((None, None));
    }

    let mut compiler = UiDocumentCompiler::default();
    for (reference, widget) in &imports.widgets {
        compiler.register_widget_import(reference.clone(), widget.clone())?;
    }
    for (reference, style) in &imports.styles {
        compiler.register_style_import(reference.clone(), style.clone())?;
    }
    let compiled = compiler.compile(document)?;
    let preview_host = UiAssetPreviewHost::new(preview_size, &document.asset.id, &compiled)?;
    Ok((Some(compiled), Some(preview_host)))
}

fn preview_size_for_preset(preview_preset: UiAssetPreviewPreset) -> UiSize {
    match preview_preset {
        UiAssetPreviewPreset::EditorDocked => UiSize::new(1280.0, 720.0),
        UiAssetPreviewPreset::EditorFloating => UiSize::new(1100.0, 780.0),
        UiAssetPreviewPreset::GameHud => UiSize::new(1920.0, 1080.0),
        UiAssetPreviewPreset::Dialog => UiSize::new(640.0, 480.0),
    }
}

fn current_preview_size(
    current: &Option<UiAssetPreviewHost>,
    preview_preset: UiAssetPreviewPreset,
) -> UiSize {
    current
        .as_ref()
        .map(UiAssetPreviewHost::preview_size)
        .unwrap_or_else(|| preview_size_for_preset(preview_preset))
}

fn ensure_asset_kind(
    expected: UiAssetKind,
    actual: UiAssetKind,
) -> Result<(), UiAssetEditorSessionError> {
    if expected != actual {
        return Err(UiAssetEditorSessionError::UnexpectedKind { expected, actual });
    }
    Ok(())
}

fn default_selection(document: &UiAssetDocument) -> UiDesignerSelectionModel {
    document
        .root
        .as_ref()
        .map(|root| selection_for_node(document, &root.node))
        .unwrap_or_default()
}

fn reconcile_selection(
    document: &UiAssetDocument,
    current: &UiDesignerSelectionModel,
) -> UiDesignerSelectionModel {
    let primary = current.primary_node_id.as_deref();
    if let Some(node_id) = primary {
        if document.nodes.contains_key(node_id) {
            let mut selection = selection_for_node(document, node_id);
            let parent = selection.parent_node_id.clone();
            for sibling in &current.sibling_node_ids {
                if sibling == node_id || !document.nodes.contains_key(sibling) {
                    continue;
                }
                if parent_for_node(document, sibling)
                    .map(|(parent_id, _)| Some(parent_id) == parent)
                    .unwrap_or(false)
                {
                    selection = selection.with_sibling(sibling.clone());
                }
            }
            return selection;
        }
    }
    default_selection(document)
}

fn build_style_inspector(
    document: &UiAssetDocument,
    selection: &UiDesignerSelectionModel,
    imports: &UiAssetCompilerImports,
    active_states: &[String],
) -> UiStyleInspectorReflectionModel {
    let Some(node_id) = selection.primary_node_id.as_deref() else {
        return UiStyleInspectorReflectionModel::default();
    };
    let Some(node) = document.nodes.get(node_id) else {
        return UiStyleInspectorReflectionModel::default();
    };

    let mut inspector = UiStyleInspectorReflectionModel::for_node(node_id.to_string());
    for class_name in &node.classes {
        inspector = inspector.with_class(class_name.clone());
    }
    for state in active_states {
        inspector = inspector.with_active_pseudo_state(state.clone());
    }
    for (path, value) in &node.style_overrides.self_values {
        inspector =
            inspector.with_inline_override(format!("self.{path}"), toml_value_to_json(value));
    }
    for (path, value) in &node.style_overrides.slot {
        inspector =
            inspector.with_inline_override(format!("slot.{path}"), toml_value_to_json(value));
    }
    for rule in matched_style_rule_entries(document, &imports.styles, node_id, active_states) {
        inspector = inspector.with_matched_rule(rule.reflection());
    }
    inspector
}

fn toml_value_to_json(value: &toml::Value) -> serde_json::Value {
    serde_json::to_value(value).unwrap_or(serde_json::Value::Null)
}

const SUPPORTED_PSEUDO_STATES: &[&str] = &[
    "hover", "focus", "pressed", "checked", "selected", "disabled",
];

fn serialize_document(document: &UiAssetDocument) -> Result<String, UiAssetEditorSessionError> {
    toml::to_string_pretty(document)
        .map_err(|error| UiAssetError::ParseToml(error.to_string()).into())
}

#[derive(Clone, Debug)]
struct LocalStyleRuleEntry {
    stylesheet_index: usize,
    rule_index: usize,
    selector: String,
}

#[derive(Clone, Debug)]
struct LocalStyleTokenEntry {
    name: String,
    literal: String,
}

fn local_style_rule_entries(document: &UiAssetDocument) -> Vec<LocalStyleRuleEntry> {
    let mut entries = Vec::new();
    for (stylesheet_index, stylesheet) in document.stylesheets.iter().enumerate() {
        for (rule_index, rule) in stylesheet.rules.iter().enumerate() {
            entries.push(LocalStyleRuleEntry {
                stylesheet_index,
                rule_index,
                selector: rule.selector.clone(),
            });
        }
    }
    entries
}

fn selected_style_rule_declaration_entries(
    document: &UiAssetDocument,
    selected_rule_index: Option<usize>,
) -> Vec<UiStyleRuleDeclarationEntry> {
    selected_rule_index
        .and_then(|index| local_style_rule_entries(document).get(index).cloned())
        .map(|entry| {
            declaration_entries(
                &document.stylesheets[entry.stylesheet_index].rules[entry.rule_index].set,
            )
        })
        .unwrap_or_default()
}

fn matched_style_rule_entries_for_selection(
    document: &UiAssetDocument,
    selection: &UiDesignerSelectionModel,
    imports: &UiAssetCompilerImports,
    active_states: &[String],
) -> Vec<MatchedStyleRuleEntry> {
    selection
        .primary_node_id
        .as_deref()
        .map(|node_id| {
            matched_style_rule_entries(document, &imports.styles, node_id, active_states)
        })
        .unwrap_or_default()
}

fn local_style_token_entries(document: &UiAssetDocument) -> Vec<LocalStyleTokenEntry> {
    document
        .tokens
        .iter()
        .map(|(name, value)| LocalStyleTokenEntry {
            name: name.clone(),
            literal: toml_value_literal(value),
        })
        .collect()
}

fn reconcile_selected_style_rule_index(
    document: &UiAssetDocument,
    current: Option<usize>,
) -> Option<usize> {
    let count = local_style_rule_entries(document).len();
    match (current, count) {
        (_, 0) => None,
        (Some(index), count) => Some(index.min(count - 1)),
        (None, _) => None,
    }
}

fn reconcile_selected_style_rule_declaration_path(
    document: &UiAssetDocument,
    selected_rule_index: Option<usize>,
    current: Option<&str>,
) -> Option<String> {
    let entries = selected_style_rule_declaration_entries(document, selected_rule_index);
    current
        .filter(|path| entries.iter().any(|entry| entry.path.as_str() == *path))
        .map(str::to_string)
}

fn reconcile_selected_matched_style_rule_index(
    entries: &[MatchedStyleRuleEntry],
    current: Option<usize>,
) -> Option<usize> {
    match (current, entries.len()) {
        (_, 0) => None,
        (Some(index), count) => Some(index.min(count - 1)),
        (None, _) => None,
    }
}

fn reconcile_selected_style_token_name(
    document: &UiAssetDocument,
    current: Option<&str>,
) -> Option<String> {
    current
        .filter(|name| document.tokens.contains_key(*name))
        .map(str::to_string)
}

fn reconcile_selected_palette_index<T>(items: &[T], current: Option<usize>) -> Option<usize> {
    match (current, items.len()) {
        (_, 0) => None,
        (Some(index), count) => Some(index.min(count - 1)),
        (None, _) => None,
    }
}

fn normalized_selector(selector: &str) -> Result<String, UiAssetEditorSessionError> {
    let trimmed = selector.trim();
    if trimmed.is_empty() || !selector_is_valid(trimmed) {
        return Err(UiAssetEditorSessionError::InvalidStyleSelector {
            selector: trimmed.to_string(),
        });
    }
    Ok(trimmed.to_string())
}

fn normalized_class_name(class_name: &str) -> Option<String> {
    let trimmed = class_name.trim();
    (!trimmed.is_empty() && !trimmed.chars().any(char::is_whitespace)).then(|| trimmed.to_string())
}

fn normalized_token_name(token_name: &str) -> Option<String> {
    let trimmed = token_name.trim();
    (!trimmed.is_empty() && !trimmed.chars().any(char::is_whitespace)).then(|| trimmed.to_string())
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

fn parse_token_literal(value_literal: &str) -> Option<Value> {
    let trimmed = value_literal.trim();
    if trimmed.is_empty() {
        return None;
    }

    let wrapped = format!("value = {trimmed}");
    toml::from_str::<toml::Table>(&wrapped)
        .ok()
        .and_then(|table| table.get("value").cloned())
        .or_else(|| Some(Value::String(trimmed.to_string())))
}

fn toml_value_literal(value: &Value) -> String {
    value.to_string()
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

fn selected_node_selector(
    document: &UiAssetDocument,
    selection: &UiDesignerSelectionModel,
) -> Option<String> {
    selection
        .primary_node_id
        .as_deref()
        .and_then(|node_id| document.nodes.get(node_id))
        .map(selector_for_node)
}

fn reference_asset_id(reference: &str) -> &str {
    reference
        .split_once('#')
        .map(|(asset_id, _)| asset_id)
        .unwrap_or(reference)
}

fn selector_for_node(node: &zircon_ui::UiNodeDefinition) -> String {
    if let Some(control_id) = node.control_id.as_deref() {
        return format!("#{control_id}");
    }

    let mut selector = selector_component_name(node).to_string();
    for class_name in &node.classes {
        selector.push('.');
        selector.push_str(class_name);
    }
    selector
}

fn selected_node_has_inline_overrides(
    document: &UiAssetDocument,
    selection: &UiDesignerSelectionModel,
) -> bool {
    selection
        .primary_node_id
        .as_deref()
        .and_then(|node_id| document.nodes.get(node_id))
        .is_some_and(|node| {
            !node.style_overrides.self_values.is_empty() || !node.style_overrides.slot.is_empty()
        })
}

fn pseudo_state_active(inspector: &UiStyleInspectorReflectionModel, state: &str) -> bool {
    inspector
        .active_pseudo_states
        .iter()
        .any(|candidate| candidate == state)
}

fn selection_summary(selection: &UiDesignerSelectionModel) -> String {
    let primary = selection
        .primary_node_id
        .clone()
        .unwrap_or_else(|| "none".to_string());
    let parent = selection
        .parent_node_id
        .clone()
        .unwrap_or_else(|| "-".to_string());
    let mount = selection.mount.clone().unwrap_or_else(|| "-".to_string());
    format!("selected {primary} • parent {parent} • mount {mount}")
}

fn build_hierarchy_items(document: &UiAssetDocument, selected: Option<&str>) -> Vec<String> {
    fn visit(
        output: &mut Vec<String>,
        document: &UiAssetDocument,
        node_id: &str,
        depth: usize,
        selected: Option<&str>,
    ) {
        let Some(node) = document.nodes.get(node_id) else {
            return;
        };
        let prefix = if selected == Some(node_id) {
            "> "
        } else {
            "  "
        };
        let label = node
            .widget_type
            .clone()
            .or_else(|| node.component_ref.clone())
            .unwrap_or_else(|| "Node".to_string());
        output.push(format!("{prefix}{}{node_id} [{label}]", "  ".repeat(depth)));
        for child in &node.children {
            visit(output, document, &child.child, depth + 1, selected);
        }
    }

    let mut items = Vec::new();
    if let Some(root) = &document.root {
        visit(&mut items, document, &root.node, 0, selected);
    }
    items
}

fn build_inspector_items(reflection: &UiAssetEditorReflectionModel) -> Vec<String> {
    let mut items = vec![
        format!("mode: {:?}", reflection.route.mode),
        format!("asset kind: {:?}", reflection.route.asset_kind),
        format!("dirty: {}", reflection.source_dirty),
        format!("undo: {}", reflection.can_undo),
        format!("redo: {}", reflection.can_redo),
        format!("preview: {}", reflection.preview_available),
    ];
    if let Some(node_id) = &reflection.selection.primary_node_id {
        items.push(format!("selection: {node_id}"));
    }
    if !reflection.style_inspector.classes.is_empty() {
        items.push(format!(
            "classes: {}",
            reflection.style_inspector.classes.join(", ")
        ));
    }
    items
}

fn build_stylesheet_items(
    inspector: &UiStyleInspectorReflectionModel,
    selector_hint: Option<String>,
) -> Vec<String> {
    let mut items = Vec::new();
    if let Some(selector_hint) = selector_hint {
        items.push(format!("selection selector: {selector_hint}"));
    }
    if !inspector.active_pseudo_states.is_empty() {
        items.push(format!(
            "states: {}",
            inspector.active_pseudo_states.join(", ")
        ));
    }
    for (path, value) in &inspector.inline_overrides {
        items.push(format!("override {path} = {value}"));
    }
    for rule in &inspector.matched_rules {
        items.push(format!(
            "{} (spec {}, order {})",
            rule.selector, rule.specificity, rule.source_order
        ));
    }
    if items.is_empty() {
        items.push("no matched stylesheet rules".to_string());
    }
    items
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

fn selected_hierarchy_index(
    document: &UiAssetDocument,
    selection: &UiDesignerSelectionModel,
) -> i32 {
    let Some(primary) = selection.primary_node_id.as_deref() else {
        return -1;
    };
    hierarchy_node_ids(document)
        .iter()
        .position(|id| id == primary)
        .map(|i| i as i32)
        .unwrap_or(-1)
}

fn hierarchy_node_ids(document: &UiAssetDocument) -> Vec<String> {
    fn visit(output: &mut Vec<String>, document: &UiAssetDocument, node_id: &str) {
        output.push(node_id.to_string());
        let Some(node) = document.nodes.get(node_id) else {
            return;
        };
        for child in &node.children {
            visit(output, document, &child.child);
        }
    }

    let mut items = Vec::new();
    if let Some(root) = &document.root {
        visit(&mut items, document, &root.node);
    }
    items
}

fn selection_for_node(document: &UiAssetDocument, node_id: &str) -> UiDesignerSelectionModel {
    let mut selection = UiDesignerSelectionModel::single(node_id.to_string());
    if let Some((parent_node_id, mount)) = parent_for_node(document, node_id) {
        selection = selection.with_parent(parent_node_id);
        if let Some(mount) = mount {
            selection = selection.with_mount(mount);
        }
    }
    selection
}

fn parent_for_node(document: &UiAssetDocument, node_id: &str) -> Option<(String, Option<String>)> {
    for (parent_id, node) in &document.nodes {
        for child in &node.children {
            if child.child == node_id {
                return Some((parent_id.clone(), child.mount.clone()));
            }
        }
    }
    None
}
