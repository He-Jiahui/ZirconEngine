use super::super::common::UiAssetStringSelectionData;

#[derive(Clone, Default)]
pub(crate) struct UiAssetCollectionPanelData {
    pub palette: UiAssetStringSelectionData,
    pub hierarchy: UiAssetStringSelectionData,
    pub preview: UiAssetStringSelectionData,
}
