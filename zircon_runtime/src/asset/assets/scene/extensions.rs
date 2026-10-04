//! 地形、瓦片与脚本绑定在场景中仅记录引用和作者设置；各运行时子系统决定是否支持以及何时实例化这些扩展。

use std::collections::BTreeMap;

use crate::asset::AssetReference;
use serde::{Deserialize, Serialize};

use super::defaults::default_true;

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct SceneTerrainAsset {
    pub terrain: AssetReference,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct SceneTileMapAsset {
    pub tilemap: AssetReference,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct SceneScriptBindingAsset {
    pub package: String,
    pub module: String,
    #[serde(default = "default_true")]
    pub enabled: bool,
    #[serde(default = "default_true")]
    pub update: bool,
    #[serde(default = "default_true")]
    pub fixed_update: bool,
    #[serde(default, skip_serializing_if = "BTreeMap::is_empty")]
    pub properties: BTreeMap<String, serde_json::Value>,
}
