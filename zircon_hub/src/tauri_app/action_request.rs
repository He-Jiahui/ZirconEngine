use std::path::PathBuf;

use serde::Deserialize;
use serde_json::Value;

use crate::error::HubError;

use super::view_model::{settings_payload_from_value, HubSettingsPayload};

#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub(crate) struct HubActionRequest {
    pub action_id: String,
    pub target_id: Option<String>,
    #[serde(default)]
    pub payload: Option<Value>,
}

#[derive(Debug, Clone)]
pub(crate) enum HubAction {
    ShowPage {
        target_id: String,
    },
    ShowProjectSubpage {
        target_id: String,
    },
    SearchProjects {
        query: String,
    },
    SetProjectFilter {
        target_id: String,
    },
    SetProjectSort {
        target_id: String,
    },
    SetProjectViewMode {
        target_id: String,
    },
    SelectProject {
        target_id: String,
    },
    OpenProjectDetail {
        target_id: String,
    },
    ViewAllProjects,
    NewProject,
    UpdateNewProjectDraft {
        payload: NewProjectDraftActionPayload,
    },
    SelectEngine {
        target_id: String,
    },
    SaveSettings {
        payload: Option<HubSettingsPayload>,
    },
    BrowseSettingsFolder {
        target_id: Option<String>,
        payload: Option<BrowseSettingsFolderPayload>,
    },
    CreateProject {
        payload: CreateProjectActionPayload,
    },
    ImportProject {
        target_id: Option<String>,
        payload: Option<ImportProjectActionPayload>,
    },
    PinProject {
        target_id: Option<String>,
        payload: Option<ProjectTargetActionPayload>,
    },
    UnpinProject {
        target_id: Option<String>,
        payload: Option<ProjectTargetActionPayload>,
    },
    RemoveFromHub {
        target_id: Option<String>,
        payload: Option<ProjectTargetActionPayload>,
    },
    RequestDelete {
        target_id: Option<String>,
        payload: Option<ProjectTargetActionPayload>,
    },
    CancelDelete {
        target_id: Option<String>,
        payload: Option<ProjectTargetActionPayload>,
    },
    ConfirmDelete {
        target_id: Option<String>,
        payload: Option<ProjectTargetActionPayload>,
    },
    OpenResource {
        target_id: Option<String>,
        payload: Option<OpenResourcePayload>,
    },
    OpenOutputFolder {
        target_id: Option<String>,
        payload: Option<OpenOutputFolderPayload>,
    },
    BuildProject {
        target_id: Option<String>,
        payload: Option<ProjectTargetActionPayload>,
    },
    PackageProject {
        target_id: Option<String>,
        payload: Option<ProjectTargetActionPayload>,
    },
    InstallDevice {
        target_id: Option<String>,
        payload: Option<ProjectTargetActionPayload>,
    },
    OpenEditor {
        target_id: Option<String>,
        payload: Option<ProjectTargetActionPayload>,
    },
}

#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub(crate) struct SearchProjectsPayload {
    #[serde(default)]
    pub query: String,
}

#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
struct SearchProjectsEnvelope {
    search: SearchProjectsPayload,
}

#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub(crate) struct NewProjectDraftActionPayload {
    #[serde(alias = "projectName")]
    pub name: String,
    pub location: PathBuf,
    pub template: String,
    pub engine_id: Option<String>,
}

#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
struct NewProjectDraftActionEnvelope {
    draft: NewProjectDraftActionPayload,
}

#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub(crate) struct CreateProjectActionPayload {
    #[serde(alias = "projectName")]
    pub name: String,
    pub location: PathBuf,
    pub template: String,
    pub engine_id: Option<String>,
}

#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
struct CreateProjectActionEnvelope {
    project: CreateProjectActionPayload,
}

#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub(crate) struct ImportProjectActionPayload {
    pub path: Option<PathBuf>,
    pub folder: Option<PathBuf>,
    pub engine_id: Option<String>,
}

#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
struct ImportProjectActionEnvelope {
    project: ImportProjectActionPayload,
}

#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub(crate) struct ProjectTargetActionPayload {
    #[serde(alias = "id")]
    pub project_id: Option<String>,
    #[serde(alias = "path")]
    pub project_path: Option<PathBuf>,
}

#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
struct ProjectTargetActionEnvelope {
    project: ProjectTargetActionPayload,
}

#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub(crate) struct BrowseSettingsFolderPayload {
    pub field: Option<String>,
    pub initial_dir: Option<PathBuf>,
    pub settings: Option<HubSettingsPayload>,
}

#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
struct BrowseSettingsFolderEnvelope {
    folder: BrowseSettingsFolderPayload,
}

#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub(crate) struct OpenResourcePayload {
    pub resource_id: Option<String>,
    pub path: Option<PathBuf>,
}

#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
struct OpenResourceEnvelope {
    resource: OpenResourcePayload,
}

#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub(crate) struct OpenOutputFolderPayload {
    pub path: Option<PathBuf>,
    pub output_dir: Option<PathBuf>,
    pub history_id: Option<String>,
}

#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
struct OpenOutputFolderEnvelope {
    output: OpenOutputFolderPayload,
}

impl HubActionRequest {
    pub(crate) fn parse(&self) -> Result<HubAction, HubError> {
        match self.action_id.trim() {
            "show-page" | "page" => Ok(HubAction::ShowPage {
                target_id: self.required_target()?.to_string(),
            }),
            "show-project-subpage" | "project-subpage" => Ok(HubAction::ShowProjectSubpage {
                target_id: self.required_target()?.to_string(),
            }),
            "search-projects" => Ok(HubAction::SearchProjects {
                query: search_projects_payload_from_value(
                    self.payload.as_ref(),
                    self.target_id.as_deref(),
                )?
                .query,
            }),
            "set-project-filter" => Ok(HubAction::SetProjectFilter {
                target_id: self.required_target()?.to_string(),
            }),
            "set-project-sort" => Ok(HubAction::SetProjectSort {
                target_id: self.required_target()?.to_string(),
            }),
            "set-project-view-mode" => Ok(HubAction::SetProjectViewMode {
                target_id: self.required_target()?.to_string(),
            }),
            "select-project" | "open-project" => Ok(HubAction::SelectProject {
                target_id: self.required_target()?.to_string(),
            }),
            "open-project-detail" => Ok(HubAction::OpenProjectDetail {
                target_id: self.required_target()?.to_string(),
            }),
            "view-all-projects" => Ok(HubAction::ViewAllProjects),
            "new-project" => Ok(HubAction::NewProject),
            "update-new-project-draft" => Ok(HubAction::UpdateNewProjectDraft {
                payload: new_project_draft_payload_from_value(self.payload.as_ref())?,
            }),
            "select-engine" => Ok(HubAction::SelectEngine {
                target_id: self.required_target()?.to_string(),
            }),
            "save-settings" => Ok(HubAction::SaveSettings {
                payload: settings_payload_from_value(self.payload.as_ref())?,
            }),
            "browse-settings-folder" => Ok(HubAction::BrowseSettingsFolder {
                target_id: self.trimmed_target(),
                payload: browse_settings_folder_payload_from_value(self.payload.as_ref())?,
            }),
            "create-project" => Ok(HubAction::CreateProject {
                payload: create_project_payload_from_value(self.payload.as_ref())?,
            }),
            "import-project" => Ok(HubAction::ImportProject {
                target_id: self.trimmed_target(),
                payload: import_project_payload_from_value(self.payload.as_ref())?,
            }),
            "pin-project" => Ok(HubAction::PinProject {
                target_id: self.trimmed_target(),
                payload: project_target_payload_from_value(self.payload.as_ref())?,
            }),
            "unpin-project" => Ok(HubAction::UnpinProject {
                target_id: self.trimmed_target(),
                payload: project_target_payload_from_value(self.payload.as_ref())?,
            }),
            "remove-from-hub" => Ok(HubAction::RemoveFromHub {
                target_id: self.trimmed_target(),
                payload: project_target_payload_from_value(self.payload.as_ref())?,
            }),
            "request-delete" => Ok(HubAction::RequestDelete {
                target_id: self.trimmed_target(),
                payload: project_target_payload_from_value(self.payload.as_ref())?,
            }),
            "cancel-delete" => Ok(HubAction::CancelDelete {
                target_id: self.trimmed_target(),
                payload: project_target_payload_from_value(self.payload.as_ref())?,
            }),
            "confirm-delete" => Ok(HubAction::ConfirmDelete {
                target_id: self.trimmed_target(),
                payload: project_target_payload_from_value(self.payload.as_ref())?,
            }),
            "open-resource" => Ok(HubAction::OpenResource {
                target_id: self.trimmed_target(),
                payload: open_resource_payload_from_value(self.payload.as_ref())?,
            }),
            "open-output-folder" => Ok(HubAction::OpenOutputFolder {
                target_id: self.trimmed_target(),
                payload: open_output_folder_payload_from_value(self.payload.as_ref())?,
            }),
            "build-project" => Ok(HubAction::BuildProject {
                target_id: self.trimmed_target(),
                payload: project_target_payload_from_value(self.payload.as_ref())?,
            }),
            "package-project" => Ok(HubAction::PackageProject {
                target_id: self.trimmed_target(),
                payload: project_target_payload_from_value(self.payload.as_ref())?,
            }),
            "install-device" => Ok(HubAction::InstallDevice {
                target_id: self.trimmed_target(),
                payload: project_target_payload_from_value(self.payload.as_ref())?,
            }),
            "open-editor" => Ok(HubAction::OpenEditor {
                target_id: self.trimmed_target(),
                payload: project_target_payload_from_value(self.payload.as_ref())?,
            }),
            _ => Err(HubError::message(format!(
                "Unknown Hub action: {}",
                self.action_id
            ))),
        }
    }

    pub(crate) fn project_target_payload(
        &self,
    ) -> Result<Option<ProjectTargetActionPayload>, HubError> {
        project_target_payload_from_value(self.payload.as_ref())
    }

    fn trimmed_target(&self) -> Option<String> {
        self.target_id
            .as_deref()
            .map(str::trim)
            .filter(|target| !target.is_empty())
            .map(str::to_string)
    }

    fn required_target(&self) -> Result<&str, HubError> {
        self.target_id
            .as_deref()
            .map(str::trim)
            .filter(|target| !target.is_empty())
            .ok_or_else(|| {
                HubError::message(format!(
                    "Target is required for Hub action: {}",
                    self.action_id
                ))
            })
    }
}

fn search_projects_payload_from_value(
    payload: Option<&Value>,
    target_id: Option<&str>,
) -> Result<SearchProjectsPayload, HubError> {
    let Some(payload) = payload else {
        return Ok(SearchProjectsPayload {
            query: target_id.unwrap_or("").to_string(),
        });
    };
    if let Some(query) = payload.as_str() {
        return Ok(SearchProjectsPayload {
            query: query.to_string(),
        });
    }
    if payload.get("search").is_some() {
        let envelope: SearchProjectsEnvelope = serde_json::from_value(payload.clone())?;
        return Ok(envelope.search);
    }
    Ok(serde_json::from_value(payload.clone())?)
}

fn new_project_draft_payload_from_value(
    payload: Option<&Value>,
) -> Result<NewProjectDraftActionPayload, HubError> {
    let Some(payload) = payload else {
        return Err(HubError::message("New Project draft payload is required"));
    };
    if payload.get("draft").is_some() {
        let envelope: NewProjectDraftActionEnvelope = serde_json::from_value(payload.clone())?;
        return Ok(envelope.draft);
    }
    Ok(serde_json::from_value(payload.clone())?)
}

fn create_project_payload_from_value(
    payload: Option<&Value>,
) -> Result<CreateProjectActionPayload, HubError> {
    let Some(payload) = payload else {
        return Err(HubError::message("Create Project payload is required"));
    };
    if payload.get("project").is_some() {
        let envelope: CreateProjectActionEnvelope = serde_json::from_value(payload.clone())?;
        return Ok(envelope.project);
    }
    Ok(serde_json::from_value(payload.clone())?)
}

fn import_project_payload_from_value(
    payload: Option<&Value>,
) -> Result<Option<ImportProjectActionPayload>, HubError> {
    let Some(payload) = payload else {
        return Ok(None);
    };
    if let Some(path) = payload.as_str() {
        return Ok(Some(ImportProjectActionPayload {
            path: Some(PathBuf::from(path)),
            folder: None,
            engine_id: None,
        }));
    }
    if payload.get("project").is_some() {
        let envelope: ImportProjectActionEnvelope = serde_json::from_value(payload.clone())?;
        return Ok(Some(envelope.project));
    }
    Ok(Some(serde_json::from_value(payload.clone())?))
}

fn project_target_payload_from_value(
    payload: Option<&Value>,
) -> Result<Option<ProjectTargetActionPayload>, HubError> {
    let Some(payload) = payload else {
        return Ok(None);
    };
    if let Some(target) = payload
        .as_str()
        .map(str::trim)
        .filter(|target| !target.is_empty())
    {
        return Ok(Some(ProjectTargetActionPayload {
            project_id: Some(target.to_string()),
            project_path: None,
        }));
    }
    if payload.get("project").is_some() {
        let envelope: ProjectTargetActionEnvelope = serde_json::from_value(payload.clone())?;
        return Ok(Some(envelope.project));
    }
    Ok(Some(serde_json::from_value(payload.clone())?))
}

fn browse_settings_folder_payload_from_value(
    payload: Option<&Value>,
) -> Result<Option<BrowseSettingsFolderPayload>, HubError> {
    let Some(payload) = payload else {
        return Ok(None);
    };
    if payload.get("folder").is_some() {
        let envelope: BrowseSettingsFolderEnvelope = serde_json::from_value(payload.clone())?;
        return Ok(Some(envelope.folder));
    }
    Ok(Some(serde_json::from_value(payload.clone())?))
}

fn open_resource_payload_from_value(
    payload: Option<&Value>,
) -> Result<Option<OpenResourcePayload>, HubError> {
    let Some(payload) = payload else {
        return Ok(None);
    };
    if let Some(target) = payload
        .as_str()
        .map(str::trim)
        .filter(|target| !target.is_empty())
    {
        return Ok(Some(OpenResourcePayload {
            resource_id: Some(target.to_string()),
            path: None,
        }));
    }
    if payload.get("resource").is_some() {
        let envelope: OpenResourceEnvelope = serde_json::from_value(payload.clone())?;
        return Ok(Some(envelope.resource));
    }
    Ok(Some(serde_json::from_value(payload.clone())?))
}

fn open_output_folder_payload_from_value(
    payload: Option<&Value>,
) -> Result<Option<OpenOutputFolderPayload>, HubError> {
    let Some(payload) = payload else {
        return Ok(None);
    };
    if let Some(path) = payload
        .as_str()
        .map(str::trim)
        .filter(|path| !path.is_empty())
    {
        return Ok(Some(OpenOutputFolderPayload {
            path: Some(PathBuf::from(path)),
            output_dir: None,
            history_id: None,
        }));
    }
    if payload.get("output").is_some() {
        let envelope: OpenOutputFolderEnvelope = serde_json::from_value(payload.clone())?;
        return Ok(Some(envelope.output));
    }
    Ok(Some(serde_json::from_value(payload.clone())?))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_create_project_payload_for_create_project_action() {
        let action = HubActionRequest {
            action_id: "create-project".to_string(),
            target_id: None,
            payload: Some(serde_json::json!({
                "project": {
                    "name": "Game",
                    "location": "E:/Projects",
                    "template": "renderable-empty",
                    "engineId": "engine"
                }
            })),
        }
        .parse()
        .expect("create-project should parse a project payload");

        let HubAction::CreateProject { payload } = action else {
            panic!("create-project should parse to the create-project action variant");
        };
        assert_eq!(payload.name, "Game");
        assert_eq!(payload.template, "renderable-empty");
        assert_eq!(payload.engine_id.as_deref(), Some("engine"));
    }

    #[test]
    fn parses_new_project_draft_payload_for_runtime_state_update() {
        let action = HubActionRequest {
            action_id: "update-new-project-draft".to_string(),
            target_id: None,
            payload: Some(serde_json::json!({
                "draft": {
                    "name": "Draft Game",
                    "location": "E:/Drafts",
                    "template": "renderable-empty",
                    "engineId": "engine"
                }
            })),
        }
        .parse()
        .expect("update-new-project-draft should parse a draft payload");

        let HubAction::UpdateNewProjectDraft { payload } = action else {
            panic!("update-new-project-draft should parse to the draft update variant");
        };
        assert_eq!(payload.name, "Draft Game");
        assert_eq!(payload.location, PathBuf::from("E:/Drafts"));
        assert_eq!(payload.template, "renderable-empty");
        assert_eq!(payload.engine_id.as_deref(), Some("engine"));
    }

    #[test]
    fn parses_search_projects_typed_payload_before_target_fallback() {
        let action = HubActionRequest {
            action_id: "search-projects".to_string(),
            target_id: Some("legacy query".to_string()),
            payload: Some(serde_json::json!({
                "query": "typed query"
            })),
        }
        .parse()
        .expect("search-projects should parse a typed search payload");

        let HubAction::SearchProjects { query } = action else {
            panic!("search-projects should parse to the search action variant");
        };
        assert_eq!(query, "typed query");
    }

    #[test]
    fn parses_browse_settings_folder_payload_for_folder_action() {
        let action = HubActionRequest {
            action_id: "browse-settings-folder".to_string(),
            target_id: None,
            payload: Some(serde_json::json!({
                "folder": {
                    "field": "defaultProjectDir",
                    "initialDir": "E:/Drafts",
                    "settings": {
                        "defaultProjectDir": "E:/Projects"
                    }
                }
            })),
        }
        .parse()
        .expect("browse-settings-folder should parse a folder payload");

        let HubAction::BrowseSettingsFolder { payload, .. } = action else {
            panic!("browse-settings-folder should parse to the browse folder action variant");
        };
        let payload = payload.expect("folder payload should be present");
        assert_eq!(payload.field.as_deref(), Some("defaultProjectDir"));
        assert_eq!(payload.initial_dir, Some(PathBuf::from("E:/Drafts")));
        assert!(payload.settings.is_some());
    }

    #[test]
    fn parses_project_target_payload_for_background_project_actions() {
        let action = HubActionRequest {
            action_id: "package-project".to_string(),
            target_id: Some("fallback-project".to_string()),
            payload: Some(serde_json::json!({
                "project": {
                    "projectId": "target-project",
                    "projectPath": "E:/Projects/Target"
                }
            })),
        }
        .parse()
        .expect("package-project should parse a typed project target payload");

        let HubAction::PackageProject { target_id, payload } = action else {
            panic!("package-project should parse to the package action variant");
        };
        assert_eq!(target_id.as_deref(), Some("fallback-project"));
        let payload = payload.expect("project target payload should be present");
        assert_eq!(payload.project_id.as_deref(), Some("target-project"));
        assert_eq!(
            payload.project_path,
            Some(PathBuf::from("E:/Projects/Target"))
        );
    }

    #[test]
    fn parses_cancel_delete_project_target_payload() {
        let action = HubActionRequest {
            action_id: "cancel-delete".to_string(),
            target_id: Some("fallback-project".to_string()),
            payload: Some(serde_json::json!({
                "project": {
                    "projectId": "target-project",
                    "projectPath": "E:/Projects/Target"
                }
            })),
        }
        .parse()
        .expect("cancel-delete should parse a typed project target payload");

        let HubAction::CancelDelete { target_id, payload } = action else {
            panic!("cancel-delete should parse to the cancel delete action variant");
        };
        assert_eq!(target_id.as_deref(), Some("fallback-project"));
        let payload = payload.expect("project target payload should be present");
        assert_eq!(payload.project_id.as_deref(), Some("target-project"));
        assert_eq!(
            payload.project_path,
            Some(PathBuf::from("E:/Projects/Target"))
        );
    }

    #[test]
    fn parses_open_output_folder_wrapped_payload_for_output_action() {
        let action = HubActionRequest {
            action_id: "open-output-folder".to_string(),
            target_id: None,
            payload: Some(serde_json::json!({
                "output": {
                    "historyId": "123:package-project:Game"
                }
            })),
        }
        .parse()
        .expect("open-output-folder should parse an output payload");

        let HubAction::OpenOutputFolder { payload, .. } = action else {
            panic!("open-output-folder should parse to the open-output action variant");
        };
        assert_eq!(
            payload
                .expect("output payload should be present")
                .history_id
                .as_deref(),
            Some("123:package-project:Game")
        );
    }

    #[test]
    fn unknown_action_is_rejected_before_runtime_routing() {
        let error = HubActionRequest {
            action_id: "upload-to-cloud".to_string(),
            target_id: None,
            payload: None,
        }
        .parse()
        .expect_err("unknown actions should not reach runtime routing");

        assert_eq!(error.to_string(), "Unknown Hub action: upload-to-cloud");
    }
}
