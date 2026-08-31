use std::sync::Arc;
use std::sync::atomic::{AtomicU64, Ordering};
use std::time::{Duration, Instant};

use crate::core::runtime::{EngineTaskGraph, EngineTaskGraphOptions};

use super::*;
use crate::asset::artifact::{
    RenderArtifactBlockCodec, RenderArtifactBlockDescriptor, RenderArtifactContentId,
    RenderArtifactPublishStatus, RenderArtifactResidencyClass, RenderArtifactStore,
    RenderArtifactStoreLimits, RenderSubresourceId,
};

const TEST_BLOCK_ALIGNMENT: u32 = 256;
static NEXT_TEST_ROOT: AtomicU64 = AtomicU64::new(1);

struct TestStoreRoot(std::path::PathBuf);

impl TestStoreRoot {
    fn new() -> Self {
        let root = std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .join("..")
            .join(".codex_tmp")
            .join("render-block-loader-tests")
            .join(format!(
                "{}-{}",
                std::process::id(),
                NEXT_TEST_ROOT.fetch_add(1, Ordering::Relaxed)
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

fn content_id_for(bytes: &[u8]) -> RenderArtifactContentId {
    RenderArtifactContentId::from_bytes(*blake3::hash(bytes).as_bytes())
}

fn descriptor(
    encoded: &[u8],
    decoded_bytes: usize,
    codec: RenderArtifactBlockCodec,
) -> RenderArtifactBlockDescriptor {
    RenderArtifactBlockDescriptor::new(
        RenderSubresourceId::TextureMipLayer { mip: 0, layer: 0 },
        content_id_for(encoded),
        codec,
        encoded.len() as u64,
        decoded_bytes as u64,
        TEST_BLOCK_ALIGNMENT,
        Arc::from("rgba8unorm"),
        RenderArtifactResidencyClass::Bootstrap,
        Vec::new(),
    )
}

fn limits() -> RenderArtifactBlockLoaderLimits {
    RenderArtifactBlockLoaderLimits::new(
        16,
        64,
        16,
        16 * 1024 * 1024,
        8 * 1024 * 1024,
        RenderArtifactStoreLimits::new(1024 * 1024, 8 * 1024 * 1024),
    )
}

fn loader(
    store: RenderArtifactStore,
    limits: RenderArtifactBlockLoaderLimits,
) -> (EngineTaskGraph, RenderArtifactBlockLoader) {
    let runtime = EngineTaskGraph::try_new(EngineTaskGraphOptions::with_worker_threads(3))
        .unwrap_or_else(|error| panic!("test execution runtime failed: {error}"));
    let loader = RenderArtifactBlockLoader::new(store, limits, &runtime)
        .unwrap_or_else(|error| panic!("render block loader failed: {error}"));
    (runtime, loader)
}

fn request(
    descriptor: RenderArtifactBlockDescriptor,
    deadline: Option<Instant>,
) -> RenderArtifactBlockRequest {
    let request = RenderArtifactBlockRequest::new(descriptor, RenderArtifactIoPriority::NORMAL);
    match deadline {
        Some(deadline) => request.with_deadline(deadline),
        None => request,
    }
}

fn dispatch_all(loader: &RenderArtifactBlockLoader) {
    let report = loader
        .dispatch_io(RenderArtifactBlockIoDispatchBudget::new(
            usize::MAX,
            u64::MAX,
        ))
        .unwrap_or_else(|error| panic!("render block dispatch failed: {error}"));
    assert_eq!(report.remaining_queued_entries, 0);
}

fn wait_for_terminal(
    loader: &RenderArtifactBlockLoader,
    ticket: &RenderArtifactBlockTicket,
) -> RenderArtifactBlockPoll {
    let deadline = Instant::now() + Duration::from_secs(3);
    loop {
        loader.maintain_deadlines(Instant::now());
        let poll = ticket.poll();
        if !matches!(poll, RenderArtifactBlockPoll::Pending(_)) {
            return poll;
        }
        assert!(Instant::now() < deadline, "render block load timed out");
        std::thread::yield_now();
    }
}

#[test]
fn render_block_loader_single_flights_raw_io_and_shares_one_decoded_owner() {
    let root = TestStoreRoot::new();
    let store = root.store();
    let bytes = vec![17_u8; 16 * 1024];
    let descriptor = descriptor(&bytes, bytes.len(), RenderArtifactBlockCodec::Raw);
    let reused_descriptor = RenderArtifactBlockDescriptor::new(
        RenderSubresourceId::TextureMipLayer { mip: 1, layer: 0 },
        descriptor.content_id(),
        descriptor.codec(),
        descriptor.encoded_bytes(),
        descriptor.decoded_bytes(),
        descriptor.alignment(),
        Arc::from(descriptor.platform_format()),
        RenderArtifactResidencyClass::Streamable,
        Vec::new(),
    );
    assert!(matches!(
        store.publish_block(&descriptor, &bytes, limits().store_limits()),
        Ok(RenderArtifactPublishStatus::Published)
    ));
    let (_runtime, loader) = loader(store, limits());

    let first = loader
        .request(request(descriptor.clone(), None))
        .unwrap_or_else(|error| panic!("first block request failed: {error}"));
    let second = loader
        .request(request(reused_descriptor, None))
        .unwrap_or_else(|error| panic!("merged block request failed: {error}"));
    dispatch_all(&loader);
    let RenderArtifactBlockPoll::Ready(first_block) = wait_for_terminal(&loader, &first) else {
        panic!("first block request did not become ready");
    };
    let RenderArtifactBlockPoll::Ready(second_block) = wait_for_terminal(&loader, &second) else {
        panic!("second block request did not become ready");
    };

    assert!(Arc::ptr_eq(first_block.bytes(), second_block.bytes()));
    assert_eq!(first_block.bytes().as_ref(), bytes.as_slice());
    assert_eq!(
        first_block.descriptor().subresource(),
        RenderSubresourceId::TextureMipLayer { mip: 0, layer: 0 }
    );
    assert_eq!(
        second_block.descriptor().subresource(),
        RenderSubresourceId::TextureMipLayer { mip: 1, layer: 0 }
    );
    let diagnostics = loader.diagnostics();
    assert_eq!(diagnostics.submitted_io_tasks, 1);
    assert_eq!(diagnostics.submitted_decode_tasks, 0);
    assert_eq!(diagnostics.merged_requests, 1);
}

#[test]
fn render_block_loader_decodes_zstd_on_the_async_compute_stage() {
    let root = TestStoreRoot::new();
    let store = root.store();
    let decoded = vec![31_u8; 64 * 1024];
    let encoded = zstd::stream::encode_all(decoded.as_slice(), 1)
        .unwrap_or_else(|error| panic!("test zstd encode failed: {error}"));
    let descriptor = descriptor(&encoded, decoded.len(), RenderArtifactBlockCodec::Zstd);
    store
        .publish_block(&descriptor, &encoded, limits().store_limits())
        .unwrap_or_else(|error| panic!("compressed block publication failed: {error}"));
    let (_runtime, loader) = loader(store, limits());

    let ticket = loader
        .request(request(descriptor, None))
        .unwrap_or_else(|error| panic!("compressed block request failed: {error}"));
    dispatch_all(&loader);
    let RenderArtifactBlockPoll::Ready(block) = wait_for_terminal(&loader, &ticket) else {
        panic!("compressed block request did not become ready");
    };

    assert_eq!(block.bytes().as_ref(), decoded.as_slice());
    let diagnostics = loader.diagnostics();
    assert_eq!(diagnostics.submitted_io_tasks, 1);
    assert_eq!(diagnostics.submitted_decode_tasks, 1);
    assert_eq!(diagnostics.decoded_bytes, decoded.len() as u64);
}

#[test]
fn render_block_loader_rejects_retained_byte_overcommit_before_scheduling() {
    let root = TestStoreRoot::new();
    let bytes = vec![41_u8; 4096];
    let descriptor = descriptor(&bytes, bytes.len(), RenderArtifactBlockCodec::Raw);
    let constrained = RenderArtifactBlockLoaderLimits::new(
        1,
        1,
        1,
        1024,
        8 * 1024 * 1024,
        RenderArtifactStoreLimits::new(1024 * 1024, 8 * 1024 * 1024),
    );
    let (_runtime, loader) = loader(root.store(), constrained);

    assert!(matches!(
        loader.request(request(descriptor, None)),
        Err(RenderArtifactBlockAdmissionError::RetainedBytesCapacityExceeded { .. })
    ));
    assert_eq!(loader.diagnostics().submitted_io_tasks, 0);
}

#[test]
fn render_block_loader_expires_ticket_frontier_without_scanning_entries() {
    let root = TestStoreRoot::new();
    let bytes = vec![43_u8; 4096];
    let descriptor = descriptor(&bytes, bytes.len(), RenderArtifactBlockCodec::Raw);
    let (_runtime, loader) = loader(root.store(), limits());
    let deadline = Instant::now();
    let ticket = loader
        .request(request(descriptor, Some(deadline)))
        .unwrap_or_else(|error| panic!("deadline request failed: {error}"));

    let report = loader.maintain_deadlines(deadline);

    assert_eq!(report.expired_tickets, 1);
    assert!(matches!(
        ticket.poll(),
        RenderArtifactBlockPoll::Cancelled(RenderArtifactBlockCancelReason::Deadline)
    ));
    assert_eq!(loader.diagnostics().live_entries, 0);
}

#[test]
fn render_block_loader_surfaces_store_verification_failure_without_payload_clone() {
    let root = TestStoreRoot::new();
    let store = root.store();
    let bytes = vec![47_u8; 4096];
    let published = descriptor(&bytes, bytes.len(), RenderArtifactBlockCodec::Raw);
    store
        .publish_block(&published, &bytes, limits().store_limits())
        .unwrap_or_else(|error| panic!("verification fixture publication failed: {error}"));
    let mismatched = RenderArtifactBlockDescriptor::new(
        published.subresource(),
        published.content_id(),
        published.codec(),
        published.encoded_bytes() + 1,
        published.decoded_bytes() + 1,
        published.alignment(),
        Arc::from(published.platform_format()),
        published.residency(),
        Vec::new(),
    );
    let (_runtime, loader) = loader(store, limits());

    let ticket = loader
        .request(request(mismatched, None))
        .unwrap_or_else(|error| panic!("mismatched block request admission failed: {error}"));
    dispatch_all(&loader);
    let RenderArtifactBlockPoll::Failed(failure) = wait_for_terminal(&loader, &ticket) else {
        panic!("mismatched block request did not fail");
    };

    assert_eq!(
        failure.code(),
        RenderArtifactBlockFailureCode::BlockSizeMismatch
    );
    assert_eq!(loader.diagnostics().failed_entries, 1);
}

#[test]
fn render_block_loader_close_cancels_observers_and_closes_admission() {
    let root = TestStoreRoot::new();
    let bytes = vec![53_u8; 4096];
    let descriptor = descriptor(&bytes, bytes.len(), RenderArtifactBlockCodec::Raw);
    let (_runtime, loader) = loader(root.store(), limits());
    let ticket = loader
        .request(request(descriptor.clone(), None))
        .unwrap_or_else(|error| panic!("close fixture admission failed: {error}"));

    let report = loader.close();

    assert_eq!(report.cancelled_tickets, 1);
    assert!(matches!(
        ticket.poll(),
        RenderArtifactBlockPoll::Cancelled(RenderArtifactBlockCancelReason::OwnerClosed)
    ));
    assert!(matches!(
        loader.request(request(descriptor, None)),
        Err(RenderArtifactBlockAdmissionError::Closed)
    ));
}

#[test]
fn render_block_loader_rejects_decoded_bomb_quote_before_io() {
    let root = TestStoreRoot::new();
    let bytes = vec![59_u8; 1024];
    let descriptor = descriptor(&bytes, 16 * 1024 * 1024, RenderArtifactBlockCodec::Zstd);
    let (_runtime, loader) = loader(root.store(), limits());

    assert!(matches!(
        loader.request(request(descriptor, None)),
        Err(RenderArtifactBlockAdmissionError::DecodedBlockLimitExceeded { .. })
    ));
    assert_eq!(loader.diagnostics().submitted_io_tasks, 0);
}

#[path = "tests/batch.rs"]
mod batch;
