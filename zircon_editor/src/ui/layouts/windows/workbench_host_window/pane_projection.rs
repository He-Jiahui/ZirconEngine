use super::*;
use crate::ui::layouts::common::model_rc;
use crate::ui::layouts::views::{blank_viewport_chrome, scene_viewport_chrome};
use crate::ui::slint_host::{
    AnimationEditorPaneData, UiAssetActionStateData, UiAssetCanvasNodeData,
    UiAssetCanvasSlotTargetData, UiAssetCollectionPanelData, UiAssetEditorPaneData,
    UiAssetInspectorBindingData, UiAssetInspectorLayoutData, UiAssetInspectorPanelData,
    UiAssetInspectorSemanticData, UiAssetInspectorSlotData, UiAssetInspectorWidgetData,
    UiAssetMatchedStyleRuleData, UiAssetPaletteDragData, UiAssetPaneHeaderData,
    UiAssetPreviewCanvasData, UiAssetPreviewMockData, UiAssetPreviewPanelData,
    UiAssetSourceDetailData, UiAssetSourcePanelData, UiAssetStringSelectionData,
    UiAssetStylePanelData, UiAssetStyleRuleData, UiAssetStyleRuleDeclarationData,
    UiAssetStyleStateData, UiAssetStyleTokenData, UiAssetThemeSourceData,
};
use crate::ui::widgets::common::drawer_slot_key;

pub(super) fn side_pane(
    model: &WorkbenchViewModel,
    chrome: &EditorChromeSnapshot,
    slots: &[ActivityDrawerSlot],
    ui_asset_panes: &std::collections::BTreeMap<
        String,
        crate::ui::asset_editor::UiAssetEditorPanePresentation,
    >,
    animation_panes: &std::collections::BTreeMap<
        String,
        crate::ui::animation_editor::AnimationEditorPanePresentation,
    >,
) -> PaneData {
    let stack = slots
        .iter()
        .filter_map(|slot| model.tool_windows.get(slot))
        .find(|stack| {
            stack.mode != crate::ui::workbench::layout::ActivityDrawerMode::Collapsed
                && stack.active_tab.is_some()
                && !stack.tabs.is_empty()
        })
        .or_else(|| {
            slots
                .iter()
                .filter_map(|slot| model.tool_windows.get(slot))
                .find(|stack| stack.active_tab.is_some() && !stack.tabs.is_empty())
        })
        .or_else(|| {
            slots
                .iter()
                .filter_map(|slot| model.tool_windows.get(slot))
                .find(|stack| !stack.tabs.is_empty())
        });

    let Some(stack) = stack else {
        return blank_pane();
    };
    let tab = stack
        .tabs
        .iter()
        .find(|tab| tab.active)
        .or_else(|| stack.tabs.first());
    let Some(tab) = tab else {
        return blank_pane();
    };
    pane_from_tab(
        &tab.instance_id.0,
        drawer_slot_key(stack.slot),
        &tab.title,
        &tab.icon_key,
        tab.content_kind,
        tab.empty_state.as_ref(),
        find_tab_snapshot(chrome, &tab.instance_id.0),
        chrome,
        ui_asset_panes.get(&tab.instance_id.0),
        animation_panes.get(&tab.instance_id.0),
    )
}

pub(crate) fn document_pane(
    model: &WorkbenchViewModel,
    chrome: &EditorChromeSnapshot,
    ui_asset_panes: &std::collections::BTreeMap<
        String,
        crate::ui::asset_editor::UiAssetEditorPanePresentation,
    >,
    animation_panes: &std::collections::BTreeMap<
        String,
        crate::ui::animation_editor::AnimationEditorPanePresentation,
    >,
) -> PaneData {
    let tab = model
        .document_tabs
        .iter()
        .find(|tab| tab.active)
        .or_else(|| model.document_tabs.first());
    let Some(tab) = tab else {
        return blank_pane();
    };
    pane_from_tab(
        &tab.instance_id.0,
        "",
        &tab.title,
        &tab.icon_key,
        tab.content_kind,
        tab.empty_state.as_ref(),
        find_tab_snapshot(chrome, &tab.instance_id.0),
        chrome,
        ui_asset_panes.get(&tab.instance_id.0),
        animation_panes.get(&tab.instance_id.0),
    )
}

pub(super) fn pane_from_tab(
    instance_id: &str,
    slot: &str,
    title: &str,
    icon_key: &str,
    kind: ViewContentKind,
    empty_state: Option<&PaneEmptyStateModel>,
    snapshot: Option<&ViewTabSnapshot>,
    chrome: &EditorChromeSnapshot,
    ui_asset_pane: Option<&crate::ui::asset_editor::UiAssetEditorPanePresentation>,
    animation_pane: Option<&crate::ui::animation_editor::AnimationEditorPanePresentation>,
) -> PaneData {
    let (subtitle, info, show_toolbar) = pane_metadata(kind, snapshot, chrome);
    let viewport = match kind {
        ViewContentKind::Scene => scene_viewport_chrome(&chrome.scene_viewport_settings),
        _ => blank_viewport_chrome(),
    };
    let (
        empty_title,
        empty_body,
        primary_action_label,
        primary_action_id,
        secondary_action_label,
        secondary_action_id,
        secondary_hint,
    ) = empty_state
        .map(|state| {
            (
                state.title.clone(),
                state.body.clone(),
                state
                    .primary_action
                    .as_ref()
                    .map(|action| action.label.clone())
                    .unwrap_or_default(),
                state
                    .primary_action
                    .as_ref()
                    .map(action_id_from_model)
                    .unwrap_or_default(),
                state
                    .secondary_action
                    .as_ref()
                    .map(|action| action.label.clone())
                    .unwrap_or_default(),
                state
                    .secondary_action
                    .as_ref()
                    .map(action_id_from_model)
                    .unwrap_or_default(),
                state.secondary_hint.clone().unwrap_or_default(),
            )
        })
        .unwrap_or_else(|| {
            (
                String::new(),
                String::new(),
                String::new(),
                String::new(),
                String::new(),
                String::new(),
                String::new(),
            )
        });
    let ui_asset_pane = ui_asset_pane.cloned().unwrap_or_default();
    let animation_pane = animation_pane.cloned().unwrap_or_default();

    PaneData {
        id: instance_id.into(),
        slot: slot.into(),
        kind: pane_kind_key(kind).into(),
        title: title.into(),
        icon_key: icon_key.into(),
        subtitle: subtitle.into(),
        info: info.into(),
        show_empty: empty_state.is_some(),
        empty_title: SharedString::from(empty_title),
        empty_body: SharedString::from(empty_body),
        primary_action_label: SharedString::from(primary_action_label),
        primary_action_id: SharedString::from(primary_action_id),
        secondary_action_label: SharedString::from(secondary_action_label),
        secondary_action_id: SharedString::from(secondary_action_id),
        secondary_hint: SharedString::from(secondary_hint),
        show_toolbar,
        viewport,
        ui_asset: ui_asset_pane_data(ui_asset_pane),
        animation: animation_pane_data(animation_pane),
    }
}

pub(super) fn find_tab_snapshot<'a>(
    chrome: &'a EditorChromeSnapshot,
    instance_id: &str,
) -> Option<&'a ViewTabSnapshot> {
    for drawer in chrome.workbench.drawers.values() {
        if let Some(tab) = drawer
            .tabs
            .iter()
            .find(|tab| tab.instance_id.0.as_str() == instance_id)
        {
            return Some(tab);
        }
    }

    for page in &chrome.workbench.main_pages {
        match page {
            MainPageSnapshot::Workbench { workspace, .. } => {
                if let Some(tab) = find_in_workspace(workspace, instance_id) {
                    return Some(tab);
                }
            }
            MainPageSnapshot::Exclusive { view, .. } if view.instance_id.0 == instance_id => {
                return Some(view);
            }
            MainPageSnapshot::Exclusive { .. } => {}
        }
    }

    for window in &chrome.workbench.floating_windows {
        if let Some(tab) = find_in_workspace(&window.workspace, instance_id) {
            return Some(tab);
        }
    }

    None
}

fn shared_string_list(items: Vec<String>) -> slint::ModelRc<slint::SharedString> {
    model_rc(items.into_iter().map(SharedString::from).collect())
}

fn ui_asset_string_selection(
    items: Vec<String>,
    selected_index: i32,
) -> UiAssetStringSelectionData {
    UiAssetStringSelectionData {
        items: shared_string_list(items),
        selected_index,
    }
}

fn ui_asset_canvas_nodes(
    items: Vec<crate::ui::asset_editor::UiAssetEditorPreviewCanvasNode>,
) -> slint::ModelRc<UiAssetCanvasNodeData> {
    model_rc(
        items
            .into_iter()
            .map(|item| UiAssetCanvasNodeData {
                node_id: item.node_id.into(),
                label: item.label.into(),
                kind: item.kind.into(),
                x: item.x,
                y: item.y,
                width: item.width,
                height: item.height,
                depth: item.depth,
                z_index: item.z_index,
                selected: item.selected,
            })
            .collect(),
    )
}

fn ui_asset_canvas_slot_targets(
    items: Vec<crate::ui::asset_editor::UiAssetEditorPreviewCanvasSlotTarget>,
) -> slint::ModelRc<UiAssetCanvasSlotTargetData> {
    model_rc(
        items
            .into_iter()
            .map(|item| UiAssetCanvasSlotTargetData {
                label: item.label.into(),
                detail: item.detail.into(),
                x: item.x,
                y: item.y,
                width: item.width,
                height: item.height,
                selected: item.selected,
            })
            .collect(),
    )
}

fn ui_asset_pane_data(
    ui_asset_pane: crate::ui::asset_editor::UiAssetEditorPanePresentation,
) -> UiAssetEditorPaneData {
    UiAssetEditorPaneData {
        header: UiAssetPaneHeaderData {
            asset_id: ui_asset_pane.asset_id.into(),
            mode: ui_asset_pane.mode.into(),
            status: ui_asset_pane.last_error.into(),
            selection: ui_asset_pane.selection_summary.into(),
        },
        actions: UiAssetActionStateData {
            can_save: ui_asset_pane.can_save,
            can_undo: ui_asset_pane.can_undo,
            can_redo: ui_asset_pane.can_redo,
            can_insert_child: ui_asset_pane.can_insert_child,
            can_insert_after: ui_asset_pane.can_insert_after,
            can_move_up: ui_asset_pane.can_move_up,
            can_move_down: ui_asset_pane.can_move_down,
            can_reparent_into_previous: ui_asset_pane.can_reparent_into_previous,
            can_reparent_into_next: ui_asset_pane.can_reparent_into_next,
            can_reparent_outdent: ui_asset_pane.can_reparent_outdent,
            can_open_reference: ui_asset_pane.can_open_reference,
            can_convert_to_reference: ui_asset_pane.can_convert_to_reference,
            can_extract_component: ui_asset_pane.can_extract_component,
            can_promote_to_external_widget: ui_asset_pane.can_promote_to_external_widget,
            can_wrap_in_vertical_box: ui_asset_pane.can_wrap_in_vertical_box,
            can_unwrap: ui_asset_pane.can_unwrap,
            can_create_rule: ui_asset_pane.can_create_rule,
            can_extract_rule: ui_asset_pane.can_extract_rule,
        },
        collections: UiAssetCollectionPanelData {
            palette: ui_asset_string_selection(
                ui_asset_pane.palette_items,
                ui_asset_pane.palette_selected_index,
            ),
            hierarchy: ui_asset_string_selection(
                ui_asset_pane.hierarchy_items,
                ui_asset_pane.hierarchy_selected_index,
            ),
            preview: ui_asset_string_selection(
                ui_asset_pane.preview_items,
                ui_asset_pane.preview_selected_index,
            ),
        },
        source: UiAssetSourcePanelData {
            text: ui_asset_pane.source_text.into(),
            detail: UiAssetSourceDetailData {
                block_label: ui_asset_pane.source_selected_block_label.into(),
                selected_line: ui_asset_pane.source_selected_line,
                cursor_byte_offset: ui_asset_pane.source_cursor_byte_offset,
                selected_excerpt: ui_asset_pane.source_selected_excerpt.into(),
                roundtrip_status: ui_asset_pane.source_roundtrip_status.into(),
                outline: ui_asset_string_selection(
                    ui_asset_pane.source_outline_items,
                    ui_asset_pane.source_outline_selected_index,
                ),
            },
        },
        preview: UiAssetPreviewPanelData {
            preset: ui_asset_pane.preview_preset.into(),
            summary: ui_asset_pane.preview_summary.into(),
            available: ui_asset_pane.preview_available,
            canvas: UiAssetPreviewCanvasData {
                width: ui_asset_pane.preview_surface_width,
                height: ui_asset_pane.preview_surface_height,
                items: ui_asset_canvas_nodes(ui_asset_pane.preview_canvas_items),
            },
            mock: UiAssetPreviewMockData {
                subject_collection: ui_asset_string_selection(
                    ui_asset_pane.preview_mock_subject_items,
                    ui_asset_pane.preview_mock_subject_selected_index,
                ),
                subject_node_id: ui_asset_pane.preview_mock_subject_node_id.into(),
                collection: ui_asset_string_selection(
                    ui_asset_pane.preview_mock_items,
                    ui_asset_pane.preview_mock_selected_index,
                ),
                property: ui_asset_pane.preview_mock_property.into(),
                kind: ui_asset_pane.preview_mock_kind.into(),
                value: ui_asset_pane.preview_mock_value.into(),
                expression_result: ui_asset_pane.preview_mock_expression_result.into(),
                nested_collection: ui_asset_string_selection(
                    ui_asset_pane.preview_mock_nested_items,
                    ui_asset_pane.preview_mock_nested_selected_index,
                ),
                nested_key: ui_asset_pane.preview_mock_nested_key.into(),
                nested_kind: ui_asset_pane.preview_mock_nested_kind.into(),
                nested_value: ui_asset_pane.preview_mock_nested_value.into(),
                suggestion_collection: ui_asset_string_selection(
                    ui_asset_pane.preview_mock_suggestion_items,
                    -1,
                ),
                schema_items: shared_string_list(ui_asset_pane.preview_mock_schema_items),
                state_graph_items: shared_string_list(ui_asset_pane.preview_state_graph_items),
                can_edit: ui_asset_pane.preview_mock_can_edit,
                can_clear: ui_asset_pane.preview_mock_can_clear,
                nested_can_edit: ui_asset_pane.preview_mock_nested_can_edit,
                nested_can_add: ui_asset_pane.preview_mock_nested_can_add,
                nested_can_delete: ui_asset_pane.preview_mock_nested_can_delete,
            },
        },
        palette_drag: UiAssetPaletteDragData {
            target_preview_index: ui_asset_pane.palette_drag_target_preview_index,
            target_action: ui_asset_pane.palette_drag_target_action.into(),
            target_label: ui_asset_pane.palette_drag_target_label.into(),
            slot_target_items: ui_asset_canvas_slot_targets(
                ui_asset_pane.palette_drag_slot_target_items,
            ),
            candidate_items: shared_string_list(ui_asset_pane.palette_drag_candidate_items),
            candidate_selected_index: ui_asset_pane.palette_drag_candidate_selected_index,
            target_chooser_active: ui_asset_pane.palette_target_chooser_active,
        },
        style: UiAssetStylePanelData {
            states: UiAssetStyleStateData {
                hover: ui_asset_pane.style_state_hover,
                focus: ui_asset_pane.style_state_focus,
                pressed: ui_asset_pane.style_state_pressed,
                disabled: ui_asset_pane.style_state_disabled,
                selected: ui_asset_pane.style_state_selected,
            },
            class_items: shared_string_list(ui_asset_pane.style_class_items),
            theme_source: UiAssetThemeSourceData {
                collection: ui_asset_string_selection(
                    ui_asset_pane.theme_source_items,
                    ui_asset_pane.theme_source_selected_index,
                ),
                selected_source_reference: ui_asset_pane.theme_selected_source_reference.into(),
                selected_source_kind: ui_asset_pane.theme_selected_source_kind.into(),
                selected_source_token_count: ui_asset_pane.theme_selected_source_token_count,
                selected_source_rule_count: ui_asset_pane.theme_selected_source_rule_count,
                selected_source_available: ui_asset_pane.theme_selected_source_available,
                can_promote_local: ui_asset_pane.theme_can_promote_local,
                selected_source_token_items: shared_string_list(
                    ui_asset_pane.theme_selected_source_token_items,
                ),
                selected_source_rule_items: shared_string_list(
                    ui_asset_pane.theme_selected_source_rule_items,
                ),
                cascade_layer_items: shared_string_list(ui_asset_pane.theme_cascade_layer_items),
                cascade_token_items: shared_string_list(ui_asset_pane.theme_cascade_token_items),
                cascade_rule_items: shared_string_list(ui_asset_pane.theme_cascade_rule_items),
                compare_items: shared_string_list(ui_asset_pane.theme_compare_items),
                merge_preview_items: shared_string_list(ui_asset_pane.theme_merge_preview_items),
                rule_helper_items: shared_string_list(ui_asset_pane.theme_rule_helper_items),
                refactor_items: shared_string_list(ui_asset_pane.theme_refactor_items),
                promote_asset_id: ui_asset_pane.theme_promote_asset_id.into(),
                promote_document_id: ui_asset_pane.theme_promote_document_id.into(),
                promote_display_name: ui_asset_pane.theme_promote_display_name.into(),
                can_edit_promote_draft: ui_asset_pane.theme_can_edit_promote_draft,
                can_prune_duplicate_local_overrides: ui_asset_pane
                    .theme_can_prune_duplicate_local_overrides,
            },
            rule: UiAssetStyleRuleData {
                items: shared_string_list(ui_asset_pane.style_rule_items),
                selected_index: ui_asset_pane.style_rule_selected_index,
                selected_selector: ui_asset_pane.style_selected_rule_selector.into(),
                can_edit: ui_asset_pane.style_can_edit_rule,
                can_delete: ui_asset_pane.style_can_delete_rule,
            },
            matched_rule: UiAssetMatchedStyleRuleData {
                collection: ui_asset_string_selection(
                    ui_asset_pane.style_matched_rule_items,
                    ui_asset_pane.style_matched_rule_selected_index,
                ),
                selected_origin: ui_asset_pane.style_selected_matched_rule_origin.into(),
                selected_selector: ui_asset_pane.style_selected_matched_rule_selector.into(),
                selected_specificity: ui_asset_pane.style_selected_matched_rule_specificity,
                selected_source_order: ui_asset_pane.style_selected_matched_rule_source_order,
                selected_declaration_items: shared_string_list(
                    ui_asset_pane.style_selected_matched_rule_declaration_items,
                ),
            },
            rule_declaration: UiAssetStyleRuleDeclarationData {
                items: shared_string_list(ui_asset_pane.style_rule_declaration_items),
                selected_index: ui_asset_pane.style_rule_declaration_selected_index,
                selected_path: ui_asset_pane.style_selected_rule_declaration_path.into(),
                selected_value: ui_asset_pane.style_selected_rule_declaration_value.into(),
                can_edit: ui_asset_pane.style_can_edit_rule_declaration,
                can_delete: ui_asset_pane.style_can_delete_rule_declaration,
            },
            token: UiAssetStyleTokenData {
                items: shared_string_list(ui_asset_pane.style_token_items),
                selected_index: ui_asset_pane.style_token_selected_index,
                selected_name: ui_asset_pane.style_selected_token_name.into(),
                selected_value: ui_asset_pane.style_selected_token_value.into(),
                can_edit: ui_asset_pane.style_can_edit_token,
                can_delete: ui_asset_pane.style_can_delete_token,
            },
            can_create_rule: ui_asset_pane.can_create_rule,
            can_extract_rule: ui_asset_pane.can_extract_rule,
            stylesheet_items: shared_string_list(ui_asset_pane.stylesheet_items),
        },
        inspector: UiAssetInspectorPanelData {
            widget: UiAssetInspectorWidgetData {
                selected_node_id: ui_asset_pane.inspector_selected_node_id.into(),
                parent_node_id: ui_asset_pane.inspector_parent_node_id.into(),
                mount: ui_asset_pane.inspector_mount.into(),
                widget_kind: ui_asset_pane.inspector_widget_kind.into(),
                widget_label: ui_asset_pane.inspector_widget_label.into(),
                control_id: ui_asset_pane.inspector_control_id.into(),
                text_prop: ui_asset_pane.inspector_text_prop.into(),
                can_edit_control_id: ui_asset_pane.inspector_can_edit_control_id,
                can_edit_text_prop: ui_asset_pane.inspector_can_edit_text_prop,
                promote_asset_id: ui_asset_pane.inspector_promote_asset_id.into(),
                promote_component_name: ui_asset_pane.inspector_promote_component_name.into(),
                promote_document_id: ui_asset_pane.inspector_promote_document_id.into(),
                can_edit_promote_draft: ui_asset_pane.inspector_can_edit_promote_draft,
                items: shared_string_list(ui_asset_pane.inspector_items),
            },
            slot: UiAssetInspectorSlotData {
                padding: ui_asset_pane.inspector_slot_padding.into(),
                width_preferred: ui_asset_pane.inspector_slot_width_preferred.into(),
                height_preferred: ui_asset_pane.inspector_slot_height_preferred.into(),
                semantic: UiAssetInspectorSemanticData {
                    title: ui_asset_pane.inspector_slot_semantic_title.into(),
                    collection: ui_asset_string_selection(
                        ui_asset_pane.inspector_slot_semantic_items,
                        ui_asset_pane.inspector_slot_semantic_selected_index,
                    ),
                    path: ui_asset_pane.inspector_slot_semantic_path.into(),
                    value: ui_asset_pane.inspector_slot_semantic_value.into(),
                },
                kind: ui_asset_pane.inspector_slot_kind.into(),
                linear_main_weight: ui_asset_pane.inspector_slot_linear_main_weight.into(),
                linear_main_stretch: ui_asset_pane.inspector_slot_linear_main_stretch.into(),
                linear_cross_weight: ui_asset_pane.inspector_slot_linear_cross_weight.into(),
                linear_cross_stretch: ui_asset_pane.inspector_slot_linear_cross_stretch.into(),
                overlay_anchor_x: ui_asset_pane.inspector_slot_overlay_anchor_x.into(),
                overlay_anchor_y: ui_asset_pane.inspector_slot_overlay_anchor_y.into(),
                overlay_pivot_x: ui_asset_pane.inspector_slot_overlay_pivot_x.into(),
                overlay_pivot_y: ui_asset_pane.inspector_slot_overlay_pivot_y.into(),
                overlay_position_x: ui_asset_pane.inspector_slot_overlay_position_x.into(),
                overlay_position_y: ui_asset_pane.inspector_slot_overlay_position_y.into(),
                overlay_z_index: ui_asset_pane.inspector_slot_overlay_z_index.into(),
                grid_row: ui_asset_pane.inspector_slot_grid_row.into(),
                grid_column: ui_asset_pane.inspector_slot_grid_column.into(),
                grid_row_span: ui_asset_pane.inspector_slot_grid_row_span.into(),
                grid_column_span: ui_asset_pane.inspector_slot_grid_column_span.into(),
                flow_break_before: ui_asset_pane.inspector_slot_flow_break_before.into(),
                flow_alignment: ui_asset_pane.inspector_slot_flow_alignment.into(),
            },
            layout: UiAssetInspectorLayoutData {
                width_preferred: ui_asset_pane.inspector_layout_width_preferred.into(),
                height_preferred: ui_asset_pane.inspector_layout_height_preferred.into(),
                semantic: UiAssetInspectorSemanticData {
                    title: ui_asset_pane.inspector_layout_semantic_title.into(),
                    collection: ui_asset_string_selection(
                        ui_asset_pane.inspector_layout_semantic_items,
                        ui_asset_pane.inspector_layout_semantic_selected_index,
                    ),
                    path: ui_asset_pane.inspector_layout_semantic_path.into(),
                    value: ui_asset_pane.inspector_layout_semantic_value.into(),
                },
                kind: ui_asset_pane.inspector_layout_kind.into(),
                box_gap: ui_asset_pane.inspector_layout_box_gap.into(),
                scroll_axis: ui_asset_pane.inspector_layout_scroll_axis.into(),
                scroll_gap: ui_asset_pane.inspector_layout_scroll_gap.into(),
                scrollbar_visibility: ui_asset_pane.inspector_layout_scrollbar_visibility.into(),
                virtualization_item_extent: ui_asset_pane
                    .inspector_layout_virtualization_item_extent
                    .into(),
                virtualization_overscan: ui_asset_pane
                    .inspector_layout_virtualization_overscan
                    .into(),
                clip: ui_asset_pane.inspector_layout_clip.into(),
            },
            binding: UiAssetInspectorBindingData {
                collection: ui_asset_string_selection(
                    ui_asset_pane.inspector_binding_items,
                    ui_asset_pane.inspector_binding_selected_index,
                ),
                binding_id: ui_asset_pane.inspector_binding_id.into(),
                binding_event: ui_asset_pane.inspector_binding_event.into(),
                event_collection: ui_asset_string_selection(
                    ui_asset_pane.inspector_binding_event_items,
                    ui_asset_pane.inspector_binding_event_selected_index,
                ),
                binding_route: ui_asset_pane.inspector_binding_route.into(),
                binding_route_target: ui_asset_pane.inspector_binding_route_target.into(),
                binding_action_target: ui_asset_pane.inspector_binding_action_target.into(),
                route_suggestion_collection: ui_asset_string_selection(
                    ui_asset_pane.inspector_binding_route_suggestion_items,
                    -1,
                ),
                action_suggestion_collection: ui_asset_string_selection(
                    ui_asset_pane.inspector_binding_action_suggestion_items,
                    -1,
                ),
                action_kind_collection: ui_asset_string_selection(
                    ui_asset_pane.inspector_binding_action_kind_items,
                    ui_asset_pane.inspector_binding_action_kind_selected_index,
                ),
                payload_collection: ui_asset_string_selection(
                    ui_asset_pane.inspector_binding_payload_items,
                    ui_asset_pane.inspector_binding_payload_selected_index,
                ),
                payload_suggestion_collection: ui_asset_string_selection(
                    ui_asset_pane.inspector_binding_payload_suggestion_items,
                    -1,
                ),
                payload_key: ui_asset_pane.inspector_binding_payload_key.into(),
                payload_value: ui_asset_pane.inspector_binding_payload_value.into(),
                schema_items: shared_string_list(ui_asset_pane.inspector_binding_schema_items),
                can_edit: ui_asset_pane.inspector_can_edit_binding,
                can_delete: ui_asset_pane.inspector_can_delete_binding,
            },
        },
    }
}

fn animation_pane_data(
    animation_pane: crate::ui::animation_editor::AnimationEditorPanePresentation,
) -> AnimationEditorPaneData {
    AnimationEditorPaneData {
        mode: animation_pane.mode.into(),
        asset_path: animation_pane.asset_path.into(),
        status: animation_pane.status.into(),
        selection: animation_pane.selection_summary.into(),
        current_frame: animation_pane.current_frame as i32,
        timeline_start_frame: animation_pane.timeline_start_frame as i32,
        timeline_end_frame: animation_pane.timeline_end_frame as i32,
        playback_label: animation_pane.playback_label.into(),
        track_items: shared_string_list(animation_pane.track_items),
        parameter_items: shared_string_list(animation_pane.parameter_items),
        node_items: shared_string_list(animation_pane.node_items),
        state_items: shared_string_list(animation_pane.state_items),
        transition_items: shared_string_list(animation_pane.transition_items),
    }
}

pub(super) fn blank_pane() -> PaneData {
    PaneData {
        id: SharedString::default(),
        slot: SharedString::default(),
        kind: "Placeholder".into(),
        title: SharedString::default(),
        icon_key: SharedString::default(),
        subtitle: SharedString::default(),
        info: SharedString::default(),
        show_empty: false,
        empty_title: SharedString::default(),
        empty_body: SharedString::default(),
        primary_action_label: SharedString::default(),
        primary_action_id: SharedString::default(),
        secondary_action_label: SharedString::default(),
        secondary_action_id: SharedString::default(),
        secondary_hint: SharedString::default(),
        show_toolbar: false,
        viewport: blank_viewport_chrome(),
        ui_asset: ui_asset_pane_data(
            crate::ui::asset_editor::UiAssetEditorPanePresentation::default(),
        ),
        animation: animation_pane_data(
            crate::ui::animation_editor::AnimationEditorPanePresentation::default(),
        ),
    }
}

fn pane_metadata(
    kind: ViewContentKind,
    snapshot: Option<&ViewTabSnapshot>,
    chrome: &EditorChromeSnapshot,
) -> (String, String, bool) {
    match kind {
        ViewContentKind::Welcome => (
            chrome.welcome.subtitle.clone(),
            chrome.welcome.status_message.clone(),
            false,
        ),
        ViewContentKind::Project => (
            if chrome.project_overview.project_name.is_empty() {
                chrome.project_path.clone()
            } else {
                chrome.project_overview.project_name.clone()
            },
            format!(
                "{} folders • {} assets",
                chrome.project_overview.folder_count, chrome.project_overview.asset_count
            ),
            false,
        ),
        ViewContentKind::Assets => (
            chrome
                .asset_activity
                .selected_folder_id
                .clone()
                .unwrap_or_else(|| "res://".to_string()),
            format!(
                "{} folders • {} assets",
                chrome.asset_activity.visible_folders.len(),
                chrome.asset_activity.visible_assets.len()
            ),
            false,
        ),
        ViewContentKind::Hierarchy => (
            format!("{} nodes", chrome.scene_entries.len()),
            "Hierarchy selection drives Scene and Inspector".to_string(),
            false,
        ),
        ViewContentKind::Inspector => (
            "Selection Inspector".to_string(),
            chrome
                .inspector
                .as_ref()
                .map(|inspector| format!("Node {}", inspector.id))
                .unwrap_or_default(),
            false,
        ),
        ViewContentKind::Scene => (
            format!("{} x {}", chrome.viewport_size.x, chrome.viewport_size.y),
            String::new(),
            true,
        ),
        ViewContentKind::Game => (
            format!("{} x {}", chrome.viewport_size.x, chrome.viewport_size.y),
            String::new(),
            false,
        ),
        ViewContentKind::Console => ("Task Output".to_string(), chrome.status_line.clone(), false),
        ViewContentKind::PrefabEditor => (
            payload_path(snapshot).unwrap_or_else(|| "Prefab Workspace".to_string()),
            "Prefab editor host slot is ready. Asset-specific tooling is still placeholder.".into(),
            false,
        ),
        ViewContentKind::AssetBrowser => (
            chrome.asset_browser.project_name.clone(),
            format!(
                "{} folders • {} assets",
                chrome.asset_browser.visible_folders.len(),
                chrome.asset_browser.visible_assets.len()
            ),
            false,
        ),
        ViewContentKind::UiAssetEditor => (
            payload_path(snapshot).unwrap_or_else(|| "UI Asset Editor".to_string()),
            "Live UI asset preview and source editing session".to_string(),
            false,
        ),
        ViewContentKind::AnimationSequenceEditor => (
            payload_path(snapshot).unwrap_or_else(|| "Animation Sequence".to_string()),
            "Sequence timeline and property-track authoring".to_string(),
            false,
        ),
        ViewContentKind::AnimationGraphEditor => (
            payload_path(snapshot).unwrap_or_else(|| "Animation Graph".to_string()),
            "Graph and state-machine authoring surface".to_string(),
            false,
        ),
        ViewContentKind::Placeholder => (
            "Missing View".to_string(),
            "This pane was restored from layout state but the descriptor is unavailable.".into(),
            false,
        ),
    }
}

fn payload_path(snapshot: Option<&ViewTabSnapshot>) -> Option<String> {
    snapshot
        .and_then(|view| {
            view.serializable_payload
                .get("path")
                .or_else(|| view.serializable_payload.get("asset_id"))
        })
        .and_then(|value| value.as_str())
        .map(str::to_string)
}

fn action_id_from_model(action: &PaneActionModel) -> String {
    match action.binding.as_ref().map(EditorUiBinding::payload) {
        Some(EditorUiBindingPayload::MenuAction { action_id }) => action_id.clone(),
        _ => String::new(),
    }
}

fn pane_kind_key(kind: ViewContentKind) -> &'static str {
    match kind {
        ViewContentKind::Welcome => "Welcome",
        ViewContentKind::Project => "Project",
        ViewContentKind::Hierarchy => "Hierarchy",
        ViewContentKind::Inspector => "Inspector",
        ViewContentKind::Scene => "Scene",
        ViewContentKind::Game => "Game",
        ViewContentKind::Assets => "Assets",
        ViewContentKind::Console => "Console",
        ViewContentKind::PrefabEditor => "PrefabEditor",
        ViewContentKind::AssetBrowser => "AssetBrowser",
        ViewContentKind::UiAssetEditor => "UiAssetEditor",
        ViewContentKind::AnimationSequenceEditor => "AnimationSequenceEditor",
        ViewContentKind::AnimationGraphEditor => "AnimationGraphEditor",
        ViewContentKind::Placeholder => "Placeholder",
    }
}

fn find_in_workspace<'a>(
    workspace: &'a crate::ui::workbench::snapshot::DocumentWorkspaceSnapshot,
    instance_id: &str,
) -> Option<&'a ViewTabSnapshot> {
    match workspace {
        crate::ui::workbench::snapshot::DocumentWorkspaceSnapshot::Split {
            first, second, ..
        } => {
            find_in_workspace(first, instance_id).or_else(|| find_in_workspace(second, instance_id))
        }
        crate::ui::workbench::snapshot::DocumentWorkspaceSnapshot::Tabs { tabs, .. } => tabs
            .iter()
            .find(|tab| tab.instance_id.0.as_str() == instance_id),
    }
}
