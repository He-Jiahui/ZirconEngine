use std::path::Path;

use crate::error::HubError;
use crate::projects::{project_paths_match, RecentProject};
use crate::state::{HubMessage, HubMessageId, ShellMessageId};
use crate::tauri_app::action_id::HubActionId;
use crate::tauri_app::action_request::{HubActionRequest, ProjectTargetActionPayload};

use super::{recent_project_display_name, HubRuntimeSession};

impl HubRuntimeSession {
    pub(in crate::tauri_app) fn apply_request_project_target(
        &mut self,
        request: &HubActionRequest,
    ) -> Result<(), HubError> {
        let action = request.action()?;
        let payload = request.project_target_payload()?;
        self.apply_action_project_target(request.target_id.as_deref(), payload.as_ref(), action)
    }

    pub(in crate::tauri_app) fn apply_action_project_target(
        &mut self,
        target_id: Option<&str>,
        payload: Option<&ProjectTargetActionPayload>,
        action: HubActionId,
    ) -> Result<(), HubError> {
        let targets = project_target_candidates(target_id, payload);
        if targets.is_empty() {
            return Ok(());
        }
        let Some(project) = targets
            .iter()
            .find_map(|target| self.find_recent_project(target))
        else {
            return Err(HubError::status(
                HubMessage::with_params(
                    HubMessageId::Shell(ShellMessageId::UnknownRecentProjectTarget),
                    [action.as_str().to_string(), targets[0].clone()],
                ),
                Some(HubMessage::new(HubMessageId::Shell(
                    ShellMessageId::CheckActionTarget,
                ))),
            ));
        };
        self.activate_recent_project_target(project)
    }

    pub(in crate::tauri_app) fn project_target_label_from_request(
        &self,
        request: &HubActionRequest,
    ) -> Option<String> {
        let payload = request.project_target_payload().ok().flatten();
        project_target_candidates(request.target_id.as_deref(), payload.as_ref())
            .into_iter()
            .find_map(|target| {
                self.find_recent_project(&target)
                    .map(|project| recent_project_display_name(&project))
                    .or(Some(target))
            })
    }

    fn activate_recent_project_target(&mut self, project: RecentProject) -> Result<(), HubError> {
        let selected_before = self.selected_project_path.clone();
        let active_engine_before = self.config.active_engine_id.clone();
        self.selected_project_path = Some(project.path.clone());
        self.activate_project_engine_for_path(&project.path);
        self.refresh_project_context_views(
            selected_project_path_changed(
                selected_before.as_deref(),
                self.selected_project_path.as_deref(),
            ),
            self.config.active_engine_id != active_engine_before,
        )
    }
}

pub(super) fn project_target_candidates(
    target_id: Option<&str>,
    payload: Option<&ProjectTargetActionPayload>,
) -> Vec<String> {
    let mut targets = Vec::new();
    if let Some(payload) = payload {
        if let Some(path) = payload
            .project_path
            .as_ref()
            .filter(|path| !path.as_os_str().is_empty())
        {
            targets.push(path.to_string_lossy().into_owned());
        }
        push_trimmed_candidate(&mut targets, payload.project_id.as_deref());
    }
    push_trimmed_candidate(&mut targets, target_id);
    targets
}

fn push_trimmed_candidate(targets: &mut Vec<String>, candidate: Option<&str>) {
    if let Some(candidate) = candidate
        .map(str::trim)
        .filter(|candidate| !candidate.is_empty())
    {
        targets.push(candidate.to_string());
    }
}

fn selected_project_path_changed(before: Option<&Path>, after: Option<&Path>) -> bool {
    match (before, after) {
        (Some(before), Some(after)) => !project_paths_match(before, after),
        (None, None) => false,
        _ => true,
    }
}

#[cfg(test)]
#[path = "tests/action_targets.rs"]
mod tests;
