use std::error::Error;
use std::num::NonZeroU64;

use crate::entry::cli::parse_diagnostic_log_startup_args;
use zircon_runtime::diagnostic_log::DiagnosticLogFilterConfig;

use super::super::runtime_session_args::{
    parse_runtime_session_startup_args, RuntimeSessionProfile, RuntimeSessionStartupArgs,
};

pub(super) const HELP: &str = "Usage: zircon_server [OPTIONS]\n\n  --project <path>                  Project root or zircon-project.toml\n  --runtime-session-profile headless\n  --ticks <positive-count>          Exit after successful simulation ticks\n  --play-scene <relative-path>      Project-relative scene\n  --log-level <level>               Process log level\n  --log-filter <filter>             Process log filter\n  -h, --help                       Print help without loading Runtime\n\nZIRCON_RUNTIME_LIBRARY selects the versioned Runtime DLL and BuildSet sidecar.";

pub(super) struct HeadlessArgs {
    pub(super) startup: RuntimeSessionStartupArgs,
    pub(super) tick_limit: Option<NonZeroU64>,
    pub(super) log_filter: DiagnosticLogFilterConfig,
    pub(super) help: bool,
}

pub(super) fn parse(
    args: impl IntoIterator<Item = impl Into<String>>,
) -> Result<HeadlessArgs, Box<dyn Error>> {
    let diagnostic = parse_diagnostic_log_startup_args(args)?;
    let mut args = diagnostic.remaining_args;
    if !args.iter().any(|arg| {
        arg == "--runtime-session-profile" || arg.starts_with("--runtime-session-profile=")
    }) {
        args.push("--runtime-session-profile=headless".to_owned());
    }
    let mut startup = parse_runtime_session_startup_args(args)?;
    let help = startup.help_requested;
    if startup.profile != RuntimeSessionProfile::Headless {
        return Err(std::io::Error::new(
            std::io::ErrorKind::InvalidInput,
            "zircon_server requires the headless session profile",
        )
        .into());
    }
    if startup.reference_cpu_presenter || startup.play_report_pipe.is_some() {
        return Err(std::io::Error::new(
            std::io::ErrorKind::InvalidInput,
            "desktop presenter and Play report pipe are unsupported by zircon_server",
        )
        .into());
    }
    if startup.play_scene.is_some() && startup.project_root.is_none() {
        return Err(std::io::Error::new(
            std::io::ErrorKind::InvalidInput,
            "--play-scene requires --project",
        )
        .into());
    }
    let mut remaining = std::mem::take(&mut startup.remaining_args).into_iter();
    let mut tick_limit = None;
    while let Some(arg) = remaining.next() {
        let value = if arg == "--ticks" {
            remaining.next().ok_or_else(|| {
                std::io::Error::new(
                    std::io::ErrorKind::InvalidInput,
                    "--ticks requires a positive decimal count",
                )
            })?
        } else if let Some(value) = arg.strip_prefix("--ticks=") {
            value.to_owned()
        } else {
            return Err(std::io::Error::new(
                std::io::ErrorKind::InvalidInput,
                format!("unknown headless argument: {arg}"),
            )
            .into());
        };
        if tick_limit.is_some()
            || value.is_empty()
            || !value.bytes().all(|byte| byte.is_ascii_digit())
        {
            return Err(std::io::Error::new(
                std::io::ErrorKind::InvalidInput,
                "--ticks must appear once with a positive decimal count",
            )
            .into());
        }
        tick_limit = Some(value.parse::<NonZeroU64>()?);
    }
    Ok(HeadlessArgs {
        startup,
        tick_limit,
        log_filter: diagnostic.filter,
        help,
    })
}
