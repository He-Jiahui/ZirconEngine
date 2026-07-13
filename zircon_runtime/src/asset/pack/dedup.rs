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
    *blake3::hash(bytes).as_bytes()
}

#[cfg(test)]
mod tests {
    use super::zrpack_content_hash;

    #[test]
    fn zrpack_content_hash_matches_blake3_empty_input_vector() {
        assert_eq!(
            zrpack_content_hash(b""),
            [
                0xaf, 0x13, 0x49, 0xb9, 0xf5, 0xf9, 0xa1, 0xa6, 0xa0, 0x40, 0x4d, 0xea, 0x36, 0xdc,
                0xc9, 0x49, 0x9b, 0xcb, 0x25, 0xc9, 0xad, 0xc1, 0x12, 0xb7, 0xcc, 0x9a, 0x93, 0xca,
                0xe4, 0x1f, 0x32, 0x62,
            ]
        );
    }
}
