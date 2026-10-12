use crate::ui::asset_editor;
use crate::ui::retained_host as host_contract;

use super::string_selection::to_host_contract_ui_asset_string_selection;

pub(super) fn to_host_contract_ui_asset_source(
    data: &mut asset_editor::UiAssetEditorPanePresentation,
) -> host_contract::UiAssetSourcePanelData {
    host_contract::UiAssetSourcePanelData {
        text: std::mem::take(&mut data.source_text).into(),
        detail: host_contract::UiAssetSourceDetailData {
            block_label: std::mem::take(&mut data.source_selected_block_label).into(),
            selected_line: data.source_selected_line,
            cursor_byte_offset: data.source_cursor_byte_offset,
            selected_excerpt: std::mem::take(&mut data.source_selected_excerpt).into(),
            roundtrip_status: std::mem::take(&mut data.source_roundtrip_status).into(),
            outline: to_host_contract_ui_asset_string_selection(
                std::mem::take(&mut data.source_outline_items),
                data.source_outline_selected_index,
            ),
        },
    }
}
