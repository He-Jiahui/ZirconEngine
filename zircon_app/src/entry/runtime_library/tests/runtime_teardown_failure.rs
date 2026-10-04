use super::{RuntimeLibraryError, RuntimeSessionTeardownFailureState};

#[test]
fn runtime_session_teardown_failure_state_records_secondary_errors_in_the_product_ledger() {
    let state = RuntimeSessionTeardownFailureState::default();
    state.record(RuntimeLibraryError::new("surface unbind failed"));
    state.record(RuntimeLibraryError::new("session destroy failed"));

    let report = state.failure_ledger().snapshot();
    assert_eq!(report.records().len(), 2);
    assert_eq!(report.primary().unwrap().message(), "surface unbind failed");
    assert_eq!(report.secondary()[0].message(), "session destroy failed");
}
