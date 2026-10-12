use serde::{Deserialize, Serialize};

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
/// 根类策略决定组件实例能否追加调用方 class；闭合策略会在契约校验阶段拒绝追加。
pub enum UiRootClassPolicy {
    #[default]
    AppendOnly,
    Closed,
}
