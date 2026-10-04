use std::collections::BTreeMap;

use serde::{Deserialize, Serialize};

use super::super::json_value::{cache_table_to_json, json_table_to_cache, ArtifactCacheJsonValue};
use crate::asset::AssetImportError;

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
// 脚本属性使用顺序 JSON 缓存值，并保留包、模块、启用状态及两类更新开关，供场景实体缓存往返。
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
    // 属性还原会拒绝非法十进制或非有限数值；失败沿实体缓存读取路径传播，避免返回不完整的脚本绑定。
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
