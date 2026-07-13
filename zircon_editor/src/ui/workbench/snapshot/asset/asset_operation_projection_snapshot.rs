#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct AssetOperationProjectionSnapshot {
    pub asset_type_id: String,
    pub id: String,
    pub display_name: String,
    pub operation_id: String,
    pub icon_name: Option<String>,
    pub default_document: Option<String>,
}
