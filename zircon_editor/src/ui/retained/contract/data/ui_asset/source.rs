use crate::ui::retained_host::primitives::SharedString;

use super::common::UiAssetStringSelectionData;

#[derive(Clone, Default)]
pub(crate) struct UiAssetSourceDetailData {
    pub block_label: SharedString,
    pub selected_line: i32,
    pub cursor_byte_offset: i32,
    pub selected_excerpt: SharedString,
    pub roundtrip_status: SharedString,
    pub outline: UiAssetStringSelectionData,
}

#[derive(Clone, Default)]
pub(crate) struct UiAssetSourcePanelData {
    pub text: SharedString,
    pub detail: UiAssetSourceDetailData,
}
