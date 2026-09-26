use serde::{Deserialize, Serialize};

const FNV_OFFSET_BASIS: u64 = 0xcbf29ce484222325;
const FNV_PRIME: u64 = 0x100000001b3;

/// 内容变化检测用的 FNV-1a 64 位指纹，供缓存键、依赖快照和产物清单记录字节变化。
/// 它是快速非密码学摘要，不能作为防篡改或来源认证依据。
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
