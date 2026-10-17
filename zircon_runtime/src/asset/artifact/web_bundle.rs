//! Desktop publication of immutable Raw assets and portable browser metadata.
use std::collections::HashSet;
use std::fs::{self, File, OpenOptions};
use std::io::{self, BufWriter, Read, Write};
use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicU64, Ordering};

use super::{
    write_raw_asset, RawAssetCatalog, RawAssetCatalogEntry, RawAssetError, RawSceneCatalogEntry,
    RenderArtifactBlockCodec, RenderArtifactCookOutput, RenderArtifactStore,
    RenderArtifactStoreLimits, RAW_ASSET_MAX_BYTES, RAW_CATALOG_MAX_BYTES,
};
use crate::asset::{asset_kind_for_imported_asset, AssetId, ImportedAsset};
use crate::core::resource::{ResourceRecord, ResourceState};
use crate::scene::DynamicScene;

pub const WEBGPU_ASSET_TARGET: &str = "wasm32-webgpu";
static NEXT_STAGING_FILE: AtomicU64 = AtomicU64::new(1);

#[derive(Debug, thiserror::Error)]
pub enum WebBundleError {
    #[error("web bundle I/O failed: {0}")]
    Io(#[from] io::Error),
    #[error(transparent)]
    Asset(#[from] RawAssetError),
    #[error("web bundle metadata serialization failed: {0}")]
    Json(#[from] serde_json::Error),
    #[error("invalid web bundle: {0}")]
    Invalid(&'static str),
    #[error("web render artifact publication failed: {0}")]
    Render(String),
}

/// Payloads are written through the borrowed cache view. Publishing catalog.json
/// is the only visibility boundary; failed staging preserves the previous catalog.
pub struct WebBundleWriter {
    root: PathBuf,
    catalog: RawAssetCatalog,
    ids: HashSet<AssetId>,
    failed: bool,
}
impl WebBundleWriter {
    pub fn new(root: PathBuf) -> Result<Self, WebBundleError> {
        fs::create_dir_all(&root)?;
        Ok(Self {
            root,
            catalog: RawAssetCatalog {
                schema_version: 1,
                target_platform: WEBGPU_ASSET_TARGET.into(),
                assets: Vec::new(),
                scene: None,
                startup_assets: Vec::new(),
                ui_roots: Vec::new(),
            },
            ids: HashSet::new(),
            failed: false,
        })
    }
    pub fn add_asset(
        &mut self,
        record: &ResourceRecord,
        asset: &ImportedAsset,
    ) -> Result<(), WebBundleError> {
        let result = self.add_asset_inner(record, asset);
        self.failed |= result.is_err();
        result
    }
    fn add_asset_inner(
        &mut self,
        record: &ResourceRecord,
        asset: &ImportedAsset,
    ) -> Result<(), WebBundleError> {
        if self.failed || self.ids.len() >= 4096 || self.ids.contains(&record.id) {
            return Err(WebBundleError::Invalid(
                "writer failed, catalog is full, or resource ID is duplicated",
            ));
        }
        if record.kind != asset_kind_for_imported_asset(asset) {
            return Err(WebBundleError::Invalid(
                "resource kind does not match its payload",
            ));
        }
        let locator = record.primary_locator.to_string();
        if locator.len() > 4096
            || locator.starts_with(['/', '\\'])
            || locator.as_bytes().get(1) == Some(&b':')
        {
            return Err(WebBundleError::Invalid("resource locator must be portable"));
        }
        let staged = stage_payload(&self.root, RAW_ASSET_MAX_BYTES, |writer| {
            write_raw_asset(writer, asset, RAW_ASSET_MAX_BYTES).map_err(WebBundleError::from)
        })?;
        let (payload_bytes, content_hash) = staged.publish(&self.root, "assets", "zr-raw")?;
        // Desktop cache paths, diagnostics and revision counters are not runtime metadata.
        let portable_record =
            ResourceRecord::new(record.id, record.kind, record.primary_locator.clone())
                .with_dependency_ids(record.dependency_ids.clone())
                .with_state(ResourceState::Ready);
        self.catalog.assets.push(RawAssetCatalogEntry {
            record: portable_record,
            payload_bytes,
            content_hash,
        });
        self.ids.insert(record.id);
        Ok(())
    }
    pub fn set_startup_scene(
        &mut self,
        scene: &DynamicScene,
        startup_assets: Vec<AssetId>,
        ui_roots: Vec<AssetId>,
    ) -> Result<(), WebBundleError> {
        let result = (|| {
            if self.failed || startup_assets.len() > 4096 || ui_roots.len() > 8 {
                return Err(WebBundleError::Invalid(
                    "startup selection exceeds its limits",
                ));
            }
            let staged = stage_payload(&self.root, RAW_CATALOG_MAX_BYTES, |writer| {
                serde_json::to_writer(writer, scene).map_err(WebBundleError::from)
            })?;
            let (payload_bytes, content_hash) = staged.publish(&self.root, "scenes", "json")?;
            self.catalog.scene = Some(RawSceneCatalogEntry {
                payload_bytes,
                content_hash,
            });
            self.catalog.startup_assets = startup_assets;
            self.catalog.ui_roots = ui_roots;
            Ok(())
        })();
        self.failed |= result.is_err();
        result
    }
    /// Cook with revision 1 (the fresh browser registry revision) and Raw blocks.
    pub fn add_render_artifact(
        &mut self,
        output: &RenderArtifactCookOutput,
    ) -> Result<(), WebBundleError> {
        let result = (|| {
            if self.failed
                || output.manifest().target_platform() != WEBGPU_ASSET_TARGET
                || output.manifest().asset_revision() != 1
                || output
                    .blocks()
                    .iter()
                    .any(|block| block.descriptor().codec() != RenderArtifactBlockCodec::Raw)
            {
                return Err(WebBundleError::Invalid(
                    "browser artifacts require the WebGPU target, revision 1 and Raw blocks",
                ));
            }
            let store = RenderArtifactStore::new(self.root.clone());
            let limits = RenderArtifactStoreLimits::new(RAW_CATALOG_MAX_BYTES, RAW_ASSET_MAX_BYTES);
            for block in output.blocks() {
                store
                    .publish_block(block.descriptor(), block.bytes(), limits)
                    .map_err(|e| WebBundleError::Render(e.to_string()))?;
            }
            store
                .publish_manifest(output.manifest(), limits)
                .map_err(|e| WebBundleError::Render(e.to_string()))?;
            Ok(())
        })();
        self.failed |= result.is_err();
        result
    }
    pub fn publish(mut self) -> Result<RawAssetCatalog, WebBundleError> {
        if self.failed || self.catalog.scene.is_none() {
            return Err(WebBundleError::Invalid(
                "writer failed or has no startup scene",
            ));
        }
        if self
            .catalog
            .startup_assets
            .iter()
            .chain(&self.catalog.ui_roots)
            .any(|id| !self.ids.contains(id))
            || self.catalog.assets.iter().any(|entry| {
                entry
                    .record
                    .dependency_ids
                    .iter()
                    .any(|id| !self.ids.contains(id))
            })
        {
            return Err(WebBundleError::Invalid(
                "startup or dependency resource is missing",
            ));
        }
        self.catalog.assets.sort_by_key(|entry| entry.record.id);
        let bytes = serde_json::to_vec(&self.catalog)?;
        if bytes.len() as u64 > RAW_CATALOG_MAX_BYTES {
            return Err(WebBundleError::Invalid("catalog exceeds its byte limit"));
        }
        crate::core::resource::io::atomic_write(&self.root.join("catalog.json"), &bytes)?;
        Ok(self.catalog)
    }
}

struct StagedPayload {
    path: PathBuf,
    bytes: u64,
    hash: [u8; 32],
}
impl StagedPayload {
    fn publish(
        self,
        root: &Path,
        directory: &str,
        extension: &str,
    ) -> Result<(u64, [u8; 32]), WebBundleError> {
        let directory = root.join(directory);
        fs::create_dir_all(&directory)?;
        let destination = directory.join(format!(
            "{}.{}",
            blake3::Hash::from_bytes(self.hash).to_hex(),
            extension
        ));
        if destination.exists() {
            verify_existing(&destination, self.bytes, self.hash)?;
        } else if let Err(error) = fs::rename(&self.path, &destination) {
            if destination.exists() {
                verify_existing(&destination, self.bytes, self.hash)?;
            } else {
                return Err(error.into());
            }
        }
        Ok((self.bytes, self.hash))
    }
}
impl Drop for StagedPayload {
    fn drop(&mut self) {
        let _ = fs::remove_file(&self.path);
    }
}
struct PayloadWriter {
    writer: BufWriter<File>,
    hash: blake3::Hasher,
    bytes: u64,
    limit: u64,
}
impl Write for PayloadWriter {
    fn write(&mut self, bytes: &[u8]) -> io::Result<usize> {
        if self.bytes.saturating_add(bytes.len() as u64) > self.limit {
            return Err(io::Error::new(
                io::ErrorKind::InvalidData,
                "web payload exceeds its byte limit",
            ));
        }
        let count = self.writer.write(bytes)?;
        self.hash.update(&bytes[..count]);
        self.bytes += count as u64;
        Ok(count)
    }
    fn flush(&mut self) -> io::Result<()> {
        self.writer.flush()
    }
}
fn stage_payload(
    root: &Path,
    limit: u64,
    encode: impl FnOnce(&mut PayloadWriter) -> Result<(), WebBundleError>,
) -> Result<StagedPayload, WebBundleError> {
    let sequence = NEXT_STAGING_FILE
        .fetch_update(Ordering::Relaxed, Ordering::Relaxed, |n| n.checked_add(1))
        .map_err(|_| WebBundleError::Invalid("staging file identity exhausted"))?;
    let path = root.join(format!(".web-pending-{}-{sequence}", std::process::id()));
    let file = OpenOptions::new()
        .write(true)
        .create_new(true)
        .open(&path)?;
    let mut staged = StagedPayload {
        path,
        bytes: 0,
        hash: [0; 32],
    };
    let mut writer = PayloadWriter {
        writer: BufWriter::with_capacity(64 * 1024, file),
        hash: blake3::Hasher::new(),
        bytes: 0,
        limit,
    };
    encode(&mut writer)?;
    writer.flush()?;
    writer.writer.get_ref().sync_all()?;
    staged.bytes = writer.bytes;
    staged.hash = *writer.hash.finalize().as_bytes();
    drop(writer);
    Ok(staged)
}
fn verify_existing(
    path: &Path,
    expected_bytes: u64,
    expected_hash: [u8; 32],
) -> Result<(), WebBundleError> {
    let mut file = File::open(path)?;
    if file.metadata()?.len() != expected_bytes {
        return Err(WebBundleError::Invalid(
            "existing immutable payload has the wrong length",
        ));
    }
    let mut buffer = [0; 8192];
    let mut hash = blake3::Hasher::new();
    loop {
        let count = file.read(&mut buffer)?;
        if count == 0 {
            break;
        }
        hash.update(&buffer[..count]);
    }
    if hash.finalize().as_bytes() != &expected_hash {
        return Err(WebBundleError::Invalid(
            "existing immutable payload hash mismatch",
        ));
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    struct TestDirectory(PathBuf);
    impl TestDirectory {
        fn new() -> Self {
            let sequence = NEXT_STAGING_FILE.fetch_add(1, Ordering::Relaxed);
            let path = PathBuf::from(
                std::env::var_os("CARGO_TARGET_DIR").expect("registered target root"),
            )
            .join("zircon-test-output/runtime-memory-webgpu-20261016")
            .join(format!("web-bundle-{}-{sequence}", std::process::id()));
            fs::create_dir_all(&path).unwrap();
            Self(path)
        }
    }
    impl Drop for TestDirectory {
        fn drop(&mut self) {
            let parent = PathBuf::from(std::env::var_os("CARGO_TARGET_DIR").unwrap())
                .join("zircon-test-output/runtime-memory-webgpu-20261016")
                .canonicalize()
                .unwrap();
            let candidate = self.0.canonicalize().unwrap();
            assert!(candidate.starts_with(&parent) && candidate != parent);
            fs::remove_dir_all(&candidate).unwrap();
        }
    }
    #[test]
    fn startup_scene_round_trip_and_failed_publication_preserve_previous_catalog() {
        let dir = TestDirectory::new();
        let mut writer = WebBundleWriter::new(dir.0.clone()).unwrap();
        writer
            .set_startup_scene(&DynamicScene::empty(), Vec::new(), Vec::new())
            .unwrap();
        let catalog = writer.publish().unwrap();
        let previous = fs::read(dir.0.join("catalog.json")).unwrap();
        let entry = catalog.scene.as_ref().unwrap();
        let bytes = fs::read(dir.0.join(entry.relative_payload_path())).unwrap();
        assert_eq!(entry.content_hash, *blake3::hash(&bytes).as_bytes());
        let scene: DynamicScene = serde_json::from_slice(&bytes).unwrap();
        assert_eq!(scene, DynamicScene::empty());
        let mut failed = WebBundleWriter::new(dir.0.clone()).unwrap();
        assert!(failed
            .set_startup_scene(&scene, vec![], vec![AssetId::new(); 9])
            .is_err());
        assert!(failed.publish().is_err());
        assert_eq!(previous, fs::read(dir.0.join("catalog.json")).unwrap());
        assert!(fs::read_dir(&dir.0).unwrap().all(|entry| !entry
            .unwrap()
            .file_name()
            .to_string_lossy()
            .starts_with(".web-pending")));
    }
    #[test]
    fn streaming_limit_removes_staging_file_and_rejects_corrupt_reuse() {
        let dir = TestDirectory::new();
        assert!(stage_payload(&dir.0, 4, |writer| {
            writer.write_all(b"12345")?;
            Ok(())
        })
        .is_err());
        assert_eq!(fs::read_dir(&dir.0).unwrap().count(), 0);
        let staged = stage_payload(&dir.0, 32, |writer| {
            writer.write_all(b"payload")?;
            Ok(())
        })
        .unwrap();
        let (_, hash) = staged.publish(&dir.0, "assets", "zr-raw").unwrap();
        let path = dir.0.join("assets").join(format!(
            "{}.zr-raw",
            blake3::Hash::from_bytes(hash).to_hex()
        ));
        fs::write(path, b"corrupt").unwrap();
        let staged = stage_payload(&dir.0, 32, |writer| {
            writer.write_all(b"payload")?;
            Ok(())
        })
        .unwrap();
        assert!(staged.publish(&dir.0, "assets", "zr-raw").is_err());
    }
}

#[cfg(all(test, feature = "memory-validation"))]
#[path = "web_bundle/acceptance_fixture.rs"]
mod acceptance_fixture;
