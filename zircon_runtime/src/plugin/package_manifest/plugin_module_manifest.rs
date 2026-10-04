//! 模块声明和事件消费声明；运行时将其中的排序字段投影到核心模块描述符。
use serde::{Deserialize, Serialize};

use crate::core::framework::platform::RuntimeTargetMode;
use crate::core::{InitLevel, ModuleDependencySpec};

use super::PluginModuleKind;

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
/// 按事件 ID 与载荷模式声明订阅意图；处理器绑定和能力准入发生在注册端。
pub struct PluginEventConsumerManifest {
    pub consumer_id: String,
    pub event_id: String,
    pub payload_schema: String,
    #[serde(default, skip_serializing_if = "String::is_empty")]
    pub required_capability: String,
}

impl PluginEventConsumerManifest {
    pub fn new(
        consumer_id: impl Into<String>,
        event_id: impl Into<String>,
        payload_schema: impl Into<String>,
    ) -> Self {
        Self {
            consumer_id: consumer_id.into(),
            event_id: event_id.into(),
            payload_schema: payload_schema.into(),
            required_capability: String::new(),
        }
    }

    pub fn with_required_capability(mut self, capability: impl Into<String>) -> Self {
        self.required_capability = capability.into();
        self
    }
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
/// 包内模块的完整声明；种类决定运行域，目标列表限定可参与的宿主目标。
/// 初始化级别与模块依赖用于排序；能力、系统和事件供各注册链分别消费。
pub struct PluginModuleManifest {
    pub name: String,
    #[serde(default, skip_serializing_if = "String::is_empty")]
    pub description: String,
    pub kind: PluginModuleKind,
    pub crate_name: String,
    #[serde(default = "default_plugin_module_init_level")]
    pub init_level: InitLevel,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub module_dependencies: Vec<ModuleDependencySpec>,
    #[serde(default)]
    pub target_modes: Vec<RuntimeTargetMode>,
    #[serde(default)]
    pub capabilities: Vec<String>,
    #[serde(default)]
    pub system_sets: Vec<String>,
    #[serde(default)]
    pub system_anchors: Vec<String>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub event_consumers: Vec<PluginEventConsumerManifest>,
}

// 旧清单缺少初始化级别时沿用构造器的 Post 基线，避免反序列化改变注册顺序。
fn default_plugin_module_init_level() -> InitLevel {
    InitLevel::Post
}
