//! 表达包对其他插件、能力及桥接接口的声明依赖；目录投影据此构建必需接口闭包。
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
/// required 决定接口列表是否进入必需桥接闭包；能力约束由包校验链另行检查。
pub struct PluginDependencyManifest {
    pub id: String,
    pub required: bool,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub capability: Option<String>,
    #[serde(default)]
    pub interfaces: Vec<String>,
}

impl PluginDependencyManifest {
    /// 构造依赖边，不检查目标是否已注册；目录合并时再判断提供者和接口。
    pub fn new(id: impl Into<String>, required: bool) -> Self {
        Self {
            id: id.into(),
            required,
            capability: None,
            interfaces: Vec::new(),
        }
    }

    pub fn with_capability(mut self, capability: impl Into<String>) -> Self {
        self.capability = Some(capability.into());
        self
    }

    pub fn with_interface(mut self, interface_id: impl Into<String>) -> Self {
        self.interfaces.push(interface_id.into());
        self
    }

    pub fn with_interfaces<I, S>(mut self, interface_ids: I) -> Self
    where
        I: IntoIterator<Item = S>,
        S: Into<String>,
    {
        self.interfaces
            .extend(interface_ids.into_iter().map(Into::into));
        self
    }
}
