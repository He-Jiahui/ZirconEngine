use serde::{Deserialize, Serialize};

#[derive(
    Clone, Copy, Debug, Default, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize,
)]
/// 可复制和序列化的世界句柄键；构造数值本身不表示对应关卡仍登记在 LevelManager 中。
pub struct WorldHandle(pub u64);

impl WorldHandle {
    pub const fn new(value: u64) -> Self {
        Self(value)
    }

    pub const fn get(self) -> u64 {
        self.0
    }
}

#[cfg(test)]
#[path = "tests/world_handle_optimization_batch_ic_runtime612_tests.rs"]
mod optimization_batch_ic_runtime612_tests;
