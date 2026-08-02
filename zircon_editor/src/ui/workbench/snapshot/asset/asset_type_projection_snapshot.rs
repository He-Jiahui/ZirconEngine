use crate::core::asset::{
    AssetSourceWritePolicy, AssetTypeDefinition, builtin_asset_type_definition,
};
use zircon_runtime_interface::resource::ResourceKind;

#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct AssetTypeProjectionSnapshot {
    pub asset_type_id: String,
    pub display_name: String,
    pub badge: String,
    pub icon_name: String,
    pub color_token: String,
    pub source_write_policy: AssetSourceWritePolicy,
}

impl AssetTypeProjectionSnapshot {
    pub fn from_resource_kind(kind: ResourceKind) -> Self {
        builtin_asset_type_definition(kind)
            .map(Self::from_definition)
            .unwrap_or_default()
    }

    pub fn from_definition(definition: &AssetTypeDefinition) -> Self {
        Self {
            asset_type_id: definition.id().to_string(),
            display_name: definition.presentation().display_name().to_owned(),
            badge: definition.presentation().badge().to_owned(),
            icon_name: definition.presentation().icon_name().to_owned(),
            color_token: definition.presentation().color_token().to_owned(),
            source_write_policy: definition.source_write_policy(),
        }
    }
}
