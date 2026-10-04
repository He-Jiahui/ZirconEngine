//! 物理材质是作者资产与物理运行时之间的参数载体；场景碰撞引用它，不能由渲染材质的名称推断摩擦或恢复系数。

use serde::{Deserialize, Serialize};

use crate::core::framework::scene::physics::PhysicsMaterialMetadata;

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct PhysicsMaterialAsset {
    pub name: Option<String>,
    #[serde(flatten)]
    pub metadata: PhysicsMaterialMetadata,
}

impl PhysicsMaterialAsset {
    pub fn from_toml_str(document: &str) -> Result<Self, toml::de::Error> {
        toml::from_str(document)
    }

    pub fn to_toml_string(&self) -> Result<String, toml::ser::Error> {
        toml::to_string_pretty(self)
    }
}
