use zircon_runtime::core::resource::ResourceKind;

#[derive(Clone, Debug, Default)]
pub struct AssetReferenceSnapshot {
    pub uuid: String,
    pub locator: String,
    pub display_name: String,
    pub kind: Option<ResourceKind>,
    pub known_project_asset: bool,
}
