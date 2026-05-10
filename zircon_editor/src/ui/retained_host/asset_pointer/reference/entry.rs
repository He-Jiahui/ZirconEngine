#[derive(Clone, Debug, PartialEq, Eq)]
pub(crate) struct AssetReferenceListPointerEntry {
    pub asset_uuid: String,
    pub known_project_asset: bool,
}
