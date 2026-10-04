use serde::{Deserialize, Serialize};

/// World 资源注册表的局部索引；系统参数借此声明资源访问，不能跨 World 复用数值。
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, PartialOrd, Ord, Serialize, Deserialize)]
pub struct ResourceId(usize);

impl ResourceId {
    pub const fn new(index: usize) -> Self {
        Self(index)
    }

    pub const fn index(self) -> usize {
        self.0
    }
}
