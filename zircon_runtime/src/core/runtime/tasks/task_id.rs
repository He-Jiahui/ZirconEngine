use serde::{Deserialize, Serialize};

/// 标识由调用方分配；同一作用域只禁止同时活跃的重复 ID，终结后可以复用。
/// Stable identity assigned to one admitted runtime task.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct TaskId(u64);

impl TaskId {
    pub const fn new(raw: u64) -> Self {
        Self(raw)
    }

    pub const fn raw(self) -> u64 {
        self.0
    }
}
