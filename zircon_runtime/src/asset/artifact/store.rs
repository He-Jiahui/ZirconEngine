use std::fs::{self, File, OpenOptions};
use std::io::{self, BufReader, Read, Write};
use std::path::{Path, PathBuf};
use std::sync::{
    atomic::{AtomicU64, Ordering},
    Arc,
};

use bincode::Options;
use serde::{Deserialize, Serialize};

use crate::core::resource::io::atomic_write;
use crate::core::resource::{ResourceRecord, ResourceScheme};

use super::cache_payload::ArtifactCacheAsset;
use super::chunk_residency::{
    chunk_path, ArtifactChunkDescriptor, ArtifactChunkInventory, ArtifactChunkResidency,
    ArtifactChunkResidencyDiagnostics, ChunkReader, ARTIFACT_CHUNK_BYTES, ARTIFACT_CHUNK_DIRECTORY,
};
use crate::asset::project::ProjectPaths;
use crate::asset::{
    asset_kind_for_imported_asset, AssetImportError, AssetKind, AssetUri, ImportedAsset,
};

const ARTIFACT_CACHE_EXTENSION: &str = "zasset";
const ARTIFACT_CACHE_SUFFIX: &str = ".zasset";
const ARTIFACT_MANIFEST_MAGIC: &[u8] = b"ZRARTM05";
const ARTIFACT_MANIFEST_SCHEMA_VERSION: u32 = 5;
const ARTIFACT_STAGING_DIRECTORY: &str = ".staging";
const ARTIFACT_CACHE_ZSTD_LEVEL: i32 = 1;
const BLAKE3_HEX_LENGTH: usize = 64;
const ZSTD_COMPRESS_BOUND_SMALL_INPUT_BYTES: u64 = 128 * 1024;
const ZSTD_COMPRESS_BOUND_SMALL_INPUT_MARGIN_DIVISOR: u64 = 2 * 1024;
// A 2 GiB generation can require roughly 32k 64 KiB chunks when compression
// does not reduce its size. Keep malformed manifests bounded while admitting
// that maximum supported chunk inventory.
const ARTIFACT_MANIFEST_MAX_BYTES: usize = 4 * 1024 * 1024;
const ARTIFACT_RAW_PAYLOAD_MAX_BYTES: u64 = 2 * 1024 * 1024 * 1024;

static NEXT_ARTIFACT_STAGING_ID: AtomicU64 = AtomicU64::new(1);

#[derive(Clone, Debug, Default)]
pub struct ArtifactStore {
    chunk_residency: ArtifactChunkResidency,
}

pub(crate) struct PreparedArtifactWrite {
    pub(crate) locator: AssetUri,
    pub(crate) artifact_path: PathBuf,
    pub(crate) payload: Vec<u8>,
    pub(crate) raw_bytes: u64,
    pub(crate) compressed_bytes: u64,
    pub(crate) chunk_count: usize,
}

impl ArtifactStore {
    pub fn with_chunk_residency_budget(max_resident_bytes: usize) -> Self {
        Self {
            chunk_residency: ArtifactChunkResidency::with_max_resident_bytes(max_resident_bytes),
        }
    }

    pub fn write(
        &self,
        paths: &ProjectPaths,
        metadata: &ResourceRecord,
        asset: &ImportedAsset,
    ) -> Result<AssetUri, AssetImportError> {
        let prepared = self.prepare_write(paths, metadata, asset)?;
        let locator = prepared.locator.clone();
        atomic_write(&prepared.artifact_path, &prepared.payload)?;
        Ok(locator)
    }

    pub(crate) fn prepare_write(
        &self,
        paths: &ProjectPaths,
        metadata: &ResourceRecord,
        asset: &ImportedAsset,
    ) -> Result<PreparedArtifactWrite, AssetImportError> {
        let asset_kind = asset_kind_for_imported_asset(asset);
        if asset_kind != metadata.kind {
            return Err(AssetImportError::Parse(format!(
                "artifact metadata kind mismatch: record is {:?}, payload is {asset_kind:?}",
                metadata.kind
            )));
        }
        let relative_path = format!(
            "{}/{}.{}",
            asset_kind_directory(metadata.kind),
            metadata.id(),
            ARTIFACT_CACHE_EXTENSION
        );
        let artifact_uri = AssetUri::parse(&format!("lib://{relative_path}"))?;
        let artifact_path = resolve_artifact_cache_path(paths, &artifact_uri)?;
        let cache_asset = ArtifactCacheAsset::from_imported(asset)?;
        let staged = StagedArtifactPayload::write(paths.asset_artifact_root(), &cache_asset)?;
        let chunks = publish_chunks(paths.asset_artifact_root(), staged.path())?;
        let raw_bytes = staged.raw_bytes();
        let compressed_bytes = staged.compressed_bytes();
        let chunk_count = chunks.len();
        let manifest = ArtifactManifest {
            schema_version: ARTIFACT_MANIFEST_SCHEMA_VERSION,
            kind: metadata.kind,
            revision: metadata.revision,
            content_hash: staged.content_hash().to_owned(),
            raw_bytes,
            compressed_bytes,
            chunks,
        };
        let payload = serialize_manifest(&manifest)?;

        // Chunks are immutable and complete before this atomic publication. A
        // failed write therefore leaves the previously readable manifest intact.
        Ok(PreparedArtifactWrite {
            locator: artifact_uri,
            artifact_path,
            payload,
            raw_bytes,
            compressed_bytes,
            chunk_count,
        })
    }

    pub fn read(
        &self,
        paths: &ProjectPaths,
        artifact_uri: &AssetUri,
    ) -> Result<ImportedAsset, AssetImportError> {
        self.read_with_raw_payload_limit(paths, artifact_uri, ARTIFACT_RAW_PAYLOAD_MAX_BYTES)
    }

    pub(crate) fn read_with_raw_payload_limit(
        &self,
        paths: &ProjectPaths,
        artifact_uri: &AssetUri,
        max_raw_payload_bytes: u64,
    ) -> Result<ImportedAsset, AssetImportError> {
        let inventory = self.open_chunk_inventory(paths, artifact_uri)?;
        let expected_kind = inventory.kind();
        let expected_raw_bytes = inventory.raw_bytes();
        if expected_raw_bytes > max_raw_payload_bytes {
            return Err(AssetImportError::ArtifactRawPayloadLimitExceeded {
                raw_bytes: expected_raw_bytes,
                limit_bytes: max_raw_payload_bytes,
            });
        }
        let reader = ChunkReader::new(inventory, self.chunk_residency.clone());
        let decoder = zstd::stream::read::Decoder::new(reader)?;
        let mut decoder = CountingReader::new(decoder);
        let cache_asset: ArtifactCacheAsset = bincode::DefaultOptions::new()
            .with_fixint_encoding()
            .with_limit(expected_raw_bytes)
            .deserialize_from(&mut decoder)
            .map_err(AssetImportError::ArtifactCacheDeserialize)?;
        let mut trailing = [0; ARTIFACT_CHUNK_BYTES];
        if decoder.read(&mut trailing)? != 0 {
            return Err(AssetImportError::Parse(
                "artifact payload has trailing decoded bytes".to_string(),
            ));
        }
        let decoded_bytes = decoder.bytes_read();
        let mut chunk_reader = decoder.into_inner().finish().into_inner();
        chunk_reader.verify_complete()?;
        if decoded_bytes != expected_raw_bytes {
            return Err(AssetImportError::Parse(format!(
                "artifact payload raw size mismatch: decoded {decoded_bytes}, manifest records {expected_raw_bytes}"
            )));
        }
        let asset = cache_asset.into_imported()?;
        let actual_kind = asset_kind_for_imported_asset(&asset);
        if actual_kind != expected_kind {
            return Err(AssetImportError::Parse(format!(
                "artifact manifest kind mismatch: manifest is {expected_kind:?}, payload is {actual_kind:?}"
            )));
        }
        Ok(asset)
    }

    pub fn open_chunk_inventory(
        &self,
        paths: &ProjectPaths,
        artifact_uri: &AssetUri,
    ) -> Result<ArtifactChunkInventory, AssetImportError> {
        let artifact_path = resolve_artifact_cache_path(paths, artifact_uri)?;
        let manifest = read_manifest(&artifact_path)?;
        validate_manifest(artifact_uri.path(), &manifest)?;
        Ok(ArtifactChunkInventory::new(
            paths.asset_artifact_root(),
            manifest.kind,
            manifest.revision,
            manifest.content_hash,
            manifest.raw_bytes,
            manifest.compressed_bytes,
            manifest.chunks,
        ))
    }

    pub fn read_compressed_chunk(
        &self,
        inventory: &ArtifactChunkInventory,
        index: usize,
    ) -> Result<Arc<[u8]>, AssetImportError> {
        Ok(self.chunk_residency.read(inventory, index)?)
    }

    pub fn chunk_residency_diagnostics(
        &self,
    ) -> Result<ArtifactChunkResidencyDiagnostics, AssetImportError> {
        Ok(self.chunk_residency.diagnostics()?)
    }
}

#[derive(Clone, Debug, Serialize, Deserialize)]
struct ArtifactManifest {
    schema_version: u32,
    kind: AssetKind,
    revision: u64,
    content_hash: String,
    raw_bytes: u64,
    compressed_bytes: u64,
    chunks: Vec<ArtifactChunkDescriptor>,
}

struct StagedArtifactPayload {
    path: PathBuf,
    content_hash: String,
    raw_bytes: u64,
    compressed_bytes: u64,
}

impl StagedArtifactPayload {
    fn write(
        artifact_root: &Path,
        cache_asset: &ArtifactCacheAsset,
    ) -> Result<Self, AssetImportError> {
        let (path, file) = create_staging_file(artifact_root)?;
        let result: Result<Self, AssetImportError> = (|| {
            let writer = HashingWriter::new(file);
            let encoder = zstd::stream::Encoder::new(writer, ARTIFACT_CACHE_ZSTD_LEVEL)?;
            let mut raw_writer = RawPayloadLimitWriter::new(encoder);
            if let Err(error) = bincode::serialize_into(&mut raw_writer, cache_asset) {
                if raw_writer.limit_exceeded() {
                    return Err(artifact_raw_payload_size_error(
                        ARTIFACT_RAW_PAYLOAD_MAX_BYTES.saturating_add(1),
                    ));
                }
                return Err(AssetImportError::ArtifactCacheSerialize(error));
            }
            let raw_bytes = raw_writer.bytes_written();
            validate_artifact_raw_payload_bytes(raw_bytes)?;
            let encoder = raw_writer.finish();
            let writer = encoder.finish()?;
            let (file, content_hash, compressed_bytes) = writer.finish();
            validate_artifact_compressed_payload_bytes(raw_bytes, compressed_bytes)?;
            file.sync_all()?;
            Ok(Self {
                path: path.clone(),
                content_hash,
                raw_bytes,
                compressed_bytes,
            })
        })();
        if result.is_err() {
            let _ = fs::remove_file(&path);
        }
        result
    }

    fn path(&self) -> &Path {
        &self.path
    }

    fn content_hash(&self) -> &str {
        &self.content_hash
    }

    fn raw_bytes(&self) -> u64 {
        self.raw_bytes
    }

    fn compressed_bytes(&self) -> u64 {
        self.compressed_bytes
    }
}

impl Drop for StagedArtifactPayload {
    fn drop(&mut self) {
        let _ = fs::remove_file(&self.path);
    }
}

struct RawPayloadLimitWriter<W> {
    inner: W,
    bytes_written: u64,
    limit_exceeded: bool,
}

impl<W> RawPayloadLimitWriter<W> {
    fn new(inner: W) -> Self {
        Self {
            inner,
            bytes_written: 0,
            limit_exceeded: false,
        }
    }

    fn bytes_written(&self) -> u64 {
        self.bytes_written
    }

    fn limit_exceeded(&self) -> bool {
        self.limit_exceeded
    }

    fn finish(self) -> W {
        self.inner
    }
}

impl<W: Write> Write for RawPayloadLimitWriter<W> {
    fn write(&mut self, bytes: &[u8]) -> io::Result<usize> {
        let requested_bytes = u64::try_from(bytes.len()).map_err(|_| {
            self.limit_exceeded = true;
            io::Error::new(
                io::ErrorKind::InvalidData,
                "artifact raw payload write size overflow",
            )
        })?;
        let next_bytes = self
            .bytes_written
            .checked_add(requested_bytes)
            .filter(|next| *next <= ARTIFACT_RAW_PAYLOAD_MAX_BYTES)
            .ok_or_else(|| {
                self.limit_exceeded = true;
                io::Error::new(
                    io::ErrorKind::InvalidData,
                    "artifact raw payload exceeds the runtime budget",
                )
            })?;
        let written = self.inner.write(bytes)?;
        self.bytes_written = self
            .bytes_written
            .checked_add(written as u64)
            .filter(|total| *total <= next_bytes)
            .ok_or_else(|| {
                io::Error::new(
                    io::ErrorKind::InvalidData,
                    "artifact raw payload writer reported an invalid byte count",
                )
            })?;
        Ok(written)
    }

    fn flush(&mut self) -> io::Result<()> {
        self.inner.flush()
    }
}

struct HashingWriter<W> {
    inner: W,
    hasher: blake3::Hasher,
    bytes_written: u64,
}

impl<W> HashingWriter<W> {
    fn new(inner: W) -> Self {
        Self {
            inner,
            hasher: blake3::Hasher::new(),
            bytes_written: 0,
        }
    }

    fn finish(self) -> (W, String, u64) {
        (
            self.inner,
            self.hasher.finalize().to_hex().to_string(),
            self.bytes_written,
        )
    }
}

impl<W: Write> Write for HashingWriter<W> {
    fn write(&mut self, bytes: &[u8]) -> io::Result<usize> {
        let written = self.inner.write(bytes)?;
        self.hasher.update(&bytes[..written]);
        self.bytes_written += written as u64;
        Ok(written)
    }

    fn flush(&mut self) -> io::Result<()> {
        self.inner.flush()
    }
}

struct CountingReader<R> {
    inner: R,
    bytes_read: u64,
}

impl<R> CountingReader<R> {
    fn new(inner: R) -> Self {
        Self {
            inner,
            bytes_read: 0,
        }
    }

    fn bytes_read(&self) -> u64 {
        self.bytes_read
    }

    fn into_inner(self) -> R {
        self.inner
    }
}

impl<R: Read> Read for CountingReader<R> {
    fn read(&mut self, buffer: &mut [u8]) -> io::Result<usize> {
        let bytes_read = self.inner.read(buffer)?;
        self.bytes_read += bytes_read as u64;
        Ok(bytes_read)
    }
}

fn create_staging_file(artifact_root: &Path) -> Result<(PathBuf, File), AssetImportError> {
    let directory = artifact_root.join(ARTIFACT_STAGING_DIRECTORY);
    fs::create_dir_all(&directory)?;
    loop {
        let id = NEXT_ARTIFACT_STAGING_ID.fetch_add(1, Ordering::Relaxed);
        let path = directory.join(format!("artifact-{id}.staging"));
        match OpenOptions::new().write(true).create_new(true).open(&path) {
            Ok(file) => return Ok((path, file)),
            Err(error) if error.kind() == io::ErrorKind::AlreadyExists => continue,
            Err(error) => return Err(error.into()),
        }
    }
}

fn publish_chunks(
    artifact_root: &Path,
    staged_payload_path: &Path,
) -> Result<Vec<ArtifactChunkDescriptor>, AssetImportError> {
    let chunk_root = artifact_root.join(ARTIFACT_CHUNK_DIRECTORY);
    fs::create_dir_all(&chunk_root)?;
    let mut reader = BufReader::new(File::open(staged_payload_path)?);
    let mut buffer = vec![0; ARTIFACT_CHUNK_BYTES];
    let mut chunks = Vec::new();
    loop {
        let bytes_read = reader.read(&mut buffer)?;
        if bytes_read == 0 {
            break;
        }
        let bytes = &buffer[..bytes_read];
        let content_hash = blake3::hash(bytes).to_hex().to_string();
        let path = chunk_path(&chunk_root, &content_hash);
        let matches_existing_content =
            existing_chunk_matches(&path, &content_hash, bytes_read as u64)?;
        if !matches_existing_content {
            atomic_write(&path, bytes)?;
        }
        chunks.push(ArtifactChunkDescriptor::new(
            content_hash,
            bytes_read as u32,
        ));
    }
    if chunks.is_empty() {
        return Err(AssetImportError::Parse(
            "artifact compression produced no content-addressed chunks".to_string(),
        ));
    }
    Ok(chunks)
}

fn existing_chunk_matches(
    path: &Path,
    expected_content_hash: &str,
    expected_bytes: u64,
) -> Result<bool, AssetImportError> {
    let file = match File::open(path) {
        Ok(file) => file,
        Err(error) if error.kind() == io::ErrorKind::NotFound => return Ok(false),
        Err(error) => return Err(error.into()),
    };
    if file.metadata()?.len() != expected_bytes {
        return Ok(false);
    }

    let mut reader = BufReader::new(file);
    let mut buffer = [0; ARTIFACT_CHUNK_BYTES];
    let mut hasher = blake3::Hasher::new();
    loop {
        let bytes_read = reader.read(&mut buffer)?;
        if bytes_read == 0 {
            break;
        }
        hasher.update(&buffer[..bytes_read]);
    }
    Ok(hasher.finalize().to_hex().as_str() == expected_content_hash)
}

fn serialize_manifest(manifest: &ArtifactManifest) -> Result<Vec<u8>, AssetImportError> {
    let bytes = bincode::serialize(manifest).map_err(AssetImportError::ArtifactCacheSerialize)?;
    let mut payload = Vec::with_capacity(ARTIFACT_MANIFEST_MAGIC.len() + bytes.len());
    payload.extend_from_slice(ARTIFACT_MANIFEST_MAGIC);
    payload.extend_from_slice(&bytes);
    if payload.len() > ARTIFACT_MANIFEST_MAX_BYTES {
        return Err(AssetImportError::Parse(format!(
            "artifact manifest exceeds the {ARTIFACT_MANIFEST_MAX_BYTES}-byte limit"
        )));
    }
    Ok(payload)
}

fn read_manifest(path: &Path) -> Result<ArtifactManifest, AssetImportError> {
    let file = File::open(path)?;
    let max_bytes = ARTIFACT_MANIFEST_MAX_BYTES as u64;
    if file.metadata()?.len() > max_bytes {
        return Err(AssetImportError::Parse(format!(
            "artifact manifest exceeds the {ARTIFACT_MANIFEST_MAX_BYTES}-byte limit"
        )));
    }
    let mut payload = Vec::with_capacity(ARTIFACT_MANIFEST_MAX_BYTES.min(4096));
    file.take(max_bytes + 1).read_to_end(&mut payload)?;
    if payload.len() > ARTIFACT_MANIFEST_MAX_BYTES {
        return Err(AssetImportError::Parse(format!(
            "artifact manifest exceeds the {ARTIFACT_MANIFEST_MAX_BYTES}-byte limit"
        )));
    }
    let Some(bytes) = payload.strip_prefix(ARTIFACT_MANIFEST_MAGIC) else {
        return Err(AssetImportError::Parse(
            "unsupported artifact manifest format; expected versioned chunk manifest".to_string(),
        ));
    };
    bincode::DefaultOptions::new()
        .with_fixint_encoding()
        .with_limit(max_bytes)
        .reject_trailing_bytes()
        .deserialize(bytes)
        .map_err(AssetImportError::ArtifactCacheDeserialize)
}

fn validate_manifest(path: &str, manifest: &ArtifactManifest) -> Result<(), AssetImportError> {
    if !path.ends_with(ARTIFACT_CACHE_SUFFIX) {
        return Err(AssetImportError::Parse(format!(
            "unsupported artifact cache extension for {path}; expected {ARTIFACT_CACHE_SUFFIX}"
        )));
    }
    if manifest.schema_version != ARTIFACT_MANIFEST_SCHEMA_VERSION {
        return Err(AssetImportError::Parse(format!(
            "unsupported artifact manifest schema {}; expected {}",
            manifest.schema_version, ARTIFACT_MANIFEST_SCHEMA_VERSION
        )));
    }
    validate_artifact_raw_payload_bytes(manifest.raw_bytes)?;
    if !is_blake3_hex(&manifest.content_hash) || manifest.chunks.is_empty() {
        return Err(AssetImportError::Parse(
            "artifact manifest must declare a BLAKE3 content hash and at least one chunk"
                .to_string(),
        ));
    }
    if manifest.chunks.iter().any(|chunk| {
        chunk.compressed_bytes == 0
            || chunk.compressed_bytes as usize > ARTIFACT_CHUNK_BYTES
            || !is_blake3_hex(&chunk.content_hash)
    }) {
        return Err(AssetImportError::Parse(
            "artifact manifest chunks must have 1..=64 KiB byte counts and BLAKE3 identifiers"
                .to_string(),
        ));
    }
    let compressed_bytes = manifest.chunks.iter().try_fold(0_u64, |total, chunk| {
        total
            .checked_add(u64::from(chunk.compressed_bytes))
            .ok_or_else(|| AssetImportError::Parse("artifact manifest size overflow".to_string()))
    })?;
    if compressed_bytes != manifest.compressed_bytes {
        return Err(AssetImportError::Parse(format!(
            "artifact manifest compressed size mismatch: chunks total {compressed_bytes}, manifest records {}",
            manifest.compressed_bytes
        )));
    }
    validate_artifact_compressed_payload_bytes(manifest.raw_bytes, compressed_bytes)?;
    if let Some(expected_kind) = asset_kind_from_artifact_path(path) {
        if expected_kind != manifest.kind {
            return Err(AssetImportError::Parse(format!(
                "artifact manifest kind mismatch for {path}: path is {expected_kind:?}, manifest is {:?}",
                manifest.kind
            )));
        }
    }
    Ok(())
}

fn validate_artifact_raw_payload_bytes(raw_bytes: u64) -> Result<(), AssetImportError> {
    if raw_bytes == 0 || raw_bytes > ARTIFACT_RAW_PAYLOAD_MAX_BYTES {
        return Err(artifact_raw_payload_size_error(raw_bytes));
    }
    Ok(())
}

fn artifact_raw_payload_size_error(raw_bytes: u64) -> AssetImportError {
    AssetImportError::Parse(format!(
        "artifact raw payload size {raw_bytes} exceeds the supported 1..={ARTIFACT_RAW_PAYLOAD_MAX_BYTES}-byte range"
    ))
}

fn validate_artifact_compressed_payload_bytes(
    raw_bytes: u64,
    compressed_bytes: u64,
) -> Result<(), AssetImportError> {
    // This mirrors Zstd 1.5.7's ZSTD_COMPRESSBOUND: raw + raw / 256, with
    // the documented sub-128 KiB margin. The stream encoder never flushes a
    // partial frame, so this bounds the immutable chunk inventory that read
    // may open for a raw payload already admitted above.
    let small_input_margin = (raw_bytes < ZSTD_COMPRESS_BOUND_SMALL_INPUT_BYTES)
        .then(|| {
            (ZSTD_COMPRESS_BOUND_SMALL_INPUT_BYTES - raw_bytes)
                / ZSTD_COMPRESS_BOUND_SMALL_INPUT_MARGIN_DIVISOR
        })
        .unwrap_or_default();
    let compressed_bound = raw_bytes
        .checked_add(raw_bytes / 256)
        .and_then(|bound| bound.checked_add(small_input_margin))
        .ok_or_else(|| {
            AssetImportError::Parse("artifact compressed payload bound overflow".to_string())
        })?;
    if compressed_bytes > compressed_bound {
        return Err(AssetImportError::Parse(format!(
            "artifact compressed payload size {compressed_bytes} exceeds the {compressed_bound}-byte Zstd bound for {raw_bytes} raw bytes"
        )));
    }
    Ok(())
}

fn resolve_artifact_cache_path(
    paths: &ProjectPaths,
    artifact_uri: &AssetUri,
) -> Result<PathBuf, AssetImportError> {
    if artifact_uri.scheme() != ResourceScheme::Library {
        return Err(AssetImportError::UnsupportedFormat(format!(
            "artifact uri must use lib:// scheme: {artifact_uri}"
        )));
    }
    Ok(paths.asset_artifact_root().join(artifact_uri.path()))
}

fn is_blake3_hex(value: &str) -> bool {
    value.len() == BLAKE3_HEX_LENGTH && value.bytes().all(|byte| byte.is_ascii_hexdigit())
}

fn asset_kind_directory(kind: AssetKind) -> &'static str {
    match kind {
        AssetKind::Data => "data",
        AssetKind::Texture => "textures",
        AssetKind::Shader => "shaders",
        AssetKind::Material => "materials",
        AssetKind::MaterialGraph => "materials/graphs",
        AssetKind::Sound => "sound",
        AssetKind::Font => "fonts",
        AssetKind::PhysicsMaterial => "physics/materials",
        AssetKind::NavMesh => "navigation/navmeshes",
        AssetKind::NavigationSettings => "navigation/settings",
        AssetKind::Terrain => "terrain/heightfields",
        AssetKind::TerrainLayerStack => "terrain/layers",
        AssetKind::TileSet => "tilemap_2d/tilesets",
        AssetKind::TileMap => "tilemap_2d/maps",
        AssetKind::Prefab => "prefabs",
        AssetKind::Scene => "scenes",
        AssetKind::Model => "models",
        AssetKind::Mesh => "meshes",
        AssetKind::AnimationSkeleton => "animation/skeletons",
        AssetKind::AnimationClip => "animation/clips",
        AssetKind::AnimationSequence => "animation/sequences",
        AssetKind::AnimationGraph => "animation/graphs",
        AssetKind::AnimationStateMachine => "animation/state_machines",
        AssetKind::UiLayout => "ui/layouts",
        AssetKind::UiWidget => "ui/widgets",
        AssetKind::UiStyle => "ui/styles",
    }
}

fn asset_kind_from_artifact_path(path: &str) -> Option<AssetKind> {
    [
        ("textures/", AssetKind::Texture),
        ("shaders/", AssetKind::Shader),
        ("data/", AssetKind::Data),
        ("physics/materials/", AssetKind::PhysicsMaterial),
        ("materials/graphs/", AssetKind::MaterialGraph),
        ("materials/", AssetKind::Material),
        ("sound/", AssetKind::Sound),
        ("fonts/", AssetKind::Font),
        ("navigation/navmeshes/", AssetKind::NavMesh),
        ("navigation/settings/", AssetKind::NavigationSettings),
        ("terrain/heightfields/", AssetKind::Terrain),
        ("terrain/layers/", AssetKind::TerrainLayerStack),
        ("tilemap_2d/tilesets/", AssetKind::TileSet),
        ("tilemap_2d/maps/", AssetKind::TileMap),
        ("prefabs/", AssetKind::Prefab),
        ("scenes/", AssetKind::Scene),
        ("meshes/", AssetKind::Mesh),
        ("models/", AssetKind::Model),
        ("animation/skeletons/", AssetKind::AnimationSkeleton),
        ("animation/clips/", AssetKind::AnimationClip),
        ("animation/sequences/", AssetKind::AnimationSequence),
        ("animation/graphs/", AssetKind::AnimationGraph),
        (
            "animation/state_machines/",
            AssetKind::AnimationStateMachine,
        ),
        ("ui/layouts/", AssetKind::UiLayout),
        ("ui/widgets/", AssetKind::UiWidget),
        ("ui/styles/", AssetKind::UiStyle),
    ]
    .into_iter()
    .find_map(|(prefix, kind)| path.starts_with(prefix).then_some(kind))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn artifact_zstd_compress_bound_large_payload_accepts_exact_bound() {
        let raw_bytes = ZSTD_COMPRESS_BOUND_SMALL_INPUT_BYTES + 1;
        let compressed_bound = raw_bytes + raw_bytes / 256;

        assert!(validate_artifact_compressed_payload_bytes(raw_bytes, compressed_bound).is_ok());
    }

    #[test]
    fn artifact_zstd_compress_bound_large_payload_rejects_bytes_above_bound() {
        let raw_bytes = ZSTD_COMPRESS_BOUND_SMALL_INPUT_BYTES + 1;
        let compressed_bound = raw_bytes + raw_bytes / 256;

        let error = validate_artifact_compressed_payload_bytes(raw_bytes, compressed_bound + 1)
            .expect_err("bytes above the Zstd bound must be rejected");
        let AssetImportError::Parse(message) = error else {
            panic!("expected a parse error for an oversized compressed payload");
        };
        assert!(message.contains("exceeds the"));
    }
}
