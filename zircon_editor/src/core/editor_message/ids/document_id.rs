use serde::{Deserialize, Serialize};

#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
/// 文档管理器分配的运行期身份；生命周期消息携带它以便消费者回查文档，不等价于持久化资产路径。
pub struct DocumentId(u64);

impl DocumentId {
    pub const fn new(value: u64) -> Self {
        Self(value)
    }

    pub const fn value(self) -> u64 {
        self.0
    }
}
