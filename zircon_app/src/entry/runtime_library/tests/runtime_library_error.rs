use super::{RuntimeLibraryError, RuntimeLibraryErrorKind};

#[test]
fn protocol_violations_retain_a_typed_error_kind() {
    let error = RuntimeLibraryError::protocol_violation("foreign output exceeded its budget");

    assert_eq!(error.kind(), RuntimeLibraryErrorKind::ProtocolViolation);
    assert_eq!(error.to_string(), "foreign output exceeded its budget");
}

#[test]
fn unavailable_capabilities_retain_a_typed_error_kind() {
    let error = RuntimeLibraryError::capability_unavailable("no qualified surface backend");

    assert_eq!(error.kind(), RuntimeLibraryErrorKind::CapabilityUnavailable);
}
