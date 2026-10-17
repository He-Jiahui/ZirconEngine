use super::{EphemeralCacheHash, StableContentDigest};

#[test]
fn ephemeral_cache_hash_reuses_equal_runtime_inputs() {
    // 比较固定夹具在本实现内的结果；不据此证明所有不同输入均无碰撞。
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
    // 同一 BLAKE3 实现产生期望值；这里只核对包装及字节往返，不是独立摘要向量。
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
    // 仅比较值本体尺寸；不测量计算期间的分配或容器容量。
    assert_eq!(
        std::mem::size_of::<EphemeralCacheHash>(),
        std::mem::size_of::<u64>()
    );
    assert_eq!(
        std::mem::size_of::<StableContentDigest>(),
        std::mem::size_of::<[u8; 32]>()
    );
}
