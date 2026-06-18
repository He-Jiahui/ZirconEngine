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

    pub(super) fn refresh_invalidation_diagnostics(&self) -> HostInvalidationDiagnostics {
        self.state.borrow().refresh_invalidation_diagnostics
    }

    pub(super) fn set_host_refresh_diagnostics_overlay(&self, diagnostics: HostRefreshDiagnostics) {
        self.set_host_refresh_diagnostics_overlay_text(diagnostics.overlay_text().into());
    }

    fn set_host_refresh_diagnostics_overlay_text(&self, overlay_text: SharedString) {
        let mut state = self.state.borrow_mut();
        state.host_presentation.host_shell.debug_refresh_rate = overlay_text;
    }
}
