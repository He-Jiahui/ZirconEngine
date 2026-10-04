//! 特性能力束的依赖边；目录用 primary 边确认能力所有者，用其余边约束启用。
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
/// 主依赖声明所有者插件；目录要求每个特性恰有一条匹配 owner_plugin_id 的主边。
pub struct PluginFeatureDependency {
    pub plugin_id: String,
    pub capability: String,
    #[serde(default)]
    pub primary: bool,
}

impl PluginFeatureDependency {
    pub fn required(plugin_id: impl Into<String>, capability: impl Into<String>) -> Self {
        Self {
            plugin_id: plugin_id.into(),
            capability: capability.into(),
            primary: false,
        }
    }

    /// 将此依赖标记为所有者边；调用方仍须保证整束只有一个主边且目标一致。
    pub fn primary(plugin_id: impl Into<String>, capability: impl Into<String>) -> Self {
        Self {
            plugin_id: plugin_id.into(),
            capability: capability.into(),
            primary: true,
        }
    }
}
