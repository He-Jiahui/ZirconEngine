use zircon_manager::AssetRecordKind;

#[derive(Clone, Debug, Default)]
pub struct AssetReferenceSnapshot {
    pub uuid: String,
    pub locator: String,
    pub display_name: String,
    pub kind: Option<AssetRecordKind>,
    pub known_project_asset: bool,
}
