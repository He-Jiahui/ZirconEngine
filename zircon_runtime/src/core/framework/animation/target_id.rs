use std::fmt;

use serde::{Deserialize, Serialize};

use crate::core::framework::scene::EntityPath;

const ANIMATION_TARGET_NAMESPACE: &[u8] = b"zircon.animation.target.v1";
const ANIMATION_TARGET_HASH_BLOCK_BYTES: usize = 64;
const ANIMATION_TARGET_SEGMENT_LENGTH_BYTES: usize = std::mem::size_of::<u64>();
const LOWER_HEX_DIGITS: &[u8; 16] = b"0123456789abcdef";

/// Stable identity for an animation target derived from its import path.
///
/// The identity contains no scene entity handle. Importers and runtime target
/// tables may therefore independently derive the same value from the same
/// ordered path segments.
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
pub struct AnimationTargetId([u8; 16]);

impl AnimationTargetId {
    pub fn from_path(path: &EntityPath) -> Self {
        Self::from_segments(path.segments())
    }

    /// 以有序路径段生成与场景句柄无关的目标身份；段边界参与哈希，
    /// 因而导入端和骨架目标表须传入同一分段规则，而非拼接后的任意字符串。
    pub fn from_segments<I, S>(segments: I) -> Self
    where
        I: IntoIterator<Item = S>,
        S: AsRef<str>,
    {
        let mut hasher = blake3::Hasher::new();
        hasher.update(ANIMATION_TARGET_NAMESPACE);
        for segment in segments {
            update_segment_hash(&mut hasher, segment.as_ref());
        }

        let mut bytes = [0_u8; 16];
        bytes.copy_from_slice(&hasher.finalize().as_bytes()[..16]);
        Self(bytes)
    }

    pub fn as_bytes(self) -> [u8; 16] {
        self.0
    }
}

fn update_segment_hash(hasher: &mut blake3::Hasher, segment: &str) {
    let bytes = segment.as_bytes();
    let segment_length = (bytes.len() as u64).to_le_bytes();
    if bytes.len() <= ANIMATION_TARGET_HASH_BLOCK_BYTES - ANIMATION_TARGET_SEGMENT_LENGTH_BYTES {
        let mut framed = [0_u8; ANIMATION_TARGET_HASH_BLOCK_BYTES];
        framed[..ANIMATION_TARGET_SEGMENT_LENGTH_BYTES].copy_from_slice(&segment_length);
        let framed_len = ANIMATION_TARGET_SEGMENT_LENGTH_BYTES + bytes.len();
        framed[ANIMATION_TARGET_SEGMENT_LENGTH_BYTES..framed_len].copy_from_slice(bytes);
        hasher.update(&framed[..framed_len]);
    } else {
        hasher.update(&segment_length);
        hasher.update(bytes);
    }
}

impl From<&EntityPath> for AnimationTargetId {
    fn from(path: &EntityPath) -> Self {
        Self::from_path(path)
    }
}

impl fmt::Display for AnimationTargetId {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        let mut encoded = [0_u8; 32];
        for (index, byte) in self.0.iter().copied().enumerate() {
            encoded[index * 2] = LOWER_HEX_DIGITS[usize::from(byte >> 4)];
            encoded[index * 2 + 1] = LOWER_HEX_DIGITS[usize::from(byte & 0x0f)];
        }
        let encoded = std::str::from_utf8(&encoded).expect("lower hexadecimal digits are ASCII");
        formatter.write_str(encoded)
    }
}

#[cfg(test)]
#[path = "tests/target_id.rs"]
mod tests;
