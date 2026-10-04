use serde::{Deserialize, Serialize};

/// 一个 World 内部的原型索引；场景持久身份由 EntityId 承担，结构迁移后不能把此索引当作实体身份。
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, PartialOrd, Ord, Serialize, Deserialize)]
pub struct ArchetypeId(usize);

impl ArchetypeId {
    pub const EMPTY: Self = Self(0);

    pub const fn new(index: usize) -> Self {
        Self(index)
    }

    pub const fn index(self) -> usize {
        self.0
    }
}

impl Default for ArchetypeId {
    fn default() -> Self {
        Self::EMPTY
    }
}
