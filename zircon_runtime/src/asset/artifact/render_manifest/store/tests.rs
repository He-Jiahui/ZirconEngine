use std::sync::Arc;
use std::sync::atomic::{AtomicU64, Ordering};

use crate::core::resource::{ResourceId, ResourceKind, UntypedResourceHandle};

use super::*;
use crate::asset::artifact::{
    RenderArtifactBlockCodec, RenderArtifactBlockDescriptor, RenderArtifactContentId,
    RenderArtifactLayout, RenderArtifactManifest, RenderArtifactResidencyClass,
    RenderArtifactTextureBlockFormat, RenderArtifactTextureLayout, RenderSubresourceId,
};

const TEST_BLOCK_ALIGNMENT: u32 = 256;
static NEXT_TEST_STORE_ID: AtomicU64 = AtomicU64::new(1);

struct TestStoreRoot(std::path::PathBuf);

impl TestStoreRoot {
    fn new() -> Self {
        let root = std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .join("..")
            .join(".codex_tmp")
            .join("render-artifact-store-tests")
            .join(format!(
                "{}-{}",
                std::process::id(),
                NEXT_TEST_STORE_ID.fetch_add(1, Ordering::Relaxed)
            ));
        std::fs::create_dir_all(&root)
            .unwrap_or_else(|error| panic!("failed to create workspace-local test root: {error}"));
        Self(root)
    }

    fn store(&self) -> RenderArtifactStore {
        RenderArtifactStore::new(self.0.clone())
    }
}

impl Drop for TestStoreRoot {
    fn drop(&mut self) {
        if let Err(error) = std::fs::remove_dir_all(&self.0) {
            assert_eq!(error.kind(), std::io::ErrorKind::NotFound);
        }
    }
}

fn store_limits() -> RenderArtifactStoreLimits {
    RenderArtifactStoreLimits::new(1024 * 1024, 8 * 1024 * 1024)
}

fn content_id_for(bytes: &[u8]) -> RenderArtifactContentId {
    RenderArtifactContentId::from_bytes(*blake3::hash(bytes).as_bytes())
}

fn stored_texture_block(bytes: &[u8]) -> RenderArtifactBlockDescriptor {
    RenderArtifactBlockDescriptor::new(
        RenderSubresourceId::TextureMipLayer { mip: 0, layer: 0 },
        content_id_for(bytes),
        RenderArtifactBlockCodec::Raw,
        bytes.len() as u64,
        bytes.len() as u64,
        TEST_BLOCK_ALIGNMENT,
        Arc::from("rgba8unorm"),
        RenderArtifactResidencyClass::Bootstrap,
        Vec::new(),
    )
}

fn resource(label: &str, kind: ResourceKind) -> UntypedResourceHandle {
    UntypedResourceHandle::new(ResourceId::from_stable_label(label), kind)
}

fn rgba8_texture_layout(width: u32, height: u32) -> RenderArtifactLayout {
    RenderArtifactLayout::texture(RenderArtifactTextureLayout::new(
        RenderArtifactTextureBlockFormat::new(Arc::from("rgba8unorm"), 1, 1, 4),
        width,
        height,
        1,
        1,
        0,
    ))
}

#[test]
fn render_semantic_block_store_publishes_reuses_and_random_reads_one_block() {
    let root = TestStoreRoot::new();
    let store = root.store();
    let bytes = vec![19_u8; 16 * 1024];
    let descriptor = stored_texture_block(&bytes);
    let limits = store_limits();

    assert!(matches!(
        store.publish_block(&descriptor, &bytes, limits),
        Ok(RenderArtifactPublishStatus::Published)
    ));
    assert!(matches!(
        store.publish_block(&descriptor, &bytes, limits),
        Ok(RenderArtifactPublishStatus::Reused)
    ));
    let read = store
        .read_block(&descriptor, limits)
        .unwrap_or_else(|error| panic!("semantic block read failed: {error}"));
    assert_eq!(read.as_ref(), bytes.as_slice());
}

#[test]
fn render_semantic_block_store_rejects_hash_mismatch_before_publication() {
    let root = TestStoreRoot::new();
    let store = root.store();
    let expected = vec![7_u8; 4 * 1024];
    let actual = vec![8_u8; 4 * 1024];
    let descriptor = stored_texture_block(&expected);

    let result = store.publish_block(&descriptor, &actual, store_limits());

    assert!(matches!(
        result,
        Err(RenderArtifactStoreError::BlockContentHashMismatch { .. })
    ));
    assert!(!store.block_exists(descriptor.content_id()));
}

#[test]
fn render_semantic_block_store_enforces_caller_owned_read_limits() {
    let root = TestStoreRoot::new();
    let store = root.store();
    let bytes = vec![11_u8; 4 * 1024];
    let descriptor = stored_texture_block(&bytes);
    store
        .publish_block(&descriptor, &bytes, store_limits())
        .unwrap_or_else(|error| panic!("semantic block publication failed: {error}"));
    let restricted = RenderArtifactStoreLimits::new(1024 * 1024, 1024);

    let result = store.read_block(&descriptor, restricted);

    assert!(matches!(
        result,
        Err(RenderArtifactStoreError::ByteLimitExceeded {
            byte_kind: "encoded block",
            actual: 4096,
            limit: 1024,
        })
    ));
}

#[test]
fn render_semantic_manifest_publication_requires_all_blocks_and_round_trips() {
    let root = TestStoreRoot::new();
    let store = root.store();
    let bytes = vec![23_u8; 8 * 1024];
    let descriptor = stored_texture_block(&bytes);
    let texture = resource("render-artifact/store/texture", ResourceKind::Texture);
    let manifest = RenderArtifactManifest::new(
        texture,
        3,
        Arc::from("windows-dx12-sm6"),
        rgba8_texture_layout(64, 32),
        Vec::new(),
        vec![descriptor.clone()],
    )
    .unwrap_or_else(|error| panic!("stored manifest construction failed: {error}"));
    let limits = store_limits();

    let missing = store.publish_manifest(&manifest, limits);
    assert!(matches!(
        missing,
        Err(RenderArtifactStoreError::MissingPublishedBlock { .. })
    ));
    store
        .publish_block(&descriptor, &bytes, limits)
        .unwrap_or_else(|error| panic!("manifest block publication failed: {error}"));
    assert!(matches!(
        store.publish_manifest(&manifest, limits),
        Ok(RenderArtifactPublishStatus::Published)
    ));
    assert!(matches!(
        store.publish_manifest(&manifest, limits),
        Ok(RenderArtifactPublishStatus::Reused)
    ));
    let read = store
        .read_manifest(texture, 3, "windows-dx12-sm6", limits)
        .unwrap_or_else(|error| panic!("semantic manifest read failed: {error}"));
    assert_eq!(read, manifest);

    let restricted_manifest = RenderArtifactStoreLimits::new(1, 8 * 1024 * 1024);
    assert!(matches!(
        store.read_manifest(texture, 3, "windows-dx12-sm6", restricted_manifest),
        Err(RenderArtifactStoreError::ByteLimitExceeded {
            byte_kind: "manifest",
            ..
        })
    ));
}

#[test]
fn render_semantic_manifest_target_is_hashed_instead_of_becoming_a_path() {
    let root = TestStoreRoot::new();
    let store = root.store();
    let bytes = vec![29_u8; 1024];
    let descriptor = stored_texture_block(&bytes);
    let texture = resource("render-artifact/store/path-safe", ResourceKind::Texture);
    let target = "../../outside\\device";
    let manifest = RenderArtifactManifest::new(
        texture,
        1,
        Arc::from(target),
        rgba8_texture_layout(16, 16),
        Vec::new(),
        vec![descriptor.clone()],
    )
    .unwrap_or_else(|error| panic!("path-safe manifest construction failed: {error}"));
    let limits = store_limits();
    store
        .publish_block(&descriptor, &bytes, limits)
        .unwrap_or_else(|error| panic!("path-safe block publication failed: {error}"));
    store
        .publish_manifest(&manifest, limits)
        .unwrap_or_else(|error| panic!("path-safe manifest publication failed: {error}"));

    let read = store
        .read_manifest(texture, 1, target, limits)
        .unwrap_or_else(|error| panic!("path-safe manifest read failed: {error}"));
    assert_eq!(read.target_platform(), target);
    assert!(!root.0.parent().unwrap_or(&root.0).join("outside").exists());
}
