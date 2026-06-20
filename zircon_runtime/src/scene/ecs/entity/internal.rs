use serde::{Deserialize, Serialize};

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, PartialOrd, Ord, Serialize, Deserialize)]
pub struct InternalEntity {
    index: u32,
    generation: u32,
}

impl InternalEntity {
    pub const PLACEHOLDER: Self = Self::new(u32::MAX, 0);

    pub const fn new(index: u32, generation: u32) -> Self {
        Self { index, generation }
    }

    pub const fn index(self) -> u32 {
        self.index
    }

    pub const fn generation(self) -> u32 {
        self.generation
    }

    pub const fn to_bits(self) -> u64 {
        self.index as u64 | ((self.generation as u64) << 32)
    }

    pub const fn from_bits(bits: u64) -> Self {
        Self {
            index: bits as u32,
            generation: (bits >> 32) as u32,
        }
    }
}
