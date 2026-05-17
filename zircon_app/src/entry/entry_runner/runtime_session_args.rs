use std::error::Error;

const RUNTIME_SESSION_PROFILE_ARG: &str = "--runtime-session-profile";
const RUNTIME_SESSION_HELP_ARG: &str = "--help";
const RUNTIME_SESSION_SHORT_HELP_ARG: &str = "-h";

pub(super) const RUNTIME_SESSION_STARTUP_HELP: &str = "\
Usage: zircon_runtime [OPTIONS]

Options:
  --runtime-session-profile <profile>   Select runtime, editor, dev, minimal, or headless dynamic session policy
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

Profiles:
  runtime                               Default runtime preview policy
  editor                                Editor-host policy accepted by the runtime ABI
  dev                                   Runtime-owned dev diagnostics, including diagnostic-store log cadence
  minimal                               Minimal runtime session policy
  headless                              Headless runtime session policy
";

#[derive(Clone, Debug, PartialEq, Eq)]
pub(super) struct RuntimeSessionStartupArgs {
    pub(super) profile: RuntimeSessionProfile,
    pub(super) help_requested: bool,
    pub(super) remaining_args: Vec<String>,
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub(super) enum RuntimeSessionProfile {
    #[default]
    Runtime,
    Editor,
    Dev,
    Minimal,
    Headless,
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
    let mut help_requested = false;
    let mut args = args.into_iter().map(Into::into);

    while let Some(arg) = args.next() {
        if arg == RUNTIME_SESSION_HELP_ARG || arg == RUNTIME_SESSION_SHORT_HELP_ARG {
            help_requested = true;
            continue;
        }

        if arg == RUNTIME_SESSION_PROFILE_ARG {
            if profile_provided {
                return Err(
                    format!("{RUNTIME_SESSION_PROFILE_ARG} was provided more than once").into(),
                );
            }
            let Some(value) = args.next() else {
                return Err(missing_profile_value_error().into());
            };
            profile = RuntimeSessionProfile::parse(value)?;
            profile_provided = true;
            continue;
        }

        if let Some(value) = arg.strip_prefix("--runtime-session-profile=") {
            if profile_provided {
                return Err(
                    format!("{RUNTIME_SESSION_PROFILE_ARG} was provided more than once").into(),
                );
            }
            if value.is_empty() {
                return Err(missing_profile_value_error().into());
            }
            profile = RuntimeSessionProfile::parse(value)?;
            profile_provided = true;
            continue;
        }

        remaining_args.push(arg);
    }

    Ok(RuntimeSessionStartupArgs {
        profile,
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
            Self::Editor => "editor",
            Self::Dev => "dev",
            Self::Minimal => "minimal",
            Self::Headless => "headless",
        }
    }

    fn parse(value: impl AsRef<str>) -> Result<Self, Box<dyn Error>> {
        match value.as_ref().trim().to_ascii_lowercase().as_str() {
            "runtime" => Ok(Self::Runtime),
            "editor" => Ok(Self::Editor),
            "dev" => Ok(Self::Dev),
            "minimal" => Ok(Self::Minimal),
            "headless" => Ok(Self::Headless),
            _ => Err(format!(
                "unknown runtime session profile `{}`; expected runtime, editor, dev, minimal, or headless",
                value.as_ref()
            )
            .into()),
        }
    }
}

fn missing_profile_value_error() -> String {
    format!("{RUNTIME_SESSION_PROFILE_ARG} requires runtime, editor, dev, minimal, or headless")
}

#[cfg(test)]
mod tests {
    use super::{parse_runtime_session_startup_args, RuntimeSessionProfile};

    #[test]
    fn runtime_session_args_default_to_runtime_profile() {
        let parsed = parse_runtime_session_startup_args(["--log-level=debug".to_string()]).unwrap();

        assert_eq!(parsed.profile, RuntimeSessionProfile::Runtime);
        assert_eq!(parsed.profile.as_bytes(), b"runtime");
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
        assert_eq!(parsed.remaining_args, ["--leftover"]);
    }

    #[test]
    fn runtime_session_args_strip_equals_profile() {
        let parsed =
            parse_runtime_session_startup_args(["--runtime-session-profile=headless".to_string()])
                .unwrap();

        assert_eq!(parsed.profile, RuntimeSessionProfile::Headless);
        assert_eq!(parsed.profile.as_bytes(), b"headless");
        assert!(!parsed.help_requested);
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
            "editor",
            "dev",
            "minimal",
            "headless",
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
    fn runtime_session_args_reject_duplicate_profiles() {
        let error = parse_runtime_session_startup_args([
            "--runtime-session-profile=dev".to_string(),
            "--runtime-session-profile".to_string(),
            "runtime".to_string(),
        ])
        .unwrap_err();

        assert_eq!(
            error.to_string(),
            "--runtime-session-profile was provided more than once"
        );
    }

    #[test]
    fn runtime_session_args_reject_missing_profile_value() {
        let error = parse_runtime_session_startup_args(["--runtime-session-profile".to_string()])
            .unwrap_err();

        assert_eq!(
            error.to_string(),
            "--runtime-session-profile requires runtime, editor, dev, minimal, or headless"
        );
    }

    #[test]
    fn runtime_session_args_reject_unknown_profile_value() {
        let error = parse_runtime_session_startup_args([
            "--runtime-session-profile=debug-tools".to_string()
        ])
        .unwrap_err();

        assert_eq!(
            error.to_string(),
            "unknown runtime session profile `debug-tools`; expected runtime, editor, dev, minimal, or headless"
        );
    }
}
