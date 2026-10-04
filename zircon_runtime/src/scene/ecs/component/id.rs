use serde::{Deserialize, Serialize};

/// ComponentRegistry 的 World 局部索引；跨 World 转移组件时必须先重新解析目标注册表 ID。
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, PartialOrd, Ord, Serialize, Deserialize)]
pub struct ComponentId(usize);

impl ComponentId {
    pub const fn new(index: usize) -> Self {
        Self(index)
    }

    pub const fn index(self) -> usize {
        self.0
    }
}
