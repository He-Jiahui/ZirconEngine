use serde::{Deserialize, Serialize};

const FNV_OFFSET_BASIS: u64 = 0xcbf29ce484222325;
const FNV_PRIME: u64 = 0x100000001b3;

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
pub struct UiAssetFingerprint {
    pub value: u64,
}

impl UiAssetFingerprint {
    pub fn from_bytes(bytes: &[u8]) -> Self {
        let mut value = FNV_OFFSET_BASIS;
        for byte in bytes {
            value ^= u64::from(*byte);
            value = value.wrapping_mul(FNV_PRIME);
        }
        Self { value }
    }
}
