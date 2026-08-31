use super::*;
use crate::core::framework::render::{
    IblBakeArtifactContents, IblBakeArtifactDescriptor, IblBakeArtifactPayload, IblBakeKey,
};

fn request() -> IblBakeArtifactRequest {
    IblBakeArtifactRequest::new(IblBakeKey::source_cubemap(7, [11, 13, 17, 19]), 16, 5)
        .with_required_contents(IblBakeArtifactContents::SH9)
}

fn blob(descriptor: IblBakeArtifactDescriptor, fill: u8) -> IblBakeArtifactBlob {
    let payload = IblBakeArtifactPayload::decode(
        descriptor,
        &vec![fill; descriptor.expected_payload_size_bytes()],
    )
    .expect("test payload should match its descriptor");
    IblBakeArtifactBlob::from_payload(payload)
}

#[test]
fn borrowed_blob_resolution_preserves_asset_priority_and_rejection_count() {
    let request = request();
    let stale_asset = blob(
        IblBakeArtifactDescriptor::current_for_request(&request).with_algorithm_version(u64::MAX),
        0x21,
    );
    let current_asset = blob(
        IblBakeArtifactDescriptor::current_for_request(&request),
        0x42,
    );
    let runtime_cache = blob(
        IblBakeArtifactDescriptor::current_for_runtime_cache_request(&request),
        0x84,
    );

    let resolved = IblBakeArtifactResolvedPayload::resolve_borrowed_blob_sources(
        &request,
        &[stale_asset, current_asset],
        Some(&runtime_cache),
    );

    assert_eq!(
        resolved.source(),
        IblBakeArtifactSource::AssetDerivedArtifact
    );
    assert_eq!(resolved.rejected_candidate_count(), 1);
    assert_eq!(resolved.payload().unwrap().bytes()[0], 0x42);
}

#[test]
fn borrowed_blob_resolution_uses_current_runtime_cache_after_stale_assets() {
    let request = request();
    let stale_asset = blob(
        IblBakeArtifactDescriptor::current_for_request(&request).with_algorithm_version(u64::MAX),
        0x21,
    );
    let runtime_cache = blob(
        IblBakeArtifactDescriptor::current_for_runtime_cache_request(&request),
        0x84,
    );

    let resolved = IblBakeArtifactResolvedPayload::resolve_borrowed_blob_sources(
        &request,
        &[stale_asset],
        Some(&runtime_cache),
    );

    assert_eq!(resolved.source(), IblBakeArtifactSource::RuntimeCache);
    assert_eq!(resolved.rejected_candidate_count(), 1);
    assert_eq!(resolved.payload().unwrap().bytes()[0], 0x84);
}

#[test]
fn borrowed_blob_resolution_requests_compute_when_every_blob_is_stale() {
    let request = request();
    let stale_asset = blob(
        IblBakeArtifactDescriptor::current_for_request(&request).with_algorithm_version(u64::MAX),
        0x21,
    );
    let stale_runtime_cache = blob(
        IblBakeArtifactDescriptor::current_for_runtime_cache_request(&request)
            .with_algorithm_version(u64::MAX),
        0x84,
    );

    let resolved = IblBakeArtifactResolvedPayload::resolve_borrowed_blob_sources(
        &request,
        &[stale_asset],
        Some(&stale_runtime_cache),
    );

    assert_eq!(resolved.source(), IblBakeArtifactSource::RuntimeCompute);
    assert_eq!(resolved.rejected_candidate_count(), 2);
    assert!(resolved.payload().is_none());
    assert_eq!(resolved.environment_compute_dispatch_count(), 1);
}
