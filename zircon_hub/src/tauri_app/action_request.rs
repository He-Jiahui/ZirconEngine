use std::path::PathBuf;

use serde::de::DeserializeOwned;
use serde::Deserialize;
use serde_json::Value;

use crate::projects::project_template_catalog;
use crate::{
    error::HubError,
    state::{
        DeliveryMessageId, HubMessage, HubMessageId, LearnMessageId, ProjectMessageId,
        SettingsMessageId, ShellMessageId,
    },
};

use super::action_id::HubActionId;
use super::view_model::{HubSettingsActionPayload, HubSettingsPayload};

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
    UpdateSettingsDraft {
        payload: HubSettingsPayload,
    },
    SaveSettings {
        payload: Option<HubSettingsPayload>,
    },
    DiscardSettingsDraft,
    RestoreDefaultSettings,
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
#[serde(deny_unknown_fields)]
#[serde(rename_all = "camelCase")]
pub(crate) struct SearchProjectsPayload {
    #[serde(default)]
    pub query: String,
}

#[derive(Debug, Clone, Deserialize)]
#[serde(deny_unknown_fields)]
#[serde(rename_all = "camelCase")]
pub(crate) struct NewProjectDraftActionPayload {
    #[serde(alias = "projectName")]
    pub name: String,
    pub location: PathBuf,
    pub template: String,
    pub engine_id: Option<String>,
}

#[derive(Debug, Clone, Deserialize)]
#[serde(deny_unknown_fields)]
#[serde(rename_all = "camelCase")]
pub(crate) struct CreateProjectActionPayload {
    #[serde(alias = "projectName")]
    pub name: String,
    pub location: PathBuf,
    pub template: String,
    pub engine_id: Option<String>,
}

#[derive(Debug, Clone, Deserialize)]
#[serde(deny_unknown_fields)]
#[serde(rename_all = "camelCase")]
pub(crate) struct ImportProjectActionPayload {
    pub path: Option<PathBuf>,
    pub folder: Option<PathBuf>,
    pub engine_id: Option<String>,
}

#[derive(Debug, Clone, Deserialize)]
#[serde(deny_unknown_fields)]
#[serde(rename_all = "camelCase")]
pub(crate) struct ProjectTargetActionPayload {
    #[serde(alias = "id")]
    pub project_id: Option<String>,
    #[serde(alias = "path")]
    pub project_path: Option<PathBuf>,
}

#[derive(Debug, Clone, Deserialize)]
#[serde(deny_unknown_fields)]
#[serde(rename_all = "camelCase")]
pub(crate) struct BrowseSettingsFolderPayload {
    pub field: Option<String>,
    pub initial_dir: Option<PathBuf>,
    pub settings: Option<HubSettingsPayload>,
}

#[derive(Debug, Clone, Deserialize)]
#[serde(deny_unknown_fields)]
#[serde(rename_all = "camelCase")]
pub(crate) struct OpenResourcePayload {
    pub resource_id: Option<String>,
    pub path: Option<PathBuf>,
}

#[derive(Debug, Clone, Deserialize)]
#[serde(deny_unknown_fields)]
#[serde(rename_all = "camelCase")]
pub(crate) struct OpenOutputFolderPayload {
    pub path: Option<PathBuf>,
    pub output_dir: Option<PathBuf>,
    pub history_id: Option<String>,
}

impl HubActionRequest {
    pub(crate) fn action(&self) -> Result<HubActionId, HubError> {
        HubActionId::from_str(&self.action_id)
            .ok_or_else(|| HubError::message(format!("Unknown Hub action: {}", self.action_id)))
    }

    #[allow(dead_code)]
    pub(crate) fn parse(&self) -> Result<HubAction, HubError> {
        self.parse_as(self.action()?)
    }

    pub(in crate::tauri_app) fn parse_as(
        &self,
        action: HubActionId,
    ) -> Result<HubAction, HubError> {
        match action {
            HubActionId::ShowPage => Ok(HubAction::ShowPage {
                target_id: self.required_target()?.to_string(),
            }),
            HubActionId::ShowProjectSubpage => Ok(HubAction::ShowProjectSubpage {
                target_id: self.required_target()?.to_string(),
            }),
            HubActionId::SearchProjects => Ok(HubAction::SearchProjects {
                query: parse_payload::<SearchProjectsPayload>(action, self.payload.as_ref())?.query,
            }),
            HubActionId::SetProjectFilter => Ok(HubAction::SetProjectFilter {
                target_id: self.required_target()?.to_string(),
            }),
            HubActionId::SetProjectSort => Ok(HubAction::SetProjectSort {
                target_id: self.required_target()?.to_string(),
            }),
            HubActionId::SetProjectViewMode => Ok(HubAction::SetProjectViewMode {
                target_id: self.required_target()?.to_string(),
            }),
            HubActionId::SelectProject => Ok(HubAction::SelectProject {
                target_id: self.required_target()?.to_string(),
            }),
            HubActionId::OpenProjectDetail => Ok(HubAction::OpenProjectDetail {
                target_id: self.required_target()?.to_string(),
            }),
            HubActionId::ViewAllProjects => Ok(HubAction::ViewAllProjects),
            HubActionId::NewProject => Ok(HubAction::NewProject),
            HubActionId::UpdateNewProjectDraft => Ok(HubAction::UpdateNewProjectDraft {
                payload: parse_payload(action, self.payload.as_ref())?,
            }),
            HubActionId::SelectEngine => Ok(HubAction::SelectEngine {
                target_id: self.required_target()?.to_string(),
            }),
            HubActionId::UpdateSettingsDraft => Ok(HubAction::UpdateSettingsDraft {
                payload: parse_payload::<HubSettingsActionPayload>(action, self.payload.as_ref())?
                    .settings,
            }),
            HubActionId::SaveSettings => Ok(HubAction::SaveSettings {
                payload: parse_optional_payload::<HubSettingsActionPayload>(
                    action,
                    self.payload.as_ref(),
                )?
                .map(|payload| payload.settings),
            }),
            HubActionId::DiscardSettingsDraft => Ok(HubAction::DiscardSettingsDraft),
            HubActionId::RestoreDefaultSettings => Ok(HubAction::RestoreDefaultSettings),
            HubActionId::BrowseSettingsFolder => Ok(HubAction::BrowseSettingsFolder {
                target_id: self.trimmed_target(),
                payload: parse_optional_payload(action, self.payload.as_ref())?,
            }),
            HubActionId::CreateProject => Ok(HubAction::CreateProject {
                payload: parse_payload(action, self.payload.as_ref())?,
            }),
            HubActionId::ImportProject => Ok(HubAction::ImportProject {
                target_id: self.trimmed_target(),
                payload: parse_optional_payload(action, self.payload.as_ref())?,
            }),
            HubActionId::PinProject => Ok(HubAction::PinProject {
                target_id: self.trimmed_target(),
                payload: parse_optional_payload(action, self.payload.as_ref())?,
            }),
            HubActionId::UnpinProject => Ok(HubAction::UnpinProject {
                target_id: self.trimmed_target(),
                payload: parse_optional_payload(action, self.payload.as_ref())?,
            }),
            HubActionId::RemoveFromHub => Ok(HubAction::RemoveFromHub {
                target_id: self.trimmed_target(),
                payload: parse_optional_payload(action, self.payload.as_ref())?,
            }),
            HubActionId::RequestDelete => Ok(HubAction::RequestDelete {
                target_id: self.trimmed_target(),
                payload: parse_optional_payload(action, self.payload.as_ref())?,
            }),
            HubActionId::CancelDelete => Ok(HubAction::CancelDelete {
                target_id: self.trimmed_target(),
                payload: parse_optional_payload(action, self.payload.as_ref())?,
            }),
            HubActionId::ConfirmDelete => Ok(HubAction::ConfirmDelete {
                target_id: self.trimmed_target(),
                payload: parse_optional_payload(action, self.payload.as_ref())?,
            }),
            HubActionId::OpenResource => Ok(HubAction::OpenResource {
                target_id: self.trimmed_target(),
                payload: parse_optional_payload(action, self.payload.as_ref())?,
            }),
            HubActionId::OpenOutputFolder => Ok(HubAction::OpenOutputFolder {
                target_id: self.trimmed_target(),
                payload: parse_optional_payload(action, self.payload.as_ref())?,
            }),
            HubActionId::BuildProject => Ok(HubAction::BuildProject {
                target_id: self.trimmed_target(),
                payload: parse_optional_payload(action, self.payload.as_ref())?,
            }),
            HubActionId::PackageProject => Ok(HubAction::PackageProject {
                target_id: self.trimmed_target(),
                payload: parse_optional_payload(action, self.payload.as_ref())?,
            }),
            HubActionId::InstallDevice => Ok(HubAction::InstallDevice {
                target_id: self.trimmed_target(),
                payload: parse_optional_payload(action, self.payload.as_ref())?,
            }),
            HubActionId::OpenEditor => Ok(HubAction::OpenEditor {
                target_id: self.trimmed_target(),
                payload: parse_optional_payload(action, self.payload.as_ref())?,
            }),
        }
    }

    pub(crate) fn project_target_payload(
        &self,
    ) -> Result<Option<ProjectTargetActionPayload>, HubError> {
        parse_optional_payload(self.action()?, self.payload.as_ref())
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

pub(crate) trait ValidatePayload {
    fn validate(&self) -> Result<(), HubError> {
        Ok(())
    }
}

fn parse_payload<T>(action: HubActionId, payload: Option<&Value>) -> Result<T, HubError>
where
    T: DeserializeOwned + ValidatePayload,
{
    let Some(payload) = payload else {
        return Err(HubError::status(
            HubMessage::with_params(
                HubMessageId::Shell(ShellMessageId::PayloadRequiredForAction),
                [action.as_str()],
            ),
            Some(HubMessage::new(HubMessageId::Shell(
                ShellMessageId::ReviewActionPayload,
            ))),
        ));
    };
    deserialize_payload(action, payload)
}

fn parse_optional_payload<T>(
    action: HubActionId,
    payload: Option<&Value>,
) -> Result<Option<T>, HubError>
where
    T: DeserializeOwned + ValidatePayload,
{
    let Some(payload) = payload else {
        return Ok(None);
    };
    deserialize_payload(action, payload).map(Some)
}

fn deserialize_payload<T>(action: HubActionId, payload: &Value) -> Result<T, HubError>
where
    T: DeserializeOwned + ValidatePayload,
{
    let parsed: T = serde_json::from_value(payload.clone()).map_err(|error| {
        HubError::status(
            HubMessage::with_params(
                HubMessageId::Shell(ShellMessageId::InvalidPayloadForAction),
                [action.as_str().to_string(), error.to_string()],
            ),
            Some(HubMessage::new(HubMessageId::Shell(
                ShellMessageId::ReviewActionPayload,
            ))),
        )
    })?;
    parsed.validate()?;
    Ok(parsed)
}

impl ValidatePayload for SearchProjectsPayload {}

impl ValidatePayload for NewProjectDraftActionPayload {
    fn validate(&self) -> Result<(), HubError> {
        validate_project_creation_payload(&self.name, &self.location, &self.template)
    }
}

impl ValidatePayload for CreateProjectActionPayload {
    fn validate(&self) -> Result<(), HubError> {
        validate_project_creation_payload(&self.name, &self.location, &self.template)
    }
}

impl ValidatePayload for ImportProjectActionPayload {
    fn validate(&self) -> Result<(), HubError> {
        validate_optional_absolute_path(self.path.as_ref(), "Import path")?;
        validate_optional_absolute_path(self.folder.as_ref(), "Import folder")
    }
}

impl ValidatePayload for ProjectTargetActionPayload {
    fn validate(&self) -> Result<(), HubError> {
        validate_optional_absolute_path(self.project_path.as_ref(), "Project path")
    }
}

impl ValidatePayload for BrowseSettingsFolderPayload {
    fn validate(&self) -> Result<(), HubError> {
        if let Some(field) = self.field.as_deref() {
            validate_settings_folder_field(field)?;
        }
        validate_optional_absolute_path(self.initial_dir.as_ref(), "Initial directory")
    }
}

impl ValidatePayload for OpenResourcePayload {
    fn validate(&self) -> Result<(), HubError> {
        validate_optional_absolute_path(self.path.as_ref(), "Resource path")
    }
}

impl ValidatePayload for OpenOutputFolderPayload {
    fn validate(&self) -> Result<(), HubError> {
        validate_optional_absolute_path(self.path.as_ref(), "Output path")?;
        validate_optional_absolute_path(self.output_dir.as_ref(), "Output directory")
    }
}

impl ValidatePayload for HubSettingsActionPayload {}

fn validate_project_creation_payload(
    name: &str,
    location: &PathBuf,
    template: &str,
) -> Result<(), HubError> {
    if name.trim().is_empty() {
        return Err(HubError::status(
            HubMessage::new(HubMessageId::Settings(
                SettingsMessageId::ProjectNameRequired,
            )),
            Some(HubMessage::new(HubMessageId::Shell(
                ShellMessageId::ReviewActionPayload,
            ))),
        ));
    }
    validate_absolute_path(location, "Project location")?;
    if !project_template_catalog()
        .iter()
        .any(|candidate| candidate.id == template.trim())
    {
        return Err(HubError::status(
            HubMessage::with_params(
                HubMessageId::Project(ProjectMessageId::UnknownTemplate),
                [template],
            ),
            Some(HubMessage::new(HubMessageId::Shell(
                ShellMessageId::ReviewActionPayload,
            ))),
        ));
    }
    Ok(())
}

fn validate_optional_absolute_path(path: Option<&PathBuf>, label: &str) -> Result<(), HubError> {
    if let Some(path) = path.filter(|path| !path.as_os_str().is_empty()) {
        validate_absolute_path(path, label)?;
    }
    Ok(())
}

fn validate_absolute_path(path: &PathBuf, label: &str) -> Result<(), HubError> {
    if !path.is_absolute() {
        return Err(HubError::status(
            HubMessage::with_params(
                absolute_path_message_id(label),
                [path.to_string_lossy().into_owned()],
            ),
            Some(HubMessage::new(HubMessageId::Shell(
                ShellMessageId::ReviewActionPayload,
            ))),
        ));
    }
    Ok(())
}

fn validate_settings_folder_field(field: &str) -> Result<(), HubError> {
    match field.trim() {
        "defaultProjectDir"
        | "default-project-dir"
        | "project-dir"
        | "defaultSourceDir"
        | "default-source-dir"
        | "source-dir"
        | "defaultBuildOutputDir"
        | "default-build-output-dir"
        | "build-output"
        | "defaultDeviceInstallDir"
        | "default-device-install-dir"
        | "device-install" => Ok(()),
        _ => Err(HubError::status(
            HubMessage::with_params(
                HubMessageId::Settings(SettingsMessageId::UnknownFolderField),
                [field],
            ),
            Some(HubMessage::new(HubMessageId::Shell(
                ShellMessageId::ReviewActionPayload,
            ))),
        )),
    }
}

fn absolute_path_message_id(label: &str) -> HubMessageId {
    match label {
        "Project location" => HubMessageId::Project(ProjectMessageId::LocationMustBeAbsolute),
        "Project path" => HubMessageId::Project(ProjectMessageId::PathMustBeAbsolute),
        "Import path" => HubMessageId::Project(ProjectMessageId::ImportPathMustBeAbsolute),
        "Import folder" => HubMessageId::Project(ProjectMessageId::ImportFolderMustBeAbsolute),
        "Initial directory" => {
            HubMessageId::Settings(SettingsMessageId::InitialDirectoryMustBeAbsolute)
        }
        "Resource path" => HubMessageId::Learn(LearnMessageId::ResourcePathMustBeAbsolute),
        "Output path" => HubMessageId::Delivery(DeliveryMessageId::OutputPathMustBeAbsolute),
        "Output directory" => {
            HubMessageId::Delivery(DeliveryMessageId::OutputDirectoryMustBeAbsolute)
        }
        _ => HubMessageId::Project(ProjectMessageId::PathMustBeAbsolute),
    }
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
                "name": "Game",
                "location": "E:/Projects",
                "template": "renderable-empty",
                "engineId": "engine"
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
                "name": "Draft Game",
                "location": "E:/Drafts",
                "template": "renderable-empty",
                "engineId": "engine"
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
    fn parses_search_projects_typed_payload() {
        let action = HubActionRequest {
            action_id: "search-projects".to_string(),
            target_id: Some("archived query".to_string()),
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
                "field": "defaultProjectDir",
                "initialDir": "E:/Drafts",
                "settings": {
                    "defaultProjectDir": "E:/Projects"
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
    fn parses_update_settings_draft_payload_for_draft_action() {
        let action = HubActionRequest {
            action_id: "update-settings-draft".to_string(),
            target_id: None,
            payload: Some(serde_json::json!({
                "settings": {
                    "pythonPath": "",
                    "language": "Chinese"
                }
            })),
        }
        .parse()
        .expect("update-settings-draft should parse a settings payload");

        let HubAction::UpdateSettingsDraft { payload } = action else {
            panic!("update-settings-draft should parse to the settings draft action variant");
        };
        assert_eq!(payload.python_path.as_deref(), Some(""));
        assert_eq!(payload.language.as_deref(), Some("Chinese"));
    }

    #[test]
    fn parses_project_target_payload_for_background_project_actions() {
        let action = HubActionRequest {
            action_id: "package-project".to_string(),
            target_id: Some("fallback-project".to_string()),
            payload: Some(serde_json::json!({
                "projectId": "target-project",
                "projectPath": "E:/Projects/Target"
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
                "projectId": "target-project",
                "projectPath": "E:/Projects/Target"
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
    fn parses_open_output_folder_flat_payload_for_output_action() {
        let action = HubActionRequest {
            action_id: "open-output-folder".to_string(),
            target_id: None,
            payload: Some(serde_json::json!({
                "historyId": "123:package-project:Game"
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
    fn create_project_rejects_empty_name_with_recoverable_message() {
        let error = HubActionRequest {
            action_id: "create-project".to_string(),
            target_id: None,
            payload: Some(serde_json::json!({
                "name": "  ",
                "location": "E:/Projects",
                "template": "renderable-empty"
            })),
        }
        .parse()
        .expect_err("empty project names should be rejected");

        assert_eq!(error.to_string(), "Project name must not be empty");
    }

    #[test]
    fn create_project_rejects_relative_location() {
        let error = HubActionRequest {
            action_id: "create-project".to_string(),
            target_id: None,
            payload: Some(serde_json::json!({
                "name": "Game",
                "location": "projects/Game",
                "template": "renderable-empty"
            })),
        }
        .parse()
        .expect_err("relative project locations should be rejected");

        assert_eq!(
            error.to_string(),
            "Project location must be an absolute path: projects/Game"
        );
    }

    #[test]
    fn create_project_rejects_unknown_template_id() {
        let disabled_template = HubActionRequest {
            action_id: "create-project".to_string(),
            target_id: None,
            payload: Some(serde_json::json!({
                "name": "Game",
                "location": "E:/Projects",
                "template": "3d-scene"
            })),
        }
        .parse()
        .expect("disabled catalog templates should reach runtime as coming soon");
        assert!(matches!(
            disabled_template,
            HubAction::CreateProject { payload } if payload.template == "3d-scene"
        ));

        let error = HubActionRequest {
            action_id: "create-project".to_string(),
            target_id: None,
            payload: Some(serde_json::json!({
                "name": "Game",
                "location": "E:/Projects",
                "template": "not-a-template"
            })),
        }
        .parse()
        .expect_err("unknown templates should be rejected before runtime creation");

        assert_eq!(
            error.to_string(),
            "Unknown project template: not-a-template"
        );
    }

    #[test]
    fn project_target_envelope_payload_is_rejected_after_hard_cutover() {
        let error = HubActionRequest {
            action_id: "package-project".to_string(),
            target_id: None,
            payload: Some(serde_json::json!({
                "project": {
                    "projectId": "target-project"
                }
            })),
        }
        .parse()
        .expect_err("project target envelopes should be removed after hard cutover");

        assert!(error
            .to_string()
            .contains("Invalid payload for Hub action package-project"));
    }

    #[test]
    fn missing_required_payload_is_rejected_with_action_id() {
        let error = HubActionRequest {
            action_id: "create-project".to_string(),
            target_id: None,
            payload: None,
        }
        .parse()
        .expect_err("required payloads should include the action id in errors");

        assert_eq!(
            error.to_string(),
            "Payload is required for Hub action: create-project"
        );
    }

    #[test]
    fn settings_payload_requires_settings_wrapper() {
        let error = HubActionRequest {
            action_id: "update-settings-draft".to_string(),
            target_id: None,
            payload: Some(serde_json::json!({
                "pythonPath": "python"
            })),
        }
        .parse()
        .expect_err("settings actions should require a settings wrapper");

        assert!(error
            .to_string()
            .contains("Invalid payload for Hub action update-settings-draft"));
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
