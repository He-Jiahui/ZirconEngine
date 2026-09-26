use serde::{Deserialize, Serialize};

/// 输入准入的继承策略；Runtime 沿父节点继承最近的显式策略，根节点默认为 `Receive`。
/// 子节点显式 `Receive` 可覆盖父节点 `Ignore`；能否命中还取决于可见性和指针准入。
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
pub enum UiInputPolicy {
    #[default]
    Inherit,
    Receive,
    Ignore,
}
