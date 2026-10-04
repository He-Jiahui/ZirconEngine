//! 包提供的配置选项声明；扩展注册表负责校验值类型、默认值与枚举约束。
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
/// 默认值保持清单文本形式；注册时才按 value_type 解析，枚举值必须包含默认值。
pub struct PluginOptionManifest {
    pub key: String,
    pub display_name: String,
    pub value_type: String,
    pub default_value: String,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub enum_values: Vec<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub required_capability: Option<String>,
}

impl PluginOptionManifest {
    pub fn new(
        key: impl Into<String>,
        display_name: impl Into<String>,
        value_type: impl Into<String>,
        default_value: impl Into<String>,
    ) -> Self {
        Self {
            key: key.into(),
            display_name: display_name.into(),
            value_type: value_type.into(),
            default_value: default_value.into(),
            enum_values: Vec::new(),
            required_capability: None,
        }
    }

    /// 声明 enum 选项的可选值；直接构造和 TOML 解析不承担去重与成员校验。
    pub fn with_enum_values<I, S>(mut self, values: I) -> Self
    where
        I: IntoIterator<Item = S>,
        S: Into<String>,
    {
        self.enum_values = values.into_iter().map(Into::into).collect();
        self
    }

    pub fn with_required_capability(mut self, capability: impl Into<String>) -> Self {
        self.required_capability = Some(capability.into());
        self
    }
}
