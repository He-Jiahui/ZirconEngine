use zircon_ui::UiSize;

use super::entry::AssetReferenceListPointerEntry;

#[derive(Clone, Debug, PartialEq)]
pub(crate) struct AssetReferenceListPointerLayout {
    pub pane_size: UiSize,
    pub entries: Vec<AssetReferenceListPointerEntry>,
}

impl Default for AssetReferenceListPointerLayout {
    fn default() -> Self {
        Self {
            pane_size: UiSize::new(0.0, 0.0),
            entries: Vec::new(),
        }
    }
}

impl AssetReferenceListPointerLayout {
    pub(crate) fn from_references(
        references: &[crate::ui::workbench::snapshot::AssetReferenceSnapshot],
        pane_size: UiSize,
    ) -> Self {
        Self {
            pane_size,
            entries: references
                .iter()
                .map(|reference| AssetReferenceListPointerEntry {
                    asset_uuid: reference.uuid.clone(),
                    known_project_asset: reference.known_project_asset,
                })
                .collect(),
        }
    }
}
