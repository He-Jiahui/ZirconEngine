use serde::{Deserialize, Serialize};

use super::super::ArtifactCacheJsonValue;
use crate::asset::{
    AssetCodecError, AssetReference, PrefabInstanceAsset, PrefabPropertyOverrideAsset,
    TransformAsset,
};

// 覆盖值是不透明 JSON；位置式缓存不能直接读回 serde_json::Value，
// 因此与脚本属性、组件载荷一样改存顺序 JSON 缓存值。字段顺序与作者 DTO 一致，
// 无覆盖的实例保持原有字节布局。
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub(super) struct ArtifactCachePrefabInstanceAsset {
    prefab: AssetReference,
    local_transform: TransformAsset,
    overrides: Vec<ArtifactCachePrefabOverrideAsset>,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
struct ArtifactCachePrefabOverrideAsset {
    entity_path: String,
    property_path: String,
    value: ArtifactCacheJsonValue,
}

impl From<&PrefabInstanceAsset> for ArtifactCachePrefabInstanceAsset {
    fn from(asset: &PrefabInstanceAsset) -> Self {
        Self {
            prefab: asset.prefab.clone(),
            local_transform: asset.local_transform,
            overrides: asset
                .overrides
                .iter()
                .map(|value| ArtifactCachePrefabOverrideAsset {
                    entity_path: value.entity_path.clone(),
                    property_path: value.property_path.clone(),
                    value: ArtifactCacheJsonValue::from_json(&value.value),
                })
                .collect(),
        }
    }
}

impl ArtifactCachePrefabInstanceAsset {
    // 只还原记录的数据并保持序列化顺序；不解析覆盖目标，也不应用覆盖。
    pub(super) fn into_asset(self) -> Result<PrefabInstanceAsset, AssetCodecError> {
        Ok(PrefabInstanceAsset {
            prefab: self.prefab,
            local_transform: self.local_transform,
            overrides: self
                .overrides
                .into_iter()
                .map(|value| {
                    Ok(PrefabPropertyOverrideAsset {
                        entity_path: value.entity_path,
                        property_path: value.property_path,
                        value: value.value.into_json()?,
                    })
                })
                .collect::<Result<Vec<_>, AssetCodecError>>()?,
        })
    }
}
