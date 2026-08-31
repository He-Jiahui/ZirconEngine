use super::super::super::*;

impl RetainedEditorHost {
    pub(super) fn update_welcome_project_name(&mut self, value: &str) {
        update_draft_text(&mut self.startup_session.draft.project_name, value);
        self.schedule_welcome_project_probe();
        self.refresh_welcome_snapshot();
    }

    pub(super) fn update_welcome_location(&mut self, value: &str) {
        update_draft_text(&mut self.startup_session.draft.location, value);
        self.schedule_welcome_project_probe();
        self.refresh_welcome_snapshot();
    }
}

fn update_draft_text(target: &mut String, value: &str) {
    target.clear();
    target.push_str(value);
}

#[cfg(test)]
#[path = "draft/reused_text_tests.rs"]
mod reused_text_tests;
