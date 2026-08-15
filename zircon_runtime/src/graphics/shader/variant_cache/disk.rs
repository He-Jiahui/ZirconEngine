use std::fs;
use std::io;
use std::path::{Path, PathBuf};
use std::time::{SystemTime, UNIX_EPOCH};

use serde::{Deserialize, Serialize};

use crate::asset::project::ProjectPaths;
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
        let hash = shader_variant_cache_hash(canonical_string.as_str(), include_content_hashes);
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
    include_content_hashes: impl IntoIterator<Item = impl AsRef<str>>,
) -> String {
    let mut hasher = blake3::Hasher::new();
    hasher.update(canonical_string.as_bytes());
    for include_hash in include_content_hashes {
        hasher.update(b"\0");
        hasher.update(include_hash.as_ref().as_bytes());
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
    use std::path::{Path, PathBuf};
    use std::time::{SystemTime, UNIX_EPOCH};

    use crate::asset::project::ProjectPaths;
    use crate::core::framework::render::{
        GeometrySourceId, ShaderFeatureBits, ShaderPassType, ShaderQualityTier, ShaderVariantKey,
        SHADING_MODEL_ID_STANDARD_PBR,
    };
    use crate::core::resource::ResourceId;

    use super::{
        shader_cache_root_for_project, ShaderVariantCacheDisk, ShaderVariantCacheDiskKey,
        ShaderVariantCacheDiskLookup,
    };

    #[cfg(any(unix, windows))]
    #[test]
    fn shader_cache_roots_keep_the_physical_project_identity_for_relative_layouts() {
        let parent = unique_shader_cache_project_root("physical-identity");
        let physical_project = parent.join("physical-project");
        fs::create_dir_all(&physical_project).unwrap();
        let project_alias = parent.join("project-alias");
        create_directory_link(&physical_project, &project_alias);

        let default_root = shader_cache_root_for_project(&project_alias, None);
        let configured_root =
            shader_cache_root_for_project(&project_alias, Some(Path::new("derived/shaders")));
        let expected_project = ProjectPaths::resolve_existing_path(&physical_project).unwrap();

        fs::remove_dir_all(&parent).unwrap();
        assert_eq!(
            default_root,
            expected_project.join(".zircon/cache/shader_variants")
        );
        assert_eq!(configured_root, expected_project.join("derived/shaders"));
    }

    fn unique_shader_cache_project_root(case_name: &str) -> PathBuf {
        let timestamp = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_nanos();
        let path = std::env::temp_dir().join(format!(
            "zircon-shader-cache-{case_name}-{}-{timestamp}",
            std::process::id()
        ));
        if path.exists() {
            fs::remove_dir_all(&path).unwrap();
        }
        path
    }

    #[cfg(unix)]
    fn create_directory_link(target: &Path, link: &Path) {
        std::os::unix::fs::symlink(target, link).expect("create shader-cache project alias");
    }

    #[cfg(windows)]
    fn create_directory_link(target: &Path, link: &Path) {
        let command = format!(r#"mklink /J "{}" "{}""#, link.display(), target.display());
        let output = std::process::Command::new("cmd")
            .args(["/D", "/S", "/C"])
            .arg(command)
            .output()
            .expect("start mklink for shader-cache project alias");
        assert!(
            output.status.success(),
            "create shader-cache project junction failed: {}",
            String::from_utf8_lossy(&output.stderr)
        );
    }

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
            material_layout_hash: 0,
            material_option_bits: 0,
            geometry_source: GeometrySourceId::new(0),
            shading_model: SHADING_MODEL_ID_STANDARD_PBR,
            pass_type: ShaderPassType::Forward,
            features: ShaderFeatureBits::new(ShaderFeatureBits::ALPHA_TEST),
            quality: ShaderQualityTier::Medium,
            platform_token: "wgpu-test".to_string(),
        }
    }
}
