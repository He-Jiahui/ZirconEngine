#[path = "load_report/diagnostics.rs"]
pub(super) mod diagnostics;
#[path = "load_report/report.rs"]
mod report;

pub use diagnostics::RuntimeModuleLoadDiagnostic;
pub(in crate::builtin::runtime_modules) use report::RuntimeModuleLoadReport;
