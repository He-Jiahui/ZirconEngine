use std::collections::BTreeMap;

#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct ZrPackDedupTable {
    chunks: BTreeMap<[u8; 32], usize>,
}

impl ZrPackDedupTable {
    pub fn insert_or_get(&mut self, bytes: &[u8]) -> ([u8; 32], Option<usize>) {
        let hash = zrpack_content_hash(bytes);
        let existing = self.chunks.get(&hash).copied();
        if existing.is_none() {
            let index = self.chunks.len();
            self.chunks.insert(hash, index);
        }
        (hash, existing)
    }

    pub fn len(&self) -> usize {
        self.chunks.len()
    }

    pub fn is_empty(&self) -> bool {
        self.chunks.is_empty()
    }
}

pub fn zrpack_content_hash(bytes: &[u8]) -> [u8; 32] {
    let mut hash = [0u8; 32];
    for (index, seed) in ZRPACK_HASH_SEEDS.iter().enumerate() {
        let value = fnv1a64(bytes, *seed);
        hash[index * 8..index * 8 + 8].copy_from_slice(&value.to_le_bytes());
    }
    hash
}

const ZRPACK_HASH_SEEDS: [u64; 4] = [
    0xcbf2_9ce4_8422_2325,
    0x9ae1_6a3b_2f90_404f,
    0x6eed_0e9d_a4d9_4a4f,
    0xace5_929a_d4d9_8f13,
];

fn fnv1a64(bytes: &[u8], seed: u64) -> u64 {
    let mut hash = seed;
    for byte in bytes {
        hash ^= u64::from(*byte);
        hash = hash.wrapping_mul(0x100_0000_01b3);
    }
    hash
}
