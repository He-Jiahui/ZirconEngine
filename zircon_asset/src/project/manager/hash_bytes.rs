use std::collections::hash_map::DefaultHasher;
use std::hash::{Hash, Hasher};

pub(super) fn hash_bytes(bytes: &[u8]) -> String {
    let mut hasher = DefaultHasher::new();
    bytes.hash(&mut hasher);
    format!("{:016x}", hasher.finish())
}
