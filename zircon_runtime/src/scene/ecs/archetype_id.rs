use serde::{Deserialize, Serialize};

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
