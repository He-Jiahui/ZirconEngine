use super::super::super::*;

impl RetainedEditorHost {
    pub(super) fn open_startup_workbench(&mut self) {
        match self.editor_manager.dismiss_welcome_page() {
            Ok(()) => {
                self.clear_welcome_project_probe();
                self.runtime.set_session_mode(EditorSessionMode::Project);
                self.invalidate_host(HostInvalidationMask::PRESENTATION_DATA);
                self.set_status_line("Opened default workbench".to_string());
            }
            Err(error) => {
                let error = update_startup_error(&mut self.startup_session.status_message, error);
                self.restore_welcome_after_startup_failure(error);
            }
        }
    }

    // 示例视图启动退出欢迎页并取消探测；a rejected view admission restores the welcome
    // projection so the manager and retained host retain one active page.
    pub(super) fn open_startup_view(&mut self, descriptor_id: &str, status: &str) {
        match self
            .editor_manager
            .open_view(
                crate::ui::workbench::view::ViewDescriptorId::new(descriptor_id),
                None,
            )
            .map_err(|error| error.to_string())
            .and_then(|instance_id| {
                if let Err(error) = self.editor_manager.dismiss_welcome_page() {
                    let _ = self.editor_manager.close_view(&instance_id);
                    return Err(error.to_string());
                }
                self.clear_welcome_project_probe();
                Ok(instance_id)
            }) {
            Ok(_) => {
                self.runtime.set_session_mode(EditorSessionMode::Project);
                self.invalidate_host(HostInvalidationMask::PRESENTATION_DATA);
                self.set_status_line(status.to_string());
            }
            Err(error) => {
                self.startup_session.status_message = error.clone();
                self.restore_welcome_after_startup_failure(error);
            }
        }
    }

    fn restore_welcome_after_startup_failure(&mut self, error: String) {
        let error = match self.editor_manager.show_welcome_page() {
            Ok(()) => error,
            Err(restore_error) => format!(
                "{error}; startup failure could not restore the Welcome page: {restore_error}"
            ),
        };
        // Runtime and manager page state are restored together; presentation data is then
        // recomputed from the retained Welcome session.
        self.runtime.set_session_mode(EditorSessionMode::Welcome);
        self.refresh_welcome_snapshot();
        self.set_status_line(error);
    }
}

fn update_startup_error(status_message: &mut String, error: impl std::fmt::Display) -> String {
    let formatted = error.to_string();
    status_message.clone_from(&formatted);
    formatted
}

#[cfg(test)]
#[path = "startup_views/tests/reused_error_tests.rs"]
mod reused_error_tests;
