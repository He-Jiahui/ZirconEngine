use super::*;
use crate::core::resource::{ResourceId, ResourceKind, UntypedResourceHandle};

use crate::asset::artifact::{
    RenderArtifactLayout, RenderArtifactLoadScope, RenderArtifactManifest,
    RenderArtifactTextureBlockFormat, RenderArtifactTextureLayout,
};

fn request(
    descriptor: RenderArtifactBlockDescriptor,
    priority: RenderArtifactIoPriority,
) -> RenderArtifactBlockRequest {
    RenderArtifactBlockRequest::new(descriptor, priority)
}

fn dispatch_one(loader: &RenderArtifactBlockLoader) -> RenderArtifactBlockIoDispatchReport {
    loader
        .dispatch_io(RenderArtifactBlockIoDispatchBudget::new(1, u64::MAX))
        .unwrap_or_else(|error| panic!("block dispatch failed: {error}"))
}

#[test]
fn render_block_loader_batch_admission_is_atomic_before_io_dispatch() {
    let root = TestStoreRoot::new();
    let first_bytes = vec![61_u8; 4096];
    let second_bytes = vec![67_u8; 4096];
    let constrained = RenderArtifactBlockLoaderLimits::new(
        1,
        4,
        4,
        32 * 1024,
        8 * 1024 * 1024,
        RenderArtifactStoreLimits::new(1024 * 1024, 8 * 1024 * 1024),
    );
    let (_runtime, loader) = loader(root.store(), constrained);

    let result = loader.request_batch(&[
        request(
            descriptor(
                &first_bytes,
                first_bytes.len(),
                RenderArtifactBlockCodec::Raw,
            ),
            RenderArtifactIoPriority::NORMAL,
        ),
        request(
            descriptor(
                &second_bytes,
                second_bytes.len(),
                RenderArtifactBlockCodec::Raw,
            ),
            RenderArtifactIoPriority::HIGH,
        ),
    ]);

    assert!(matches!(
        result,
        Err(RenderArtifactBlockAdmissionError::EntryCapacityExceeded { capacity: 1 })
    ));
    let diagnostics = loader.diagnostics();
    assert_eq!(diagnostics.live_entries, 0);
    assert_eq!(diagnostics.live_tickets, 0);
    assert_eq!(diagnostics.queued_io_entries, 0);
    assert_eq!(diagnostics.retained_bytes, 0);
    assert_eq!(diagnostics.submitted_io_tasks, 0);
}

#[test]
fn render_block_loader_batch_single_flights_duplicate_content_before_dispatch() {
    let root = TestStoreRoot::new();
    let store = root.store();
    let bytes = vec![71_u8; 4096];
    let first = descriptor(&bytes, bytes.len(), RenderArtifactBlockCodec::Raw);
    let second = RenderArtifactBlockDescriptor::new(
        RenderSubresourceId::TextureMipLayer { mip: 1, layer: 0 },
        first.content_id(),
        first.codec(),
        first.encoded_bytes(),
        first.decoded_bytes(),
        first.alignment(),
        Arc::from(first.platform_format()),
        RenderArtifactResidencyClass::Streamable,
        Vec::new(),
    );
    store
        .publish_block(&first, &bytes, limits().store_limits())
        .unwrap_or_else(|error| panic!("duplicate fixture publication failed: {error}"));
    let (_runtime, loader) = loader(store, limits());

    let batch = loader
        .request_batch(&[
            request(first, RenderArtifactIoPriority::NORMAL),
            request(second, RenderArtifactIoPriority::HIGH),
        ])
        .unwrap_or_else(|error| panic!("duplicate batch admission failed: {error}"));

    assert_eq!(batch.tickets().len(), 2);
    assert_eq!(loader.diagnostics().queued_io_entries, 1);
    assert_eq!(dispatch_one(&loader).submitted_tasks, 1);
    let RenderArtifactBlockPoll::Ready(first_block) =
        wait_for_terminal(&loader, &batch.tickets()[0])
    else {
        panic!("first duplicate block did not become ready");
    };
    let RenderArtifactBlockPoll::Ready(second_block) =
        wait_for_terminal(&loader, &batch.tickets()[1])
    else {
        panic!("second duplicate block did not become ready");
    };
    assert!(Arc::ptr_eq(first_block.bytes(), second_block.bytes()));
    assert_eq!(loader.diagnostics().submitted_io_tasks, 1);
    assert_eq!(loader.diagnostics().merged_requests, 1);
}

#[test]
fn render_block_loader_dispatches_the_highest_priority_frontier_first() {
    let root = TestStoreRoot::new();
    let store = root.store();
    let low_bytes = vec![73_u8; 4096];
    let high_bytes = vec![79_u8; 4096];
    let low = descriptor(&low_bytes, low_bytes.len(), RenderArtifactBlockCodec::Raw);
    let high = descriptor(&high_bytes, high_bytes.len(), RenderArtifactBlockCodec::Raw);
    for (block, bytes) in [(&low, low_bytes.as_slice()), (&high, high_bytes.as_slice())] {
        store
            .publish_block(block, bytes, limits().store_limits())
            .unwrap_or_else(|error| panic!("priority fixture publication failed: {error}"));
    }
    let (_runtime, loader) = loader(store, limits());

    let batch = loader
        .request_batch(&[
            request(low, RenderArtifactIoPriority::LOW),
            request(high, RenderArtifactIoPriority::CRITICAL),
        ])
        .unwrap_or_else(|error| panic!("priority batch admission failed: {error}"));

    let report = dispatch_one(&loader);

    assert_eq!(report.submitted_tasks, 1);
    assert_eq!(report.remaining_queued_entries, 1);
    let RenderArtifactBlockPoll::Ready(high_block) =
        wait_for_terminal(&loader, &batch.tickets()[1])
    else {
        panic!("critical block did not become ready first");
    };
    assert_eq!(high_block.bytes().as_ref(), high_bytes.as_slice());
    assert!(matches!(
        batch.tickets()[0].poll(),
        RenderArtifactBlockPoll::Pending(RenderArtifactBlockLoadStage::QueuedIo)
    ));
}

#[test]
fn render_load_plan_batch_maps_to_one_atomic_loader_admission() {
    let root = TestStoreRoot::new();
    let store = root.store();
    let bytes = vec![83_u8; 4096];
    let block = descriptor(&bytes, bytes.len(), RenderArtifactBlockCodec::Raw);
    let resource = UntypedResourceHandle::new(
        ResourceId::from_stable_label("render-loader/plan-batch"),
        ResourceKind::Texture,
    );
    let manifest = RenderArtifactManifest::new(
        resource,
        1,
        Arc::from("windows-dx12-sm6"),
        RenderArtifactLayout::texture(RenderArtifactTextureLayout::new(
            RenderArtifactTextureBlockFormat::new(Arc::from("rgba8unorm"), 1, 1, 4),
            32,
            32,
            1,
            1,
            0,
        )),
        Vec::new(),
        vec![block.clone()],
    )
    .unwrap_or_else(|error| panic!("plan-batch manifest failed: {error}"));
    let plan = manifest
        .load_plan(RenderArtifactLoadScope::Bootstrap)
        .unwrap_or_else(|error| panic!("plan-batch plan failed: {error}"));
    store
        .publish_block(&block, &bytes, limits().store_limits())
        .unwrap_or_else(|error| panic!("plan-batch publication failed: {error}"));
    let (_runtime, loader) = loader(store, limits());

    let tickets = loader
        .request_load_batch(&plan.batches()[0], RenderArtifactIoPriority::HIGH, None)
        .unwrap_or_else(|error| panic!("plan-batch admission failed: {error}"));

    assert_eq!(tickets.tickets().len(), 1);
    assert_eq!(loader.diagnostics().queued_io_entries, 1);
    assert_eq!(dispatch_one(&loader).submitted_tasks, 1);
    assert!(matches!(
        wait_for_terminal(&loader, &tickets.tickets()[0]),
        RenderArtifactBlockPoll::Ready(_)
    ));
}
