use super::super::common::UiAssetStringSelectionData;

#[derive(Clone, Default)]
pub(crate) struct UiAssetCollectionPanelData {
    pub palette: UiAssetStringSelectionData,
    pub palette_visible_start: usize,
    pub hierarchy: UiAssetStringSelectionData,
    pub preview: UiAssetStringSelectionData,
}
