//! 解析动态 Runtime 会话的进程参数；物理项目路径与会话创建仍由各产品 runner 持有。

use std::{
    error::Error,
    fmt::{self, Display, Formatter},
    path::{Path, PathBuf},
};

use zircon_runtime::asset::project::ProjectPaths;
use zircon_runtime_interface::project::RelPath;

const RUNTIME_SESSION_PROJECT_ARG: &str = "--project";
const RUNTIME_SESSION_PROFILE_ARG: &str = "--runtime-session-profile";
const RUNTIME_SESSION_PLAY_SCENE_ARG: &str = "--play-scene";
const RUNTIME_SESSION_PLAY_REPORT_PIPE_ARG: &str = "--play-report-pipe";
const RUNTIME_SESSION_REFERENCE_CPU_PRESENTER_ARG: &str = "--reference-cpu-presenter";
const RUNTIME_SESSION_HELP_ARG: &str = "--help";
const RUNTIME_SESSION_SHORT_HELP_ARG: &str = "-h";

pub(super) const RUNTIME_SESSION_STARTUP_HELP: &str = "\
Usage: zircon_runtime [OPTIONS]

Options:
  --project <path>                     Load a Zircon project root or zircon-project.toml and run its default scene
  --project=<path>                     Load the same project input with an equals-form argument
  --runtime-session-profile <profile>   Select runtime, runtime-pipelined, editor, dev, minimal, or headless dynamic session policy
  --runtime-session-profile=<profile>   Select the same dynamic session policy with an equals-form argument
  --play-scene <relative-path>          Load a versioned Play snapshot relative to the selected project root
  --play-scene=<relative-path>          Load the same project-relative Play snapshot with an equals-form argument
  --play-report-pipe <name>             Select a logical Play startup report outlet
  --play-report-pipe=<name>             Select the same logical Play startup report outlet with an equals-form argument
  --reference-cpu-presenter              Explicitly use the degraded CPU copy presenter when a qualified native backend is unavailable
  --log-level <level>                   Select verbose, debug, log, warn, error, or off process logging
  --log-filter <filter>                 Select comma-separated log filters such as warn,zircon_runtime::asset=debug
  -h, --help                            Print this help without loading the dynamic runtime library

Environment:
  ZIRCON_RUNTIME_LIBRARY                Override the dynamic runtime library with a path relative to the product executable or an absolute path
  ZIRCON_LOG_FILTER                     Override scoped process log filters
  ZIRCON_LOG                            Alias for scoped process log filters when ZIRCON_LOG_FILTER is unset
  RUST_LOG                              Bevy-style fallback scoped log filter when Zircon filter variables are unset
  ZIRCON_LOG_LEVEL                      Override the minimum process log level
  ZIRCON_RUNTIME_CAPTURE_FRAME_PNG      Write the first successfully presented runtime frame to a PNG path; relative paths resolve from the open project root, or launch directory without a project
  ZIRCON_RUNTIME_EXIT_AFTER_FIRST_FRAME Exit after the first successfully presented runtime frame
  ZIRCON_RUNTIME_EXIT_AFTER_PRESENTED_FRAMES Exit after a positive decimal count of successfully presented runtime frames

Profiles:
  runtime                               Default runtime preview policy
  runtime-pipelined                     Render-owner pipelined runtime preview policy
  editor                                Editor-host policy accepted by the runtime ABI
  dev                                   Runtime-owned dev diagnostics, including diagnostic-store log cadence
  minimal                               Minimal runtime session policy
  headless                              Headless runtime session policy
";

#[derive(Clone, Debug, PartialEq, Eq)]
pub(super) struct RuntimeSessionStartupArgs {
    pub(super) profile: RuntimeSessionProfile,
    pub(super) project_root: Option<PathBuf>,
    pub(super) play_scene: Option<RelPath>,
    pub(super) play_report_pipe: Option<String>,
    pub(super) reference_cpu_presenter: bool,
    pub(super) help_requested: bool,
    pub(super) remaining_args: Vec<String>,
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub(super) enum RuntimeSessionProfile {
    #[default]
    Runtime,
    RuntimePipelined,
    Editor,
    Dev,
    Minimal,
    Headless,
}

#[derive(Debug)]
pub(super) struct RuntimeStartupArgumentError {
    argument: &'static str,
    requested: String,
    cause: &'static str,
    recovery: &'static str,
}

impl RuntimeStartupArgumentError {
    fn new(
        argument: &'static str,
        requested: impl Into<String>,
        cause: &'static str,
        recovery: &'static str,
    ) -> Self {
        Self {
            argument,
            requested: requested.into(),
            cause,
            recovery,
        }
    }
}

impl Display for RuntimeStartupArgumentError {
    fn fmt(&self, formatter: &mut Formatter<'_>) -> fmt::Result {
        write!(
            formatter,
            "runtime startup diagnostic: component=runtime_app argument={} requested={} cause={} recovery={}",
            self.argument, self.requested, self.cause, self.recovery
        )
    }
}

impl Error for RuntimeStartupArgumentError {}

pub(super) fn unknown_runtime_argument_error(
    argument: impl Into<String>,
) -> RuntimeStartupArgumentError {
    RuntimeStartupArgumentError::new(
        "<unknown>",
        argument,
        "unsupported runtime argument",
        "run zircon_runtime --help to inspect supported startup arguments",
    )
}

pub(super) fn invalid_runtime_project_root_error(
    project_root: &Path,
) -> RuntimeStartupArgumentError {
    RuntimeStartupArgumentError::new(
        RUNTIME_SESSION_PROJECT_ARG,
        ProjectPaths::display_path(project_root)
            .display()
            .to_string(),
        "project input is not an existing directory or zircon-project.toml",
        "provide an existing project-root directory or zircon-project.toml after --project",
    )
}

pub(super) fn missing_runtime_project_manifest_error(
    project_root: &Path,
) -> RuntimeStartupArgumentError {
    RuntimeStartupArgumentError::new(
        RUNTIME_SESSION_PROJECT_ARG,
        ProjectPaths::display_path(project_root)
            .display()
            .to_string(),
        "project manifest zircon-project.toml is missing",
        "provide a Zircon project root containing zircon-project.toml",
    )
}

/// 从产品参数中提取 Runtime ABI 会话意图；不负责检查项目存在或启动报告出口可用性。
pub(super) fn parse_runtime_session_startup_args<I, S>(
    args: I,
) -> Result<RuntimeSessionStartupArgs, Box<dyn Error>>
where
    I: IntoIterator<Item = S>,
    S: Into<String>,
{
    let mut remaining_args = Vec::new();
    let mut profile = RuntimeSessionProfile::default();
    let mut profile_provided = false;
    let mut project_root = None;
    let mut play_scene = None;
    let mut play_report_pipe = None;
    let mut reference_cpu_presenter = false;
    let mut help_requested = false;
    let mut args = args.into_iter().map(Into::into);

    while let Some(arg) = args.next() {
        if arg == RUNTIME_SESSION_HELP_ARG || arg == RUNTIME_SESSION_SHORT_HELP_ARG {
            help_requested = true;
            continue;
        }

        if arg == RUNTIME_SESSION_REFERENCE_CPU_PRESENTER_ARG {
            if reference_cpu_presenter {
                return Err(duplicate_reference_cpu_presenter_error().into());
            }
            reference_cpu_presenter = true;
            continue;
        }

        if arg == RUNTIME_SESSION_PROJECT_ARG {
            if project_root.is_some() {
                return Err(duplicate_project_value_error().into());
            }
            let Some(value) = args.next() else {
                return Err(missing_project_value_error().into());
            };
            if value.trim().is_empty() {
                return Err(empty_project_value_error().into());
            }
            project_root = Some(PathBuf::from(value));
            continue;
        }

        if let Some(value) = arg.strip_prefix("--project=") {
            if project_root.is_some() {
                return Err(duplicate_project_value_error().into());
            }
            if value.trim().is_empty() {
                return Err(empty_project_value_error().into());
            }
            project_root = Some(PathBuf::from(value));
            continue;
        }

        if arg == RUNTIME_SESSION_PLAY_SCENE_ARG {
            if play_scene.is_some() {
                return Err(duplicate_play_scene_value_error().into());
            }
            let Some(value) = args.next() else {
                return Err(missing_play_scene_value_error().into());
            };
            // Play 场景仅允许规范化项目相对路径；根路径解析由实际产品启动阶段执行。
            play_scene = Some(parse_play_scene_value(&value)?);
            continue;
        }

        if let Some(value) = arg.strip_prefix("--play-scene=") {
            if play_scene.is_some() {
                return Err(duplicate_play_scene_value_error().into());
            }
            play_scene = Some(parse_play_scene_value(value)?);
            continue;
        }

        if arg == RUNTIME_SESSION_PLAY_REPORT_PIPE_ARG {
            if play_report_pipe.is_some() {
                return Err(duplicate_play_report_pipe_value_error().into());
            }
            let Some(value) = args.next() else {
                return Err(missing_play_report_pipe_value_error().into());
            };
            play_report_pipe = Some(parse_play_report_pipe_value(&value)?);
            continue;
        }

        if let Some(value) = arg.strip_prefix("--play-report-pipe=") {
            if play_report_pipe.is_some() {
                return Err(duplicate_play_report_pipe_value_error().into());
            }
            play_report_pipe = Some(parse_play_report_pipe_value(value)?);
            continue;
        }

        if arg == RUNTIME_SESSION_PROFILE_ARG {
            if profile_provided {
                return Err(duplicate_profile_value_error().into());
            }
            let Some(value) = args.next() else {
                return Err(missing_profile_value_error().into());
            };
            if value.trim().is_empty() {
                return Err(empty_profile_value_error().into());
            }
            profile = RuntimeSessionProfile::parse(value)?;
            profile_provided = true;
            continue;
        }

        if let Some(value) = arg.strip_prefix("--runtime-session-profile=") {
            if profile_provided {
                return Err(duplicate_profile_value_error().into());
            }
            if value.trim().is_empty() {
                return Err(empty_profile_value_error().into());
            }
            profile = RuntimeSessionProfile::parse(value)?;
            profile_provided = true;
            continue;
        }

        remaining_args.push(arg);
    }

    Ok(RuntimeSessionStartupArgs {
        profile,
        project_root,
        play_scene,
        play_report_pipe,
        reference_cpu_presenter,
        help_requested,
        remaining_args,
    })
}

pub(super) fn play_startup_requires_project_error(
    argument: &'static str,
) -> RuntimeStartupArgumentError {
    RuntimeStartupArgumentError::new(
        argument,
        "<provided-without-project>",
        "Play startup input requires --project",
        "provide a project root before selecting a Play scene or Play report outlet",
    )
}

fn parse_play_scene_value(value: impl AsRef<str>) -> Result<RelPath, RuntimeStartupArgumentError> {
    let value = value.as_ref();
    if value.trim().is_empty() {
        return Err(empty_play_scene_value_error());
    }
    RelPath::parse(value).map_err(|_| {
        RuntimeStartupArgumentError::new(
            RUNTIME_SESSION_PLAY_SCENE_ARG,
            value,
            "Play scene must be a normalized project-relative path",
            "provide a nonempty relative path without a platform prefix, . or .. components",
        )
    })
}

fn parse_play_report_pipe_value(
    value: impl AsRef<str>,
) -> Result<String, RuntimeStartupArgumentError> {
    let value = value.as_ref();
    if value.trim().is_empty() {
        return Err(empty_play_report_pipe_value_error());
    }
    Ok(value.to_owned())
}

impl RuntimeSessionProfile {
    pub(super) const fn as_bytes(self) -> &'static [u8] {
        self.as_str().as_bytes()
    }

    pub(super) const fn as_str(self) -> &'static str {
        match self {
            Self::Runtime => "runtime",
            Self::RuntimePipelined => "runtime-pipelined",
            Self::Editor => "editor",
            Self::Dev => "dev",
            Self::Minimal => "minimal",
            Self::Headless => "headless",
        }
    }

    fn parse(value: impl AsRef<str>) -> Result<Self, Box<dyn Error>> {
        match value.as_ref().trim().to_ascii_lowercase().as_str() {
            "runtime" => Ok(Self::Runtime),
            "runtime-pipelined" => Ok(Self::RuntimePipelined),
            "editor" => Ok(Self::Editor),
            "dev" => Ok(Self::Dev),
            "minimal" => Ok(Self::Minimal),
            "headless" => Ok(Self::Headless),
            _ => Err(RuntimeStartupArgumentError::new(
                RUNTIME_SESSION_PROFILE_ARG,
                value.as_ref(),
                "unsupported runtime session profile",
                "choose runtime, runtime-pipelined, editor, dev, minimal, or headless",
            )
            .into()),
        }
    }
}

fn duplicate_project_value_error() -> RuntimeStartupArgumentError {
    RuntimeStartupArgumentError::new(
        RUNTIME_SESSION_PROJECT_ARG,
        "<multiple>",
        "project root was provided more than once",
        "provide exactly one project root after --project",
    )
}

fn missing_profile_value_error() -> RuntimeStartupArgumentError {
    RuntimeStartupArgumentError::new(
        RUNTIME_SESSION_PROFILE_ARG,
        "<missing>",
        "missing runtime session profile",
        "provide runtime, runtime-pipelined, editor, dev, minimal, or headless after --runtime-session-profile",
    )
}

fn duplicate_profile_value_error() -> RuntimeStartupArgumentError {
    RuntimeStartupArgumentError::new(
        RUNTIME_SESSION_PROFILE_ARG,
        "<multiple>",
        "runtime session profile was provided more than once",
        "provide exactly one --runtime-session-profile value",
    )
}

fn empty_profile_value_error() -> RuntimeStartupArgumentError {
    RuntimeStartupArgumentError::new(
        RUNTIME_SESSION_PROFILE_ARG,
        "<empty>",
        "missing runtime session profile",
        "provide runtime, runtime-pipelined, editor, dev, minimal, or headless after --runtime-session-profile",
    )
}

fn missing_project_value_error() -> RuntimeStartupArgumentError {
    RuntimeStartupArgumentError::new(
        RUNTIME_SESSION_PROJECT_ARG,
        "<missing>",
        "missing project root path",
        "provide an existing Zircon project root after --project",
    )
}

fn empty_project_value_error() -> RuntimeStartupArgumentError {
    RuntimeStartupArgumentError::new(
        RUNTIME_SESSION_PROJECT_ARG,
        "<empty>",
        "missing project root path",
        "provide an existing Zircon project root after --project",
    )
}

fn missing_play_scene_value_error() -> RuntimeStartupArgumentError {
    RuntimeStartupArgumentError::new(
        RUNTIME_SESSION_PLAY_SCENE_ARG,
        "<missing>",
        "missing project-relative Play scene path",
        "provide a normalized project-relative path after --play-scene",
    )
}

fn duplicate_play_scene_value_error() -> RuntimeStartupArgumentError {
    RuntimeStartupArgumentError::new(
        RUNTIME_SESSION_PLAY_SCENE_ARG,
        "<multiple>",
        "Play scene was provided more than once",
        "provide exactly one --play-scene value",
    )
}

fn empty_play_scene_value_error() -> RuntimeStartupArgumentError {
    RuntimeStartupArgumentError::new(
        RUNTIME_SESSION_PLAY_SCENE_ARG,
        "<empty>",
        "missing project-relative Play scene path",
        "provide a normalized project-relative path after --play-scene",
    )
}

fn missing_play_report_pipe_value_error() -> RuntimeStartupArgumentError {
    RuntimeStartupArgumentError::new(
        RUNTIME_SESSION_PLAY_REPORT_PIPE_ARG,
        "<missing>",
        "missing Play startup report outlet name",
        "provide a nonempty report outlet name after --play-report-pipe",
    )
}

fn duplicate_play_report_pipe_value_error() -> RuntimeStartupArgumentError {
    RuntimeStartupArgumentError::new(
        RUNTIME_SESSION_PLAY_REPORT_PIPE_ARG,
        "<multiple>",
        "Play startup report outlet was provided more than once",
        "provide exactly one --play-report-pipe value",
    )
}

fn empty_play_report_pipe_value_error() -> RuntimeStartupArgumentError {
    RuntimeStartupArgumentError::new(
        RUNTIME_SESSION_PLAY_REPORT_PIPE_ARG,
        "<empty>",
        "missing Play startup report outlet name",
        "provide a nonempty report outlet name after --play-report-pipe",
    )
}

fn duplicate_reference_cpu_presenter_error() -> RuntimeStartupArgumentError {
    RuntimeStartupArgumentError::new(
        RUNTIME_SESSION_REFERENCE_CPU_PRESENTER_ARG,
        "<multiple>",
        "reference CPU presenter was enabled more than once",
        "provide --reference-cpu-presenter at most once",
    )
}

#[cfg(test)]
#[path = "tests/runtime_session_args.rs"]
mod tests;
