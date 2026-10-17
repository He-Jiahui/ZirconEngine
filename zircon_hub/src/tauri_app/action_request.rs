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

/// Admission limits for WebView-controlled JSON before it is cloned into a
/// typed payload.  The walk below is iterative so a hostile nesting shape
/// cannot consume the native thread's call stack.
const MAX_PAYLOAD_BYTES: usize = 256 * 1024;
const MAX_PAYLOAD_DEPTH: usize = 32;
const MAX_PAYLOAD_NODES: usize = 4 * 1024;
const MAX_PAYLOAD_STRING_BYTES: usize = 64 * 1024;

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
    CancelBackgroundTask {
        task_id: u64,
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
    /// A Hub-generated action/source-build receipt id. The native side uses
    /// this only as a selector and resolves the recorded output path itself.
    pub receipt_id: Option<String>,
    /// A narrow, server-defined capability for a configured output root.
    pub capability: Option<OpenOutputFolderCapability>,
    /// Required only by `source-engine-output`; this is an engine registry id,
    /// never a filesystem path.
    pub engine_id: Option<String>,
}

#[derive(Debug, Clone, Copy, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "kebab-case")]
pub(crate) enum OpenOutputFolderCapability {
    DefaultBuildOutput,
    DefaultDeviceInstall,
    SourceEngineOutput,
}

impl HubActionRequest {
    pub(crate) fn action(&self) -> Result<HubActionId, HubError> {
        HubActionId::from_str(&self.action_id)
            .ok_or_else(|| HubError::message(format!("Unknown Hub action: {}", self.action_id)))
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
            HubActionId::OpenOutputFolder => {
                if self.trimmed_target().is_some() {
                    return Err(invalid_output_folder_payload(
                        "targetId is not accepted; use receiptId or capability",
                    ));
                }
                Ok(HubAction::OpenOutputFolder {
                    payload: parse_optional_payload(action, self.payload.as_ref())?,
                })
            }
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
            HubActionId::CancelBackgroundTask => Ok(HubAction::CancelBackgroundTask {
                task_id: self.required_task_id()?,
            }),
        }
    }

    #[cfg(test)]
    pub(in crate::tauri_app) fn parse(&self) -> Result<HubAction, HubError> {
        let action_id = self.action()?;
        self.parse_as(action_id)
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

    fn required_task_id(&self) -> Result<u64, HubError> {
        let target = self.required_target()?;
        let task_id = target.parse::<u64>().map_err(|_| {
            HubError::message(format!(
                "Task id must be an unsigned integer for Hub action {}: {target}",
                self.action_id
            ))
        })?;
        if task_id == 0 {
            return Err(HubError::message(format!(
                "Task id must be non-zero for Hub action {}",
                self.action_id
            )));
        }
        Ok(task_id)
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
    validate_payload_budget(action, payload)?;
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

fn validate_payload_budget(action: HubActionId, payload: &Value) -> Result<(), HubError> {
    let mut stack = vec![(payload, 0usize)];
    let mut nodes = 0usize;
    let mut bytes = 0usize;

    while let Some((value, depth)) = stack.pop() {
        if depth > MAX_PAYLOAD_DEPTH {
            return Err(payload_budget_error(action, "nesting depth"));
        }
        nodes = nodes.saturating_add(1);
        if nodes > MAX_PAYLOAD_NODES {
            return Err(payload_budget_error(action, "node count"));
        }

        let add_bytes = |bytes: &mut usize, amount: usize| {
            *bytes = bytes.saturating_add(amount);
            *bytes <= MAX_PAYLOAD_BYTES
        };

        let within_budget = match value {
            Value::Null => add_bytes(&mut bytes, 4),
            Value::Bool(value) => add_bytes(&mut bytes, if *value { 4 } else { 5 }),
            Value::Number(number) => add_bytes(&mut bytes, number.to_string().len()),
            Value::String(text) => {
                text.len() <= MAX_PAYLOAD_STRING_BYTES
                    && add_bytes(&mut bytes, text.len().saturating_add(2))
            }
            Value::Array(values) => {
                if values.len() > MAX_PAYLOAD_NODES {
                    return Err(payload_budget_error(action, "array item count"));
                }
                for child in values {
                    stack.push((child, depth.saturating_add(1)));
                }
                add_bytes(&mut bytes, values.len().saturating_add(2))
            }
            Value::Object(values) => {
                if values.len() > MAX_PAYLOAD_NODES {
                    return Err(payload_budget_error(action, "object member count"));
                }
                let mut object_bytes = 2usize;
                for (key, child) in values {
                    object_bytes = object_bytes.saturating_add(key.len()).saturating_add(4);
                    stack.push((child, depth.saturating_add(1)));
                }
                add_bytes(&mut bytes, object_bytes)
            }
        };

        if !within_budget {
            return Err(payload_budget_error(action, "serialized bytes"));
        }
    }

    Ok(())
}

fn payload_budget_error(action: HubActionId, dimension: &str) -> HubError {
    HubError::status(
        HubMessage::with_params(
            HubMessageId::Shell(ShellMessageId::InvalidPayloadForAction),
            [
                action.as_str().to_string(),
                format!("payload exceeds the Hub budget ({dimension})"),
            ],
        ),
        Some(HubMessage::new(HubMessageId::Shell(
            ShellMessageId::ReviewActionPayload,
        ))),
    )
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
        let receipt_id = self
            .receipt_id
            .as_deref()
            .map(str::trim)
            .filter(|value| !value.is_empty());
        let has_capability = self.capability.is_some();
        if receipt_id.is_some() == has_capability {
            return Err(invalid_output_folder_payload(
                "provide exactly one Hub receiptId or capability",
            ));
        }
        if let Some(receipt_id) = receipt_id {
            if receipt_id.len() > 512 {
                return Err(invalid_output_folder_payload(
                    "receiptId exceeds the 512-byte limit",
                ));
            }
            if self.engine_id.is_some() {
                return Err(invalid_output_folder_payload(
                    "engineId is only valid with source-engine-output",
                ));
            }
        }
        match self.capability {
            Some(OpenOutputFolderCapability::SourceEngineOutput) => {
                let Some(engine_id) = self
                    .engine_id
                    .as_deref()
                    .map(str::trim)
                    .filter(|value| !value.is_empty())
                else {
                    return Err(invalid_output_folder_payload(
                        "source-engine-output requires engineId",
                    ));
                };
                if engine_id.len() > 256 {
                    return Err(invalid_output_folder_payload(
                        "engineId exceeds the 256-byte limit",
                    ));
                }
            }
            Some(
                OpenOutputFolderCapability::DefaultBuildOutput
                | OpenOutputFolderCapability::DefaultDeviceInstall,
            ) => {
                if self.engine_id.is_some() {
                    return Err(invalid_output_folder_payload(
                        "engineId is only valid with source-engine-output",
                    ));
                }
            }
            None => {}
        }
        Ok(())
    }
}

fn invalid_output_folder_payload(detail: &str) -> HubError {
    HubError::status(
        HubMessage::with_params(
            HubMessageId::Shell(ShellMessageId::InvalidPayloadForAction),
            ["open-output-folder", detail],
        ),
        Some(HubMessage::new(HubMessageId::Shell(
            ShellMessageId::ReviewActionPayload,
        ))),
    )
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
#[path = "tests/action_request.rs"]
mod tests;
