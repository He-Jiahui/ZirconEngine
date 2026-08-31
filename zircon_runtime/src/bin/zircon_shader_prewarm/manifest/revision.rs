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
        update_delimited_hash(&mut hasher, hash.as_bytes());
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
    update_delimited_hash(&mut hasher, &base_revision.to_le_bytes());
    for hash in include_content_hashes {
        update_delimited_hash(&mut hasher, hash.as_bytes());
    }
    non_zero_revision_from_hash(hasher.finalize())
}

fn non_zero_revision_from_bytes(bytes: &[u8]) -> u64 {
    non_zero_revision_from_hash(blake3::hash(bytes))
}

const DELIMITED_HASH_STACK_CAPACITY: usize = 65;

fn update_delimited_hash(hasher: &mut blake3::Hasher, value: &[u8]) {
    if value.len() < DELIMITED_HASH_STACK_CAPACITY {
        let mut buffered = [0_u8; DELIMITED_HASH_STACK_CAPACITY];
        buffered[..value.len()].copy_from_slice(value);
        buffered[value.len()] = 0;
        hasher.update(&buffered[..value.len() + 1]);
    } else {
        hasher.update(value);
        hasher.update(&[0]);
    }
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

#[cfg(test)]
mod tests {
    use super::*;

    fn legacy_content_revision(include_content_hashes: &[String]) -> u64 {
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

    fn legacy_base_revision(base_revision: u64, include_content_hashes: &[String]) -> u64 {
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

    #[test]
    fn optimization_batch_20260830ey_runtime564_preserves_content_revision_bytes() {
        let values = vec![
            String::new(),
            "short".to_owned(),
            "x".repeat(64),
            "y".repeat(65),
            "z".repeat(129),
        ];
        assert_eq!(
            asset_scan_revision_from_content_hashes(&values),
            legacy_content_revision(&values)
        );
    }

    #[test]
    fn optimization_batch_20260830ey_runtime564_preserves_base_revision_bytes() {
        let values = vec!["a".repeat(64), "b".repeat(65), "c".repeat(3)];
        assert_eq!(
            asset_scan_revision_from_base_revision_and_content_hashes(41, &values),
            legacy_base_revision(41, &values)
        );
        assert_eq!(
            asset_scan_revision_from_base_revision_and_content_hashes(41, &[]),
            41
        );
    }

    #[test]
    fn optimization_batch_20260830ey_runtime564_batches_short_values_into_one_update() {
        let source = include_str!("revision.rs");
        assert!(source.contains("buffered[value.len()] = 0"));
        assert!(source.contains("hasher.update(&buffered[..value.len() + 1])"));
    }
}
