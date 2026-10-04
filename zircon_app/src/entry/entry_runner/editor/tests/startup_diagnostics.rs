use crate::entry::product_shutdown::{
    ProductFailureLedger, ProductFailureSeverity, ProductHostPhase,
};

use super::{finish_editor_host, record_editor_host_failure};

#[test]
fn editor_finish_preserves_host_and_shutdown_failures_in_order() {
    let failures = ProductFailureLedger::default();
    let host_result =
        Err::<(), Box<dyn std::error::Error>>(std::io::Error::other("editor host failed").into());
    record_editor_host_failure(&failures, &host_result);
    failures.record(
        ProductHostPhase::DestroyingRuntime,
        ProductFailureSeverity::Terminal,
        "runtime_session",
        "session destroy failed",
    );

    let error = finish_editor_host("project=test", host_result, failures.snapshot())
        .expect_err("the combined editor failure report must fail");
    let diagnostic = error.to_string();
    assert!(diagnostic.contains("recorded=2 suppressed=0"));
    assert!(diagnostic.contains("owner=editor_host message=editor host failed"));
    assert!(diagnostic.contains("owner=runtime_session message=session destroy failed"));
}

#[test]
fn editor_finish_preserves_success_when_the_failure_report_is_empty() {
    let failures = ProductFailureLedger::default();

    assert_eq!(
        finish_editor_host("project=test", Ok(7_u8), failures.snapshot()).unwrap(),
        7
    );
}
