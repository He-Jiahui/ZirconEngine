use std::fs::{self, File, OpenOptions};
use std::io::{self, Read, Write};
use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicU64, Ordering};
use std::time::{SystemTime, UNIX_EPOCH};

use serde::{Deserialize, Serialize};

use crate::asset::project::ProjectPaths;
use crate::core::framework::render::{ShaderVariantKey, ShaderVariantPrewarmSourceId};

const SHADER_VARIANT_CACHE_SCHEMA_VERSION: u32 = 3;
const SHADER_VARIANT_CACHE_DIR: &str = "shader_variants";
const SHADER_VARIANT_CACHE_PAYLOAD_SUFFIX: &str = "wgsl.zst";
const SHADER_VARIANT_CACHE_MANIFEST_SUFFIX: &str = "manifest";
const SHADER_VARIANT_CACHE_ZSTD_LEVEL: i32 = 3;
const SHADER_VARIANT_CACHE_MAX_MANIFEST_BYTES: u64 = 64 * 1024;
const SHADER_VARIANT_CACHE_MAX_COMPRESSED_BYTES: u64 = 8 * 1024 * 1024;
const SHADER_VARIANT_CACHE_MAX_DECODED_BYTES: u64 = 32 * 1024 * 1024;
static SHADER_VARIANT_CACHE_STAGING_COUNTER: AtomicU64 = AtomicU64::new(0);

#[derive(Clone, Debug, PartialEq, Eq)]
pub(crate) struct ShaderVariantCacheDiskKey {
    pub(crate) hash: String,
    pub(crate) canonical_string: String,
    pub(crate) source_id: ShaderVariantPrewarmSourceId,
    pub(crate) source_hash: String,
    pub(crate) template_revision: String,
    pub(crate) naga_version: String,
    pub(crate) wgpu_version: String,
}

impl ShaderVariantCacheDiskKey {
    pub(crate) fn from_variant_key(
        key: &ShaderVariantKey,
        source_hash: impl Into<String>,
        include_content_hashes: &[String],
        template_revision: impl Into<String>,
        naga_version: impl Into<String>,
        wgpu_version: impl Into<String>,
    ) -> Self {
        crate::profile_scope!("render", "shader_pipeline", "disk_cache_key");
        let canonical_string = key.canonical_string();
        let source_hash = source_hash.into();
        let template_revision = template_revision.into();
        let naga_version = naga_version.into();
        let wgpu_version = wgpu_version.into();
        let source_id = ShaderVariantPrewarmSourceId::from_cache_contract(
            &source_hash,
            include_content_hashes,
            &template_revision,
            &naga_version,
            &wgpu_version,
        );
        let hash = shader_variant_cache_hash(canonical_string.as_str(), &source_id);
        Self {
            hash,
            canonical_string,
            source_id,
            source_hash,
            template_revision,
            naga_version,
            wgpu_version,
        }
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub(crate) struct ShaderVariantCacheDiskEntry {
    pub(crate) wgsl_source: String,
    pub(crate) meta: ShaderVariantCacheDiskMeta,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub(crate) enum ShaderVariantCacheDiskLookup {
    Hit(ShaderVariantCacheDiskEntry),
    Miss,
    Error(ShaderVariantCacheDiskError),
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub(crate) enum ShaderVariantCacheDiskError {
    Io(String),
    Json(String),
    Utf8(String),
    Compression(String),
    SchemaMismatch { expected: u32, actual: u32 },
    KeyMismatch,
    SourceHashMismatch,
    PayloadHashMismatch,
    CorruptTarget(String),
    BudgetExceeded { what: &'static str, limit: u64 },
}

impl From<io::Error> for ShaderVariantCacheDiskError {
    fn from(error: io::Error) -> Self {
        Self::Io(error.to_string())
    }
}

#[derive(Clone, Debug)]
pub(crate) struct ShaderVariantCacheDisk {
    root: PathBuf,
    fallback_roots: Vec<PathBuf>,
    schema_version: u32,
}

impl ShaderVariantCacheDisk {
    pub(crate) fn new(cache_root: impl Into<PathBuf>) -> Self {
        Self {
            root: cache_root.into(),
            fallback_roots: Vec::new(),
            schema_version: SHADER_VARIANT_CACHE_SCHEMA_VERSION,
        }
    }

    pub(crate) fn with_fallback_roots(
        cache_root: impl Into<PathBuf>,
        fallback_roots: impl IntoIterator<Item = impl Into<PathBuf>>,
    ) -> Self {
        Self {
            root: cache_root.into(),
            fallback_roots: fallback_roots.into_iter().map(Into::into).collect(),
            schema_version: SHADER_VARIANT_CACHE_SCHEMA_VERSION,
        }
    }

    pub(crate) fn default_project_root(project_root: &Path) -> PathBuf {
        shader_cache_root_for_project(
            project_root,
            std::env::var_os("ZR_SHADER_CACHE_DIR")
                .filter(|path| !path.is_empty())
                .as_deref()
                .map(Path::new),
        )
    }

    pub(crate) fn default_staged_project_root(project_root: &Path) -> PathBuf {
        project_relative_shader_cache_root(
            project_root,
            Path::new("cache").join(SHADER_VARIANT_CACHE_DIR),
        )
    }

    pub(crate) fn lookup(&self, key: &ShaderVariantCacheDiskKey) -> ShaderVariantCacheDiskLookup {
        crate::profile_scope!("render", "shader_pipeline", "disk_cache_lookup");
        match self.read_entry_at(&self.root, key) {
            Ok(Some(entry)) => ShaderVariantCacheDiskLookup::Hit(entry),
            Ok(None) => {
                for fallback_root in &self.fallback_roots {
                    match self.read_entry_at(fallback_root, key) {
                        Ok(Some(entry)) => return ShaderVariantCacheDiskLookup::Hit(entry),
                        Ok(None) => {}
                        Err(error) => return ShaderVariantCacheDiskLookup::Error(error),
                    }
                }
                ShaderVariantCacheDiskLookup::Miss
            }
            Err(error) => {
                self.remove_entry_files(key);
                ShaderVariantCacheDiskLookup::Error(error)
            }
        }
    }

    pub(crate) fn write(
        &self,
        key: &ShaderVariantCacheDiskKey,
        wgsl_source: &str,
    ) -> Result<ShaderVariantCacheDiskEntry, ShaderVariantCacheDiskError> {
        crate::profile_scope!("render", "shader_pipeline", "disk_cache_write");
        let source_hash_matches = {
            crate::profile_scope!("render", "shader_pipeline", "disk_cache_write_source_hash");
            shader_source_hash(wgsl_source) == key.source_hash
        };
        if !source_hash_matches {
            return Err(ShaderVariantCacheDiskError::SourceHashMismatch);
        }
        if wgsl_source.len() as u64 > SHADER_VARIANT_CACHE_MAX_DECODED_BYTES {
            return Err(ShaderVariantCacheDiskError::BudgetExceeded {
                what: "decoded shader payload",
                limit: SHADER_VARIANT_CACHE_MAX_DECODED_BYTES,
            });
        }
        let path = self.entry_path(key);
        fs::create_dir_all(&path.directory)?;
        let compressed = {
            crate::profile_scope!("render", "shader_pipeline", "disk_cache_compress");
            zstd::stream::encode_all(wgsl_source.as_bytes(), SHADER_VARIANT_CACHE_ZSTD_LEVEL)
                .map_err(|error| ShaderVariantCacheDiskError::Compression(error.to_string()))?
        };
        if compressed.len() as u64 > SHADER_VARIANT_CACHE_MAX_COMPRESSED_BYTES {
            return Err(ShaderVariantCacheDiskError::BudgetExceeded {
                what: "compressed shader payload",
                limit: SHADER_VARIANT_CACHE_MAX_COMPRESSED_BYTES,
            });
        }
        crate::profile_counter!(
            "render",
            "shader_disk_cache_write_source_bytes",
            wgsl_source.len()
        );
        crate::profile_counter!(
            "render",
            "shader_disk_cache_write_compressed_bytes",
            compressed.len()
        );
        let meta = ShaderVariantCacheDiskMeta {
            schema_version: self.schema_version,
            hash: key.hash.clone(),
            canonical_string: key.canonical_string.clone(),
            source_id: key.source_id.clone(),
            source_hash: key.source_hash.clone(),
            template_revision: key.template_revision.clone(),
            naga_version: key.naga_version.clone(),
            wgpu_version: key.wgpu_version.clone(),
            compressed_bytes: compressed.len() as u64,
            decoded_bytes: wgsl_source.len() as u64,
            payload_hash: blake3::hash(&compressed).to_hex().to_string(),
            created_unix_seconds: unix_seconds_now(),
        };
        let manifest_bytes = {
            crate::profile_scope!("render", "shader_pipeline", "disk_cache_metadata_encode");
            serde_json::to_vec_pretty(&meta)
                .map_err(|error| ShaderVariantCacheDiskError::Json(error.to_string()))?
        };
        if manifest_bytes.len() as u64 > SHADER_VARIANT_CACHE_MAX_MANIFEST_BYTES {
            return Err(ShaderVariantCacheDiskError::BudgetExceeded {
                what: "shader cache manifest",
                limit: SHADER_VARIANT_CACHE_MAX_MANIFEST_BYTES,
            });
        }
        // The payload is immutable. The manifest is the sole publication point.
        crate::profile_scope!("render", "shader_pipeline", "disk_cache_payload_commit");
        atomic_create_or_verify(&path.payload, &compressed)?;
        crate::profile_scope!("render", "shader_pipeline", "disk_cache_metadata_commit");
        atomic_manifest_commit(&path.manifest, &manifest_bytes)?;
        self.read_entry_at(&self.root, key)?.ok_or_else(|| {
            ShaderVariantCacheDiskError::CorruptTarget(
                "manifest commit did not publish a readable cache entry".to_string(),
            )
        })
    }

    fn read_entry_at(
        &self,
        root: &Path,
        key: &ShaderVariantCacheDiskKey,
    ) -> Result<Option<ShaderVariantCacheDiskEntry>, ShaderVariantCacheDiskError> {
        let path = self.entry_path_at(root, key);
        // v2's split payload/meta layout is intentionally a truthful miss.
        if !path.manifest.exists() {
            return Ok(None);
        }
        if !path.payload.exists() {
            return Err(ShaderVariantCacheDiskError::CorruptTarget(
                "manifest exists without immutable payload".to_string(),
            ));
        }
        let manifest_bytes = {
            crate::profile_scope!("render", "shader_pipeline", "disk_cache_metadata_read");
            read_bounded(
                &path.manifest,
                SHADER_VARIANT_CACHE_MAX_MANIFEST_BYTES,
                "manifest",
            )?
        };
        crate::profile_counter!(
            "render",
            "shader_disk_cache_meta_bytes",
            manifest_bytes.len()
        );
        let meta = {
            crate::profile_scope!("render", "shader_pipeline", "disk_cache_metadata_decode");
            serde_json::from_slice::<ShaderVariantCacheDiskMeta>(&manifest_bytes)
                .map_err(|error| ShaderVariantCacheDiskError::Json(error.to_string()))?
        };
        if meta.schema_version != self.schema_version {
            return Err(ShaderVariantCacheDiskError::SchemaMismatch {
                expected: self.schema_version,
                actual: meta.schema_version,
            });
        }
        if meta.hash != key.hash
            || meta.canonical_string != key.canonical_string
            || meta.source_id != key.source_id
            || meta.source_hash != key.source_hash
            || meta.template_revision != key.template_revision
            || meta.naga_version != key.naga_version
            || meta.wgpu_version != key.wgpu_version
        {
            return Err(ShaderVariantCacheDiskError::KeyMismatch);
        }
        if meta.compressed_bytes > SHADER_VARIANT_CACHE_MAX_COMPRESSED_BYTES
            || meta.decoded_bytes > SHADER_VARIANT_CACHE_MAX_DECODED_BYTES
        {
            return Err(ShaderVariantCacheDiskError::BudgetExceeded {
                what: "shader cache payload",
                limit: SHADER_VARIANT_CACHE_MAX_COMPRESSED_BYTES,
            });
        }
        let compressed = {
            crate::profile_scope!("render", "shader_pipeline", "disk_cache_payload_read");
            read_bounded(
                &path.payload,
                SHADER_VARIANT_CACHE_MAX_COMPRESSED_BYTES,
                "compressed payload",
            )?
        };
        crate::profile_counter!(
            "render",
            "shader_disk_cache_compressed_bytes",
            compressed.len()
        );
        if compressed.len() as u64 != meta.compressed_bytes
            || blake3::hash(&compressed).to_hex().to_string() != meta.payload_hash
        {
            return Err(ShaderVariantCacheDiskError::PayloadHashMismatch);
        }
        let source = {
            crate::profile_scope!("render", "shader_pipeline", "disk_cache_decompress");
            let mut decoder = zstd::stream::read::Decoder::new(&compressed[..])
                .map_err(|error| ShaderVariantCacheDiskError::Compression(error.to_string()))?;
            let mut source = Vec::new();
            decoder
                .by_ref()
                .take(SHADER_VARIANT_CACHE_MAX_DECODED_BYTES + 1)
                .read_to_end(&mut source)?;
            if source.len() as u64 > SHADER_VARIANT_CACHE_MAX_DECODED_BYTES {
                return Err(ShaderVariantCacheDiskError::BudgetExceeded {
                    what: "decoded shader payload",
                    limit: SHADER_VARIANT_CACHE_MAX_DECODED_BYTES,
                });
            }
            source
        };
        crate::profile_counter!("render", "shader_disk_cache_decoded_bytes", source.len());
        if source.len() as u64 != meta.decoded_bytes {
            return Err(ShaderVariantCacheDiskError::CorruptTarget(
                "decoded length does not match manifest".to_string(),
            ));
        }
        let wgsl_source = String::from_utf8(source)
            .map_err(|error| ShaderVariantCacheDiskError::Utf8(error.to_string()))?;
        let source_hash_matches = {
            crate::profile_scope!("render", "shader_pipeline", "disk_cache_payload_rehash");
            shader_source_hash(&wgsl_source) == key.source_hash
        };
        if !source_hash_matches {
            return Err(ShaderVariantCacheDiskError::SourceHashMismatch);
        }
        Ok(Some(ShaderVariantCacheDiskEntry { wgsl_source, meta }))
    }

    fn entry_path(&self, key: &ShaderVariantCacheDiskKey) -> ShaderVariantCacheDiskPath {
        self.entry_path_at(&self.root, key)
    }

    fn entry_path_at(
        &self,
        root: &Path,
        key: &ShaderVariantCacheDiskKey,
    ) -> ShaderVariantCacheDiskPath {
        let shard = key.hash.get(0..2).unwrap_or("00");
        let directory = root.join(format!("v{}", self.schema_version)).join(shard);
        ShaderVariantCacheDiskPath {
            payload: directory.join(format!(
                "{}.{}",
                key.hash, SHADER_VARIANT_CACHE_PAYLOAD_SUFFIX
            )),
            manifest: directory.join(format!(
                "{}.{}",
                key.hash, SHADER_VARIANT_CACHE_MANIFEST_SUFFIX
            )),
            directory,
        }
    }

    fn remove_entry_files(&self, key: &ShaderVariantCacheDiskKey) {
        self.remove_entry_files_at(&self.root, key);
    }

    fn remove_entry_files_at(&self, root: &Path, key: &ShaderVariantCacheDiskKey) {
        let path = self.entry_path_at(root, key);
        let _ = fs::remove_file(path.payload);
        let _ = fs::remove_file(path.manifest);
    }
}

fn shader_cache_root_for_project(project_root: &Path, configured_root: Option<&Path>) -> PathBuf {
    match configured_root {
        Some(root) if root.is_absolute() => ProjectPaths::resolve_path(root)
            .map(|root| root.into_operation_path())
            .unwrap_or_else(|_| root.to_path_buf()),
        Some(root) => project_relative_shader_cache_root(project_root, root),
        None => project_relative_shader_cache_root(
            project_root,
            Path::new(".zircon")
                .join("cache")
                .join(SHADER_VARIANT_CACHE_DIR),
        ),
    }
}

fn project_relative_shader_cache_root(project_root: &Path, relative: impl AsRef<Path>) -> PathBuf {
    let relative = relative.as_ref();
    ProjectPaths::resolve_path(project_root)
        .and_then(|root| ProjectPaths::resolve_path_from(&root, relative))
        .map(|root| root.into_operation_path())
        .unwrap_or_else(|_| project_root.join(relative))
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub(crate) struct ShaderVariantCacheDiskMeta {
    pub(crate) schema_version: u32,
    pub(crate) hash: String,
    pub(crate) canonical_string: String,
    pub(crate) source_id: ShaderVariantPrewarmSourceId,
    pub(crate) source_hash: String,
    pub(crate) template_revision: String,
    pub(crate) naga_version: String,
    pub(crate) wgpu_version: String,
    pub(crate) compressed_bytes: u64,
    pub(crate) decoded_bytes: u64,
    pub(crate) payload_hash: String,
    pub(crate) created_unix_seconds: u64,
}

struct ShaderVariantCacheDiskPath {
    directory: PathBuf,
    payload: PathBuf,
    manifest: PathBuf,
}

fn shader_variant_cache_hash(
    canonical_string: &str,
    source_id: &ShaderVariantPrewarmSourceId,
) -> String {
    let mut hasher = blake3::Hasher::new();
    hash_field(&mut hasher, canonical_string.as_bytes());
    hash_field(&mut hasher, source_id.as_str().as_bytes());
    hasher.finalize().to_hex().to_string()
}

fn hash_field(hasher: &mut blake3::Hasher, field: &[u8]) {
    hasher.update(&(field.len() as u64).to_le_bytes());
    hasher.update(field);
}

fn shader_source_hash(wgsl_source: &str) -> String {
    blake3::hash(wgsl_source.as_bytes()).to_hex().to_string()
}

fn read_bounded(
    path: &Path,
    limit: u64,
    what: &'static str,
) -> Result<Vec<u8>, ShaderVariantCacheDiskError> {
    let file = File::open(path)?;
    let mut bytes = Vec::new();
    file.take(limit + 1).read_to_end(&mut bytes)?;
    if bytes.len() as u64 > limit {
        return Err(ShaderVariantCacheDiskError::BudgetExceeded { what, limit });
    }
    Ok(bytes)
}

fn unique_staging_path(path: &Path) -> PathBuf {
    let nonce = SHADER_VARIANT_CACHE_STAGING_COUNTER.fetch_add(1, Ordering::Relaxed);
    let file_name = path
        .file_name()
        .and_then(|name| name.to_str())
        .unwrap_or("entry");
    path.with_file_name(format!(".{file_name}.tmp-{}-{nonce}", std::process::id()))
}

fn atomic_create_or_verify(path: &Path, bytes: &[u8]) -> Result<(), ShaderVariantCacheDiskError> {
    atomic_create_or_verify_with_hook(path, bytes, None)
}

fn atomic_create_or_verify_with_hook(
    path: &Path,
    bytes: &[u8],
    after_initial_check: Option<&dyn Fn()>,
) -> Result<(), ShaderVariantCacheDiskError> {
    if path.exists() {
        let existing = read_bounded(
            path,
            SHADER_VARIANT_CACHE_MAX_COMPRESSED_BYTES,
            "immutable payload",
        )?;
        return if existing == bytes {
            Ok(())
        } else {
            Err(ShaderVariantCacheDiskError::CorruptTarget(
                "immutable payload conflict differs from requested bytes".to_string(),
            ))
        };
    }
    if let Some(after_initial_check) = after_initial_check {
        after_initial_check();
    }
    let temp_path = unique_staging_path(path);
    let result = (|| {
        let mut file = OpenOptions::new()
            .write(true)
            .create_new(true)
            .open(&temp_path)?;
        file.write_all(bytes)?;
        file.sync_all()?;
        match install_staged_file(&temp_path, path) {
            Ok(()) => {
                sync_parent(path);
                Ok(())
            }
            Err(_error) if path.exists() => {
                let existing = read_bounded(
                    path,
                    SHADER_VARIANT_CACHE_MAX_COMPRESSED_BYTES,
                    "immutable payload",
                )?;
                if existing == bytes {
                    Ok(())
                } else {
                    Err(ShaderVariantCacheDiskError::CorruptTarget(
                        "immutable payload rename conflict differs from requested bytes"
                            .to_string(),
                    ))
                }
            }
            Err(error) => Err(ShaderVariantCacheDiskError::Io(error.to_string())),
        }
    })();
    let _ = fs::remove_file(&temp_path);
    result
}

fn atomic_manifest_commit(path: &Path, bytes: &[u8]) -> Result<(), ShaderVariantCacheDiskError> {
    atomic_manifest_commit_with_hook(path, bytes, None)
}

fn atomic_manifest_commit_with_hook(
    path: &Path,
    bytes: &[u8],
    after_initial_check: Option<&dyn Fn()>,
) -> Result<(), ShaderVariantCacheDiskError> {
    if path.exists() {
        let existing = read_bounded(path, SHADER_VARIANT_CACHE_MAX_MANIFEST_BYTES, "manifest")?;
        return if existing == bytes {
            Ok(())
        } else {
            Err(ShaderVariantCacheDiskError::CorruptTarget(
                "manifest rename conflict differs from requested identity".to_string(),
            ))
        };
    }
    if let Some(after_initial_check) = after_initial_check {
        after_initial_check();
    }
    let temp_path = unique_staging_path(path);
    let result = (|| {
        let mut file = OpenOptions::new()
            .write(true)
            .create_new(true)
            .open(&temp_path)?;
        file.write_all(bytes)?;
        file.sync_all()?;
        match install_staged_file(&temp_path, path) {
            Ok(()) => {
                sync_parent(path);
                Ok(())
            }
            Err(_error) if path.exists() => {
                let existing =
                    read_bounded(path, SHADER_VARIANT_CACHE_MAX_MANIFEST_BYTES, "manifest")?;
                if existing == bytes {
                    Ok(())
                } else {
                    Err(ShaderVariantCacheDiskError::CorruptTarget(
                        "manifest rename conflict differs from requested identity".to_string(),
                    ))
                }
            }
            Err(error) => Err(ShaderVariantCacheDiskError::Io(error.to_string())),
        }
    })();
    let _ = fs::remove_file(&temp_path);
    result
}

fn install_staged_file(temp_path: &Path, path: &Path) -> io::Result<()> {
    // `rename` replaces an existing destination on Windows. A hard link is
    // the standard-library create-only primitive there (CreateHardLinkW), so
    // an existing target returns an error instead of being clobbered. Removing
    // the staging name after a successful link leaves the fully synced inode
    // at the immutable content-addressed destination. If the filesystem does
    // not support hard links, the typed I/O error is retained; no overwrite
    // fallback is safe for an immutable payload or the sole manifest commit.
    fs::hard_link(temp_path, path)
}

fn sync_parent(path: &Path) {
    if let Some(parent) = path.parent() {
        if let Ok(file) = File::open(parent) {
            let _ = file.sync_all();
        }
    }
}

fn unix_seconds_now() -> u64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map_or(0, |duration| duration.as_secs())
}

#[cfg(test)]
#[path = "tests/disk.rs"]
mod tests;
