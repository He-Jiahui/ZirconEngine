mod host_window;
mod invalidation;
mod overlay;
mod refresh;

pub(crate) use host_window::{
    HostWindowDiagnostic, HostWindowDiagnosticQueue, HostWindowDiagnosticSeverity,
};
pub(crate) use invalidation::HostInvalidationDiagnostics;
pub(crate) use overlay::STARTUP_REFRESH_DIAGNOSTICS_OVERLAY;
pub(crate) use refresh::HostRefreshDiagnostics;

#[cfg(test)]
#[path = "diagnostics_tests.rs"]
mod tests;
