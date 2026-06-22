use crate::ui::asset_editor;
use crate::ui::layouts::common::model_rc;
use crate::ui::retained_host as host_contract;
use crate::ui::retained_host::primitives::ModelRc;

use super::string_selection::{
    to_host_contract_shared_string_list, to_host_contract_ui_asset_string_selection,
};

fn to_host_contract_ui_asset_prop_state_rows(
    items: Vec<asset_editor::UiAssetEditorWidgetPropStateItem>,
) -> ModelRc<host_contract::UiAssetInspectorWidgetPropStateData> {
    model_rc(
        items
            .into_iter()
            .map(|item| host_contract::UiAssetInspectorWidgetPropStateData {
                kind: item.kind.into(),
                path: item.path.into(),
                value: item.value.into(),
                display: item.display.into(),
            })
            .collect(),
    )
}

fn to_host_contract_ui_asset_inspector_semantic(
    title: String,
    items: Vec<String>,
    selected_index: i32,
    path: String,
    value: String,
) -> host_contract::UiAssetInspectorSemanticData {
    host_contract::UiAssetInspectorSemanticData {
        title: title.into(),
        collection: to_host_contract_ui_asset_string_selection(items, selected_index),
        path: path.into(),
        value: value.into(),
    }
}

pub(super) fn to_host_contract_ui_asset_inspector_panel(
    data: &mut asset_editor::UiAssetEditorPanePresentation,
) -> host_contract::UiAssetInspectorPanelData {
    host_contract::UiAssetInspectorPanelData {
        widget: host_contract::UiAssetInspectorWidgetData {
            selected_node_id: std::mem::take(&mut data.inspector_selected_node_id).into(),
            parent_node_id: std::mem::take(&mut data.inspector_parent_node_id).into(),
            mount: std::mem::take(&mut data.inspector_mount).into(),
            widget_kind: std::mem::take(&mut data.inspector_widget_kind).into(),
            widget_label: std::mem::take(&mut data.inspector_widget_label).into(),
            control_id: std::mem::take(&mut data.inspector_control_id).into(),
            text_prop: std::mem::take(&mut data.inspector_text_prop).into(),
            component_root_class_policy: std::mem::take(
                &mut data.inspector_component_root_class_policy,
            )
            .into(),
            can_edit_control_id: data.inspector_can_edit_control_id,
            can_edit_text_prop: data.inspector_can_edit_text_prop,
            can_edit_component_root_class_policy: data
                .inspector_can_edit_component_root_class_policy,
            promote_asset_id: std::mem::take(&mut data.inspector_promote_asset_id).into(),
            promote_component_name: std::mem::take(&mut data.inspector_promote_component_name)
                .into(),
            promote_document_id: std::mem::take(&mut data.inspector_promote_document_id).into(),
            can_edit_promote_draft: data.inspector_can_edit_promote_draft,
            prop_state_rows: to_host_contract_ui_asset_prop_state_rows(std::mem::take(
                &mut data.inspector_widget_prop_state_rows,
            )),
            prop_state_items: to_host_contract_shared_string_list(std::mem::take(
                &mut data.inspector_widget_prop_state_items,
            )),
            items: to_host_contract_shared_string_list(std::mem::take(&mut data.inspector_items)),
        },
        slot: host_contract::UiAssetInspectorSlotData {
            padding: std::mem::take(&mut data.inspector_slot_padding).into(),
            width_preferred: std::mem::take(&mut data.inspector_slot_width_preferred).into(),
            height_preferred: std::mem::take(&mut data.inspector_slot_height_preferred).into(),
            semantic: to_host_contract_ui_asset_inspector_semantic(
                std::mem::take(&mut data.inspector_slot_semantic_title),
                std::mem::take(&mut data.inspector_slot_semantic_items),
                data.inspector_slot_semantic_selected_index,
                std::mem::take(&mut data.inspector_slot_semantic_path),
                std::mem::take(&mut data.inspector_slot_semantic_value),
            ),
            kind: std::mem::take(&mut data.inspector_slot_kind).into(),
            linear_main_weight: std::mem::take(&mut data.inspector_slot_linear_main_weight).into(),
            linear_main_stretch: std::mem::take(&mut data.inspector_slot_linear_main_stretch)
                .into(),
            linear_cross_weight: std::mem::take(&mut data.inspector_slot_linear_cross_weight)
                .into(),
            linear_cross_stretch: std::mem::take(&mut data.inspector_slot_linear_cross_stretch)
                .into(),
            overlay_anchor_x: std::mem::take(&mut data.inspector_slot_overlay_anchor_x).into(),
            overlay_anchor_y: std::mem::take(&mut data.inspector_slot_overlay_anchor_y).into(),
            overlay_pivot_x: std::mem::take(&mut data.inspector_slot_overlay_pivot_x).into(),
            overlay_pivot_y: std::mem::take(&mut data.inspector_slot_overlay_pivot_y).into(),
            overlay_position_x: std::mem::take(&mut data.inspector_slot_overlay_position_x).into(),
            overlay_position_y: std::mem::take(&mut data.inspector_slot_overlay_position_y).into(),
            overlay_z_index: std::mem::take(&mut data.inspector_slot_overlay_z_index).into(),
            grid_row: std::mem::take(&mut data.inspector_slot_grid_row).into(),
            grid_column: std::mem::take(&mut data.inspector_slot_grid_column).into(),
            grid_row_span: std::mem::take(&mut data.inspector_slot_grid_row_span).into(),
            grid_column_span: std::mem::take(&mut data.inspector_slot_grid_column_span).into(),
            flow_break_before: std::mem::take(&mut data.inspector_slot_flow_break_before).into(),
            flow_alignment: std::mem::take(&mut data.inspector_slot_flow_alignment).into(),
        },
        layout: host_contract::UiAssetInspectorLayoutData {
            width_preferred: std::mem::take(&mut data.inspector_layout_width_preferred).into(),
            height_preferred: std::mem::take(&mut data.inspector_layout_height_preferred).into(),
            semantic: to_host_contract_ui_asset_inspector_semantic(
                std::mem::take(&mut data.inspector_layout_semantic_title),
                std::mem::take(&mut data.inspector_layout_semantic_items),
                data.inspector_layout_semantic_selected_index,
                std::mem::take(&mut data.inspector_layout_semantic_path),
                std::mem::take(&mut data.inspector_layout_semantic_value),
            ),
            kind: std::mem::take(&mut data.inspector_layout_kind).into(),
            box_gap: std::mem::take(&mut data.inspector_layout_box_gap).into(),
            scroll_axis: std::mem::take(&mut data.inspector_layout_scroll_axis).into(),
            scroll_gap: std::mem::take(&mut data.inspector_layout_scroll_gap).into(),
            scrollbar_visibility: std::mem::take(&mut data.inspector_layout_scrollbar_visibility)
                .into(),
            virtualization_item_extent: std::mem::take(
                &mut data.inspector_layout_virtualization_item_extent,
            )
            .into(),
            virtualization_overscan: std::mem::take(
                &mut data.inspector_layout_virtualization_overscan,
            )
            .into(),
            clip: std::mem::take(&mut data.inspector_layout_clip).into(),
        },
        binding: host_contract::UiAssetInspectorBindingData {
            collection: to_host_contract_ui_asset_string_selection(
                std::mem::take(&mut data.inspector_binding_items),
                data.inspector_binding_selected_index,
            ),
            binding_id: std::mem::take(&mut data.inspector_binding_id).into(),
            binding_event: std::mem::take(&mut data.inspector_binding_event).into(),
            event_collection: to_host_contract_ui_asset_string_selection(
                std::mem::take(&mut data.inspector_binding_event_items),
                data.inspector_binding_event_selected_index,
            ),
            binding_route: std::mem::take(&mut data.inspector_binding_route).into(),
            binding_route_target: std::mem::take(&mut data.inspector_binding_route_target).into(),
            binding_action_target: std::mem::take(&mut data.inspector_binding_action_target).into(),
            route_suggestion_collection: to_host_contract_ui_asset_string_selection(
                std::mem::take(&mut data.inspector_binding_route_suggestion_items),
                -1,
            ),
            action_suggestion_collection: to_host_contract_ui_asset_string_selection(
                std::mem::take(&mut data.inspector_binding_action_suggestion_items),
                -1,
            ),
            action_kind_collection: to_host_contract_ui_asset_string_selection(
                std::mem::take(&mut data.inspector_binding_action_kind_items),
                data.inspector_binding_action_kind_selected_index,
            ),
            payload_collection: to_host_contract_ui_asset_string_selection(
                std::mem::take(&mut data.inspector_binding_payload_items),
                data.inspector_binding_payload_selected_index,
            ),
            payload_suggestion_collection: to_host_contract_ui_asset_string_selection(
                std::mem::take(&mut data.inspector_binding_payload_suggestion_items),
                -1,
            ),
            payload_key: std::mem::take(&mut data.inspector_binding_payload_key).into(),
            payload_value: std::mem::take(&mut data.inspector_binding_payload_value).into(),
            schema_items: to_host_contract_shared_string_list(std::mem::take(
                &mut data.inspector_binding_schema_items,
            )),
            can_edit: data.inspector_can_edit_binding,
            can_delete: data.inspector_can_delete_binding,
        },
    }
}
