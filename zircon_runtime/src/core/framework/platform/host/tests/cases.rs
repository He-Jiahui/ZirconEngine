use super::{
    PlatformHostEvidence, PlatformHostEvidenceError, PlatformHostObservedCapabilities,
    PLATFORM_HOST_BACKEND_VERSION_MAX_BYTES,
};

#[test]
fn platform_host_evidence_rejects_empty_and_unbounded_backend_versions() {
    let observed = PlatformHostObservedCapabilities::new(true, true, true);

    assert_eq!(
        PlatformHostEvidence::new(observed).with_backend_version(""),
        Err(PlatformHostEvidenceError::EmptyBackendVersion)
    );

    let too_long = "v".repeat(PLATFORM_HOST_BACKEND_VERSION_MAX_BYTES + 1);
    assert_eq!(
        PlatformHostEvidence::new(observed).with_backend_version(too_long),
        Err(PlatformHostEvidenceError::BackendVersionTooLong {
            actual: PLATFORM_HOST_BACKEND_VERSION_MAX_BYTES + 1,
            maximum: PLATFORM_HOST_BACKEND_VERSION_MAX_BYTES,
        })
    );
}
