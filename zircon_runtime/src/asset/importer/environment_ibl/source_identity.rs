#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(super) struct EnvironmentIblSourceIdentity {
    revision: u64,
    hash_words: [u32; 4],
}

impl EnvironmentIblSourceIdentity {
    pub(super) const fn revision(self) -> u64 {
        self.revision
    }

    pub(super) const fn hash_words(self) -> [u32; 4] {
        self.hash_words
    }
}

pub(super) fn derive_source_identity(
    bytes: &[u8],
    face_size: u32,
    mip_count: u32,
) -> EnvironmentIblSourceIdentity {
    let mut hasher = blake3::Hasher::new();
    hasher.update(bytes);
    let revision = digest_revision(hasher.finalize());
    hasher.update(&face_size.to_le_bytes());
    hasher.update(&mip_count.to_le_bytes());
    let hash_words = digest_hash_words(hasher.finalize());
    EnvironmentIblSourceIdentity {
        revision,
        hash_words,
    }
}

fn digest_revision(digest: blake3::Hash) -> u64 {
    let bytes = digest.as_bytes();
    u64::from_le_bytes([
        bytes[0], bytes[1], bytes[2], bytes[3], bytes[4], bytes[5], bytes[6], bytes[7],
    ])
    .max(1)
}

fn digest_hash_words(digest: blake3::Hash) -> [u32; 4] {
    let bytes = digest.as_bytes();
    [
        u32::from_le_bytes([bytes[0], bytes[1], bytes[2], bytes[3]]),
        u32::from_le_bytes([bytes[4], bytes[5], bytes[6], bytes[7]]),
        u32::from_le_bytes([bytes[8], bytes[9], bytes[10], bytes[11]]),
        u32::from_le_bytes([bytes[12], bytes[13], bytes[14], bytes[15]]),
    ]
}

#[cfg(test)]
mod tests {
    use super::derive_source_identity;

    #[test]
    fn combined_source_identity_preserves_legacy_cache_keys() {
        let cases: &[(&[u8], u32, u32)] = &[
            (b"", 1, 1),
            (b"same HDR source bytes", 64, 7),
            (b"same HDR source bytes", 128, 8),
            (b"changed HDR source bytes", 256, 9),
        ];

        for &(source, face_size, mip_count) in cases {
            let identity = derive_source_identity(source, face_size, mip_count);
            assert_eq!(
                identity.revision(),
                legacy_source_revision(source),
                "source revision changed for face_size={face_size}, mip_count={mip_count}"
            );
            assert_eq!(
                identity.hash_words(),
                legacy_source_hash_words(source, face_size, mip_count),
                "source hash changed for face_size={face_size}, mip_count={mip_count}"
            );
        }
    }

    #[test]
    fn combined_source_identity_keeps_revision_layout_independent() {
        let source = b"same HDR source bytes";
        let compact = derive_source_identity(source, 64, 7);
        let large = derive_source_identity(source, 256, 9);

        assert_ne!(compact.revision(), 0);
        assert_eq!(compact.revision(), large.revision());
        assert_ne!(compact.hash_words(), large.hash_words());
    }

    fn legacy_source_hash_words(bytes: &[u8], face_size: u32, mip_count: u32) -> [u32; 4] {
        let mut hasher = blake3::Hasher::new();
        hasher.update(bytes);
        hasher.update(&face_size.to_le_bytes());
        hasher.update(&mip_count.to_le_bytes());
        digest_hash_words(hasher.finalize())
    }

    fn legacy_source_revision(bytes: &[u8]) -> u64 {
        let digest = blake3::hash(bytes);
        let bytes = digest.as_bytes();
        u64::from_le_bytes([
            bytes[0], bytes[1], bytes[2], bytes[3], bytes[4], bytes[5], bytes[6], bytes[7],
        ])
        .max(1)
    }

    fn digest_hash_words(digest: blake3::Hash) -> [u32; 4] {
        let bytes = digest.as_bytes();
        [
            u32::from_le_bytes([bytes[0], bytes[1], bytes[2], bytes[3]]),
            u32::from_le_bytes([bytes[4], bytes[5], bytes[6], bytes[7]]),
            u32::from_le_bytes([bytes[8], bytes[9], bytes[10], bytes[11]]),
            u32::from_le_bytes([bytes[12], bytes[13], bytes[14], bytes[15]]),
        ]
    }
}
