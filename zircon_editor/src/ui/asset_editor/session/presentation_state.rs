use crate::ui::asset_editor::UiAssetEditorReflectionModel;
use zircon_runtime::ui::template::UiAssetDocument;

use super::{
    binding_inspector::build_binding_fields,
    hierarchy_projection::{
        build_hierarchy_items, build_inspector_items, selected_hierarchy_index, selection_summary,
    },
    inspector_fields::build_inspector_fields,
    inspector_semantics::{
        build_layout_semantic_group, build_slot_semantic_group,
        build_structured_layout_semantic_fields, build_structured_slot_semantic_fields,
    },
    palette_drop::{
        build_palette_drag_slot_target_overlays,
        can_insert_palette_item_for_node as can_insert_palette_item_at_node,
    },
    palette_target_chooser::UiAssetPaletteTargetChooser,
    presentation::{
        UiAssetEditorPanePresentation, UiAssetEditorPreviewCanvasNode,
        UiAssetEditorPreviewCanvasSlotTarget,
    },
    preview_host::UiAssetPreviewHost,
    preview_mock::{build_preview_mock_fields, build_preview_state_graph_items},
    preview_projection::build_preview_projection,
    promote_widget::can_promote_selected_component_to_external_widget,
    source_sync::{build_source_outline, build_source_selection_summary},
    style_inspection::{
        build_stylesheet_items, local_style_rule_entries, local_style_token_entries,
        matched_style_rule_entries_for_selection, pseudo_state_active,
        selected_node_has_inline_overrides, selected_node_selector, MatchedStyleRuleEntry,
    },
    style_rule_declarations::declaration_entries,
    theme_authoring::{
        build_imported_theme_local_merge_preview, build_theme_refactor_items,
        build_theme_rule_helper_items, can_prune_duplicate_local_theme_overrides,
    },
    theme_cascade_inspection::build_theme_cascade_inspection,
    theme_compare::build_theme_compare_items,
    theme_summary::{build_theme_source_details, build_theme_summary},
    tree_editing::{
        build_palette_entries, can_convert_selected_node_to_reference,
        can_extract_selected_node_to_component, move_selected_node,
        reparent_selected_node as tree_reparent_selected_node, unwrap_selected_node,
        wrap_selected_node, PaletteInsertMode, UiTreeMoveDirection, UiTreeReparentDirection,
    },
    ui_asset_editor_session::UiAssetEditorSession,
};

impl UiAssetEditorSession {
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
            nodes: Vec::new(),
            center_column_node: Default::default(),
            designer_panel_node: Default::default(),
            designer_canvas_panel_node: Default::default(),
            inspector_panel_node: Default::default(),
            stylesheet_panel_node: Default::default(),
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
}

fn can_apply_tree_document_edit(
    document: &UiAssetDocument,
    edit: impl FnOnce(&mut UiAssetDocument) -> bool,
) -> bool {
    let mut document = document.clone();
    edit(&mut document)
}

fn palette_insert_mode_action(mode: PaletteInsertMode) -> &'static str {
    match mode {
        PaletteInsertMode::Child => "palette.insert.child",
        PaletteInsertMode::After => "palette.insert.after",
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
