use std::collections::BTreeMap;

use serde::{Deserialize, Serialize};

use super::super::json_value::{ArtifactCacheJsonValue, cache_table_to_json, json_table_to_cache};
use crate::asset::AssetImportError;

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub(super) struct ArtifactCacheSceneScriptBindingAsset {
    package: String,
    module: String,
    enabled: bool,
    update: bool,
    fixed_update: bool,
    properties: BTreeMap<String, ArtifactCacheJsonValue>,
}

impl From<&crate::asset::SceneScriptBindingAsset> for ArtifactCacheSceneScriptBindingAsset {
    fn from(asset: &crate::asset::SceneScriptBindingAsset) -> Self {
        Self {
            package: asset.package.clone(),
            module: asset.module.clone(),
            enabled: asset.enabled,
            update: asset.update,
            fixed_update: asset.fixed_update,
            properties: json_table_to_cache(&asset.properties),
        }
    }
}

impl ArtifactCacheSceneScriptBindingAsset {
    pub(super) fn into_asset(
        self,
    ) -> Result<crate::asset::SceneScriptBindingAsset, AssetImportError> {
        Ok(crate::asset::SceneScriptBindingAsset {
            package: self.package,
            module: self.module,
            enabled: self.enabled,
            update: self.update,
            fixed_update: self.fixed_update,
            properties: cache_table_to_json(self.properties)?,
        })
    }
}
