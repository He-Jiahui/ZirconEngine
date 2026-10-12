use crate::ui::asset_editor;
use crate::ui::retained_host as host_contract;

pub(super) fn to_host_contract_ui_asset_designer_tools(
    data: &mut asset_editor::UiAssetEditorPanePresentation,
) -> host_contract::UiAssetDesignerToolStateData {
    host_contract::UiAssetDesignerToolStateData {
        mode: std::mem::take(&mut data.designer_tool_mode).into(),
        can_select: data.can_designer_select,
        can_resize_slot: data.can_designer_resize_slot,
        can_preview_interact: data.can_designer_preview_interact,
    }
}
