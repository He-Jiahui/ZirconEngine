use serde::{Deserialize, Serialize};

#[derive(
    Clone, Copy, Debug, Default, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize,
)]
/// serde 按单个数值键持久化，默认值为零，公开元组字段允许直接构造任意数值。
/// 复制、反序列化或读取该键都不会延长 World 寿命，使用前由管理器核实登记状态。
/// 可复制和序列化的世界句柄键；构造数值本身不表示对应关卡仍登记在 LevelManager 中。
pub struct WorldHandle(pub u64);

impl WorldHandle {
    /// 包装任意 `value` 并返回句柄键，不分配关卡或查询管理器。
    /// 默认管理器的新键分配另由受检递增计数器完成。
    pub const fn new(value: u64) -> Self {
        Self(value)
    }

    /// 按值取出数值键，保持可复制句柄的纯值语义。
    pub const fn get(self) -> u64 {
        self.0
    }
}

#[cfg(test)]
#[path = "tests/world_handle_perf.rs"]
mod optimization_batch_ic_runtime612_tests;
