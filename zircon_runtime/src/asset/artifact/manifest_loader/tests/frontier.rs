use super::*;

fn request(
    resource: UntypedResourceHandle,
    revision: u64,
    priority: RenderArtifactIoPriority,
) -> RenderArtifactManifestRequest {
    RenderArtifactManifestRequest::new(resource, revision, Arc::from(TARGET_PLATFORM), priority)
}

fn distinct_resource(label: &str) -> UntypedResourceHandle {
    UntypedResourceHandle::new(ResourceId::from_stable_label(label), ResourceKind::Texture)
}

#[test]
fn render_manifest_batch_admission_is_atomic_before_io_dispatch() {
    let root = TestStoreRoot::new();
    let constrained =
        RenderArtifactManifestLoaderLimits::new(1, 4, 4, 4 * 1024 * 1024, store_limits());
    let (_runtime, loader) = loader(root.store(), constrained);

    let result = loader.request_batch(&[
        request(resource(), 1, RenderArtifactIoPriority::LOW),
        request(
            distinct_resource("render-manifest-loader/atomic-second"),
            1,
            RenderArtifactIoPriority::HIGH,
        ),
    ]);

    assert!(matches!(
        result,
        Err(RenderArtifactManifestAdmissionError::EntryCapacityExceeded { capacity: 1 })
    ));
    let diagnostics = loader.diagnostics();
    assert_eq!(diagnostics.live_entries, 0);
    assert_eq!(diagnostics.live_tickets, 0);
    assert_eq!(diagnostics.queued_io_entries, 0);
    assert_eq!(diagnostics.reserved_retained_bytes, 0);
    assert_eq!(diagnostics.submitted_io_tasks, 0);
}

#[test]
fn render_manifest_dispatches_the_highest_priority_frontier_first() {
    let root = TestStoreRoot::new();
    let store = root.store();
    let low_resource = distinct_resource("render-manifest-loader/priority-low");
    let high_resource = distinct_resource("render-manifest-loader/priority-high");
    fixture_manifest_for(&store, low_resource, 1, 17);
    fixture_manifest_for(&store, high_resource, 1, 19);
    let (_runtime, loader) = loader(store, loader_limits());
    let batch = loader
        .request_batch(&[
            request(low_resource, 1, RenderArtifactIoPriority::LOW),
            request(high_resource, 1, RenderArtifactIoPriority::CRITICAL),
        ])
        .unwrap_or_else(|error| panic!("manifest priority batch failed: {error}"));

    let report = loader
        .dispatch_io(RenderArtifactManifestIoDispatchBudget::new(1))
        .unwrap_or_else(|error| panic!("manifest priority dispatch failed: {error}"));

    assert_eq!(report.submitted_tasks, 1);
    assert_eq!(report.remaining_queued_entries, 1);
    assert!(matches!(
        wait_for_terminal(&loader, &batch.tickets()[1]),
        RenderArtifactManifestPoll::Ready(_)
    ));
    assert!(matches!(
        batch.tickets()[0].poll(),
        RenderArtifactManifestPoll::Pending(RenderArtifactManifestLoadStage::QueuedIo)
    ));
}
