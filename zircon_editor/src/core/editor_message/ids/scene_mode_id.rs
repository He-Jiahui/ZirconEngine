use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
/// 场景交互模式的消息身份；模式登记和激活规则由场景模式拥有者验证，此包装只保留标识。
pub struct SceneModeId(String);

impl SceneModeId {
    pub fn new(value: impl Into<String>) -> Self {
        Self(value.into())
    }

    pub fn as_str(&self) -> &str {
        &self.0
    }
}
