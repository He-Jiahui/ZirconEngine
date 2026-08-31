use std::sync::Arc;
use std::sync::atomic::{AtomicU64, Ordering};

use crate::asset::{AssetUri, TextureAsset, TextureUploadSupport};
use crate::core::resource::{ResourceId, ResourceKind, UntypedResourceHandle};

use super::*;
use crate::asset::artifact::{
    RenderArtifactStoreLimits, RenderArtifactTextureCookSettings, cook_texture_render_artifact,
};

static NEXT_TEST_STORE_ID: AtomicU64 = AtomicU64::new(1);

struct TestStoreRoot(std::path::PathBuf);

impl TestStoreRoot {
    fn new() -> Self {
        let root = std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .join("..")
            .join(".codex_tmp")
            .join("render-cook-publication-tests")
            .join(format!(
                "{}-{}",
                std::process::id(),
                NEXT_TEST_STORE_ID.fetch_add(1, Ordering::Relaxed)
            ));
        std::fs::create_dir_all(&root)
            .unwrap_or_else(|error| panic!("failed to create cook publication root: {error}"));
        Self(root)
    }
}

impl Drop for TestStoreRoot {
    fn drop(&mut self) {
        if let Err(error) = std::fs::remove_dir_all(&self.0) {
            assert_eq!(error.kind(), std::io::ErrorKind::NotFound);
        }
    }
}

fn texture_resource() -> UntypedResourceHandle {
    UntypedResourceHandle::new(
        ResourceId::from_stable_label("render-cook/publication"),
        ResourceKind::Texture,
    )
}

#[test]
fn render_cook_publication_writes_verified_blocks_then_reuses_the_immutable_bundle() {
    let root = TestStoreRoot::new();
    let store = RenderArtifactStore::new(root.0.clone());
    let limits = RenderArtifactStoreLimits::new(1024 * 1024, 1024 * 1024);
    let uri = AssetUri::parse("res://textures/published")
        .unwrap_or_else(|error| panic!("invalid publication test URI: {error}"));
    let texture = TextureAsset::new_rgba8(uri, 2, 2, vec![37_u8; 16]);
    let output = cook_texture_render_artifact(
        texture_resource(),
        4,
        texture,
        RenderArtifactTextureCookSettings::new(
            Arc::from("windows-dx12-sm6"),
            0,
            256,
            TextureUploadSupport::uncompressed_only(),
        ),
    )
    .unwrap_or_else(|error| panic!("publication fixture cook failed: {error}"));

    let first = publish_render_artifact_cook_output(&store, &output, limits)
        .unwrap_or_else(|error| panic!("first cook publication failed: {error}"));
    let second = publish_render_artifact_cook_output(&store, &output, limits)
        .unwrap_or_else(|error| panic!("reused cook publication failed: {error}"));

    assert_eq!(first.published_blocks(), 1);
    assert_eq!(first.reused_blocks(), 0);
    assert_eq!(first.published_encoded_bytes(), 16);
    assert_eq!(first.manifest(), RenderArtifactPublishStatus::Published);
    assert_eq!(second.published_blocks(), 0);
    assert_eq!(second.reused_blocks(), 1);
    assert_eq!(second.reused_encoded_bytes(), 16);
    assert_eq!(second.manifest(), RenderArtifactPublishStatus::Reused);
    let loaded = store
        .read_manifest(texture_resource(), 4, "windows-dx12-sm6", limits)
        .unwrap_or_else(|error| panic!("published cook manifest read failed: {error}"));
    assert_eq!(loaded, *output.manifest());
}
