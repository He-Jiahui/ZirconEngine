use std::path::{Path, PathBuf};
use std::process::Command;
use std::sync::OnceLock;

use zircon_runtime_interface::hub_protocol::{HubSessionToken, HUB_PROTOCOL_VERSION_V1};
use zircon_runtime_interface::project::{
    ProjectActivationOperationId, ProjectActivationOperationIdGenerator, ProjectLaunchInstanceId,
    ProjectLaunchIntent, ProjectLaunchProfile, ProjectLaunchSource,
};

use crate::error::HubError;
use crate::process::SupervisedChild;
use crate::projects::CreateProjectRequest;

const HUB_PROTOCOL_ARGUMENT: &str = "--hub-protocol";
const HUB_SESSION_ARGUMENT: &str = "--hub-session";
const PROJECT_LAUNCH_INTENT_ARGUMENT: &str = "--project-launch-intent";

static PROJECT_LAUNCH_OPERATION_IDS: OnceLock<ProjectActivationOperationIdGenerator> =
    OnceLock::new();

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum EditorLaunchRequest {
    Project(ProjectLaunchIntent),
}

impl EditorLaunchRequest {
    pub fn open_project(project_path: impl Into<PathBuf>) -> Result<Self, HubError> {
        Ok(Self::Project(ProjectLaunchIntent::open_existing(
            next_project_launch_operation_id()?,
            ProjectLaunchSource::Hub,
            ProjectLaunchProfile::Normal,
            project_path,
        )?))
    }

    pub fn create_project(request: CreateProjectRequest) -> Result<Self, HubError> {
        Ok(Self::Project(ProjectLaunchIntent::create_project(
            next_project_launch_operation_id()?,
            ProjectLaunchSource::Hub,
            ProjectLaunchProfile::Normal,
            request.project_name,
            request.location,
            request.template,
        )?))
    }

    pub fn intent(&self) -> &ProjectLaunchIntent {
        match self {
            Self::Project(intent) => intent,
        }
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct EditorLaunchCommand {
    pub executable: PathBuf,
    pub args: Vec<String>,
}

impl EditorLaunchCommand {
    pub fn new(
        executable: impl Into<PathBuf>,
        request: EditorLaunchRequest,
    ) -> Result<Self, HubError> {
        let args = vec![
            PROJECT_LAUNCH_INTENT_ARGUMENT.to_string(),
            serde_json::to_string(request.intent())?,
        ];
        Ok(Self {
            executable: executable.into(),
            args,
        })
    }

    pub fn from_staged_engine(
        engine_root: impl AsRef<Path>,
        request: EditorLaunchRequest,
    ) -> Result<Self, HubError> {
        Self::new(
            engine_root
                .as_ref()
                .join(platform_executable_name("zircon_editor")),
            request,
        )
    }

    pub fn command_line(&self) -> Vec<String> {
        std::iter::once(self.executable.to_string_lossy().into_owned())
            .chain(self.args.iter().cloned())
            .collect()
    }

    /// Adds the v1 Hub handshake arguments without changing the launch request itself.
    pub fn with_hub_handshake(mut self, session: HubSessionToken) -> Self {
        self.args.extend([
            HUB_SESSION_ARGUMENT.to_string(),
            session.to_string(),
            HUB_PROTOCOL_ARGUMENT.to_string(),
            HUB_PROTOCOL_VERSION_V1.to_string(),
        ]);
        self
    }
}

fn next_project_launch_operation_id() -> Result<ProjectActivationOperationId, HubError> {
    PROJECT_LAUNCH_OPERATION_IDS
        .get_or_init(|| ProjectActivationOperationIdGenerator::new(ProjectLaunchInstanceId::new()))
        .allocate()
        .ok_or_else(|| HubError::message("project launch operation sequence is exhausted"))
}

pub(crate) fn launch_editor(command: &EditorLaunchCommand) -> Result<SupervisedChild, HubError> {
    let mut process = Command::new(&command.executable);
    process.args(&command.args);
    SupervisedChild::spawn(&mut process, "Editor")
}

fn platform_executable_name(stem: &str) -> String {
    if cfg!(target_os = "windows") {
        format!("{stem}.exe")
    } else {
        stem.to_string()
    }
}

pub fn staged_editor_executable(configured_engine_root: impl AsRef<Path>) -> PathBuf {
    configured_engine_root
        .as_ref()
        .join(platform_executable_name("zircon_editor"))
}

pub fn staged_editor_executable_exists(configured_engine_root: impl AsRef<Path>) -> bool {
    staged_editor_executable(configured_engine_root).is_file()
}

#[cfg(test)]
#[path = "tests/editor_launch.rs"]
mod tests;
