use super::super::super::*;

impl RetainedEditorHost {
    pub(super) fn open_startup_workbench(&mut self) {
        let _ = self.editor_manager.dismiss_welcome_page();
        self.runtime.set_session_mode(EditorSessionMode::Project);
        self.invalidate_host(HostInvalidationMask::PRESENTATION_DATA);
        self.set_status_line("Opened default workbench".to_string());
    }

    pub(super) fn open_startup_view(&mut self, descriptor_id: &str, status: &str) {
        match self
            .editor_manager
            .dismiss_welcome_page()
            .map_err(|error| error.to_string())
            .and_then(|_| {
                self.editor_manager
                    .open_view(
                        crate::ui::workbench::view::ViewDescriptorId::new(descriptor_id),
                        None,
                    )
                    .map_err(|error| error.to_string())
            }) {
            Ok(_) => {
                self.runtime.set_session_mode(EditorSessionMode::Project);
                self.invalidate_host(HostInvalidationMask::PRESENTATION_DATA);
                self.set_status_line(status.to_string());
            }
            Err(error) => {
                self.startup_session.status_message = error.clone();
                self.refresh_welcome_snapshot();
                self.set_status_line(error);
            }
        }
    }
}
