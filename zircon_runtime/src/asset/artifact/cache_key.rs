use std::collections::hash_map::DefaultHasher;
use std::hash::{Hash, Hasher};

const LOWER_HEX_DIGITS: &[u8; 16] = b"0123456789abcdef";

// TODO: [CR-ASSET-ARTIFACT-0002] 查明此键是否仍要接入 ArtifactStore；目前检索到的调用仅在测试中，正式库工件按资源 ID 与修订定位。
/// 表达源内容、导入器版本和导入配置共同决定的缓存身份候选。
#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub struct LibraryCacheKey {
    source_hash: String,
    importer_version: u32,
    config_hash: String,
}

impl LibraryCacheKey {
    pub fn new(
        source_hash: impl Into<String>,
        importer_version: u32,
        config_hash: impl Into<String>,
    ) -> Self {
        Self {
            source_hash: source_hash.into(),
            importer_version,
            config_hash: config_hash.into(),
        }
    }

    /// 生成固定宽度的小写十六进制指纹；若用于持久命名，调用方需确认哈希算法的跨版本稳定性。
    pub fn fingerprint(&self) -> String {
        let mut hasher = DefaultHasher::new();
        self.hash(&mut hasher);
        fixed_lower_hex(hasher.finish())
    }
}

fn fixed_lower_hex(value: u64) -> String {
    let mut fingerprint = String::with_capacity(16);
    for shift in (0..=60).rev().step_by(4) {
        let nibble = ((value >> shift) & 0x0f) as usize;
        fingerprint.push(char::from(LOWER_HEX_DIGITS[nibble]));
    }
    fingerprint
}

#[cfg(test)]
#[path = "tests/cache_key.rs"]
mod tests;
