//! 这个摘要写入 .zmeta 的 source_digest，并在重定位前与源字节比较；因此它属于跨运行版本的持久化身份约定。

use std::collections::hash_map::DefaultHasher;
use std::hash::{Hash, Hasher};

const LOWER_HEX_DIGITS: &[u8; 16] = b"0123456789abcdef";

// TODO: [CR-ASSET-TYPESPROJECT-0002] 确认持久化 source_digest 的算法/版本契约；当前 DefaultHasher 摘要写入 .zmeta，重定位再按当前构建比较，缺少可迁移的算法标识。证据：load_or_create_meta.rs 与 relocation.rs::validate_relocatable_source。
pub(super) fn hash_bytes(bytes: &[u8]) -> String {
    let mut hasher = DefaultHasher::new();
    bytes.hash(&mut hasher);
    fixed_lower_hex(hasher.finish())
}

fn fixed_lower_hex(value: u64) -> String {
    let mut encoded = String::with_capacity(16);
    for shift in (0..=60).rev().step_by(4) {
        let nibble = ((value >> shift) & 0x0f) as usize;
        encoded.push(char::from(LOWER_HEX_DIGITS[nibble]));
    }
    encoded
}

#[cfg(test)]
#[path = "tests/hash_bytes.rs"]
mod tests;
