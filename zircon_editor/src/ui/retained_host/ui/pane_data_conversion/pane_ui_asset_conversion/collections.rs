use crate::ui::asset_editor;
use crate::ui::retained_host as host_contract;

use super::string_selection::to_host_contract_ui_asset_string_selection;

pub(super) fn to_host_contract_ui_asset_collections(
    data: &mut asset_editor::UiAssetEditorPanePresentation,
) -> host_contract::UiAssetCollectionPanelData {
    host_contract::UiAssetCollectionPanelData {
        palette: to_host_contract_ui_asset_string_selection(
            std::mem::take(&mut data.palette_items),
            data.palette_selected_index,
        ),
        hierarchy: to_host_contract_ui_asset_string_selection(
            std::mem::take(&mut data.hierarchy_items),
            data.hierarchy_selected_index,
        ),
        preview: to_host_contract_ui_asset_string_selection(
            std::mem::take(&mut data.preview_items),
            data.preview_selected_index,
        ),
    }
}
