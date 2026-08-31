use std::sync::Arc;
use std::sync::atomic::{AtomicU64, Ordering};
use std::time::{Duration, Instant};

use crate::core::resource::{ResourceId, ResourceKind, UntypedResourceHandle};
use crate::core::runtime::{EngineTaskGraph, EngineTaskGraphOptions};

use super::*;
use crate::asset::artifact::{
    RenderArtifactBlockCodec, RenderArtifactBlockDescriptor, RenderArtifactContentId,
    RenderArtifactIoPriority, RenderArtifactLayout, RenderArtifactManifest,
    RenderArtifactPublishStatus, RenderArtifactResidencyClass, RenderArtifactStore,
    RenderArtifactStoreLimits, RenderArtifactTextureBlockFormat, RenderArtifactTextureLayout,
    RenderSubresourceId,
};

const TARGET_PLATFORM: &str = "windows-dx12-sm6";
static NEXT_TEST_ROOT: AtomicU64 = AtomicU64::new(1);

struct TestStoreRoot(std::path::PathBuf);

impl TestStoreRoot {
    fn new() -> Self {
        let root = std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .join("..")
            .join(".codex_tmp")
            .join("render-manifest-loader-tests")
            .join(format!(
                "{}-{}",
                std::process::id(),
                NEXT_TEST_ROOT.fetch_add(1, Ordering::Relaxed)
            ));
        std::fs::create_dir_all(&root)
            .unwrap_or_else(|error| panic!("failed to create manifest loader root: {error}"));
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

fn resource() -> UntypedResourceHandle {
    UntypedResourceHandle::new(
        ResourceId::from_stable_label("render-manifest-loader/texture"),
        ResourceKind::Texture,
    )
}

fn store_limits() -> RenderArtifactStoreLimits {
    RenderArtifactStoreLimits::new(1024 * 1024, 1024 * 1024)
}

fn loader_limits() -> RenderArtifactManifestLoaderLimits {
    RenderArtifactManifestLoaderLimits::new(8, 32, 8, 16 * 1024 * 1024, store_limits())
}

fn fixture_manifest(store: &RenderArtifactStore) -> RenderArtifactManifest {
    fixture_manifest_for(store, resource(), 7, 11)
}

fn fixture_manifest_for(
    store: &RenderArtifactStore,
    resource: UntypedResourceHandle,
    asset_revision: u64,
    seed: u8,
) -> RenderArtifactManifest {
    let bytes = [seed; 4];
    let block = RenderArtifactBlockDescriptor::new(
        RenderSubresourceId::TextureMipLayer { mip: 0, layer: 0 },
        RenderArtifactContentId::from_bytes(*blake3::hash(&bytes).as_bytes()),
        RenderArtifactBlockCodec::Raw,
        bytes.len() as u64,
        bytes.len() as u64,
        256,
        Arc::from("rgba8unorm"),
        RenderArtifactResidencyClass::Bootstrap,
        Vec::new(),
    );
    let manifest = RenderArtifactManifest::new(
        resource,
        asset_revision,
        Arc::from(TARGET_PLATFORM),
        RenderArtifactLayout::texture(RenderArtifactTextureLayout::new(
            RenderArtifactTextureBlockFormat::new(Arc::from("rgba8unorm"), 1, 1, 4),
            1,
            1,
            1,
            1,
            0,
        )),
        Vec::new(),
        vec![block.clone()],
    )
    .unwrap_or_else(|error| panic!("manifest loader fixture is invalid: {error}"));
    assert_eq!(
        store
            .publish_block(&block, &bytes, store_limits())
            .unwrap_or_else(|error| panic!("manifest loader block publish failed: {error}")),
        RenderArtifactPublishStatus::Published
    );
    store
        .publish_manifest(&manifest, store_limits())
        .unwrap_or_else(|error| panic!("manifest loader fixture publish failed: {error}"));
    manifest
}

fn loader(
    store: RenderArtifactStore,
    limits: RenderArtifactManifestLoaderLimits,
) -> (EngineTaskGraph, RenderArtifactManifestLoader) {
    let runtime = EngineTaskGraph::try_new(EngineTaskGraphOptions::with_worker_threads(2))
        .unwrap_or_else(|error| panic!("manifest loader runtime failed: {error}"));
    let loader = RenderArtifactManifestLoader::new(store, limits, &runtime)
        .unwrap_or_else(|error| panic!("manifest loader init failed: {error}"));
    (runtime, loader)
}

fn manifest_request(
    resource: UntypedResourceHandle,
    asset_revision: u64,
) -> RenderArtifactManifestRequest {
    RenderArtifactManifestRequest::new(
        resource,
        asset_revision,
        Arc::from(TARGET_PLATFORM),
        RenderArtifactIoPriority::NORMAL,
    )
}

fn dispatch_all(loader: &RenderArtifactManifestLoader) {
    loader
        .dispatch_io(RenderArtifactManifestIoDispatchBudget::new(usize::MAX))
        .unwrap_or_else(|error| panic!("manifest I/O dispatch failed: {error}"));
}

fn wait_for_terminal(
    loader: &RenderArtifactManifestLoader,
    ticket: &RenderArtifactManifestTicket,
) -> RenderArtifactManifestPoll {
    let deadline = Instant::now() + Duration::from_secs(3);
    loop {
        loader.maintain_deadlines(Instant::now());
        let poll = ticket.poll();
        if !matches!(poll, RenderArtifactManifestPoll::Pending(_)) {
            return poll;
        }
        assert!(Instant::now() < deadline, "manifest load timed out");
        std::thread::yield_now();
    }
}

#[test]
fn render_manifest_loader_single_flights_identity_and_shares_one_manifest_owner() {
    let root = TestStoreRoot::new();
    let store = root.store();
    let expected = fixture_manifest(&store);
    let (_runtime, loader) = loader(store, loader_limits());

    let first = loader
        .request(manifest_request(resource(), 7))
        .unwrap_or_else(|error| panic!("first manifest request failed: {error}"));
    let second = loader
        .request(manifest_request(resource(), 7))
        .unwrap_or_else(|error| panic!("merged manifest request failed: {error}"));
    dispatch_all(&loader);
    let RenderArtifactManifestPoll::Ready(first_manifest) = wait_for_terminal(&loader, &first)
    else {
        panic!("first manifest request did not become ready");
    };
    let RenderArtifactManifestPoll::Ready(second_manifest) = wait_for_terminal(&loader, &second)
    else {
        panic!("second manifest request did not become ready");
    };

    assert!(Arc::ptr_eq(&first_manifest, &second_manifest));
    assert_eq!(first_manifest.as_ref(), &expected);
    assert_eq!(loader.diagnostics().submitted_io_tasks, 1);
    assert_eq!(loader.diagnostics().merged_requests, 1);
}

#[test]
fn render_manifest_loader_reports_missing_identity_as_typed_failure() {
    let root = TestStoreRoot::new();
    let (_runtime, loader) = loader(root.store(), loader_limits());
    let ticket = loader
        .request(manifest_request(resource(), 9))
        .unwrap_or_else(|error| panic!("missing manifest admission failed: {error}"));
    dispatch_all(&loader);

    let RenderArtifactManifestPoll::Failed(failure) = wait_for_terminal(&loader, &ticket) else {
        panic!("missing manifest request did not fail");
    };

    assert_eq!(failure.code(), RenderArtifactManifestFailureCode::NotFound);
    assert_eq!(loader.diagnostics().failed_entries, 1);
}

#[test]
fn render_manifest_loader_expires_deadline_frontier_without_entry_scan() {
    let root = TestStoreRoot::new();
    let (_runtime, loader) = loader(root.store(), loader_limits());
    let deadline = Instant::now();
    let ticket = loader
        .request(manifest_request(resource(), 7).with_deadline(deadline))
        .unwrap_or_else(|error| panic!("deadline manifest request failed: {error}"));

    let report = loader.maintain_deadlines(deadline);

    assert_eq!(report.expired_tickets, 1);
    assert!(matches!(
        ticket.poll(),
        RenderArtifactManifestPoll::Cancelled(RenderArtifactManifestCancelReason::Deadline)
    ));
    assert_eq!(loader.diagnostics().live_entries, 0);
}

#[test]
fn render_manifest_loader_rejects_retention_configuration_before_scope_creation() {
    let root = TestStoreRoot::new();
    let runtime = EngineTaskGraph::try_new(EngineTaskGraphOptions::with_worker_threads(2))
        .unwrap_or_else(|error| panic!("manifest loader runtime failed: {error}"));
    let constrained = RenderArtifactManifestLoaderLimits::new(1, 1, 1, 1024, store_limits());

    assert!(matches!(
        RenderArtifactManifestLoader::new(root.store(), constrained, &runtime),
        Err(RenderArtifactManifestLoaderInitError::RetainedBytesCapacityTooSmall { .. })
    ));
}

#[test]
fn render_manifest_loader_close_cancels_observers_and_closes_admission() {
    let root = TestStoreRoot::new();
    let (_runtime, loader) = loader(root.store(), loader_limits());
    let ticket = loader
        .request(manifest_request(resource(), 7))
        .unwrap_or_else(|error| panic!("close manifest request failed: {error}"));

    let report = loader.close();

    assert_eq!(report.cancelled_tickets, 1);
    assert!(matches!(
        ticket.poll(),
        RenderArtifactManifestPoll::Cancelled(RenderArtifactManifestCancelReason::OwnerClosed)
    ));
    assert!(matches!(
        loader.request(manifest_request(resource(), 7)),
        Err(RenderArtifactManifestAdmissionError::Closed)
    ));
}

#[path = "tests/frontier.rs"]
mod frontier;
