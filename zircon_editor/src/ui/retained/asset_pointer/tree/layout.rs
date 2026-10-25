use zircon_runtime_interface::ui::layout::UiSize;

#[derive(Clone, Debug, PartialEq)]
pub(crate) struct AssetFolderTreePointerLayout {
    pub pane_size: UiSize,
    pub folder_ids: Vec<String>,
    pub rows: Vec<AssetFolderTreePointerRow>,
    pub catalog_revision: u64,
}

#[derive(Clone, Debug, PartialEq)]
pub(crate) struct AssetFolderTreePointerRow {
    pub depth: usize,
    pub has_children: bool,
    pub expanded: bool,
}

impl Default for AssetFolderTreePointerLayout {
    fn default() -> Self {
        Self {
            pane_size: UiSize::new(0.0, 0.0),
            folder_ids: Vec::new(),
            rows: Vec::new(),
            catalog_revision: 0,
        }
    }
}

impl AssetFolderTreePointerLayout {
    pub(crate) fn from_snapshot(
        snapshot: &crate::ui::workbench::snapshot::AssetWorkspaceSnapshot,
        pane_size: UiSize,
    ) -> Self {
        let mut folder_ids = Vec::with_capacity(snapshot.folder_tree.len());
        folder_ids.extend(
            snapshot
                .folder_tree
                .iter()
                .map(|folder| folder.folder_id.clone()),
        );
        Self {
            pane_size,
            folder_ids,
            rows: snapshot
                .folder_tree
                .iter()
                .map(|folder| AssetFolderTreePointerRow {
                    depth: folder.depth,
                    has_children: folder.has_children,
                    expanded: folder.expanded,
                })
                .collect(),
            catalog_revision: snapshot.catalog_revision,
        }
    }
}

#[cfg(test)]
#[path = "tests/layout.rs"]
mod tests;
