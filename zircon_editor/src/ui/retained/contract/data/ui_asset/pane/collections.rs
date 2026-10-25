use super::super::common::UiAssetStringSelectionData;

#[derive(Clone, Default)]
pub(crate) struct UiAssetCollectionPanelData {
    pub palette: UiAssetStringSelectionData,
    pub palette_visible_start: usize,
    pub palette_search: Option<crate::ui::asset_editor::palette::UiAssetPaletteSearchSnapshot>,
    pub hierarchy: UiAssetStringSelectionData,
    pub preview: UiAssetStringSelectionData,
}
