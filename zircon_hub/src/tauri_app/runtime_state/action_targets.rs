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
mod tests {
    use std::{fs, path::PathBuf};

    use crate::{
        projects::RecentProject, settings::HubConfig, state::TaskOperationKind,
        tauri_app::HubActionRequest,
    };

    use super::super::HubRuntimeSession;

    #[test]
    fn explicit_project_target_updates_selected_project_without_overwriting_task_status() {
        let temp = temp_test_dir("zircon-hub-action-targets-explicit-project");
        let selected = create_project_root(&temp, "Selected");
        let target = create_project_root(&temp, "Target");
        let mut session = session_with_projects(
            &temp,
            &[("Selected", selected.clone()), ("Target", target.clone())],
            &selected,
        );
        session.task_status = crate::state::TaskStatus::running_operation(
            "Packaging",
            crate::state::HubMessage::new(crate::state::HubMessageId::Delivery(
                crate::state::DeliveryMessageId::CopyingProjectToPackage,
            )),
            TaskOperationKind::Project,
            "Target",
        );

        session
            .apply_request_project_target(&HubActionRequest {
                action_id: "package-project".to_string(),
                target_id: Some(target.to_string_lossy().into_owned()),
                payload: None,
            })
            .expect("explicit target should resolve to a recent project");

        assert_eq!(
            session.selected_project_path.as_deref(),
            Some(target.as_path())
        );
        assert_eq!(session.task_status.label, "Packaging");
        assert!(session.task_status.running);

        fs::remove_dir_all(temp).unwrap();
    }

    #[test]
    fn project_target_payload_takes_precedence_over_target_id_for_background_actions() {
        let temp = temp_test_dir("zircon-hub-action-targets-payload-project");
        let fallback = create_project_root(&temp, "Fallback");
        let target = create_project_root(&temp, "Target");
        let mut session = session_with_projects(
            &temp,
            &[("Fallback", fallback.clone()), ("Target", target.clone())],
            &fallback,
        );

        session
            .apply_request_project_target(&HubActionRequest {
                action_id: "package-project".to_string(),
                target_id: Some(fallback.to_string_lossy().into_owned()),
                payload: Some(serde_json::json!({
                    "projectPath": target
                })),
            })
            .expect("typed project payload should select the payload project before target_id");

        assert_eq!(
            session.selected_project_path.as_deref(),
            Some(target.as_path())
        );

        fs::remove_dir_all(temp).unwrap();
    }

    #[test]
    fn project_target_payload_path_takes_precedence_over_project_id() {
        let temp = temp_test_dir("zircon-hub-action-targets-payload-path");
        let fallback = create_project_root(&temp, "Fallback");
        let target = create_project_root(&temp, "Target");
        let mut session = session_with_projects(
            &temp,
            &[("Fallback", fallback.clone()), ("Target", target.clone())],
            &fallback,
        );

        session
            .apply_request_project_target(&HubActionRequest {
                action_id: "package-project".to_string(),
                target_id: None,
                payload: Some(serde_json::json!({
                    "projectId": fallback,
                    "projectPath": target
                })),
            })
            .expect("projectPath should resolve before projectId when both are present");

        assert_eq!(
            session.selected_project_path.as_deref(),
            Some(target.as_path())
        );

        fs::remove_dir_all(temp).unwrap();
    }

    fn session_with_projects(
        temp: &std::path::Path,
        projects: &[(&str, PathBuf)],
        selected_project: &std::path::Path,
    ) -> HubRuntimeSession {
        let config_path = temp.join("hub.toml");
        let shared_recent_projects_path = temp.join("recent_projects.json");
        let mut config = HubConfig::default();
        config.settings.default_build_output_dir = temp.join("out");
        config.recent_projects = projects
            .iter()
            .map(|(name, path)| RecentProject::fixture(*name, path, 1))
            .collect();
        config.runtime.selected_project_path = Some(selected_project.to_path_buf());
        config.save(&config_path).unwrap();
        fs::write(
            &shared_recent_projects_path,
            r#"{"protocol_version":1,"projects":[]}"#,
        )
        .unwrap();
        HubRuntimeSession::load_from_paths(config_path, shared_recent_projects_path).unwrap()
    }

    fn create_project_root(temp: &std::path::Path, name: &str) -> PathBuf {
        let project = temp.join(name);
        fs::create_dir_all(project.join("Assets")).unwrap();
        fs::write(
            project.join("zircon-project.toml"),
            format!("name = \"{name}\"\n"),
        )
        .unwrap();
        project
    }

    fn temp_test_dir(prefix: &str) -> PathBuf {
        let path = std::env::temp_dir().join(format!(
            "{prefix}-{}-{}",
            std::process::id(),
            crate::projects::now_unix_ms()
        ));
        let _ = fs::remove_dir_all(&path);
        fs::create_dir_all(&path).unwrap();
        path
    }
}
