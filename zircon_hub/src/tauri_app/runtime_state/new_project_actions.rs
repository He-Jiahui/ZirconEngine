use std::path::PathBuf;

use crate::error::HubError;
use crate::projects::project_template_catalog;
use crate::state::ProjectSubpage;
use crate::tauri_app::action_request::{CreateProjectActionPayload, NewProjectDraftActionPayload};

use super::HubRuntimeSession;

impl HubRuntimeSession {
    pub(super) fn update_new_project_draft(
        &mut self,
        payload: NewProjectDraftActionPayload,
    ) -> Result<(), HubError> {
        self.apply_new_project_draft_fields(
            payload.name,
            payload.location,
            payload.template,
            payload.engine_id,
        );
        self.project_subpage = ProjectSubpage::NewProject;
        self.pending_delete_project_path = None;
        self.persist(None)
    }

    pub(super) fn remember_create_project_payload(&mut self, payload: &CreateProjectActionPayload) {
        self.apply_new_project_draft_fields(
            payload.name.clone(),
            payload.location.clone(),
            payload.template.clone(),
            payload.engine_id.clone(),
        );
    }

    fn apply_new_project_draft_fields(
        &mut self,
        name: String,
        location: PathBuf,
        template: String,
        engine_id: Option<String>,
    ) {
        self.new_project_name = name.trim().to_string();
        self.new_project_location = location;
        if project_template_catalog()
            .iter()
            .any(|candidate| candidate.id == template)
        {
            self.selected_template_id = template;
        }
        self.new_project_engine_id =
            engine_id.filter(|id| self.config.engines.iter().any(|engine| engine.id == *id));
    }
}
