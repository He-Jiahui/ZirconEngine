use super::*;

#[test]
fn runtime_startup_failure_before_handle_has_no_cleanup_owner() {
    let error = RuntimeSessionCreateFailure::from(RuntimeLibraryError::protocol_violation(
        "invalid output",
    ));
    assert!(!error.cleanup_pending());
    assert!(error.cleanup_recovery_context().is_none());
    assert!(error.retry_cleanup().is_ok());
    assert!(error.cleanup_receipt().is_none());
    assert!(error
        .source()
        .unwrap()
        .downcast_ref::<RuntimeLibraryError>()
        .is_some());
}
