use super::super::super::*;

impl RetainedEditorHost {
    pub(in crate::ui::retained_host::app) fn present_welcome_surface(
        &mut self,
        status_message: impl Into<String>,
    ) -> Result<(), String> {
        self.startup_session.recent_projects = self
            .editor_manager
            .recent_projects_snapshot()
            .map_err(|error| error.to_string())?;
        self.startup_session.status_message = status_message.into();
        self.editor_manager
            .show_welcome_page()
            .map_err(|error| error.to_string())?;
        if !self.runtime.editor_snapshot().project_open {
            self.runtime.set_session_mode(EditorSessionMode::Welcome);
        }
        self.refresh_welcome_snapshot();
        Ok(())
    }
}
