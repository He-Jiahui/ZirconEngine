use crate::core::framework::render::{IblBakeArtifactContents, IblBakeArtifactRequest, IblBakeKey};

use super::{
    IblSourceCubemapBundleManifest, IblSourceCubemapBundleManifestError, IblSourceImageIdentity,
    IBL_SOURCE_CUBEMAP_BUNDLE_MANIFEST_SIZE,
};

#[test]
fn manifest_round_trip_preserves_request_source_identity_and_payload_stamps() {
    let request = request(17, [3, 5, 7, 11]);
    let source_image = IblSourceImageIdentity::new(2048, 1024, 11);
    let source = b"source payload";
    let derived = b"derived payload";
    let manifest = IblSourceCubemapBundleManifest::new(&request, source_image, source, derived);

    let bytes = manifest.encode();
    let decoded = IblSourceCubemapBundleManifest::decode(&bytes).expect("manifest must decode");

    assert_eq!(bytes.len(), IBL_SOURCE_CUBEMAP_BUNDLE_MANIFEST_SIZE);
    assert_eq!(decoded, manifest);
    assert!(decoded.matches(&request, source_image));
    assert!(decoded.source().matches_bytes(source));
    assert!(decoded.derived().matches_bytes(derived));
}

#[test]
fn manifest_rejects_same_length_payload_corruption_at_consumption_boundary() {
    let request = request(17, [3, 5, 7, 11]);
    let manifest = IblSourceCubemapBundleManifest::new(
        &request,
        IblSourceImageIdentity::new(2048, 1024, 11),
        b"source payload",
        b"derived payload",
    );

    assert!(!manifest.source().matches_bytes(b"source payloae"));
    assert!(!manifest.derived().matches_bytes(b"derived payloae"));
}

#[test]
fn manifest_rejects_corrupted_wire_before_fields_are_consumed() {
    let request = request(17, [3, 5, 7, 11]);
    let mut bytes = IblSourceCubemapBundleManifest::new(
        &request,
        IblSourceImageIdentity::new(2048, 1024, 11),
        b"source payload",
        b"derived payload",
    )
    .encode();
    bytes[40] ^= 0x80;

    assert_eq!(
        IblSourceCubemapBundleManifest::decode(&bytes),
        Err(IblSourceCubemapBundleManifestError::ChecksumMismatch)
    );
}

#[test]
fn manifest_keeps_stale_request_and_source_metadata_distinct() {
    let request = request(17, [3, 5, 7, 11]);
    let source_image = IblSourceImageIdentity::new(2048, 1024, 11);
    let manifest = IblSourceCubemapBundleManifest::new(
        &request,
        source_image,
        b"source payload",
        b"derived payload",
    );

    assert!(!manifest.matches(&request(18, [3, 5, 7, 11]), source_image));
    assert!(!manifest.matches(&request, IblSourceImageIdentity::new(2048, 1024, 12)));
}

fn request(revision: u64, source_hash: [u32; 4]) -> IblBakeArtifactRequest {
    IblBakeArtifactRequest::new(IblBakeKey::source_cubemap(revision, source_hash), 256, 9)
        .with_pmrem_layout(128, 8)
        .with_required_contents(IblBakeArtifactContents::PMREM_SH9_IEM)
}
