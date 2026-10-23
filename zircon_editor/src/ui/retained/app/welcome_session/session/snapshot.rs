use super::super::super::*;

impl RetainedEditorHost {
    pub(in crate::ui::retained_host::app) fn refresh_welcome_snapshot(&mut self) {
        let snapshot = self.startup_session.welcome_pane_snapshot(false);
        self.runtime.set_welcome_snapshot(snapshot);
        self.invalidate_host(HostInvalidationMask::PRESENTATION_DATA);
        // A lifecycle wake collects data without present damage. Welcome's
        // native validation/preview and control states also need a paint.
        let damage = self
            .ui
            .get_host_presentation_generation()
            .structure()
            .host_layout
            .document_region_frame
            .clone();
        self.ui.request_frame_update_region(damage);
    }
}
