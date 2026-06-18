use std::fs;
use std::io;
use std::path::{Path, PathBuf};
use std::time::{SystemTime, UNIX_EPOCH};

use serde::{Deserialize, Serialize};

use crate::core::framework::render::ShaderVariantKey;

const SHADER_VARIANT_CACHE_SCHEMA_VERSION: u32 = 1;
const SHADER_VARIANT_CACHE_DIR: &str = "shader_variants";
const SHADER_VARIANT_CACHE_WGSL_SUFFIX: &str = "wgsl.zst";
const SHADER_VARIANT_CACHE_META_SUFFIX: &str = "meta";
const SHADER_VARIANT_CACHE_ZSTD_LEVEL: i32 = 3;

#[derive(Clone, Debug, PartialEq, Eq)]
pub(crate) struct ShaderVariantCacheDiskKey {
    pub(crate) hash: String,
    pub(crate) canonical_string: String,
}

impl ShaderVariantCacheDiskKey {
    pub(crate) fn from_variant_key(
        key: &ShaderVariantKey,
        include_content_hashes: impl IntoIterator<Item = impl AsRef<str>>,
    ) -> Self {
        let canonical_string = key.canonical_string();
        let hash = shader_variant_cache_hash(
            canonical_string.as_str(),
            include_content_hashes
                .into_iter()
                .map(|hash| hash.as_ref().to_string()),
        );
        Self {
            hash,
            canonical_string,
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
        std::env::var_os("ZR_SHADER_CACHE_DIR")
            .map(PathBuf::from)
            .unwrap_or_else(|| {
                project_root
                    .join(".zircon-cache")
                    .join(SHADER_VARIANT_CACHE_DIR)
            })
    }

    pub(crate) fn default_staged_project_root(project_root: &Path) -> PathBuf {
        project_root.join("cache").join(SHADER_VARIANT_CACHE_DIR)
    }

    pub(crate) fn lookup(&self, key: &ShaderVariantCacheDiskKey) -> ShaderVariantCacheDiskLookup {
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
        template_revision: impl Into<String>,
        naga_version: impl Into<String>,
        wgpu_version: impl Into<String>,
    ) -> Result<ShaderVariantCacheDiskEntry, ShaderVariantCacheDiskError> {
        let path = self.entry_path(key);
        fs::create_dir_all(&path.directory)?;
        let meta = ShaderVariantCacheDiskMeta {
            schema_version: self.schema_version,
            hash: key.hash.clone(),
            canonical_string: key.canonical_string.clone(),
            template_revision: template_revision.into(),
            naga_version: naga_version.into(),
            wgpu_version: wgpu_version.into(),
            created_unix_seconds: unix_seconds_now(),
        };
        let compressed =
            zstd::stream::encode_all(wgsl_source.as_bytes(), SHADER_VARIANT_CACHE_ZSTD_LEVEL)
                .map_err(|error| ShaderVariantCacheDiskError::Compression(error.to_string()))?;
        let meta_bytes = serde_json::to_vec_pretty(&meta)
            .map_err(|error| ShaderVariantCacheDiskError::Json(error.to_string()))?;
        atomic_write(&path.wgsl, &compressed)?;
        atomic_write(&path.meta, &meta_bytes)?;
        Ok(ShaderVariantCacheDiskEntry {
            wgsl_source: wgsl_source.to_string(),
            meta,
        })
    }

    fn read_entry_at(
        &self,
        root: &Path,
        key: &ShaderVariantCacheDiskKey,
    ) -> Result<Option<ShaderVariantCacheDiskEntry>, ShaderVariantCacheDiskError> {
        let path = self.entry_path_at(root, key);
        if !path.wgsl.exists() || !path.meta.exists() {
            return Ok(None);
        }
        let meta_bytes = fs::read(&path.meta)?;
        let meta = serde_json::from_slice::<ShaderVariantCacheDiskMeta>(&meta_bytes)
            .map_err(|error| ShaderVariantCacheDiskError::Json(error.to_string()))?;
        if meta.schema_version != self.schema_version {
            return Err(ShaderVariantCacheDiskError::SchemaMismatch {
                expected: self.schema_version,
                actual: meta.schema_version,
            });
        }
        if meta.hash != key.hash || meta.canonical_string != key.canonical_string {
            return Err(ShaderVariantCacheDiskError::KeyMismatch);
        }
        let compressed = fs::read(&path.wgsl)?;
        let source = zstd::stream::decode_all(&compressed[..])
            .map_err(|error| ShaderVariantCacheDiskError::Compression(error.to_string()))?;
        let wgsl_source = String::from_utf8(source)
            .map_err(|error| ShaderVariantCacheDiskError::Utf8(error.to_string()))?;
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
            wgsl: directory.join(format!("{}.{}", key.hash, SHADER_VARIANT_CACHE_WGSL_SUFFIX)),
            meta: directory.join(format!("{}.{}", key.hash, SHADER_VARIANT_CACHE_META_SUFFIX)),
            directory,
        }
    }

    fn remove_entry_files(&self, key: &ShaderVariantCacheDiskKey) {
        self.remove_entry_files_at(&self.root, key);
    }

    fn remove_entry_files_at(&self, root: &Path, key: &ShaderVariantCacheDiskKey) {
        let path = self.entry_path_at(root, key);
        let _ = fs::remove_file(path.wgsl);
        let _ = fs::remove_file(path.meta);
    }
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub(crate) struct ShaderVariantCacheDiskMeta {
    pub(crate) schema_version: u32,
    pub(crate) hash: String,
    pub(crate) canonical_string: String,
    pub(crate) template_revision: String,
    pub(crate) naga_version: String,
    pub(crate) wgpu_version: String,
    pub(crate) created_unix_seconds: u64,
}

struct ShaderVariantCacheDiskPath {
    directory: PathBuf,
    wgsl: PathBuf,
    meta: PathBuf,
}

fn shader_variant_cache_hash(
    canonical_string: &str,
    include_content_hashes: impl IntoIterator<Item = String>,
) -> String {
    let mut hasher = blake3::Hasher::new();
    hasher.update(canonical_string.as_bytes());
    for include_hash in include_content_hashes {
        hasher.update(b"\0");
        hasher.update(include_hash.as_bytes());
    }
    hasher.finalize().to_hex().to_string()
}

fn atomic_write(path: &Path, bytes: &[u8]) -> Result<(), ShaderVariantCacheDiskError> {
    let temp_path = path.with_extension(format!(
        "{}.tmp",
        path.extension()
            .and_then(|extension| extension.to_str())
            .unwrap_or("tmp")
    ));
    fs::write(&temp_path, bytes)?;
    match fs::rename(&temp_path, path) {
        Ok(()) => Ok(()),
        Err(error) if path.exists() => {
            let _ = fs::remove_file(temp_path);
            let _ = error;
            Ok(())
        }
        Err(error) => {
            let _ = fs::remove_file(temp_path);
            Err(ShaderVariantCacheDiskError::Io(error.to_string()))
        }
    }
}

fn unix_seconds_now() -> u64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map_or(0, |duration| duration.as_secs())
}

#[cfg(test)]
mod tests {
    use std::fs;

    use crate::core::framework::render::{
        GeometrySourceId, ShaderFeatureBits, ShaderPassType, ShaderQualityTier, ShaderVariantKey,
        SHADING_MODEL_ID_STANDARD_PBR,
    };
    use crate::core::resource::ResourceId;

    use super::{ShaderVariantCacheDisk, ShaderVariantCacheDiskKey, ShaderVariantCacheDiskLookup};

    #[test]
    fn render_shader_variant_cache_hits_disk_after_restart() {
        let root = std::env::temp_dir().join(format!(
            "zircon_shader_variant_cache_test_{}",
            std::process::id()
        ));
        let _ = fs::remove_dir_all(&root);
        let key = ShaderVariantCacheDiskKey::from_variant_key(&variant_key(), ["include-a"]);
        let cache = ShaderVariantCacheDisk::new(&root);

        cache
            .write(
                &key,
                "fn main() {}",
                "template-r1",
                "naga-test",
                "wgpu-test",
            )
            .expect("write variant cache");

        let restarted = ShaderVariantCacheDisk::new(&root);
        let lookup = restarted.lookup(&key);

        match lookup {
            ShaderVariantCacheDiskLookup::Hit(entry) => {
                assert_eq!(entry.wgsl_source, "fn main() {}");
                assert_eq!(entry.meta.canonical_string, key.canonical_string);
            }
            other => panic!("expected disk hit, got {other:?}"),
        }
        let _ = fs::remove_dir_all(root);
    }

    #[test]
    fn render_shader_variant_cache_treats_corrupt_entry_as_miss_after_cleanup() {
        let root = std::env::temp_dir().join(format!(
            "zircon_shader_variant_cache_corrupt_test_{}",
            std::process::id()
        ));
        let _ = fs::remove_dir_all(&root);
        let key = ShaderVariantCacheDiskKey::from_variant_key(&variant_key(), ["include-a"]);
        let cache = ShaderVariantCacheDisk::new(&root);
        cache
            .write(
                &key,
                "fn main() {}",
                "template-r1",
                "naga-test",
                "wgpu-test",
            )
            .expect("write variant cache");
        let shard = key.hash.get(0..2).unwrap_or("00");
        fs::write(
            root.join("v1")
                .join(shard)
                .join(format!("{}.meta", key.hash)),
            b"{ invalid json",
        )
        .expect("corrupt meta");

        assert!(matches!(
            cache.lookup(&key),
            ShaderVariantCacheDiskLookup::Error(_)
        ));
        assert!(matches!(
            cache.lookup(&key),
            ShaderVariantCacheDiskLookup::Miss
        ));

        let _ = fs::remove_dir_all(root);
    }

    fn variant_key() -> ShaderVariantKey {
        ShaderVariantKey {
            material_shader: ResourceId::from_stable_label("res://materials/cache-test.wgsl"),
            material_revision: 3,
            geometry_source: GeometrySourceId::new(0),
            shading_model: SHADING_MODEL_ID_STANDARD_PBR,
            pass_type: ShaderPassType::Forward,
            features: ShaderFeatureBits::new(ShaderFeatureBits::ALPHA_TEST),
            quality: ShaderQualityTier::Medium,
            platform_token: "wgpu-test".to_string(),
        }
    }
}
