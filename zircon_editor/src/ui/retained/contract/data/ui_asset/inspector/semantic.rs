use crate::ui::retained_host::primitives::SharedString;

use super::super::common::UiAssetStringSelectionData;

#[derive(Clone, Default)]
pub(crate) struct UiAssetInspectorSemanticData {
    pub title: SharedString,
    pub collection: UiAssetStringSelectionData,
    pub path: SharedString,
    pub value: SharedString,
}
