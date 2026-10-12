#[path = "diagnostics/host_window.rs"]
mod host_window;
#[path = "diagnostics/invalidation.rs"]
mod invalidation;
#[path = "diagnostics/overlay.rs"]
mod overlay;
#[path = "diagnostics/refresh.rs"]
mod refresh;

pub(crate) use host_window::{
    HostWindowDiagnostic, HostWindowDiagnosticQueue, HostWindowDiagnosticSeverity,
};
pub(crate) use invalidation::HostInvalidationDiagnostics;
pub(crate) use overlay::STARTUP_REFRESH_DIAGNOSTICS_OVERLAY;
pub(crate) use refresh::HostRefreshDiagnostics;

#[cfg(test)]
#[path = "tests/diagnostics.rs"]
mod tests;
