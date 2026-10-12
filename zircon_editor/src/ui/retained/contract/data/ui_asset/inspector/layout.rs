use crate::ui::retained_host::primitives::SharedString;

use super::semantic::UiAssetInspectorSemanticData;

#[derive(Clone, Default)]
pub(crate) struct UiAssetInspectorLayoutData {
    pub width_preferred: SharedString,
    pub height_preferred: SharedString,
    pub semantic: UiAssetInspectorSemanticData,
    pub kind: SharedString,
    pub box_gap: SharedString,
    pub scroll_axis: SharedString,
    pub scroll_gap: SharedString,
    pub scrollbar_visibility: SharedString,
    pub virtualization_item_extent: SharedString,
    pub virtualization_overscan: SharedString,
    pub clip: SharedString,
}
