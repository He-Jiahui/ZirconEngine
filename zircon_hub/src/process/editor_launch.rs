use std::path::{Path, PathBuf};
use std::process::{Child, Command};

use zircon_runtime_interface::hub_protocol::{HubSessionToken, HUB_PROTOCOL_VERSION_V1};

use crate::error::HubError;
use crate::projects::CreateProjectRequest;

const HUB_PROTOCOL_ARGUMENT: &str = "--hub-protocol";
const HUB_SESSION_ARGUMENT: &str = "--hub-session";

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum EditorLaunchRequest {
    OpenProject { project_path: PathBuf },
    CreateProject(CreateProjectRequest),
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct EditorLaunchCommand {
    pub executable: PathBuf,
    pub args: Vec<String>,
}

impl EditorLaunchCommand {
    pub fn new(executable: impl Into<PathBuf>, request: EditorLaunchRequest) -> Self {
        let mut args = Vec::new();
        match request {
            EditorLaunchRequest::OpenProject { project_path } => {
                args.push("--project".to_string());
                args.push(project_path.to_string_lossy().into_owned());
            }
            EditorLaunchRequest::CreateProject(request) => {
                args.push("--create-project".to_string());
                args.push("--project-name".to_string());
                args.push(request.project_name);
                args.push("--location".to_string());
                args.push(request.location.to_string_lossy().into_owned());
                args.push("--template".to_string());
                args.push(request.template.as_editor_arg().to_string());
            }
        }
        Self {
            executable: executable.into(),
            args,
        }
    }

    pub fn from_staged_engine(engine_root: impl AsRef<Path>, request: EditorLaunchRequest) -> Self {
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
    ) -> Self {
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
        let command = EditorLaunchCommand::new(
            "E:/Engine/ZirconEngine/zircon_editor.exe",
            EditorLaunchRequest::OpenProject {
                project_path: PathBuf::from("E:/Projects/My Game"),
            },
        );

        assert_eq!(command.args, vec!["--project", "E:/Projects/My Game"]);
    }

    #[test]
    fn editor_create_args_match_editor_startup_contract() {
        let command = EditorLaunchCommand::new(
            "zircon_editor.exe",
            EditorLaunchRequest::CreateProject(CreateProjectRequest::new(
                "My Game",
                "E:/Projects",
                ProjectTemplate::RenderableEmpty,
            )),
        );

        assert_eq!(
            command.args,
            vec![
                "--create-project",
                "--project-name",
                "My Game",
                "--location",
                "E:/Projects",
                "--template",
                "renderable-empty",
            ]
        );
    }

    #[test]
    fn hub_handshake_arguments_use_the_typed_token_and_protocol_v1() {
        let token = HubSessionToken::from_str("0d9a5890-0e44-4e2a-b77e-3e5d4fdf1e52")
            .expect("parse deterministic test token");
        let command = EditorLaunchCommand::new(
            "zircon_editor.exe",
            EditorLaunchRequest::OpenProject {
                project_path: PathBuf::from("E:/Projects/My Game"),
            },
        )
        .with_hub_handshake(token);

        assert_eq!(
            command.args,
            vec![
                "--project",
                "E:/Projects/My Game",
                "--hub-session",
                "0d9a5890-0e44-4e2a-b77e-3e5d4fdf1e52",
                "--hub-protocol",
                "1",
            ]
        );
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
