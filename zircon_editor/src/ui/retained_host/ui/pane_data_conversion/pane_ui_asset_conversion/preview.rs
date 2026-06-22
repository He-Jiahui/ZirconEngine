use crate::ui::asset_editor;
use crate::ui::layouts::common::model_rc;
use crate::ui::retained_host as host_contract;
use crate::ui::retained_host::primitives::ModelRc;

use super::string_selection::{
    to_host_contract_shared_string_list, to_host_contract_ui_asset_string_selection,
};

fn to_host_contract_ui_asset_canvas_nodes(
    items: Vec<asset_editor::UiAssetEditorPreviewCanvasNode>,
) -> ModelRc<host_contract::UiAssetCanvasNodeData> {
    model_rc(
        items
            .into_iter()
            .map(|item| host_contract::UiAssetCanvasNodeData {
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

pub(super) fn to_host_contract_ui_asset_preview_panel(
    data: &mut asset_editor::UiAssetEditorPanePresentation,
) -> host_contract::UiAssetPreviewPanelData {
    host_contract::UiAssetPreviewPanelData {
        preset: std::mem::take(&mut data.preview_preset).into(),
        summary: std::mem::take(&mut data.preview_summary).into(),
        available: data.preview_available,
        canvas: host_contract::UiAssetPreviewCanvasData {
            width: data.preview_surface_width,
            height: data.preview_surface_height,
            items: to_host_contract_ui_asset_canvas_nodes(std::mem::take(
                &mut data.preview_canvas_items,
            )),
        },
        mock: host_contract::UiAssetPreviewMockData {
            subject_collection: to_host_contract_ui_asset_string_selection(
                std::mem::take(&mut data.preview_mock_subject_items),
                data.preview_mock_subject_selected_index,
            ),
            subject_node_id: std::mem::take(&mut data.preview_mock_subject_node_id).into(),
            collection: to_host_contract_ui_asset_string_selection(
                std::mem::take(&mut data.preview_mock_items),
                data.preview_mock_selected_index,
            ),
            property: std::mem::take(&mut data.preview_mock_property).into(),
            kind: std::mem::take(&mut data.preview_mock_kind).into(),
            value: std::mem::take(&mut data.preview_mock_value).into(),
            expression_result: std::mem::take(&mut data.preview_mock_expression_result).into(),
            nested_collection: to_host_contract_ui_asset_string_selection(
                std::mem::take(&mut data.preview_mock_nested_items),
                data.preview_mock_nested_selected_index,
            ),
            nested_key: std::mem::take(&mut data.preview_mock_nested_key).into(),
            nested_kind: std::mem::take(&mut data.preview_mock_nested_kind).into(),
            nested_value: std::mem::take(&mut data.preview_mock_nested_value).into(),
            suggestion_collection: to_host_contract_ui_asset_string_selection(
                std::mem::take(&mut data.preview_mock_suggestion_items),
                -1,
            ),
            schema_items: to_host_contract_shared_string_list(std::mem::take(
                &mut data.preview_mock_schema_items,
            )),
            state_graph_items: to_host_contract_shared_string_list(std::mem::take(
                &mut data.preview_state_graph_items,
            )),
            can_edit: data.preview_mock_can_edit,
            can_clear: data.preview_mock_can_clear,
            nested_can_edit: data.preview_mock_nested_can_edit,
            nested_can_add: data.preview_mock_nested_can_add,
            nested_can_delete: data.preview_mock_nested_can_delete,
        },
    }
}
