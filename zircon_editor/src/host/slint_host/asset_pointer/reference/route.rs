#[derive(Clone, Debug, PartialEq, Eq)]
pub(crate) enum AssetPointerReferenceRoute {
    Item {
        row_index: usize,
        asset_uuid: String,
    },
    ListSurface,
}
