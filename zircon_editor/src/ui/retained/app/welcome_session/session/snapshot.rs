use super::super::super::*;

impl RetainedEditorHost {
    pub(in crate::ui::retained_host::app) fn refresh_welcome_snapshot(&mut self) {
        let snapshot = self.startup_session.welcome_pane_snapshot(false);
        self.runtime.set_welcome_snapshot(snapshot);
        self.invalidate_host(HostInvalidationMask::PRESENTATION_DATA);
    }
}
