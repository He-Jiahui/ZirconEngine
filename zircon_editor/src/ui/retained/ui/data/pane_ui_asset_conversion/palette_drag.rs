use crate::ui::asset_editor;
use crate::ui::layouts::common::model_rc;
use crate::ui::retained_host as host_contract;
use crate::ui::retained_host::primitives::ModelRc;

use super::string_selection::to_host_contract_shared_string_list;

fn to_host_contract_ui_asset_canvas_slot_targets(
    items: Vec<asset_editor::UiAssetEditorPreviewCanvasSlotTarget>,
) -> ModelRc<host_contract::UiAssetCanvasSlotTargetData> {
    model_rc(
        items
            .into_iter()
            .map(|item| host_contract::UiAssetCanvasSlotTargetData {
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

pub(super) fn to_host_contract_ui_asset_palette_drag(
    data: &mut asset_editor::UiAssetEditorPanePresentation,
) -> host_contract::UiAssetPaletteDragData {
    host_contract::UiAssetPaletteDragData {
        target_preview_index: data.palette_drag_target_preview_index,
        target_action: std::mem::take(&mut data.palette_drag_target_action).into(),
        target_label: std::mem::take(&mut data.palette_drag_target_label).into(),
        slot_target_items: to_host_contract_ui_asset_canvas_slot_targets(std::mem::take(
            &mut data.palette_drag_slot_target_items,
        )),
        candidate_items: to_host_contract_shared_string_list(std::mem::take(
            &mut data.palette_drag_candidate_items,
        )),
        candidate_selected_index: data.palette_drag_candidate_selected_index,
        target_chooser_active: data.palette_target_chooser_active,
    }
}
