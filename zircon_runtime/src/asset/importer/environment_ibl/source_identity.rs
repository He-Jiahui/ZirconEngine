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
    derive_source_identity_with_optional_format(bytes, None, face_size, mip_count)
}

pub(super) fn derive_source_identity_with_format(
    bytes: &[u8],
    format_identity: u32,
    face_size: u32,
    mip_count: u32,
) -> EnvironmentIblSourceIdentity {
    derive_source_identity_with_optional_format(bytes, Some(format_identity), face_size, mip_count)
}

fn derive_source_identity_with_optional_format(
    bytes: &[u8],
    format_identity: Option<u32>,
    face_size: u32,
    mip_count: u32,
) -> EnvironmentIblSourceIdentity {
    let mut hasher = blake3::Hasher::new();
    hasher.update(bytes);
    if let Some(format_identity) = format_identity {
        hasher.update(&format_identity.to_le_bytes());
    }
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
    use super::{derive_source_identity, derive_source_identity_with_format};

    #[test]
    fn combined_source_identity_is_deterministic_for_source_format_and_layout() {
        let cases: &[(&[u8], u32, u32, u32)] = &[
            (b"", 11, 1, 1),
            (b"same HDR source bytes", 11, 64, 7),
            (b"same HDR source bytes", 12, 128, 8),
            (b"changed HDR source bytes", 11, 256, 9),
        ];

        for &(source, format_identity, face_size, mip_count) in cases {
            let identity =
                derive_source_identity_with_format(source, format_identity, face_size, mip_count);
            assert_eq!(
                identity.revision(),
                expected_source_revision(source, format_identity),
                "source revision changed for format={format_identity}"
            );
            assert_eq!(
                identity.hash_words(),
                expected_source_hash_words(source, format_identity, face_size, mip_count),
                "source hash changed for face_size={face_size}, mip_count={mip_count}"
            );
        }
    }

    #[test]
    fn combined_source_identity_keeps_revision_layout_independent() {
        let source = b"same HDR source bytes";
        let compact = derive_source_identity_with_format(source, 11, 64, 7);
        let large = derive_source_identity_with_format(source, 11, 256, 9);

        assert_ne!(compact.revision(), 0);
        assert_eq!(compact.revision(), large.revision());
        assert_ne!(compact.hash_words(), large.hash_words());
    }

    #[test]
    fn combined_source_identity_distinguishes_resolved_decoder_format() {
        let source = b"same source bytes";
        let hdr = derive_source_identity_with_format(source, 11, 256, 9);
        let exr = derive_source_identity_with_format(source, 12, 256, 9);

        assert_ne!(hdr.revision(), exr.revision());
        assert_ne!(hdr.hash_words(), exr.hash_words());
    }

    #[test]
    fn container_source_identity_preserves_the_pre_format_artifact_key() {
        let source = b"same container bytes";
        let identity = derive_source_identity(source, 64, 7);

        let mut revision_hasher = blake3::Hasher::new();
        revision_hasher.update(source);
        let revision_digest = revision_hasher.finalize();
        assert_eq!(identity.revision(), super::digest_revision(revision_digest));

        revision_hasher.update(&64_u32.to_le_bytes());
        revision_hasher.update(&7_u32.to_le_bytes());
        assert_eq!(
            identity.hash_words(),
            super::digest_hash_words(revision_hasher.finalize())
        );
    }

    fn expected_source_hash_words(
        bytes: &[u8],
        format_identity: u32,
        face_size: u32,
        mip_count: u32,
    ) -> [u32; 4] {
        let mut hasher = blake3::Hasher::new();
        hasher.update(bytes);
        hasher.update(&format_identity.to_le_bytes());
        hasher.update(&face_size.to_le_bytes());
        hasher.update(&mip_count.to_le_bytes());
        digest_hash_words(hasher.finalize())
    }

    fn expected_source_revision(bytes: &[u8], format_identity: u32) -> u64 {
        let mut hasher = blake3::Hasher::new();
        hasher.update(bytes);
        hasher.update(&format_identity.to_le_bytes());
        let digest = hasher.finalize();
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
