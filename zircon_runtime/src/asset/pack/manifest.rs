use serde::{Deserialize, Serialize};
use std::collections::{BTreeMap, HashSet};
use std::fmt;

pub const ZRPACK_MAGIC: [u8; 4] = *b"ZRPK";
pub const ZRPACK_FORMAT_VERSION: u32 = 1;

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct ZrPackManifest {
    pub version: u32,
    pub chunks: Vec<ZrChunkEntry>,
    pub total_size: u64,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct ZrChunkEntry {
    pub hash: [u8; 32],
    pub offset: u64,
    pub size: u32,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct ZrPackDocumentManifest {
    pub pack: ZrPackManifest,
    pub assets: Vec<ZrPackAssetEntry>,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct ZrPackAssetEntry {
    pub path: String,
    pub chunk_hash: [u8; 32],
    pub size: u64,
}

impl ZrPackManifest {
    pub fn new(version: u32, total_size: u64) -> Self {
        Self {
            version,
            chunks: Vec::new(),
            total_size,
        }
    }

    pub fn with_chunk(mut self, chunk: ZrChunkEntry) -> Self {
        self.chunks.push(chunk);
        self
    }

    pub fn covered_bytes(&self) -> u64 {
        self.chunks.iter().map(|chunk| u64::from(chunk.size)).sum()
    }

    pub fn is_complete_byte_plan(&self) -> bool {
        self.covered_bytes() == self.total_size
    }
}

impl ZrChunkEntry {
    pub fn new(hash: [u8; 32], offset: u64, size: u32) -> Self {
        Self { hash, offset, size }
    }

    pub fn end_offset(&self) -> Option<u64> {
        self.offset.checked_add(u64::from(self.size))
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum ZrPackError {
    InvalidMagic,
    UnsupportedVersion(u32),
    HeaderTooSmall,
    ManifestOutOfBounds,
    ManifestTrailingBytes,
    ManifestDecode(String),
    UnsafeAssetPath(String),
    UnnormalizedAssetPath { path: String, normalized: String },
    UnsortedAssetPaths,
    DuplicateAssetPath(String),
    DuplicateChunkHash,
    UnsortedChunkHashes,
    PackTotalSizeMismatch,
    PackChunkTableMismatch,
    PayloadExtentMismatch,
    DeltaRemovedAssetsMismatch,
    DeltaChangedAssetsMismatch,
    DeltaChunkTableMismatch,
    MissingChunk(String),
    ChunkOutOfBounds(String),
    ChunkHashMismatch(String),
    AssetNotFound(String),
    DeltaBaseManifestMismatch,
    DeltaTargetManifestMismatch,
    SizeOverflow,
}

impl ZrPackDocumentManifest {
    pub fn new(pack: ZrPackManifest, assets: Vec<ZrPackAssetEntry>) -> Self {
        Self { pack, assets }
    }

    pub fn asset(&self, path: &str) -> Option<&ZrPackAssetEntry> {
        self.assets
            .binary_search_by(|asset| asset.path.as_str().cmp(path))
            .ok()
            .map(|index| &self.assets[index])
    }
}

impl ZrPackAssetEntry {
    pub fn new(path: impl Into<String>, chunk_hash: [u8; 32], size: u64) -> Self {
        Self {
            path: path.into(),
            chunk_hash,
            size,
        }
    }
}

impl fmt::Display for ZrPackError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::InvalidMagic => write!(formatter, "zrpack header magic is invalid"),
            Self::UnsupportedVersion(version) => {
                write!(formatter, "zrpack format version {version} is unsupported")
            }
            Self::HeaderTooSmall => write!(formatter, "zrpack header is too small"),
            Self::ManifestOutOfBounds => {
                write!(formatter, "zrpack manifest range is out of bounds")
            }
            Self::ManifestTrailingBytes => {
                write!(
                    formatter,
                    "zrpack manifest must end at the artifact boundary"
                )
            }
            Self::ManifestDecode(error) => {
                write!(formatter, "failed to decode zrpack manifest: {error}")
            }
            Self::UnsafeAssetPath(path) => {
                write!(
                    formatter,
                    "zrpack asset path {path} must be a safe relative asset path"
                )
            }
            Self::UnnormalizedAssetPath { path, normalized } => {
                write!(
                    formatter,
                    "zrpack asset path {path} must use normalized relative asset path {normalized}"
                )
            }
            Self::UnsortedAssetPaths => {
                write!(formatter, "zrpack asset paths must be sorted by asset path")
            }
            Self::DuplicateAssetPath(path) => {
                write!(formatter, "zrpack asset path {path} is duplicated")
            }
            Self::DuplicateChunkHash => {
                write!(formatter, "zrpack chunk hash is duplicated")
            }
            Self::UnsortedChunkHashes => {
                write!(
                    formatter,
                    "zrpack chunk hashes must be sorted by chunk hash"
                )
            }
            Self::PackTotalSizeMismatch => {
                write!(
                    formatter,
                    "zrpack pack total size does not match chunk table"
                )
            }
            Self::PackChunkTableMismatch => {
                write!(
                    formatter,
                    "zrpack chunk table does not match manifest assets"
                )
            }
            Self::PayloadExtentMismatch => {
                write!(
                    formatter,
                    "zrpack payload extent does not match manifest offset"
                )
            }
            Self::DeltaRemovedAssetsMismatch => {
                write!(
                    formatter,
                    "zrpack delta removed assets do not match base and target manifests"
                )
            }
            Self::DeltaChangedAssetsMismatch => {
                write!(
                    formatter,
                    "zrpack delta changed assets do not match base and target manifests"
                )
            }
            Self::DeltaChunkTableMismatch => {
                write!(
                    formatter,
                    "zrpack delta chunk table does not match changed assets"
                )
            }
            Self::MissingChunk(path) => {
                write!(formatter, "zrpack asset {path} references a missing chunk")
            }
            Self::ChunkOutOfBounds(path) => write!(
                formatter,
                "zrpack asset {path} chunk range is out of bounds"
            ),
            Self::ChunkHashMismatch(path) => write!(
                formatter,
                "zrpack asset {path} chunk payload does not match its content hash"
            ),
            Self::AssetNotFound(path) => write!(formatter, "zrpack asset {path} was not found"),
            Self::DeltaBaseManifestMismatch => {
                write!(
                    formatter,
                    "zrpack delta base manifest does not match the installed pack"
                )
            }
            Self::DeltaTargetManifestMismatch => {
                write!(
                    formatter,
                    "zrpack delta target manifest could not be reconstructed"
                )
            }
            Self::SizeOverflow => {
                write!(formatter, "zrpack size does not fit into the binary format")
            }
        }
    }
}

impl std::error::Error for ZrPackError {}

pub(crate) fn validate_zrpack_document_manifest(
    manifest: &ZrPackDocumentManifest,
) -> Result<(), ZrPackError> {
    if manifest.pack.version != ZRPACK_FORMAT_VERSION {
        return Err(ZrPackError::UnsupportedVersion(manifest.pack.version));
    }
    validate_zrpack_asset_entries(&manifest.assets)?;
    validate_zrpack_chunk_table(manifest)
}

pub(crate) fn validate_zrpack_asset_entries(
    assets: &[ZrPackAssetEntry],
) -> Result<(), ZrPackError> {
    let mut seen_paths = HashSet::new();
    for asset in assets {
        validate_zrpack_asset_path(&asset.path)?;
        if !seen_paths.insert(asset.path.as_str()) {
            return Err(ZrPackError::DuplicateAssetPath(asset.path.clone()));
        }
    }
    if assets
        .windows(2)
        .any(|pair| pair[0].path.as_str() > pair[1].path.as_str())
    {
        return Err(ZrPackError::UnsortedAssetPaths);
    }
    Ok(())
}

pub(crate) fn validate_zrpack_asset_path_list(paths: &[String]) -> Result<(), ZrPackError> {
    let mut seen_paths = HashSet::new();
    for path in paths {
        validate_zrpack_asset_path(path)?;
        if !seen_paths.insert(path.as_str()) {
            return Err(ZrPackError::DuplicateAssetPath(path.clone()));
        }
    }
    if paths
        .windows(2)
        .any(|pair| pair[0].as_str() > pair[1].as_str())
    {
        return Err(ZrPackError::UnsortedAssetPaths);
    }
    Ok(())
}

pub(crate) fn validate_zrpack_asset_path(path: &str) -> Result<(), ZrPackError> {
    let normalized = normalized_zrpack_asset_path(path);
    if !is_safe_normalized_zrpack_asset_path(&normalized) {
        return Err(ZrPackError::UnsafeAssetPath(path.to_string()));
    }
    if normalized != path {
        return Err(ZrPackError::UnnormalizedAssetPath {
            path: path.to_string(),
            normalized,
        });
    }
    Ok(())
}

fn validate_zrpack_chunk_table(manifest: &ZrPackDocumentManifest) -> Result<(), ZrPackError> {
    let mut seen_chunk_hashes = HashSet::new();
    for chunk in &manifest.pack.chunks {
        if !seen_chunk_hashes.insert(&chunk.hash) {
            return Err(ZrPackError::DuplicateChunkHash);
        }
    }
    if manifest
        .pack
        .chunks
        .windows(2)
        .any(|pair| pair[0].hash > pair[1].hash)
    {
        return Err(ZrPackError::UnsortedChunkHashes);
    }

    let total_size = manifest
        .pack
        .chunks
        .iter()
        .try_fold(0_u64, |total, chunk| {
            total
                .checked_add(u64::from(chunk.size))
                .ok_or(ZrPackError::SizeOverflow)
        })?;
    if manifest.pack.total_size != total_size {
        return Err(ZrPackError::PackTotalSizeMismatch);
    }

    let chunk_sizes = manifest
        .pack
        .chunks
        .iter()
        .map(|chunk| (chunk.hash, u64::from(chunk.size)))
        .collect::<BTreeMap<_, _>>();
    let mut asset_hashes = HashSet::new();
    for asset in &manifest.assets {
        let chunk_size = chunk_sizes
            .get(&asset.chunk_hash)
            .ok_or_else(|| ZrPackError::MissingChunk(asset.path.clone()))?;
        if asset.size != *chunk_size {
            return Err(ZrPackError::ChunkOutOfBounds(asset.path.clone()));
        }
        asset_hashes.insert(asset.chunk_hash);
    }

    if asset_hashes != chunk_sizes.keys().copied().collect::<HashSet<_>>() {
        return Err(ZrPackError::PackChunkTableMismatch);
    }

    Ok(())
}

fn normalized_zrpack_asset_path(path: &str) -> String {
    path.trim().replace('\\', "/")
}

fn is_safe_normalized_zrpack_asset_path(path: &str) -> bool {
    !path.is_empty()
        && !path.starts_with('/')
        && !path.contains(':')
        && path
            .split('/')
            .all(|part| !part.is_empty() && part != "." && part != "..")
}

#[cfg(test)]
mod optimization_tests {
    use std::collections::BTreeSet;
    use std::hint::black_box;
    use std::time::{Duration, Instant};

    use super::*;

    const PATH_ADMISSION_COUNT: usize = 65_536;
    const UNIQUE_PATH_COUNT: usize = 8_192;
    const SAMPLE_COUNT: usize = 17;

    fn percentile_95(samples: &mut [Duration]) -> Duration {
        samples.sort_unstable();
        samples[(samples.len() - 1) * 95 / 100]
    }

    fn asset_paths() -> Vec<String> {
        (0..PATH_ADMISSION_COUNT)
            .map(|index| {
                format!(
                    "generated/assets/with/a/long/shared/prefix/artifact_{:05}.bin",
                    (index * 4_099) % UNIQUE_PATH_COUNT
                )
            })
            .collect()
    }

    fn ordered_unique_count(paths: &[String]) -> usize {
        let mut unique = BTreeSet::new();
        paths
            .iter()
            .filter(|path| unique.insert(path.as_str()))
            .count()
    }

    fn hash_unique_count(paths: &[String]) -> usize {
        let mut unique = HashSet::new();
        paths
            .iter()
            .filter(|path| unique.insert(path.as_str()))
            .count()
    }

    #[test]
    fn optimization_batch_20260826ab_runtime04_hash_manifest_validation_preserves_first_duplicate_error(
    ) {
        let assets = vec![
            ZrPackAssetEntry::new("assets/a.bin", [1; 32], 1),
            ZrPackAssetEntry::new("assets/b.bin", [2; 32], 1),
            ZrPackAssetEntry::new("assets/a.bin", [3; 32], 1),
        ];

        assert_eq!(
            validate_zrpack_asset_entries(&assets),
            Err(ZrPackError::DuplicateAssetPath("assets/a.bin".to_string()))
        );
    }

    #[test]
    fn optimization_batch_20260826ab_runtime04_pack_manifest_uses_hash_membership_and_sorted_windows(
    ) {
        let source = include_str!("manifest.rs");
        let production = source.split("#[cfg(test)]").next().unwrap();

        assert!(production.contains("use std::collections::{BTreeMap, HashSet};"));
        assert_eq!(production.matches("HashSet::new()").count(), 4);
        assert!(production.contains("collect::<HashSet<_>>()"));
        assert!(production.matches(".windows(2)").count() >= 2);
        assert!(!production.contains("BTreeSet"));
    }

    #[test]
    #[ignore = "release performance evidence"]
    fn optimization_batch_20260826ab_runtime04_pack_manifest_hash_validation_performance_evidence()
    {
        let paths = asset_paths();
        assert_eq!(ordered_unique_count(&paths), hash_unique_count(&paths));

        let mut ordered_samples = Vec::with_capacity(SAMPLE_COUNT);
        let mut hash_samples = Vec::with_capacity(SAMPLE_COUNT);
        for sample in 0..SAMPLE_COUNT {
            if sample % 2 == 0 {
                let started = Instant::now();
                black_box(ordered_unique_count(black_box(&paths)));
                ordered_samples.push(started.elapsed());

                let started = Instant::now();
                black_box(hash_unique_count(black_box(&paths)));
                hash_samples.push(started.elapsed());
            } else {
                let started = Instant::now();
                black_box(hash_unique_count(black_box(&paths)));
                hash_samples.push(started.elapsed());

                let started = Instant::now();
                black_box(ordered_unique_count(black_box(&paths)));
                ordered_samples.push(started.elapsed());
            }
        }

        let ordered_p95 = percentile_95(&mut ordered_samples);
        let hash_p95 = percentile_95(&mut hash_samples);
        println!(
            "RUNTIME04_PACK_MANIFEST_HASH_VALIDATION_BENCH_V1 \
             admissions={PATH_ADMISSION_COUNT} unique_paths={UNIQUE_PATH_COUNT} \
             borrowed_identity=true sorted_windows_preserved=true \
             ordered_p95_ns={} hash_p95_ns={}",
            ordered_p95.as_nanos(),
            hash_p95.as_nanos(),
        );
        assert!(
            hash_p95.as_nanos() * 100 <= ordered_p95.as_nanos() * 60,
            "hash-validation P95 {:?} exceeded 60% of ordered-validation P95 {:?}",
            hash_p95,
            ordered_p95,
        );
    }
}
