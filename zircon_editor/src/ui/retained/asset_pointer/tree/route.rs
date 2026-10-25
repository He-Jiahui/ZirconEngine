#[derive(Clone, Debug, PartialEq, Eq)]
pub(crate) enum AssetPointerTreeRoute {
    Folder {
        row_index: usize,
        folder_id: String,
    },
    Disclosure {
        row_index: usize,
        folder_id: String,
        expanded: bool,
        expected_catalog_revision: u64,
    },
    TreeSurface,
}
