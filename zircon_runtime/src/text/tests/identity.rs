use super::{EphemeralCacheHash, StableContentDigest};

#[test]
fn ephemeral_cache_hash_reuses_equal_runtime_inputs() {
    assert_eq!(
        EphemeralCacheHash::from_hashable("same text"),
        EphemeralCacheHash::from_hashable("same text")
    );
    assert_ne!(
        EphemeralCacheHash::from_hashable("same text"),
        EphemeralCacheHash::from_hashable("different text")
    );
}

#[test]
fn stable_content_digest_preserves_blake3_artifact_bytes() {
    let expected = *blake3::hash(b"stable artifact bytes").as_bytes();
    let digest = StableContentDigest::blake3(b"stable artifact bytes");

    assert_eq!(digest.as_bytes(), &expected);
    assert_eq!(
        StableContentDigest::from_bytes(expected).into_bytes(),
        expected
    );
}

#[test]
fn identity_newtypes_add_no_storage_overhead() {
    assert_eq!(
        std::mem::size_of::<EphemeralCacheHash>(),
        std::mem::size_of::<u64>()
    );
    assert_eq!(
        std::mem::size_of::<StableContentDigest>(),
        std::mem::size_of::<[u8; 32]>()
    );
}
