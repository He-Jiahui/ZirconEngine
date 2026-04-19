use zircon_runtime::ui::layout::UiSize;

use crate::ui::slint_host::asset_pointer::AssetListViewMode;

#[derive(Clone, Debug, PartialEq)]
pub(crate) struct AssetContentListPointerLayout {
    pub pane_size: UiSize,
    pub view_mode: AssetListViewMode,
    pub folder_ids: Vec<String>,
    pub item_ids: Vec<String>,
}

impl Default for AssetContentListPointerLayout {
    fn default() -> Self {
        Self {
            pane_size: UiSize::new(0.0, 0.0),
            view_mode: AssetListViewMode::List,
            folder_ids: Vec::new(),
            item_ids: Vec::new(),
        }
    }
}

impl AssetContentListPointerLayout {
    pub(crate) fn from_snapshot(
        snapshot: &crate::ui::workbench::snapshot::AssetWorkspaceSnapshot,
        pane_size: UiSize,
    ) -> Self {
        Self {
            pane_size,
            view_mode: snapshot.view_mode.into(),
            folder_ids: snapshot
                .visible_folders
                .iter()
                .map(|folder| folder.folder_id.clone())
                .collect(),
            item_ids: snapshot
                .visible_assets
                .iter()
                .map(|item| item.uuid.clone())
                .collect(),
        }
    }
}
