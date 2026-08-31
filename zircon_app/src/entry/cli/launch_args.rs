use std::{error::Error, path::Path, str::FromStr, sync::OnceLock};

use zircon_editor::{
    core::commandlet::{parse_commandlet_args, CommandletReport, CommandletRequest},
    EditorGuiStartupRequest,
};
use zircon_runtime::{
    asset::{project::ProjectPaths, AssetUri},
    diagnostic_log::DiagnosticLogFilterConfig,
};
use zircon_runtime_interface::{
    hub_protocol::{HubSessionToken, HUB_PROTOCOL_VERSION_V1},
    project::{
        ProjectActivationOperationId, ProjectActivationOperationIdGenerator,
        ProjectLaunchInstanceId, ProjectLaunchIntent, ProjectLaunchProfile, ProjectLaunchSource,
        ProjectLaunchTarget, ProjectTemplateId,
    },
};

use crate::entry::cli::parse_diagnostic_log_startup_args;

static CLI_PROJECT_LAUNCH_OPERATION_IDS: OnceLock<ProjectActivationOperationIdGenerator> =
    OnceLock::new();

/// The complete typed routing decision for the editor executable. Diagnostic arguments are
/// parsed first so process logging is initialized before the selected route can load a host.
#[derive(Clone, Debug)]
pub(crate) struct EditorLaunchArgs {
    diagnostic_filter: DiagnosticLogFilterConfig,
    remaining_args: Vec<String>,
}

/// A launch route is chosen once and then consumed by the process host without reparsing argv.
#[derive(Clone, Debug)]
pub(crate) enum EditorLaunchRoute {
    Help,
    Commandlet(CommandletRequest),
    CommandletRejected(CommandletReport),
    Gui(EditorGuiLaunchIntent),
}

impl EditorLaunchArgs {
    pub(crate) fn parse<I, S>(args: I) -> Result<Self, Box<dyn Error>>
    where
        I: IntoIterator<Item = S>,
        S: Into<String>,
    {
        let diagnostic_args = parse_diagnostic_log_startup_args(args)?;
        Ok(Self {
            diagnostic_filter: diagnostic_args.filter,
            remaining_args: diagnostic_args.remaining_args,
        })
    }

    pub(crate) fn diagnostic_filter(&self) -> &DiagnosticLogFilterConfig {
        &self.diagnostic_filter
    }

    pub(crate) fn route(self) -> Result<EditorLaunchRoute, Box<dyn Error>> {
        EditorLaunchRoute::parse(self.remaining_args)
    }
}

impl EditorLaunchRoute {
    fn parse(args: Vec<String>) -> Result<Self, Box<dyn Error>> {
        if args
            .iter()
            .any(|arg| matches!(arg.as_str(), "--help" | "-h"))
        {
            return Ok(Self::Help);
        }

        match parse_commandlet_args(args.iter().cloned()) {
            Ok(Some(request)) => Ok(Self::Commandlet(request)),
            Err(report) => Ok(Self::CommandletRejected(report)),
            Ok(None) => EditorGuiStartupRequestArgs::parse_intent(args.clone())
                .map(Self::Gui)
                .map_err(|error| editor_startup_argument_error(&args, error)),
        }
    }
}

/// GUI startup data that must reach the retained host without becoming a second argv parser.
#[derive(Clone, Debug, PartialEq, Eq)]
pub(crate) struct EditorGuiLaunchIntent {
    startup_request: Option<EditorGuiStartupRequest>,
    startup_scene_uri: Option<AssetUri>,
    layout_preset: Option<String>,
    hub_handshake: Option<EditorHubLaunchHandshake>,
}

/// The already-validated Hub launch context carried from argv to the retained host.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) struct EditorHubLaunchHandshake {
    session: HubSessionToken,
}

impl EditorHubLaunchHandshake {
    fn new(session: HubSessionToken) -> Self {
        Self { session }
    }

    pub(crate) fn session(&self) -> HubSessionToken {
        self.session
    }
}

impl EditorGuiLaunchIntent {
    fn new(
        startup_request: Option<EditorGuiStartupRequest>,
        startup_scene_uri: Option<AssetUri>,
        layout_preset: Option<String>,
        hub_handshake: Option<EditorHubLaunchHandshake>,
    ) -> Self {
        Self {
            startup_request,
            startup_scene_uri,
            layout_preset,
            hub_handshake,
        }
    }

    pub(crate) fn startup_scene_uri(&self) -> Option<&AssetUri> {
        self.startup_scene_uri.as_ref()
    }

    pub(crate) fn layout_preset(&self) -> Option<&str> {
        self.layout_preset.as_deref()
    }

    pub(crate) fn hub_handshake(&self) -> Option<EditorHubLaunchHandshake> {
        self.hub_handshake
    }

    pub(crate) fn into_parts(
        self,
    ) -> (
        Option<EditorGuiStartupRequest>,
        Option<AssetUri>,
        Option<String>,
        Option<EditorHubLaunchHandshake>,
    ) {
        (
            self.startup_request,
            self.startup_scene_uri,
            self.layout_preset,
            self.hub_handshake,
        )
    }

    #[cfg(test)]
    fn into_startup_request(self) -> Option<EditorGuiStartupRequest> {
        self.startup_request
    }
}

/// The GUI-only portion of the launch intent. The process host consumes its resulting editor
/// request and owns project preparation, runtime loading, and retained-host lifetime.
#[derive(Clone, Debug, PartialEq, Eq)]
pub(crate) struct EditorGuiStartupRequestArgs;

impl EditorGuiStartupRequestArgs {
    /// Test-only projection for startup preparation fixtures. Product startup must route through
    /// `EditorLaunchArgs` so scene and layout intent cannot be discarded.
    #[cfg(test)]
    pub(crate) fn parse<I>(args: I) -> Result<Option<EditorGuiStartupRequest>, Box<dyn Error>>
    where
        I: IntoIterator<Item = String>,
    {
        Self::parse_intent(args).map(EditorGuiLaunchIntent::into_startup_request)
    }

    pub(crate) fn parse_intent<I>(args: I) -> Result<EditorGuiLaunchIntent, Box<dyn Error>>
    where
        I: IntoIterator<Item = String>,
    {
        let mut args = args.into_iter();
        let mut project_path = None;
        let mut project_launch_intent = None;
        let mut builtin_view = None;
        let mut create_project = false;
        let mut project_name = None;
        let mut location = None;
        let mut template = None;
        let mut startup_scene_uri = None;
        let mut layout_preset = None;
        let mut hub_session = None;
        let mut hub_protocol_supplied = false;
        let mut saw_gui_arg = false;

        while let Some(arg) = args.next() {
            match arg.as_str() {
                "--project-launch-intent" => {
                    if project_launch_intent.is_some() {
                        return Err("--project-launch-intent was provided more than once".into());
                    }
                    let Some(value) = args.next() else {
                        return Err("--project-launch-intent requires a JSON payload".into());
                    };
                    project_launch_intent = Some(serde_json::from_str(&value).map_err(|error| {
                        format!(
                            "--project-launch-intent requires a valid supported project launch intent: {error}"
                        )
                    })?);
                    saw_gui_arg = true;
                }
                "--project" => {
                    if project_path.is_some() {
                        return Err("--project was provided more than once".into());
                    }
                    let Some(value) = args.next() else {
                        return Err("--project requires a project path".into());
                    };
                    if value.trim().is_empty() {
                        return Err("--project requires a non-empty project path".into());
                    }
                    project_path = Some(value);
                    saw_gui_arg = true;
                }
                "--builtin-view" => {
                    if builtin_view.is_some() {
                        return Err("--builtin-view was provided more than once".into());
                    }
                    let Some(value) = args.next() else {
                        return Err("--builtin-view requires a view descriptor id".into());
                    };
                    if value.trim().is_empty() {
                        return Err("--builtin-view requires a non-empty view descriptor id".into());
                    }
                    builtin_view = Some(value);
                    saw_gui_arg = true;
                }
                "--create-project" => {
                    if create_project {
                        return Err("--create-project was provided more than once".into());
                    }
                    create_project = true;
                    saw_gui_arg = true;
                }
                "--project-name" => {
                    if project_name.is_some() {
                        return Err("--project-name was provided more than once".into());
                    }
                    let Some(value) = args.next() else {
                        return Err("--project-name requires a value".into());
                    };
                    if value.trim().is_empty() {
                        return Err("--project-name requires a non-empty value".into());
                    }
                    project_name = Some(value);
                    saw_gui_arg = true;
                }
                "--location" => {
                    if location.is_some() {
                        return Err("--location was provided more than once".into());
                    }
                    let Some(value) = args.next() else {
                        return Err("--location requires a directory".into());
                    };
                    if value.trim().is_empty() {
                        return Err("--location requires a non-empty directory".into());
                    }
                    location = Some(value);
                    saw_gui_arg = true;
                }
                "--template" => {
                    if template.is_some() {
                        return Err("--template was provided more than once".into());
                    }
                    let Some(value) = args.next() else {
                        return Err("--template requires a value".into());
                    };
                    if value != "renderable-empty" {
                        return Err(format!("unsupported project template `{value}`").into());
                    }
                    template = Some(value);
                    saw_gui_arg = true;
                }
                "--scene" => {
                    if startup_scene_uri.is_some() {
                        return Err("--scene was provided more than once".into());
                    }
                    let Some(value) = args.next() else {
                        return Err("--scene requires a scene asset URI".into());
                    };
                    if value.trim().is_empty() {
                        return Err("--scene requires a non-empty scene asset URI".into());
                    }
                    startup_scene_uri = Some(AssetUri::parse(&value).map_err(|error| {
                        format!("--scene requires a valid scene asset URI: {error}")
                    })?);
                    saw_gui_arg = true;
                }
                "--layout" => {
                    if layout_preset.is_some() {
                        return Err("--layout was provided more than once".into());
                    }
                    let Some(value) = args.next() else {
                        return Err("--layout requires a preset id".into());
                    };
                    if value.trim().is_empty() {
                        return Err("--layout requires a non-empty preset id".into());
                    }
                    layout_preset = Some(value);
                    saw_gui_arg = true;
                }
                "--hub-session" => {
                    if hub_session.is_some() {
                        return Err("--hub-session was provided more than once".into());
                    }
                    let Some(value) = args.next() else {
                        return Err("--hub-session requires a canonical UUID v4 token".into());
                    };
                    hub_session = Some(HubSessionToken::from_str(&value).map_err(|error| {
                        format!("--hub-session requires a canonical UUID v4 token: {error}")
                    })?);
                    saw_gui_arg = true;
                }
                "--hub-protocol" => {
                    if hub_protocol_supplied {
                        return Err("--hub-protocol was provided more than once".into());
                    }
                    let Some(value) = args.next() else {
                        return Err("--hub-protocol requires protocol version 1".into());
                    };
                    let version = value
                        .parse::<u32>()
                        .map_err(|_| "--hub-protocol requires protocol version 1".to_string())?;
                    if version != HUB_PROTOCOL_VERSION_V1 {
                        return Err(format!(
                            "unsupported Hub protocol version {version}; expected {HUB_PROTOCOL_VERSION_V1}"
                        )
                        .into());
                    }
                    hub_protocol_supplied = true;
                    saw_gui_arg = true;
                }
                other => {
                    return Err(format!("unknown editor GUI startup argument `{other}`").into());
                }
            }
        }

        if !saw_gui_arg {
            return Ok(EditorGuiLaunchIntent::new(None, None, layout_preset, None));
        }
        let hub_handshake = match (hub_session, hub_protocol_supplied) {
            (Some(session), true) => Some(EditorHubLaunchHandshake::new(session)),
            (Some(_), false) => return Err("--hub-session requires --hub-protocol 1".into()),
            (None, true) => return Err("--hub-protocol requires --hub-session".into()),
            (None, false) => None,
        };
        if project_launch_intent.is_some()
            && (project_path.is_some()
                || create_project
                || project_name.is_some()
                || location.is_some()
                || template.is_some()
                || builtin_view.is_some())
        {
            return Err(
                "--project-launch-intent cannot be combined with --project, --create-project, --project-name, --location, --template, or --builtin-view"
                    .into(),
            );
        }
        if let Some(project_launch_intent) = project_launch_intent {
            if hub_handshake.is_some() && project_launch_intent.source() != ProjectLaunchSource::Hub
            {
                return Err(
                    "--hub-session requires a Hub-originated --project-launch-intent".into(),
                );
            }
            if startup_scene_uri.is_some()
                && !matches!(
                    project_launch_intent.target(),
                    ProjectLaunchTarget::OpenExisting { .. }
                )
            {
                return Err("--scene requires an existing project launch target".into());
            }
            return Ok(EditorGuiLaunchIntent::new(
                Some(EditorGuiStartupRequest::project(project_launch_intent)),
                startup_scene_uri,
                layout_preset,
                hub_handshake,
            ));
        }
        let project_launch_source = if hub_handshake.is_some() {
            ProjectLaunchSource::Hub
        } else {
            ProjectLaunchSource::Cli
        };
        if hub_handshake.is_some() {
            return Err("--hub-session requires --project-launch-intent".into());
        }
        if startup_scene_uri.is_some() && project_path.is_none() {
            return Err("--scene requires --project".into());
        }
        if create_project {
            if project_path.is_some() || builtin_view.is_some() {
                return Err(
                    "--project and --builtin-view cannot be combined with --create-project".into(),
                );
            }
            let Some(project_name) = project_name else {
                return Err("--create-project requires --project-name".into());
            };
            let Some(location) = location else {
                return Err("--create-project requires --location".into());
            };
            if template.as_deref() != Some("renderable-empty") {
                return Err("--create-project requires --template renderable-empty".into());
            }
            let project_intent = ProjectLaunchIntent::create_project(
                next_project_launch_operation_id()?,
                project_launch_source,
                ProjectLaunchProfile::Normal,
                project_name,
                location,
                ProjectTemplateId::RenderableEmpty,
            )?;
            return Ok(EditorGuiLaunchIntent::new(
                Some(EditorGuiStartupRequest::project(project_intent)),
                startup_scene_uri,
                layout_preset,
                hub_handshake,
            ));
        }
        if project_name.is_some() || location.is_some() || template.is_some() {
            return Err(
                "--project-name, --location, and --template require --create-project".into(),
            );
        }
        if project_path.is_some() && builtin_view.is_some() {
            return Err("--project cannot be combined with --builtin-view".into());
        }
        if let Some(descriptor_id) = builtin_view {
            return Ok(EditorGuiLaunchIntent::new(
                Some(EditorGuiStartupRequest::open_builtin_view(descriptor_id)),
                startup_scene_uri,
                layout_preset,
                hub_handshake,
            ));
        }
        let Some(project_path) = project_path else {
            return Ok(EditorGuiLaunchIntent::new(
                None,
                None,
                layout_preset,
                hub_handshake,
            ));
        };
        let project_intent = ProjectLaunchIntent::open_existing(
            next_project_launch_operation_id()?,
            project_launch_source,
            ProjectLaunchProfile::Normal,
            project_path,
        )?;
        Ok(EditorGuiLaunchIntent::new(
            Some(EditorGuiStartupRequest::project(project_intent)),
            startup_scene_uri,
            layout_preset,
            hub_handshake,
        ))
    }
}

fn next_project_launch_operation_id() -> Result<ProjectActivationOperationId, std::io::Error> {
    CLI_PROJECT_LAUNCH_OPERATION_IDS
        .get_or_init(|| ProjectActivationOperationIdGenerator::new(ProjectLaunchInstanceId::new()))
        .allocate()
        .ok_or_else(|| std::io::Error::other("project launch operation sequence is exhausted"))
}

pub(crate) fn editor_startup_argument_error(
    args: &[String],
    source: Box<dyn Error>,
) -> Box<dyn Error> {
    EditorLaunchArgumentError {
        requested: editor_startup_argument_summary(args),
        cause: redact_editor_startup_argument_cause(args, source.to_string()),
    }
    .into()
}

#[derive(Debug)]
struct EditorLaunchArgumentError {
    requested: String,
    cause: String,
}

impl std::fmt::Display for EditorLaunchArgumentError {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            formatter,
            "editor startup diagnostic: component=editor_app requested={} cause={} recovery=provide one valid editor startup mode and run zircon_editor --help to inspect supported arguments",
            self.requested, self.cause
        )
    }
}

impl Error for EditorLaunchArgumentError {}

fn editor_startup_argument_summary(args: &[String]) -> String {
    if args.is_empty() {
        return "<empty>".to_string();
    }

    let mut display_path_next = false;
    let mut redact_project_intent_next = false;
    args.iter()
        .map(|argument| {
            if redact_project_intent_next {
                redact_project_intent_next = false;
                return "<project-launch-intent>".to_string();
            }
            if display_path_next {
                display_path_next = false;
                return editor_startup_path_display(argument);
            }
            redact_project_intent_next = argument == "--project-launch-intent";
            display_path_next = editor_startup_argument_is_path_flag(argument);
            argument.clone()
        })
        .collect::<Vec<_>>()
        .join(" ")
}

fn redact_editor_startup_argument_cause(args: &[String], mut cause: String) -> String {
    let mut display_path_next = false;
    let mut redact_project_intent_next = false;
    for argument in args {
        if redact_project_intent_next {
            cause = cause.replace(argument, "<project-launch-intent>");
            redact_project_intent_next = false;
        } else if display_path_next {
            cause = cause.replace(argument, &editor_startup_path_display(argument));
            display_path_next = false;
        } else {
            redact_project_intent_next = argument == "--project-launch-intent";
            display_path_next = editor_startup_argument_is_path_flag(argument);
        }
    }
    cause
}

fn editor_startup_argument_is_path_flag(argument: &str) -> bool {
    matches!(argument, "--project" | "--automation" | "--location")
}

fn editor_startup_path_display(argument: &str) -> String {
    ProjectPaths::display_path(Path::new(argument))
        .display()
        .to_string()
}

#[cfg(test)]
mod tests {
    use std::path::Path;

    use super::{EditorGuiStartupRequestArgs, EditorLaunchArgs, EditorLaunchRoute};
    use zircon_editor::{
        core::commandlet::{CommandletExitCode, CommandletStatus},
        EditorGuiStartupRequest,
    };
    use zircon_runtime::diagnostic_log::{DiagnosticLogFilter, DiagnosticLogLevel};
    use zircon_runtime_interface::project::{
        ProjectActivationOperationId, ProjectActivationOperationIdGenerator,
        ProjectLaunchInstanceId, ProjectLaunchIntent, ProjectLaunchProfile, ProjectLaunchSource,
    };

    fn next_test_project_operation_id() -> ProjectActivationOperationId {
        ProjectActivationOperationIdGenerator::new(ProjectLaunchInstanceId::new())
            .allocate()
            .expect("fresh test operation identity")
    }

    #[test]
    fn unified_launch_args_route_run_to_the_editor_core_commandlet() {
        let route = EditorLaunchArgs::parse([
            "--run",
            "migrate-assets",
            "--project",
            "fixture",
            "--dry-run",
        ])
        .unwrap()
        .route()
        .unwrap();

        let EditorLaunchRoute::Commandlet(request) = route else {
            panic!("--run should route to the editor core commandlet");
        };
        assert_eq!(request.command(), "migrate-assets");
    }

    #[test]
    fn unified_launch_args_preserve_authoring_automation_typed_arguments() {
        let route = EditorLaunchArgs::parse([
            "--run",
            "authoring-automation",
            "--project",
            "fixture-project",
            "--automation",
            "fixture-automation.json",
        ])
        .unwrap()
        .route()
        .unwrap();

        let EditorLaunchRoute::Commandlet(request) = route else {
            panic!("authoring automation should route through the editor core commandlet");
        };
        assert_eq!(request.command(), "authoring-automation");
        assert_eq!(request.project_root(), Some(Path::new("fixture-project")));
        assert_eq!(
            request.automation_path(),
            Some(Path::new("fixture-automation.json"))
        );
    }

    #[test]
    fn unified_launch_args_preserve_json_parameter_errors_for_commandlets() {
        let route =
            EditorLaunchArgs::parse(["--run", "unknown", "--project", "fixture", "--dry-run"])
                .unwrap()
                .route()
                .unwrap();

        let EditorLaunchRoute::CommandletRejected(report) = route else {
            panic!("unknown --run target should return the commandlet JSON report");
        };
        assert_eq!(report.exit_code(), CommandletExitCode::InvalidArguments);
        assert_eq!(report.status(), CommandletStatus::InvalidArguments);
    }

    #[test]
    fn unified_launch_args_initialize_diagnostics_before_routing_gui() {
        let launch_args =
            EditorLaunchArgs::parse(["--project", "fixture-project", "--log-level", "warn"])
                .unwrap();

        assert_eq!(
            launch_args.diagnostic_filter().minimum,
            DiagnosticLogFilter::Minimum(DiagnosticLogLevel::Warn)
        );
        let route = launch_args.route().unwrap();
        assert!(matches!(route, EditorLaunchRoute::Gui(_)));
    }

    #[test]
    fn unified_launch_args_carry_a_layout_preset_to_the_gui_host() {
        let route = EditorLaunchArgs::parse(["--project", "fixture-project", "--layout", "debug"])
            .unwrap()
            .route()
            .unwrap();

        let EditorLaunchRoute::Gui(intent) = route else {
            panic!("a GUI launch should carry the requested layout preset");
        };
        assert_eq!(intent.layout_preset(), Some("debug"));
        assert!(matches!(
            intent.into_parts().0,
            Some(EditorGuiStartupRequest::Project { .. })
        ));
    }

    #[test]
    fn unified_launch_args_carry_a_project_scene_to_the_gui_host() {
        let route = EditorLaunchArgs::parse([
            "--project",
            "fixture-project",
            "--scene",
            "res://scenes/main.scene.toml",
        ])
        .unwrap()
        .route()
        .unwrap();

        let EditorLaunchRoute::Gui(intent) = route else {
            panic!("a GUI launch should carry the requested project scene");
        };
        assert_eq!(
            intent
                .startup_scene_uri()
                .map(ToString::to_string)
                .as_deref(),
            Some("res://scenes/main.scene.toml")
        );
        assert!(matches!(
            intent.into_parts().0,
            Some(EditorGuiStartupRequest::Project { .. })
        ));
    }

    #[test]
    fn unified_launch_args_carry_a_verified_hub_handshake_to_the_gui_host() {
        let transmitted = ProjectLaunchIntent::open_existing(
            next_test_project_operation_id(),
            ProjectLaunchSource::Hub,
            ProjectLaunchProfile::Normal,
            "fixture-project",
        )
        .unwrap();
        let route = EditorLaunchArgs::parse([
            "--hub-protocol",
            "1",
            "--project-launch-intent",
            &serde_json::to_string(&transmitted).unwrap(),
            "--hub-session",
            "0d9a5890-0e44-4e2a-b77e-3e5d4fdf1e52",
        ])
        .unwrap()
        .route()
        .unwrap();

        let EditorLaunchRoute::Gui(intent) = route else {
            panic!("a Hub launch should use the GUI route");
        };
        assert_eq!(
            intent
                .hub_handshake()
                .map(|handshake| handshake.session().to_string()),
            Some("0d9a5890-0e44-4e2a-b77e-3e5d4fdf1e52".to_string())
        );
        assert!(matches!(
            intent.into_parts().0,
            Some(EditorGuiStartupRequest::Project { intent }) if intent == transmitted
        ));
    }

    #[test]
    fn gui_launch_intent_rejects_incomplete_invalid_or_unscoped_hub_handshakes() {
        for (args, expected) in [
            (
                vec![
                    "--project".to_string(),
                    "fixture".to_string(),
                    "--hub-session".to_string(),
                    "0d9a5890-0e44-4e2a-b77e-3e5d4fdf1e52".to_string(),
                ],
                "--hub-session requires --hub-protocol 1",
            ),
            (
                vec!["--hub-protocol".to_string(), "1".to_string()],
                "--hub-protocol requires --hub-session",
            ),
            (
                vec![
                    "--project".to_string(),
                    "fixture".to_string(),
                    "--hub-session".to_string(),
                    "0d9a5890-0e44-4e2a-b77e-3e5d4fdf1e52".to_string(),
                    "--hub-protocol".to_string(),
                    "2".to_string(),
                ],
                "unsupported Hub protocol version 2; expected 1",
            ),
            (
                vec![
                    "--hub-session".to_string(),
                    "0d9a5890-0e44-4e2a-b77e-3e5d4fdf1e52".to_string(),
                    "--hub-protocol".to_string(),
                    "1".to_string(),
                ],
                "--hub-session requires --project-launch-intent",
            ),
        ] {
            let error = EditorGuiStartupRequestArgs::parse_intent(args).unwrap_err();
            assert_eq!(error.to_string(), expected);
        }

        let error = EditorGuiStartupRequestArgs::parse_intent([
            "--project".to_string(),
            "fixture".to_string(),
            "--hub-session".to_string(),
            "not-a-uuid".to_string(),
            "--hub-protocol".to_string(),
            "1".to_string(),
        ])
        .unwrap_err();
        assert!(error
            .to_string()
            .starts_with("--hub-session requires a canonical UUID v4 token:"));
    }

    #[test]
    fn hub_handshake_preserves_the_transmitted_project_operation_id() {
        let transmitted = ProjectLaunchIntent::open_existing(
            next_test_project_operation_id(),
            ProjectLaunchSource::Hub,
            ProjectLaunchProfile::Safe,
            "E:/Projects/My Game",
        )
        .unwrap();
        let payload = serde_json::to_string(&transmitted).unwrap();

        let parsed = EditorGuiStartupRequestArgs::parse_intent([
            "--project-launch-intent".to_string(),
            payload,
            "--hub-session".to_string(),
            "0d9a5890-0e44-4e2a-b77e-3e5d4fdf1e52".to_string(),
            "--hub-protocol".to_string(),
            "1".to_string(),
        ])
        .unwrap();

        let Some(EditorGuiStartupRequest::Project { intent }) = parsed.into_parts().0 else {
            panic!("a Hub launch should preserve its project launch intent");
        };
        assert_eq!(intent.operation_id(), transmitted.operation_id());
        assert_eq!(intent.profile(), ProjectLaunchProfile::Safe);
    }

    #[test]
    fn hub_handshake_rejects_legacy_project_arguments() {
        let error = EditorGuiStartupRequestArgs::parse_intent([
            "--project".to_string(),
            "fixture-project".to_string(),
            "--hub-session".to_string(),
            "0d9a5890-0e44-4e2a-b77e-3e5d4fdf1e52".to_string(),
            "--hub-protocol".to_string(),
            "1".to_string(),
        ])
        .unwrap_err();

        assert_eq!(
            error.to_string(),
            "--hub-session requires --project-launch-intent"
        );
    }

    #[test]
    fn hub_intent_diagnostics_redact_the_serialized_project_payload() {
        let transmitted = ProjectLaunchIntent::open_existing(
            next_test_project_operation_id(),
            ProjectLaunchSource::Cli,
            ProjectLaunchProfile::Normal,
            "E:/Private Projects/Secret Game",
        )
        .unwrap();
        let payload = serde_json::to_string(&transmitted).unwrap();

        let error = EditorLaunchArgs::parse([
            "--project-launch-intent".to_string(),
            payload,
            "--hub-session".to_string(),
            "0d9a5890-0e44-4e2a-b77e-3e5d4fdf1e52".to_string(),
            "--hub-protocol".to_string(),
            "1".to_string(),
        ])
        .unwrap()
        .route()
        .unwrap_err();

        let diagnostic = error.to_string();
        assert!(diagnostic.contains("--project-launch-intent <project-launch-intent>"));
        assert!(!diagnostic.contains("E:/Private Projects/Secret Game"));
    }

    #[test]
    fn gui_launch_intent_rejects_invalid_or_unscoped_project_scenes() {
        for (args, expected) in [
            (
                vec!["--scene".to_string()],
                "--scene requires a scene asset URI",
            ),
            (
                vec![
                    "--project".to_string(),
                    "fixture".to_string(),
                    "--scene".to_string(),
                    " ".to_string(),
                ],
                "--scene requires a non-empty scene asset URI",
            ),
            (
                vec![
                    "--scene".to_string(),
                    "res://scenes/main.scene.toml".to_string(),
                ],
                "--scene requires --project",
            ),
            (
                vec![
                    "--project".to_string(),
                    "fixture".to_string(),
                    "--scene".to_string(),
                    "res://scenes/one.scene.toml".to_string(),
                    "--scene".to_string(),
                    "res://scenes/two.scene.toml".to_string(),
                ],
                "--scene was provided more than once",
            ),
        ] {
            let error = EditorGuiStartupRequestArgs::parse_intent(args).unwrap_err();
            assert_eq!(error.to_string(), expected);
        }

        let error = EditorGuiStartupRequestArgs::parse_intent([
            "--project".to_string(),
            "fixture".to_string(),
            "--scene".to_string(),
            "not-an-asset-uri".to_string(),
        ])
        .unwrap_err();
        assert!(error
            .to_string()
            .starts_with("--scene requires a valid scene asset URI:"));
    }

    #[test]
    fn gui_launch_intent_rejects_missing_empty_or_duplicate_layout_presets() {
        for (args, expected) in [
            (
                vec!["--layout".to_string()],
                "--layout requires a preset id",
            ),
            (
                vec!["--layout".to_string(), " ".to_string()],
                "--layout requires a non-empty preset id",
            ),
            (
                vec![
                    "--layout".to_string(),
                    "debug".to_string(),
                    "--layout".to_string(),
                    "authoring".to_string(),
                ],
                "--layout was provided more than once",
            ),
        ] {
            let error = EditorGuiStartupRequestArgs::parse_intent(args).unwrap_err();
            assert_eq!(error.to_string(), expected);
        }
    }

    #[test]
    fn unified_launch_args_route_help_before_host_or_commandlet_construction() {
        let route = EditorLaunchArgs::parse(["--run", "plugin-list", "--help"])
            .unwrap()
            .route()
            .unwrap();

        assert!(matches!(route, EditorLaunchRoute::Help));
    }
}
