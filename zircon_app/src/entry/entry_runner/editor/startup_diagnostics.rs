//! 编辑器产品边界在 owner 清理正常返回后汇总宿主结果与共享账本。
//! 会话 destroy 的进程终止诊断由 Drop 失败路径直接输出，不会返回本报告入口。

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

/// 在释放 Runtime 会话之前写入宿主失败，以保留它与随后销毁错误的先后顺序。
pub(super) fn record_editor_host_failure<T, E: Display>(
    failures: &ProductFailureLedger,
    host_result: &Result<T, E>,
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

/// 仅在宿主结果和传入账本都成功后返回成功；持有会话的调用方须在清理正常返回后传入最终快照。
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
#[path = "tests/startup_diagnostics.rs"]
mod tests;
