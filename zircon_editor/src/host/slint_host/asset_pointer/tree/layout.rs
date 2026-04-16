use zircon_ui::UiSize;

#[derive(Clone, Debug, PartialEq)]
pub(crate) struct AssetFolderTreePointerLayout {
    pub pane_size: UiSize,
    pub folder_ids: Vec<String>,
}

impl Default for AssetFolderTreePointerLayout {
    fn default() -> Self {
        Self {
            pane_size: UiSize::new(0.0, 0.0),
            folder_ids: Vec::new(),
        }
    }
}

impl AssetFolderTreePointerLayout {
    pub(crate) fn from_snapshot(
        snapshot: &crate::workbench::snapshot::AssetWorkspaceSnapshot,
        pane_size: UiSize,
    ) -> Self {
        Self {
            pane_size,
            folder_ids: snapshot
                .folder_tree
                .iter()
                .map(|folder| folder.folder_id.clone())
                .collect(),
        }
    }
}
