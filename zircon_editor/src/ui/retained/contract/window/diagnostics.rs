use super::super::diagnostics::{
    HostInvalidationDiagnostics, HostWindowDiagnostic, HostWindowDiagnosticSeverity,
};
use super::UiHostWindow;

impl UiHostWindow {
    pub(crate) fn set_host_refresh_invalidation_diagnostics(
        &self,
        diagnostics: HostInvalidationDiagnostics,
    ) {
        self.state.borrow_mut().refresh_invalidation_diagnostics = diagnostics;
    }

    pub(in crate::ui::retained_host::host_contract) fn refresh_invalidation_diagnostics(
        &self,
    ) -> HostInvalidationDiagnostics {
        self.state.borrow().refresh_invalidation_diagnostics
    }

    pub(in crate::ui::retained_host::host_contract) fn record_host_diagnostic(
        &self,
        severity: HostWindowDiagnosticSeverity,
        message: impl Into<String>,
    ) {
        self.state
            .borrow_mut()
            .host_window_diagnostics
            .push(HostWindowDiagnostic::new(severity, message));
    }

    pub(in crate::ui::retained_host) fn take_host_diagnostics(&self) -> Vec<HostWindowDiagnostic> {
        self.state.borrow_mut().host_window_diagnostics.drain()
    }
}
