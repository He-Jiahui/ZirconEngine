use crate::ui::asset_editor;
use crate::ui::retained_host as host_contract;

use super::string_selection::to_host_contract_ui_asset_string_selection;

pub(super) fn to_host_contract_ui_asset_collections(
    data: &mut asset_editor::UiAssetEditorPanePresentation,
) -> host_contract::UiAssetCollectionPanelData {
    let (palette, palette_visible_start) = if let Some(snapshot) = &data.palette_search {
        (
            to_host_contract_ui_asset_string_selection(
                snapshot
                    .rows()
                    .map(|(_, entry)| entry.label.clone())
                    .collect(),
                snapshot
                    .selected_row()
                    .and_then(|row| i32::try_from(row).ok())
                    .unwrap_or(-1),
            ),
            0, // Snapshot rows are already bounded to the current page.
        )
    } else {
        (
            to_host_contract_ui_asset_string_selection(
                std::mem::take(&mut data.palette_items),
                data.palette_selected_index,
            ),
            data.palette_visible_start,
        )
    };
    host_contract::UiAssetCollectionPanelData {
        palette_search: data.palette_search.clone(),
        palette_visible_start,
        palette,
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
