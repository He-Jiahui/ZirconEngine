use crate::ui::asset_editor;
use crate::ui::retained_host as host_contract;

pub(super) fn to_host_contract_ui_asset_header(
    data: &mut asset_editor::UiAssetEditorPanePresentation,
) -> host_contract::UiAssetPaneHeaderData {
    host_contract::UiAssetPaneHeaderData {
        asset_id: std::mem::take(&mut data.asset_id).into(),
        mode: std::mem::take(&mut data.mode).into(),
        status: std::mem::take(&mut data.last_error).into(),
        selection: std::mem::take(&mut data.selection_summary).into(),
        shell_state: std::mem::take(&mut data.shell_state).into(),
        emergency_summary: std::mem::take(&mut data.emergency_summary).into(),
    }
}
