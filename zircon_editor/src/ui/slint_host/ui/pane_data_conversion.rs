use crate::ui::asset_editor;
use crate::ui::layouts::common::model_rc;
use crate::ui::layouts::views::{ViewTemplateFrameData, ViewTemplateNodeData};
use crate::ui::slint_host as slint_ui;
use slint::{Model, ModelRc};

fn map_model_rc<T, U, F>(model: &ModelRc<T>, mut map: F) -> ModelRc<U>
where
    T: Clone + 'static,
    U: Clone + 'static,
    F: FnMut(T) -> U,
{
    model_rc(
        (0..model.row_count())
            .filter_map(|row| model.row_data(row))
            .map(&mut map)
            .collect(),
    )
}

fn to_slint_scene_node(
    node: crate::ui::layouts::windows::workbench_host_window::SceneNodeData,
) -> slint_ui::SceneNodeData {
    slint_ui::SceneNodeData {
        id: node.id,
        name: node.name,
        depth: node.depth,
        selected: node.selected,
    }
}

fn to_slint_scene_nodes(
    nodes: &ModelRc<crate::ui::layouts::windows::workbench_host_window::SceneNodeData>,
) -> ModelRc<slint_ui::SceneNodeData> {
    map_model_rc(nodes, to_slint_scene_node)
}

pub(super) fn to_slint_hierarchy_pane(
    data: crate::ui::layouts::windows::workbench_host_window::HierarchyPaneViewData,
) -> slint_ui::HierarchyPaneData {
    slint_ui::HierarchyPaneData {
        nodes: map_model_rc(&data.nodes, to_slint_template_node),
        hierarchy_nodes: to_slint_scene_nodes(&data.hierarchy_nodes),
    }
}

pub(super) fn to_slint_inspector_pane(
    data: crate::ui::layouts::windows::workbench_host_window::InspectorPaneViewData,
) -> slint_ui::InspectorPaneData {
    slint_ui::InspectorPaneData {
        nodes: map_model_rc(&data.nodes, to_slint_template_node),
        info: data.info,
        inspector_name: data.inspector_name,
        inspector_parent: data.inspector_parent,
        inspector_x: data.inspector_x,
        inspector_y: data.inspector_y,
        inspector_z: data.inspector_z,
        delete_enabled: data.delete_enabled,
    }
}

pub(super) fn to_slint_console_pane(
    data: crate::ui::layouts::windows::workbench_host_window::ConsolePaneViewData,
) -> slint_ui::ConsolePaneData {
    slint_ui::ConsolePaneData {
        nodes: map_model_rc(&data.nodes, to_slint_template_node),
        status_text: data.status_text,
    }
}

pub(super) fn to_slint_assets_activity_pane(
    data: crate::ui::layouts::windows::workbench_host_window::AssetsActivityPaneViewData,
) -> slint_ui::AssetsActivityPaneData {
    slint_ui::AssetsActivityPaneData {
        nodes: map_model_rc(&data.nodes, to_slint_template_node),
    }
}

pub(super) fn to_slint_animation_editor_pane(
    data: crate::ui::layouts::windows::workbench_host_window::AnimationEditorPaneViewData,
) -> slint_ui::AnimationEditorPaneData {
    slint_ui::AnimationEditorPaneData {
        nodes: map_model_rc(&data.nodes, to_slint_template_node),
        mode: data.mode,
        asset_path: data.asset_path,
        status: data.status,
        selection: data.selection,
        current_frame: data.current_frame,
        timeline_start_frame: data.timeline_start_frame,
        timeline_end_frame: data.timeline_end_frame,
        playback_label: data.playback_label,
        track_items: data.track_items,
        parameter_items: data.parameter_items,
        node_items: data.node_items,
        state_items: data.state_items,
        transition_items: data.transition_items,
    }
}

fn to_slint_shared_string_list(items: Vec<String>) -> ModelRc<slint::SharedString> {
    model_rc(items.into_iter().map(slint::SharedString::from).collect())
}

pub(super) fn to_slint_asset_browser_pane(
    data: crate::ui::layouts::windows::workbench_host_window::AssetBrowserPaneViewData,
) -> slint_ui::AssetBrowserPaneData {
    slint_ui::AssetBrowserPaneData {
        nodes: map_model_rc(&data.nodes, to_slint_template_node),
    }
}

fn to_slint_template_frame(frame: ViewTemplateFrameData) -> slint_ui::TemplateNodeFrameData {
    slint_ui::TemplateNodeFrameData {
        x: frame.x,
        y: frame.y,
        width: frame.width,
        height: frame.height,
    }
}

fn to_slint_template_node(data: ViewTemplateNodeData) -> slint_ui::TemplatePaneNodeData {
    slint_ui::TemplatePaneNodeData {
        node_id: data.node_id,
        control_id: data.control_id,
        role: data.role,
        text: data.text,
        dispatch_kind: data.dispatch_kind,
        action_id: data.action_id,
        surface_variant: data.surface_variant,
        text_tone: data.text_tone,
        button_variant: data.button_variant,
        font_size: data.font_size,
        font_weight: data.font_weight,
        text_align: data.text_align,
        overflow: data.overflow,
        corner_radius: data.corner_radius,
        border_width: data.border_width,
        frame: to_slint_template_frame(data.frame),
    }
}

fn to_slint_template_nodes(items: Vec<ViewTemplateNodeData>) -> ModelRc<slint_ui::TemplatePaneNodeData> {
    model_rc(items.into_iter().map(to_slint_template_node).collect())
}

pub(super) fn to_slint_project_overview_pane(
    data: crate::ui::layouts::windows::workbench_host_window::ProjectOverviewPaneViewData,
) -> slint_ui::ProjectOverviewPaneData {
    slint_ui::ProjectOverviewPaneData {
        nodes: map_model_rc(&data.nodes, to_slint_template_node),
    }
}

fn to_slint_ui_asset_string_selection(
    items: Vec<String>,
    selected_index: i32,
) -> slint_ui::UiAssetStringSelectionData {
    slint_ui::UiAssetStringSelectionData {
        items: to_slint_shared_string_list(items),
        selected_index,
    }
}

fn to_slint_ui_asset_canvas_nodes(
    items: Vec<asset_editor::UiAssetEditorPreviewCanvasNode>,
) -> ModelRc<slint_ui::UiAssetCanvasNodeData> {
    model_rc(
        items
            .into_iter()
            .map(|item| slint_ui::UiAssetCanvasNodeData {
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

fn to_slint_ui_asset_canvas_slot_targets(
    items: Vec<asset_editor::UiAssetEditorPreviewCanvasSlotTarget>,
) -> ModelRc<slint_ui::UiAssetCanvasSlotTargetData> {
    model_rc(
        items
            .into_iter()
            .map(|item| slint_ui::UiAssetCanvasSlotTargetData {
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

pub(super) fn to_slint_ui_asset_pane(
    data: asset_editor::UiAssetEditorPanePresentation,
) -> slint_ui::UiAssetEditorPaneData {
    slint_ui::UiAssetEditorPaneData {
        nodes: to_slint_template_nodes(data.nodes),
        center_column_node: to_slint_template_node(data.center_column_node),
        designer_panel_node: to_slint_template_node(data.designer_panel_node),
        designer_canvas_panel_node: to_slint_template_node(data.designer_canvas_panel_node),
        inspector_panel_node: to_slint_template_node(data.inspector_panel_node),
        stylesheet_panel_node: to_slint_template_node(data.stylesheet_panel_node),
        header: slint_ui::UiAssetPaneHeaderData {
            asset_id: data.asset_id.into(),
            mode: data.mode.into(),
            status: data.last_error.into(),
            selection: data.selection_summary.into(),
        },
        actions: slint_ui::UiAssetActionStateData {
            can_save: data.can_save,
            can_undo: data.can_undo,
            can_redo: data.can_redo,
            can_insert_child: data.can_insert_child,
            can_insert_after: data.can_insert_after,
            can_move_up: data.can_move_up,
            can_move_down: data.can_move_down,
            can_reparent_into_previous: data.can_reparent_into_previous,
            can_reparent_into_next: data.can_reparent_into_next,
            can_reparent_outdent: data.can_reparent_outdent,
            can_open_reference: data.can_open_reference,
            can_convert_to_reference: data.can_convert_to_reference,
            can_extract_component: data.can_extract_component,
            can_promote_to_external_widget: data.can_promote_to_external_widget,
            can_wrap_in_vertical_box: data.can_wrap_in_vertical_box,
            can_unwrap: data.can_unwrap,
            can_create_rule: data.can_create_rule,
            can_extract_rule: data.can_extract_rule,
        },
        collections: slint_ui::UiAssetCollectionPanelData {
            palette: to_slint_ui_asset_string_selection(
                data.palette_items,
                data.palette_selected_index,
            ),
            hierarchy: to_slint_ui_asset_string_selection(
                data.hierarchy_items,
                data.hierarchy_selected_index,
            ),
            preview: to_slint_ui_asset_string_selection(
                data.preview_items,
                data.preview_selected_index,
            ),
        },
        source: slint_ui::UiAssetSourcePanelData {
            text: data.source_text.into(),
            detail: slint_ui::UiAssetSourceDetailData {
                block_label: data.source_selected_block_label.into(),
                selected_line: data.source_selected_line,
                cursor_byte_offset: data.source_cursor_byte_offset,
                selected_excerpt: data.source_selected_excerpt.into(),
                roundtrip_status: data.source_roundtrip_status.into(),
                outline: to_slint_ui_asset_string_selection(
                    data.source_outline_items,
                    data.source_outline_selected_index,
                ),
            },
        },
        preview: slint_ui::UiAssetPreviewPanelData {
            preset: data.preview_preset.into(),
            summary: data.preview_summary.into(),
            available: data.preview_available,
            canvas: slint_ui::UiAssetPreviewCanvasData {
                width: data.preview_surface_width,
                height: data.preview_surface_height,
                items: to_slint_ui_asset_canvas_nodes(data.preview_canvas_items),
            },
            mock: slint_ui::UiAssetPreviewMockData {
                subject_collection: to_slint_ui_asset_string_selection(
                    data.preview_mock_subject_items,
                    data.preview_mock_subject_selected_index,
                ),
                subject_node_id: data.preview_mock_subject_node_id.into(),
                collection: to_slint_ui_asset_string_selection(
                    data.preview_mock_items,
                    data.preview_mock_selected_index,
                ),
                property: data.preview_mock_property.into(),
                kind: data.preview_mock_kind.into(),
                value: data.preview_mock_value.into(),
                expression_result: data.preview_mock_expression_result.into(),
                nested_collection: to_slint_ui_asset_string_selection(
                    data.preview_mock_nested_items,
                    data.preview_mock_nested_selected_index,
                ),
                nested_key: data.preview_mock_nested_key.into(),
                nested_kind: data.preview_mock_nested_kind.into(),
                nested_value: data.preview_mock_nested_value.into(),
                suggestion_collection: to_slint_ui_asset_string_selection(
                    data.preview_mock_suggestion_items,
                    -1,
                ),
                schema_items: to_slint_shared_string_list(data.preview_mock_schema_items),
                state_graph_items: to_slint_shared_string_list(data.preview_state_graph_items),
                can_edit: data.preview_mock_can_edit,
                can_clear: data.preview_mock_can_clear,
                nested_can_edit: data.preview_mock_nested_can_edit,
                nested_can_add: data.preview_mock_nested_can_add,
                nested_can_delete: data.preview_mock_nested_can_delete,
            },
        },
        palette_drag: slint_ui::UiAssetPaletteDragData {
            target_preview_index: data.palette_drag_target_preview_index,
            target_action: data.palette_drag_target_action.into(),
            target_label: data.palette_drag_target_label.into(),
            slot_target_items: to_slint_ui_asset_canvas_slot_targets(
                data.palette_drag_slot_target_items,
            ),
            candidate_items: to_slint_shared_string_list(data.palette_drag_candidate_items),
            candidate_selected_index: data.palette_drag_candidate_selected_index,
            target_chooser_active: data.palette_target_chooser_active,
        },
        style: slint_ui::UiAssetStylePanelData {
            states: slint_ui::UiAssetStyleStateData {
                hover: data.style_state_hover,
                focus: data.style_state_focus,
                pressed: data.style_state_pressed,
                disabled: data.style_state_disabled,
                selected: data.style_state_selected,
            },
            class_items: to_slint_shared_string_list(data.style_class_items),
            theme_source: slint_ui::UiAssetThemeSourceData {
                collection: to_slint_ui_asset_string_selection(
                    data.theme_source_items,
                    data.theme_source_selected_index,
                ),
                selected_source_reference: data.theme_selected_source_reference.into(),
                selected_source_kind: data.theme_selected_source_kind.into(),
                selected_source_token_count: data.theme_selected_source_token_count,
                selected_source_rule_count: data.theme_selected_source_rule_count,
                selected_source_available: data.theme_selected_source_available,
                can_promote_local: data.theme_can_promote_local,
                selected_source_token_items: to_slint_shared_string_list(
                    data.theme_selected_source_token_items,
                ),
                selected_source_rule_items: to_slint_shared_string_list(
                    data.theme_selected_source_rule_items,
                ),
                cascade_layer_items: to_slint_shared_string_list(data.theme_cascade_layer_items),
                cascade_token_items: to_slint_shared_string_list(data.theme_cascade_token_items),
                cascade_rule_items: to_slint_shared_string_list(data.theme_cascade_rule_items),
                compare_items: to_slint_shared_string_list(data.theme_compare_items),
                merge_preview_items: to_slint_shared_string_list(data.theme_merge_preview_items),
                rule_helper_items: to_slint_shared_string_list(data.theme_rule_helper_items),
                refactor_items: to_slint_shared_string_list(data.theme_refactor_items),
                promote_asset_id: data.theme_promote_asset_id.into(),
                promote_document_id: data.theme_promote_document_id.into(),
                promote_display_name: data.theme_promote_display_name.into(),
                can_edit_promote_draft: data.theme_can_edit_promote_draft,
                can_prune_duplicate_local_overrides: data.theme_can_prune_duplicate_local_overrides,
            },
            rule: slint_ui::UiAssetStyleRuleData {
                items: to_slint_shared_string_list(data.style_rule_items),
                selected_index: data.style_rule_selected_index,
                selected_selector: data.style_selected_rule_selector.into(),
                can_edit: data.style_can_edit_rule,
                can_delete: data.style_can_delete_rule,
            },
            matched_rule: slint_ui::UiAssetMatchedStyleRuleData {
                collection: to_slint_ui_asset_string_selection(
                    data.style_matched_rule_items,
                    data.style_matched_rule_selected_index,
                ),
                selected_origin: data.style_selected_matched_rule_origin.into(),
                selected_selector: data.style_selected_matched_rule_selector.into(),
                selected_specificity: data.style_selected_matched_rule_specificity,
                selected_source_order: data.style_selected_matched_rule_source_order,
                selected_declaration_items: to_slint_shared_string_list(
                    data.style_selected_matched_rule_declaration_items,
                ),
            },
            rule_declaration: slint_ui::UiAssetStyleRuleDeclarationData {
                items: to_slint_shared_string_list(data.style_rule_declaration_items),
                selected_index: data.style_rule_declaration_selected_index,
                selected_path: data.style_selected_rule_declaration_path.into(),
                selected_value: data.style_selected_rule_declaration_value.into(),
                can_edit: data.style_can_edit_rule_declaration,
                can_delete: data.style_can_delete_rule_declaration,
            },
            token: slint_ui::UiAssetStyleTokenData {
                items: to_slint_shared_string_list(data.style_token_items),
                selected_index: data.style_token_selected_index,
                selected_name: data.style_selected_token_name.into(),
                selected_value: data.style_selected_token_value.into(),
                can_edit: data.style_can_edit_token,
                can_delete: data.style_can_delete_token,
            },
            can_create_rule: data.can_create_rule,
            can_extract_rule: data.can_extract_rule,
            stylesheet_items: to_slint_shared_string_list(data.stylesheet_items),
        },
        inspector: slint_ui::UiAssetInspectorPanelData {
            widget: slint_ui::UiAssetInspectorWidgetData {
                selected_node_id: data.inspector_selected_node_id.into(),
                parent_node_id: data.inspector_parent_node_id.into(),
                mount: data.inspector_mount.into(),
                widget_kind: data.inspector_widget_kind.into(),
                widget_label: data.inspector_widget_label.into(),
                control_id: data.inspector_control_id.into(),
                text_prop: data.inspector_text_prop.into(),
                can_edit_control_id: data.inspector_can_edit_control_id,
                can_edit_text_prop: data.inspector_can_edit_text_prop,
                promote_asset_id: data.inspector_promote_asset_id.into(),
                promote_component_name: data.inspector_promote_component_name.into(),
                promote_document_id: data.inspector_promote_document_id.into(),
                can_edit_promote_draft: data.inspector_can_edit_promote_draft,
                items: to_slint_shared_string_list(data.inspector_items),
            },
            slot: slint_ui::UiAssetInspectorSlotData {
                padding: data.inspector_slot_padding.into(),
                width_preferred: data.inspector_slot_width_preferred.into(),
                height_preferred: data.inspector_slot_height_preferred.into(),
                semantic: slint_ui::UiAssetInspectorSemanticData {
                    title: data.inspector_slot_semantic_title.into(),
                    collection: to_slint_ui_asset_string_selection(
                        data.inspector_slot_semantic_items,
                        data.inspector_slot_semantic_selected_index,
                    ),
                    path: data.inspector_slot_semantic_path.into(),
                    value: data.inspector_slot_semantic_value.into(),
                },
                kind: data.inspector_slot_kind.into(),
                linear_main_weight: data.inspector_slot_linear_main_weight.into(),
                linear_main_stretch: data.inspector_slot_linear_main_stretch.into(),
                linear_cross_weight: data.inspector_slot_linear_cross_weight.into(),
                linear_cross_stretch: data.inspector_slot_linear_cross_stretch.into(),
                overlay_anchor_x: data.inspector_slot_overlay_anchor_x.into(),
                overlay_anchor_y: data.inspector_slot_overlay_anchor_y.into(),
                overlay_pivot_x: data.inspector_slot_overlay_pivot_x.into(),
                overlay_pivot_y: data.inspector_slot_overlay_pivot_y.into(),
                overlay_position_x: data.inspector_slot_overlay_position_x.into(),
                overlay_position_y: data.inspector_slot_overlay_position_y.into(),
                overlay_z_index: data.inspector_slot_overlay_z_index.into(),
                grid_row: data.inspector_slot_grid_row.into(),
                grid_column: data.inspector_slot_grid_column.into(),
                grid_row_span: data.inspector_slot_grid_row_span.into(),
                grid_column_span: data.inspector_slot_grid_column_span.into(),
                flow_break_before: data.inspector_slot_flow_break_before.into(),
                flow_alignment: data.inspector_slot_flow_alignment.into(),
            },
            layout: slint_ui::UiAssetInspectorLayoutData {
                width_preferred: data.inspector_layout_width_preferred.into(),
                height_preferred: data.inspector_layout_height_preferred.into(),
                semantic: slint_ui::UiAssetInspectorSemanticData {
                    title: data.inspector_layout_semantic_title.into(),
                    collection: to_slint_ui_asset_string_selection(
                        data.inspector_layout_semantic_items,
                        data.inspector_layout_semantic_selected_index,
                    ),
                    path: data.inspector_layout_semantic_path.into(),
                    value: data.inspector_layout_semantic_value.into(),
                },
                kind: data.inspector_layout_kind.into(),
                box_gap: data.inspector_layout_box_gap.into(),
                scroll_axis: data.inspector_layout_scroll_axis.into(),
                scroll_gap: data.inspector_layout_scroll_gap.into(),
                scrollbar_visibility: data.inspector_layout_scrollbar_visibility.into(),
                virtualization_item_extent: data.inspector_layout_virtualization_item_extent.into(),
                virtualization_overscan: data.inspector_layout_virtualization_overscan.into(),
                clip: data.inspector_layout_clip.into(),
            },
            binding: slint_ui::UiAssetInspectorBindingData {
                collection: to_slint_ui_asset_string_selection(
                    data.inspector_binding_items,
                    data.inspector_binding_selected_index,
                ),
                binding_id: data.inspector_binding_id.into(),
                binding_event: data.inspector_binding_event.into(),
                event_collection: to_slint_ui_asset_string_selection(
                    data.inspector_binding_event_items,
                    data.inspector_binding_event_selected_index,
                ),
                binding_route: data.inspector_binding_route.into(),
                binding_route_target: data.inspector_binding_route_target.into(),
                binding_action_target: data.inspector_binding_action_target.into(),
                route_suggestion_collection: to_slint_ui_asset_string_selection(
                    data.inspector_binding_route_suggestion_items,
                    -1,
                ),
                action_suggestion_collection: to_slint_ui_asset_string_selection(
                    data.inspector_binding_action_suggestion_items,
                    -1,
                ),
                action_kind_collection: to_slint_ui_asset_string_selection(
                    data.inspector_binding_action_kind_items,
                    data.inspector_binding_action_kind_selected_index,
                ),
                payload_collection: to_slint_ui_asset_string_selection(
                    data.inspector_binding_payload_items,
                    data.inspector_binding_payload_selected_index,
                ),
                payload_suggestion_collection: to_slint_ui_asset_string_selection(
                    data.inspector_binding_payload_suggestion_items,
                    -1,
                ),
                payload_key: data.inspector_binding_payload_key.into(),
                payload_value: data.inspector_binding_payload_value.into(),
                schema_items: to_slint_shared_string_list(data.inspector_binding_schema_items),
                can_edit: data.inspector_can_edit_binding,
                can_delete: data.inspector_can_delete_binding,
            },
        },
    }
}
