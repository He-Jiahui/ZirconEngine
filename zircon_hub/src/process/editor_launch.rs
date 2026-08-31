use std::path::{Path, PathBuf};
use std::process::{Child, Command};
use std::sync::OnceLock;

use zircon_runtime_interface::hub_protocol::{HubSessionToken, HUB_PROTOCOL_VERSION_V1};
use zircon_runtime_interface::project::{
    ProjectActivationOperationId, ProjectActivationOperationIdGenerator, ProjectLaunchInstanceId,
    ProjectLaunchIntent, ProjectLaunchProfile, ProjectLaunchSource,
};

use crate::error::HubError;
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
            request.template.pack_id(),
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

    pub fn from_preferred_engine(
        configured_engine_root: impl AsRef<Path>,
        request: EditorLaunchRequest,
    ) -> Result<Self, HubError> {
        let configured = configured_engine_root
            .as_ref()
            .join(platform_executable_name("zircon_editor"));
        let executable = sibling_editor_executable().unwrap_or(configured);
        Self::new(executable, request)
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

pub fn launch_editor(command: &EditorLaunchCommand) -> Result<Child, HubError> {
    Ok(Command::new(&command.executable)
        .args(&command.args)
        .spawn()?)
}

fn platform_executable_name(stem: &str) -> String {
    if cfg!(target_os = "windows") {
        format!("{stem}.exe")
    } else {
        stem.to_string()
    }
}

pub fn preferred_editor_executable(configured_engine_root: impl AsRef<Path>) -> PathBuf {
    sibling_editor_executable().unwrap_or_else(|| {
        configured_engine_root
            .as_ref()
            .join(platform_executable_name("zircon_editor"))
    })
}

pub fn preferred_editor_executable_exists(configured_engine_root: impl AsRef<Path>) -> bool {
    preferred_editor_executable(configured_engine_root).is_file()
}

fn sibling_editor_executable() -> Option<PathBuf> {
    let executable = std::env::current_exe().ok()?;
    let sibling = executable
        .parent()?
        .join(platform_executable_name("zircon_editor"));
    sibling.is_file().then_some(sibling)
}

#[cfg(test)]
mod tests {
    use std::str::FromStr;

    use super::*;
    use crate::projects::{CreateProjectRequest, ProjectTemplate};

    #[test]
    fn editor_launch_args_preserve_project_paths_with_spaces() {
        let request = EditorLaunchRequest::open_project("E:/Projects/My Game").unwrap();
        let operation_id = request.intent().operation_id();
        let command =
            EditorLaunchCommand::new("E:/Engine/ZirconEngine/zircon_editor.exe", request).unwrap();

        assert_eq!(command.args[0], PROJECT_LAUNCH_INTENT_ARGUMENT);
        let transmitted = serde_json::from_str::<ProjectLaunchIntent>(&command.args[1]).unwrap();
        assert_eq!(transmitted.operation_id(), operation_id);
        assert_eq!(transmitted.source(), ProjectLaunchSource::Hub);
        assert!(matches!(
            transmitted.target(),
            zircon_runtime_interface::project::ProjectLaunchTarget::OpenExisting { requested_path }
                if requested_path == Path::new("E:/Projects/My Game")
        ));
    }

    #[test]
    fn editor_create_args_match_editor_startup_contract() {
        let command = EditorLaunchCommand::new(
            "zircon_editor.exe",
            EditorLaunchRequest::create_project(CreateProjectRequest::new(
                "My Game",
                "E:/Projects",
                ProjectTemplate::RenderableEmpty,
            ))
            .unwrap(),
        )
        .unwrap();

        let transmitted = serde_json::from_str::<ProjectLaunchIntent>(&command.args[1]).unwrap();
        assert!(matches!(
            transmitted.target(),
            zircon_runtime_interface::project::ProjectLaunchTarget::CreateProject {
                project_name,
                location,
                template: zircon_runtime_interface::project::ProjectTemplateId::RenderableEmpty,
            } if project_name == "My Game" && location == Path::new("E:/Projects")
        ));
    }

    #[test]
    fn hub_handshake_arguments_use_the_typed_token_and_protocol_v1() {
        let token = HubSessionToken::from_str("0d9a5890-0e44-4e2a-b77e-3e5d4fdf1e52")
            .expect("parse deterministic test token");
        let request = EditorLaunchRequest::open_project("E:/Projects/My Game").unwrap();
        let operation_id = request.intent().operation_id();
        let command = EditorLaunchCommand::new("zircon_editor.exe", request)
            .unwrap()
            .with_hub_handshake(token);

        assert_eq!(
            command.args[2..]
                .iter()
                .map(String::as_str)
                .collect::<Vec<_>>(),
            vec![
                "--hub-session",
                "0d9a5890-0e44-4e2a-b77e-3e5d4fdf1e52",
                "--hub-protocol",
                "1",
            ]
        );
        let transmitted = serde_json::from_str::<ProjectLaunchIntent>(&command.args[1]).unwrap();
        assert_eq!(transmitted.operation_id(), operation_id);
    }

    #[test]
    fn preferred_editor_executable_falls_back_to_configured_engine_root() {
        let executable = preferred_editor_executable("E:/configured/ZirconEngine");

        assert_eq!(
            executable,
            PathBuf::from("E:/configured/ZirconEngine")
                .join(platform_executable_name("zircon_editor"))
        );
    }

    #[test]
    fn preferred_editor_executable_reports_missing_configured_fallback() {
        let missing =
            std::env::temp_dir().join(format!("zircon_hub_missing_editor_{}", std::process::id()));

        assert!(!preferred_editor_executable_exists(missing));
    }
}
