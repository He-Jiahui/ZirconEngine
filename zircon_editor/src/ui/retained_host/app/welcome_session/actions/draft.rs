use super::super::super::*;

impl RetainedEditorHost {
    pub(super) fn update_welcome_project_name(&mut self, value: &str) {
        self.startup_session.draft.project_name = value.to_string();
        self.schedule_welcome_project_probe();
        self.refresh_welcome_snapshot();
    }

    pub(super) fn update_welcome_location(&mut self, value: &str) {
        self.startup_session.draft.location = value.to_string();
        self.schedule_welcome_project_probe();
        self.refresh_welcome_snapshot();
    }
}
