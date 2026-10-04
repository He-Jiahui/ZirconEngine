//! 清单的热重载策略决定是否保存旧实例状态；默认保留状态，Stateless 明确跳过迁移，Disabled 在停用前拒绝请求。
use serde::{Deserialize, Serialize};

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum VmPluginHotReloadPolicy {
    Disabled,
    Stateless,
    #[default]
    PreserveState,
}
