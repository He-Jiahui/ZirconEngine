pub(super) const ASSET_SCAN_INITIAL_RESOURCE_REVISION: u64 = 1;

pub(super) fn asset_scan_revision_from_source_digest(source_digest: &str) -> u64 {
    let source_digest = source_digest.trim();
    if source_digest.is_empty() {
        return ASSET_SCAN_INITIAL_RESOURCE_REVISION;
    }
    non_zero_revision_from_bytes(source_digest.as_bytes())
}

pub(super) fn asset_scan_revision_from_content_hashes(include_content_hashes: &[String]) -> u64 {
    if include_content_hashes.is_empty() {
        return ASSET_SCAN_INITIAL_RESOURCE_REVISION;
    }

    let mut hasher = blake3::Hasher::new();
    for hash in include_content_hashes {
        hasher.update(hash.as_bytes());
        hasher.update(&[0]);
    }
    non_zero_revision_from_hash(hasher.finalize())
}

pub(super) fn asset_scan_revision_from_base_revision_and_content_hashes(
    base_revision: u64,
    include_content_hashes: &[String],
) -> u64 {
    if include_content_hashes.is_empty() {
        return base_revision;
    }

    let mut hasher = blake3::Hasher::new();
    hasher.update(&base_revision.to_le_bytes());
    hasher.update(&[0]);
    for hash in include_content_hashes {
        hasher.update(hash.as_bytes());
        hasher.update(&[0]);
    }
    non_zero_revision_from_hash(hasher.finalize())
}

fn non_zero_revision_from_bytes(bytes: &[u8]) -> u64 {
    non_zero_revision_from_hash(blake3::hash(bytes))
}

fn non_zero_revision_from_hash(hash: blake3::Hash) -> u64 {
    let mut bytes = [0; 8];
    bytes.copy_from_slice(&hash.as_bytes()[..8]);
    let revision = u64::from_le_bytes(bytes);
    if revision == 0 {
        ASSET_SCAN_INITIAL_RESOURCE_REVISION
    } else {
        revision
    }
}
