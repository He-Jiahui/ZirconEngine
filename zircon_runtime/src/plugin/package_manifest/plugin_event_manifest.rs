//! 包提供的事件目录契约；注册器校验命名空间与事件条目后才向扩展注册表公开。
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
/// 命名空间和版本共同描述事件目录；事件消费者另在模块声明中引用事件与载荷模式。
pub struct PluginEventCatalogManifest {
    pub namespace: String,
    pub version: u32,
    #[serde(default)]
    pub events: Vec<PluginEventManifest>,
}

impl PluginEventCatalogManifest {
    pub fn empty(namespace: impl Into<String>, version: u32) -> Self {
        Self {
            namespace: namespace.into(),
            version,
            events: Vec::new(),
        }
    }
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
/// 事件标识与载荷模式供注册及消费者匹配；显示名称只用于展示。
pub struct PluginEventManifest {
    pub id: String,
    pub display_name: String,
    #[serde(default)]
    pub payload_schema: String,
}
