use std::collections::{btree_map::Entry, BTreeMap};

/// 为内容哈希分配首次出现的序号；重复内容返回同一序号，序号只对当前表实例有效。
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct ZrPackDedupTable {
    chunks: BTreeMap<[u8; 32], usize>,
}

impl ZrPackDedupTable {
    /// 返回哈希以及已有 chunk 的索引；首次出现的内容在插入时占用当前长度。
    pub fn insert_or_get(&mut self, bytes: &[u8]) -> ([u8; 32], Option<usize>) {
        let hash = zrpack_content_hash(bytes);
        let index = self.chunks.len();
        match self.chunks.entry(hash) {
            Entry::Occupied(entry) => (hash, Some(*entry.get())),
            Entry::Vacant(entry) => {
                entry.insert(index);
                (hash, None)
            }
        }
    }

    pub fn len(&self) -> usize {
        self.chunks.len()
    }

    pub fn is_empty(&self) -> bool {
        self.chunks.is_empty()
    }
}

/// pack 格式使用 BLAKE3 内容摘要作为 chunk 去重键。
pub fn zrpack_content_hash(bytes: &[u8]) -> [u8; 32] {
    *blake3::hash(bytes).as_bytes()
}

#[cfg(test)]
#[path = "tests/dedup.rs"]
mod tests;

#[cfg(test)]
#[path = "dedup/tests/optimization_tests.rs"]
mod optimization_tests;
