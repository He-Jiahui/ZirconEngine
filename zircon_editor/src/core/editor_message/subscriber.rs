use serde::{Deserialize, Serialize};

#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
/// 一条总线内分配的订阅身份；注销后不能继续请求该目标，跨总线复用相同数字不代表同一订阅。
pub struct EditorSubscriberId(u64);

impl EditorSubscriberId {
    pub const fn new(value: u64) -> Self {
        Self(value)
    }

    pub const fn value(self) -> u64 {
        self.0
    }
}
