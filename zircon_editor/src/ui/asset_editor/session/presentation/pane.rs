// Final pane DTO mapping consumes the presentation domain artifacts.
use crate::ui::retained_host::ui_perf::{record_current_ui_perf_counter, UiPerfCounter};

use super::super::super::{
    presentation::UiAssetEditorPanePresentation, UiAssetEditorReflectionModel,
};
use super::super::{
    hierarchy_projection::{build_hierarchy_items, selected_hierarchy_index, selection_summary},
    style_inspection::pseudo_state_active,
    ui_asset_editor_session::UiAssetEditorSession,
};
use super::inspector::UiAssetInspectorPaneData;

#[cfg(test)]
#[path = "pane/owned_reflection_move_tests.rs"]
mod owned_reflection_move_tests;

struct OwnedPaneReflectionFields {
    asset_id: String,
    external_conflict_summary: String,
    stale_import_items: Vec<String>,
    emergency_summary: String,
    style_class_items: Vec<String>,
    last_error: String,
}

fn take_owned_pane_reflection_fields(
    reflection: &mut UiAssetEditorReflectionModel,
) -> OwnedPaneReflectionFields {
    OwnedPaneReflectionFields {
        asset_id: std::mem::take(&mut reflection.route.asset_id),
        external_conflict_summary: std::mem::take(&mut reflection.external_conflict_summary),
        stale_import_items: std::mem::take(&mut reflection.stale_import_items),
        emergency_summary: std::mem::take(&mut reflection.emergency_summary),
        style_class_items: std::mem::take(&mut reflection.style_inspector.classes),
        last_error: reflection.last_error.take().unwrap_or_default(),
    }
}

impl UiAssetEditorSession {
    pub fn pane_presentation(&self) -> UiAssetEditorPanePresentation {
        zircon_runtime::profile_scope!("editor", "asset_editor.presentation", "pane_presentation",);
        record_current_ui_perf_counter(UiPerfCounter::AssetEditorPanePresentationBuildCount, 1.0);
        let mut reflection = self.reflection_pane_presentation();
        let preview = self.preview_pane_presentation();
        let palette_entries = self.palette_catalog.entries();
        let source = self.source_pane_presentation();
        let style = self.style_pane_presentation();
        let theme = self.theme_pane_presentation();
        let commands = self.command_availability(style.has_selected_node_selector);
        let inspector =
            self.inspector_pane_presentation(&reflection, commands.can_promote_to_external_widget);
        let UiAssetInspectorPaneData {
            preview_mock_fields,
            preview_state_graph_items,
            inspector_fields,
            binding_fields,
            runtime_report,
            slot_semantic_group,
            structured_slot_semantic,
            slot_semantic_selected_index,
            slot_semantic_path,
            slot_semantic_value,
            layout_semantic_group,
            structured_layout_semantic,
            layout_semantic_selected_index,
            layout_semantic_path,
            layout_semantic_value,
            widget_prop_state_items: inspector_widget_prop_state_items,
            widget_prop_state_rows: inspector_widget_prop_state_rows,
            inspector_items,
            component_root_class_policy,
            can_edit_component_root_class_policy,
            promote_asset_id: inspector_promote_asset_id,
            promote_component_name: inspector_promote_component_name,
            promote_document_id: inspector_promote_document_id,
            can_edit_promote_draft: inspector_can_edit_promote_draft,
        } = inspector;
        let can_save = reflection.source_dirty && reflection.last_error.is_none();
        let style_state_hover = pseudo_state_active(&reflection.style_inspector, "hover");
        let style_state_focus = pseudo_state_active(&reflection.style_inspector, "focus");
        let style_state_pressed = pseudo_state_active(&reflection.style_inspector, "pressed");
        let style_state_disabled = pseudo_state_active(&reflection.style_inspector, "disabled");
        let style_state_selected = pseudo_state_active(&reflection.style_inspector, "selected");
        let selected_node_id = reflection.selection.primary_node_id.as_deref();
        let selection_summary = selection_summary(&reflection.selection);
        let hierarchy_items = build_hierarchy_items(&self.last_valid_document, selected_node_id);
        let owned_reflection = take_owned_pane_reflection_fields(&mut reflection);
        UiAssetEditorPanePresentation {
            nodes: Vec::new(),
            center_column_node: Default::default(),
            designer_panel_node: Default::default(),
            designer_canvas_panel_node: Default::default(),
            inspector_panel_node: Default::default(),
            stylesheet_panel_node: Default::default(),
            asset_id: owned_reflection.asset_id,
            mode: format!("{:?}", reflection.route.mode),
            source_dirty: reflection.source_dirty,
            has_external_conflict: reflection.has_external_conflict,
            external_conflict_summary: owned_reflection.external_conflict_summary,
            stale_import_items: owned_reflection.stale_import_items,
            can_reload_from_disk: reflection.can_reload_from_disk,
            can_keep_local_and_save: reflection.can_keep_local_and_save,
            can_save_local_copy: reflection.can_save_local_copy,
            can_open_diff_snapshot: reflection.can_open_diff_snapshot,
            can_save,
            can_undo: reflection.can_undo,
            can_redo: reflection.can_redo,
            shell_state: reflection.shell_state.label().to_string(),
            emergency_summary: owned_reflection.emergency_summary,
            can_emergency_reload: reflection.can_emergency_reload,
            can_emergency_revert: reflection.can_emergency_revert,
            can_emergency_open_asset_browser: reflection.can_emergency_open_asset_browser,
            can_insert_child: commands.can_insert_child,
            can_insert_after: commands.can_insert_after,
            can_move_up: commands.can_move_up,
            can_move_down: commands.can_move_down,
            can_reparent_into_previous: commands.can_reparent_into_previous,
            can_reparent_into_next: commands.can_reparent_into_next,
            can_reparent_outdent: commands.can_reparent_outdent,
            can_open_reference: commands.can_open_reference,
            can_convert_to_reference: commands.can_convert_to_reference,
            can_extract_component: commands.can_extract_component,
            can_promote_to_external_widget: commands.can_promote_to_external_widget,
            can_wrap_in_vertical_box: commands.can_wrap_in_vertical_box,
            can_unwrap: commands.can_unwrap,
            can_create_rule: commands.can_create_rule,
            can_extract_rule: commands.can_extract_rule,
            preview_available: reflection.preview_available,
            designer_tool_mode: reflection.designer_tool_mode.label().to_string(),
            can_designer_select: reflection.can_designer_select,
            can_designer_resize_slot: reflection.can_designer_resize_slot,
            can_designer_preview_interact: reflection.can_designer_preview_interact,
            style_state_hover,
            style_state_focus,
            style_state_pressed,
            style_state_disabled,
            style_state_selected,
            style_class_items: owned_reflection.style_class_items,
            style_rule_items: style.rule_items,
            style_rule_selected_index: style.rule_selected_index,
            style_selected_rule_id: style.selected_rule_id,
            style_selected_rule_selector: style.selected_rule_selector,
            style_can_edit_rule: style.can_edit_rule,
            style_can_delete_rule: style.can_delete_rule,
            style_matched_rule_items: style.matched_rule_items,
            style_matched_rule_selected_index: style.matched_rule_selected_index,
            style_selected_matched_rule_origin: style.selected_matched_rule_origin,
            style_selected_matched_rule_selector: style.selected_matched_rule_selector,
            style_selected_matched_rule_specificity: style.selected_matched_rule_specificity,
            style_selected_matched_rule_source_order: style.selected_matched_rule_source_order,
            style_selected_matched_rule_declaration_items: style
                .selected_matched_rule_declaration_items,
            style_rule_declaration_items: style.rule_declaration_items,
            style_rule_declaration_selected_index: style.rule_declaration_selected_index,
            style_selected_rule_declaration_path: style.selected_rule_declaration_path,
            style_selected_rule_declaration_value: style.selected_rule_declaration_value,
            style_can_edit_rule_declaration: style.can_edit_rule_declaration,
            style_can_delete_rule_declaration: style.can_delete_rule_declaration,
            style_token_items: style.token_items,
            style_token_selected_index: style.token_selected_index,
            style_selected_token_name: style.selected_token_name,
            style_selected_token_value: style.selected_token_value,
            style_can_edit_token: style.can_edit_token,
            style_can_delete_token: style.can_delete_token,
            theme_source_items: theme.source_items,
            theme_source_selected_index: theme.source_selected_index,
            theme_selected_source_reference: theme.selected_source_reference,
            theme_selected_source_kind: theme.selected_source_kind,
            theme_selected_source_token_count: theme.selected_source_token_count,
            theme_selected_source_rule_count: theme.selected_source_rule_count,
            theme_selected_source_available: theme.selected_source_available,
            theme_can_promote_local: theme.can_promote_local,
            theme_selected_source_token_items: theme.selected_source_token_items,
            theme_selected_source_rule_items: theme.selected_source_rule_items,
            theme_cascade_layer_items: theme.cascade_layer_items,
            theme_cascade_token_items: theme.cascade_token_items,
            theme_cascade_rule_items: theme.cascade_rule_items,
            theme_compare_items: theme.compare_items,
            theme_merge_preview_items: theme.merge_preview_items,
            theme_rule_helper_items: theme.rule_helper_items,
            theme_refactor_items: theme.refactor_items,
            theme_promote_asset_id: theme.promote_asset_id,
            theme_promote_document_id: theme.promote_document_id,
            theme_promote_display_name: theme.promote_display_name,
            theme_can_edit_promote_draft: theme.can_edit_promote_draft,
            theme_can_prune_duplicate_local_overrides: theme.can_prune_duplicate_local_overrides,
            last_error: owned_reflection.last_error,
            selection_summary,
            source_text: self.source_buffer.text().to_string(),
            preview_preset: reflection.route.preview_preset.label().to_string(),
            source_selected_block_label: source.selected_block_label,
            source_selected_line: source.selected_line,
            source_cursor_byte_offset: self.source_cursor_byte_offset.min(i32::MAX as usize) as i32,
            source_selected_excerpt: source.selected_excerpt,
            source_roundtrip_status: source.roundtrip_status,
            source_outline_items: source.outline_items,
            source_outline_selected_index: source.outline_selected_index,
            structured_diagnostic_items: source.structured_diagnostic_items,
            preview_surface_width: preview.surface_width,
            preview_surface_height: preview.surface_height,
            preview_canvas_items: preview.canvas_items,
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
            preview_summary: preview.summary,
            preview_interact_node_id: self
                .last_preview_interact_dispatch
                .as_ref()
                .map(|dispatch| dispatch.node_id.clone())
                .unwrap_or_default(),
            preview_interact_event: self
                .last_preview_interact_dispatch
                .as_ref()
                .map(|dispatch| dispatch.event.native_name().to_string())
                .unwrap_or_default(),
            preview_interact_route: self
                .last_preview_interact_dispatch
                .as_ref()
                .map(|dispatch| dispatch.route.clone())
                .unwrap_or_default(),
            preview_interact_action: self
                .last_preview_interact_dispatch
                .as_ref()
                .map(|dispatch| dispatch.action.clone())
                .unwrap_or_default(),
            preview_interact_side_effect: self
                .last_preview_interact_dispatch
                .as_ref()
                .map(|dispatch| format!("{:?}", dispatch.side_effect_class))
                .unwrap_or_default(),
            preview_interact_payload_items: self
                .last_preview_interact_dispatch
                .as_ref()
                .map(|dispatch| dispatch.payload_items.clone())
                .unwrap_or_default(),
            preview_interact_target_items: self
                .last_preview_interact_dispatch
                .as_ref()
                .map(|dispatch| dispatch.target_items.clone())
                .unwrap_or_default(),
            action_policy_items: runtime_report.action_policy_items,
            capability_explanation_items: runtime_report.capability_explanation_items,
            host_enforcement_items: runtime_report.host_enforcement_items,
            unsafe_action_guidance_items: runtime_report.unsafe_action_guidance_items,
            locale_preview_items: runtime_report.locale_preview_items,
            locale_preview_selected_locale: runtime_report.locale_preview_selected_locale,
            locale_preview_selected_index: runtime_report.locale_preview_selected_index,
            locale_dependency_items: runtime_report.locale_dependency_items,
            locale_extraction_items: runtime_report.locale_extraction_items,
            locale_diagnostic_items: runtime_report.locale_diagnostic_items,
            resource_dependency_items: runtime_report.resource_dependency_items,
            resource_diagnostic_items: runtime_report.resource_diagnostic_items,
            palette_selected_index: self
                .selected_palette_index
                .map(|index| index as i32)
                .unwrap_or(-1),
            palette_drag_target_preview_index: preview.palette_drag_target_preview_index,
            palette_drag_target_action: preview.palette_drag_target_action,
            palette_drag_target_label: preview.palette_drag_target_label,
            palette_drag_slot_target_items: preview.palette_drag_slot_target_items,
            palette_drag_candidate_items: preview.palette_drag_candidate_items,
            palette_drag_candidate_selected_index: preview.palette_drag_candidate_selected_index,
            palette_target_chooser_active: preview.palette_target_chooser_active,
            hierarchy_selected_index: selected_hierarchy_index(
                &self.last_valid_document,
                &self.selection,
            ),
            preview_selected_index: preview.selected_index,
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
            inspector_slot_semantic_selected_index: slot_semantic_selected_index,
            inspector_slot_semantic_path: slot_semantic_path,
            inspector_slot_semantic_value: slot_semantic_value,
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
            inspector_layout_semantic_selected_index: layout_semantic_selected_index,
            inspector_layout_semantic_path: layout_semantic_path,
            inspector_layout_semantic_value: layout_semantic_value,
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
            inspector_component_root_class_policy: component_root_class_policy,
            inspector_can_edit_control_id: inspector_fields.can_edit_control_id,
            inspector_can_edit_text_prop: inspector_fields.can_edit_text_prop,
            inspector_can_edit_component_root_class_policy: can_edit_component_root_class_policy,
            inspector_promote_asset_id,
            inspector_promote_component_name,
            inspector_promote_document_id,
            inspector_can_edit_promote_draft,
            inspector_widget_prop_state_items,
            inspector_widget_prop_state_rows,
            palette_items: palette_entries
                .iter()
                .map(|entry| entry.label.clone())
                .collect(),
            hierarchy_items,
            inspector_items,
            stylesheet_items: style.stylesheet_items,
            preview_items: preview.items,
        }
    }
}
