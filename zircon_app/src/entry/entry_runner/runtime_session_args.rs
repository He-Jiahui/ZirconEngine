use std::{
    error::Error,
    fmt::{self, Display, Formatter},
    path::{Path, PathBuf},
};

const RUNTIME_SESSION_PROJECT_ARG: &str = "--project";
const RUNTIME_SESSION_PROFILE_ARG: &str = "--runtime-session-profile";
const RUNTIME_SESSION_HELP_ARG: &str = "--help";
const RUNTIME_SESSION_SHORT_HELP_ARG: &str = "-h";

pub(super) const RUNTIME_SESSION_STARTUP_HELP: &str = "\
Usage: zircon_runtime [OPTIONS]

Options:
  --project <path>                     Load a Zircon project root and run its default scene
  --project=<path>                     Load the same project root with an equals-form argument
  --runtime-session-profile <profile>   Select runtime, runtime-pipelined, editor, dev, minimal, or headless dynamic session policy
  --runtime-session-profile=<profile>   Select the same dynamic session policy with an equals-form argument
  --log-level <level>                   Select verbose, debug, log, warn, error, or off process logging
  --log-filter <filter>                 Select comma-separated log filters such as warn,zircon_runtime::asset=debug
  -h, --help                            Print this help without loading the dynamic runtime library

Environment:
  ZIRCON_RUNTIME_LIBRARY                Override the dynamic runtime library path
  ZIRCON_LOG_FILTER                     Override scoped process log filters
  ZIRCON_LOG                            Alias for scoped process log filters when ZIRCON_LOG_FILTER is unset
  RUST_LOG                              Bevy-style fallback scoped log filter when Zircon filter variables are unset
  ZIRCON_LOG_LEVEL                      Override the minimum process log level
  ZIRCON_RUNTIME_CAPTURE_FRAME_PNG      Write the first successfully presented runtime frame to this PNG path

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
        project_root.display().to_string(),
        "project root is not an existing directory",
        "provide an existing project-root directory after --project",
    )
}

pub(super) fn missing_runtime_project_manifest_error(
    project_root: &Path,
) -> RuntimeStartupArgumentError {
    RuntimeStartupArgumentError::new(
        RUNTIME_SESSION_PROJECT_ARG,
        project_root.display().to_string(),
        "project manifest zircon-project.toml is missing",
        "provide a Zircon project root containing zircon-project.toml",
    )
}

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
    let mut help_requested = false;
    let mut args = args.into_iter().map(Into::into);

    while let Some(arg) = args.next() {
        if arg == RUNTIME_SESSION_HELP_ARG || arg == RUNTIME_SESSION_SHORT_HELP_ARG {
            help_requested = true;
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
        help_requested,
        remaining_args,
    })
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

#[cfg(test)]
mod tests {
    use super::{parse_runtime_session_startup_args, RuntimeSessionProfile};

    #[test]
    fn runtime_session_args_default_to_runtime_profile() {
        let parsed = parse_runtime_session_startup_args(["--log-level=debug".to_string()]).unwrap();

        assert_eq!(parsed.profile, RuntimeSessionProfile::Runtime);
        assert_eq!(parsed.profile.as_bytes(), b"runtime");
        assert_eq!(parsed.project_root, None);
        assert!(!parsed.help_requested);
        assert_eq!(parsed.remaining_args, ["--log-level=debug"]);
    }

    #[test]
    fn runtime_session_args_strip_space_separated_profile() {
        let parsed = parse_runtime_session_startup_args([
            "--runtime-session-profile".to_string(),
            "dev".to_string(),
            "--leftover".to_string(),
        ])
        .unwrap();

        assert_eq!(parsed.profile, RuntimeSessionProfile::Dev);
        assert_eq!(parsed.profile.as_bytes(), b"dev");
        assert_eq!(parsed.project_root, None);
        assert_eq!(parsed.remaining_args, ["--leftover"]);
    }

    #[test]
    fn runtime_session_args_strip_equals_profile() {
        let parsed =
            parse_runtime_session_startup_args(["--runtime-session-profile=headless".to_string()])
                .unwrap();

        assert_eq!(parsed.profile, RuntimeSessionProfile::Headless);
        assert_eq!(parsed.profile.as_bytes(), b"headless");
        assert_eq!(parsed.project_root, None);
        assert!(!parsed.help_requested);
        assert!(parsed.remaining_args.is_empty());
    }

    #[test]
    fn runtime_session_args_accept_pipelined_runtime_profile() {
        let parsed = parse_runtime_session_startup_args([
            "--runtime-session-profile=runtime-pipelined".to_string(),
        ])
        .unwrap();

        assert_eq!(parsed.profile, RuntimeSessionProfile::RuntimePipelined);
        assert_eq!(parsed.profile.as_bytes(), b"runtime-pipelined");
        assert!(parsed.remaining_args.is_empty());
    }

    #[test]
    fn runtime_session_args_strip_space_separated_project_root() {
        let parsed = parse_runtime_session_startup_args([
            "--project".to_string(),
            "examples/vampire".to_string(),
            "--runtime-session-profile=dev".to_string(),
        ])
        .unwrap();

        assert_eq!(parsed.profile, RuntimeSessionProfile::Dev);
        assert_eq!(
            parsed.project_root.as_deref(),
            Some(std::path::Path::new("examples/vampire"))
        );
        assert!(parsed.remaining_args.is_empty());
    }

    #[test]
    fn runtime_session_args_strip_equals_project_root() {
        let parsed =
            parse_runtime_session_startup_args(["--project=examples/vampire".to_string()]).unwrap();

        assert_eq!(
            parsed.project_root.as_deref(),
            Some(std::path::Path::new("examples/vampire"))
        );
        assert!(parsed.remaining_args.is_empty());
    }

    #[test]
    fn runtime_session_args_strip_help_request() {
        let parsed = parse_runtime_session_startup_args([
            "--help".to_string(),
            "--runtime-session-profile=dev".to_string(),
            "-h".to_string(),
        ])
        .unwrap();

        assert_eq!(parsed.profile, RuntimeSessionProfile::Dev);
        assert!(parsed.help_requested);
        assert!(parsed.remaining_args.is_empty());
    }

    #[test]
    fn runtime_session_help_lists_profiles_and_diagnostic_inputs() {
        for expected in [
            "--runtime-session-profile",
            "runtime",
            "runtime-pipelined",
            "editor",
            "dev",
            "minimal",
            "headless",
            "--project",
            "--log-level",
            "--log-filter",
            "ZIRCON_RUNTIME_LIBRARY",
            "ZIRCON_LOG_FILTER",
            "ZIRCON_LOG",
            "RUST_LOG",
            "ZIRCON_LOG_LEVEL",
        ] {
            assert!(
                super::RUNTIME_SESSION_STARTUP_HELP.contains(expected),
                "runtime help should mention `{expected}`"
            );
        }
    }

    #[test]
    fn runtime_session_args_reject_duplicate_project_roots() {
        let error = parse_runtime_session_startup_args([
            "--project=examples/vampire".to_string(),
            "--project".to_string(),
            "examples/other".to_string(),
        ])
        .unwrap_err();

        assert_eq!(
            error.to_string(),
            "runtime startup diagnostic: component=runtime_app argument=--project requested=<multiple> cause=project root was provided more than once recovery=provide exactly one project root after --project"
        );
    }

    #[test]
    fn runtime_session_args_reject_missing_project_root() {
        let error = parse_runtime_session_startup_args(["--project".to_string()]).unwrap_err();

        assert_eq!(
            error.to_string(),
            "runtime startup diagnostic: component=runtime_app argument=--project requested=<missing> cause=missing project root path recovery=provide an existing Zircon project root after --project"
        );
    }

    #[test]
    fn runtime_session_args_reject_empty_project_root() {
        for args in [
            vec!["--project=".to_string()],
            vec!["--project".to_string(), "  ".to_string()],
            vec!["--project=  ".to_string()],
        ] {
            let error = parse_runtime_session_startup_args(args).unwrap_err();

            assert_eq!(
                error.to_string(),
                "runtime startup diagnostic: component=runtime_app argument=--project requested=<empty> cause=missing project root path recovery=provide an existing Zircon project root after --project"
            );
        }
    }

    #[test]
    fn runtime_session_args_reject_duplicate_profiles() {
        let error = parse_runtime_session_startup_args([
            "--runtime-session-profile=dev".to_string(),
            "--runtime-session-profile".to_string(),
            "runtime".to_string(),
        ])
        .unwrap_err();

        assert_eq!(
            error.to_string(),
            "runtime startup diagnostic: component=runtime_app argument=--runtime-session-profile requested=<multiple> cause=runtime session profile was provided more than once recovery=provide exactly one --runtime-session-profile value"
        );
    }

    #[test]
    fn runtime_session_args_reject_missing_profile_value() {
        let error = parse_runtime_session_startup_args(["--runtime-session-profile".to_string()])
            .unwrap_err();

        assert_eq!(
            error.to_string(),
            "runtime startup diagnostic: component=runtime_app argument=--runtime-session-profile requested=<missing> cause=missing runtime session profile recovery=provide runtime, runtime-pipelined, editor, dev, minimal, or headless after --runtime-session-profile"
        );
    }

    #[test]
    fn runtime_session_args_reject_empty_profile_value() {
        for args in [
            vec!["--runtime-session-profile=".to_string()],
            vec!["--runtime-session-profile".to_string(), "  ".to_string()],
            vec!["--runtime-session-profile=  ".to_string()],
        ] {
            let error = parse_runtime_session_startup_args(args).unwrap_err();

            assert_eq!(
                error.to_string(),
                "runtime startup diagnostic: component=runtime_app argument=--runtime-session-profile requested=<empty> cause=missing runtime session profile recovery=provide runtime, runtime-pipelined, editor, dev, minimal, or headless after --runtime-session-profile"
            );
        }
    }

    #[test]
    fn runtime_session_args_reject_unknown_profile_value() {
        let error = parse_runtime_session_startup_args([
            "--runtime-session-profile=debug-tools".to_string()
        ])
        .unwrap_err();

        assert_eq!(
            error.to_string(),
            "runtime startup diagnostic: component=runtime_app argument=--runtime-session-profile requested=debug-tools cause=unsupported runtime session profile recovery=choose runtime, runtime-pipelined, editor, dev, minimal, or headless"
        );
    }
}
