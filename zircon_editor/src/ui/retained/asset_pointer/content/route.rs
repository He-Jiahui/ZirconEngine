#[derive(Clone, Debug, PartialEq, Eq)]
pub(crate) enum AssetPointerContentRoute {
    Folder {
        row_index: usize,
        folder_index: usize,
        folder_id: String,
    },
    Item {
        row_index: usize,
        item_index: usize,
        asset_uuid: String,
    },
    ContentSurface,
}
