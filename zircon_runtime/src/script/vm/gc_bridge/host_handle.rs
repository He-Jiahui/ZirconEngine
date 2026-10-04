use serde::{Deserialize, Serialize};

/// Stable host-owned identity encoded as generation bits followed by slot-index bits.
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
pub struct HostHandle {
    index: u32,
    generation: u32,
}

impl HostHandle {
    // HostRegistry 用 index 定位槽位、用 generation 拒绝旧句柄；两者合并为脚本侧的 u64 身份。
    const INDEX_MASK: u64 = u32::MAX as u64;
    const GENERATION_SHIFT: u32 = u32::BITS;

    pub const fn from_parts(index: u32, generation: u32) -> Self {
        Self { index, generation }
    }

    pub const fn from_raw(raw: u64) -> Self {
        Self {
            index: (raw & Self::INDEX_MASK) as u32,
            generation: (raw >> Self::GENERATION_SHIFT) as u32,
        }
    }

    pub const fn index(self) -> u32 {
        self.index
    }

    pub const fn generation(self) -> u32 {
        self.generation
    }

    pub const fn into_raw(self) -> u64 {
        ((self.generation as u64) << Self::GENERATION_SHIFT) | self.index as u64
    }
}

#[cfg(test)]
#[path = "tests/host_handle.rs"]
mod tests;
