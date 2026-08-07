use crate::ui::retained_host::primitives::SharedString;

use super::super::diagnostics::{HostInvalidationDiagnostics, HostRefreshDiagnostics};
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

    pub(in crate::ui::retained_host::host_contract) fn set_host_refresh_diagnostics_overlay(
        &self,
        diagnostics: HostRefreshDiagnostics,
    ) {
        self.set_host_refresh_diagnostics_overlay_text(diagnostics.overlay_text().into());
    }

    fn set_host_refresh_diagnostics_overlay_text(&self, overlay_text: SharedString) {
        self.state
            .borrow_mut()
            .replace_diagnostics_overlay_text(overlay_text);
    }
}
