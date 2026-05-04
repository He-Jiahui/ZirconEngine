use std::error::Error;

use zircon_runtime::diagnostic_log::{DiagnosticLogFilter, DIAGNOSTIC_LOG_LEVEL_ENV};

#[derive(Clone, Debug, PartialEq, Eq)]
pub(super) struct DiagnosticLogStartupArgs {
    pub(super) filter: DiagnosticLogFilter,
    pub(super) remaining_args: Vec<String>,
}

pub(super) fn parse_diagnostic_log_startup_args<I, S>(
    args: I,
) -> Result<DiagnosticLogStartupArgs, Box<dyn Error>>
where
    I: IntoIterator<Item = S>,
    S: Into<String>,
{
    let mut remaining_args = Vec::new();
    let mut filter = DiagnosticLogFilter::from_env_or_default();
    let mut log_level_provided = false;
    let mut args = args.into_iter().map(Into::into);

    while let Some(arg) = args.next() {
        if arg == "--log-level" {
            if log_level_provided {
                return Err("--log-level was provided more than once".into());
            }
            let Some(value) = args.next() else {
                return Err(format!(
                    "--log-level requires verbose, debug, log, warn, error, or off; {DIAGNOSTIC_LOG_LEVEL_ENV} accepts the same values"
                )
                .into());
            };
            filter = DiagnosticLogFilter::parse(value)?;
            log_level_provided = true;
            continue;
        }

        if let Some(value) = arg.strip_prefix("--log-level=") {
            if log_level_provided {
                return Err("--log-level was provided more than once".into());
            }
            filter = DiagnosticLogFilter::parse(value)?;
            log_level_provided = true;
            continue;
        }

        remaining_args.push(arg);
    }

    Ok(DiagnosticLogStartupArgs {
        filter,
        remaining_args,
    })
}

#[cfg(test)]
mod tests {
    use super::parse_diagnostic_log_startup_args;
    use zircon_runtime::diagnostic_log::{DiagnosticLogFilter, DiagnosticLogLevel};

    #[test]
    fn diagnostic_log_startup_args_strip_space_separated_level() {
        let parsed = parse_diagnostic_log_startup_args([
            "--operation".to_string(),
            "Window.Layout.Reset".to_string(),
            "--log-level".to_string(),
            "warn".to_string(),
            "--headless".to_string(),
        ])
        .unwrap();

        assert_eq!(
            parsed.filter,
            DiagnosticLogFilter::Minimum(DiagnosticLogLevel::Warn)
        );
        assert_eq!(
            parsed.remaining_args,
            ["--operation", "Window.Layout.Reset", "--headless"]
        );
    }

    #[test]
    fn diagnostic_log_startup_args_strip_equals_level() {
        let parsed = parse_diagnostic_log_startup_args([
            "--log-level=debug".to_string(),
            "--list-operations".to_string(),
            "--headless".to_string(),
        ])
        .unwrap();

        assert_eq!(
            parsed.filter,
            DiagnosticLogFilter::Minimum(DiagnosticLogLevel::Debug)
        );
        assert_eq!(parsed.remaining_args, ["--list-operations", "--headless"]);
    }

    #[test]
    fn diagnostic_log_startup_args_reject_duplicate_levels() {
        let error = parse_diagnostic_log_startup_args([
            "--log-level=debug".to_string(),
            "--log-level".to_string(),
            "warn".to_string(),
        ])
        .unwrap_err();

        assert_eq!(error.to_string(), "--log-level was provided more than once");
    }

    #[test]
    fn diagnostic_log_startup_args_reject_missing_level_value() {
        let error = parse_diagnostic_log_startup_args(["--log-level".to_string()]).unwrap_err();

        assert_eq!(
            error.to_string(),
            "--log-level requires verbose, debug, log, warn, error, or off; ZIRCON_LOG_LEVEL accepts the same values"
        );
    }
}
