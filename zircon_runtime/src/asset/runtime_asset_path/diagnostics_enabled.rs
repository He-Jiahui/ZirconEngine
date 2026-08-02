use crate::diagnostic_log::{DiagnosticLogLevel, diagnostic_log_allows, write_diagnostic_log};

const DIAGNOSTIC_SCOPE: &str = "runtime_asset_path";

pub(super) fn verbose_enabled() -> bool {
    diagnostic_log_allows(DiagnosticLogLevel::Verbose)
}

pub(super) fn write_verbose(message: impl AsRef<str>) {
    write_diagnostic_log(DIAGNOSTIC_SCOPE, message);
}
