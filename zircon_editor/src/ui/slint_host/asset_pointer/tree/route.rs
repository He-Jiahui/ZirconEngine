#[derive(Clone, Debug, PartialEq, Eq)]
pub(crate) enum AssetPointerTreeRoute {
    Folder { row_index: usize, folder_id: String },
    TreeSurface,
}
