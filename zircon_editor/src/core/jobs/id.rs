//! 在任务系统、通知绑定和依赖规格之间传递类型化任务身份；数值包装不证明该任务属于当前系统或仍在保留历史中，准入端需要核验。
use serde::{Deserialize, Serialize};

#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
pub struct JobId(u64);

impl JobId {
    pub const fn new(value: u64) -> Self {
        Self(value)
    }

    pub const fn value(self) -> u64 {
        self.0
    }
}
