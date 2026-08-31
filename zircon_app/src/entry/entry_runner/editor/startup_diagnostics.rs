use std::error::Error;
use std::fmt::{self, Display, Formatter};

use crate::entry::product_shutdown::{
    ProductFailureLedger, ProductFailureReport, ProductFailureSeverity, ProductHostPhase,
};

#[derive(Debug)]
pub(super) struct EditorStartupDiagnosticError {
    component: &'static str,
    requested: String,
    cause: String,
    recovery: &'static str,
}

impl Display for EditorStartupDiagnosticError {
    fn fmt(&self, formatter: &mut Formatter<'_>) -> fmt::Result {
        write!(
            formatter,
            "editor startup diagnostic: component={} requested={} cause={} recovery={}",
            self.component, self.requested, self.cause, self.recovery
        )
    }
}

impl Error for EditorStartupDiagnosticError {}

pub(super) fn editor_startup_diagnostic_error(
    component: &'static str,
    requested: impl Into<String>,
    cause: impl Into<String>,
    recovery: &'static str,
) -> EditorStartupDiagnosticError {
    EditorStartupDiagnosticError {
        component,
        requested: requested.into(),
        cause: cause.into(),
        recovery,
    }
}

pub(super) fn record_editor_host_failure<T>(
    failures: &ProductFailureLedger,
    host_result: &Result<T, Box<dyn Error>>,
) {
    if let Err(error) = host_result {
        failures.record(
            ProductHostPhase::Running,
            ProductFailureSeverity::Terminal,
            "editor_host",
            error,
        );
    }
}

pub(super) fn finish_editor_host<T>(
    requested: &str,
    host_result: Result<T, Box<dyn Error>>,
    failure_report: ProductFailureReport,
) -> Result<T, Box<dyn Error>> {
    if failure_report.is_empty() {
        return host_result;
    }
    Err(editor_startup_diagnostic_error(
        "editor_process",
        requested,
        format!("terminal failure ledger: {failure_report}"),
        "inspect every reported editor/runtime terminal failure, repair the lowest owner, and retry zircon_editor",
    )
    .into())
}

pub(super) fn editor_host_startup_error(
    requested: &str,
    source: Box<dyn Error>,
) -> EditorStartupDiagnosticError {
    editor_startup_diagnostic_error(
        "editor_host",
        requested,
        format!("editor host execution failed: {source}"),
        "verify the requested project or view and the staged editor assets before retrying zircon_editor",
    )
}

#[cfg(test)]
mod tests {
    use crate::entry::product_shutdown::{
        ProductFailureLedger, ProductFailureSeverity, ProductHostPhase,
    };

    use super::{finish_editor_host, record_editor_host_failure};

    #[test]
    fn editor_finish_preserves_host_and_shutdown_failures_in_order() {
        let failures = ProductFailureLedger::default();
        let host_result = Err::<(), Box<dyn std::error::Error>>(
            std::io::Error::other("editor host failed").into(),
        );
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
}
