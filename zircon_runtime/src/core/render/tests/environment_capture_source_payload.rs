use super::*;
use crate::core::framework::render::{
    RenderEnvironmentCaptureRequest, SourceCubemapPrefilterQuality,
};

fn request() -> RenderEnvironmentCaptureRequest {
    RenderEnvironmentCaptureRequest::with_revisions(
        "atrium",
        [0.0; 3],
        0.1,
        200.0,
        16,
        SourceCubemapPrefilterQuality::Normal,
        1,
        1,
        1,
    )
    .unwrap()
    .with_persistence_output_uri("res://probes/atrium.zcube")
    .unwrap()
}

#[test]
fn source_payload_is_exact_and_consuming() {
    let request = request();
    let mip_count = source_cubemap_mip_count(request.face_size());
    let expected = checked_source_payload_byte_len(request.face_size(), mip_count).unwrap();
    let payload = RenderEnvironmentCaptureSourcePayload::new(
        RenderEnvironmentCaptureHandle::new(7).unwrap(),
        RenderEnvironmentCaptureOutputIdentity::from_request(&request),
        request.face_size(),
        mip_count,
        vec![3; expected],
    )
    .unwrap();

    assert_eq!(payload.handle().get(), 7);
    assert_eq!(payload.output().capture_id(), "atrium");
    assert_eq!(payload.source_rgba16f_bytes().len(), expected);
    assert_eq!(payload.into_source_rgba16f_bytes(), vec![3; expected]);
}

#[test]
fn source_payload_rejects_missing_bytes() {
    let request = request();
    let mip_count = source_cubemap_mip_count(request.face_size());
    let expected = checked_source_payload_byte_len(request.face_size(), mip_count).unwrap();
    let error = RenderEnvironmentCaptureSourcePayload::new(
        RenderEnvironmentCaptureHandle::new(7).unwrap(),
        RenderEnvironmentCaptureOutputIdentity::from_request(&request),
        request.face_size(),
        mip_count,
        vec![0; expected - 1],
    )
    .unwrap_err();

    assert_eq!(
        error,
        RenderEnvironmentCaptureSourcePayloadError::PayloadLengthMismatch {
            expected,
            actual: expected - 1,
        }
    );
}
