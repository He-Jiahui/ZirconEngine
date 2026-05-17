use std::error::Error;

use zircon_runtime::diagnostic_log::{
    DiagnosticLogFilter, DiagnosticLogFilterConfig, DIAGNOSTIC_LOG_ENV, DIAGNOSTIC_LOG_FILTER_ENV,
    DIAGNOSTIC_LOG_LEVEL_ENV, RUST_LOG_ENV,
};

#[derive(Clone, Debug, PartialEq, Eq)]
pub(super) struct DiagnosticLogStartupArgs {
    pub(super) filter: DiagnosticLogFilterConfig,
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
    let mut filter = DiagnosticLogFilterConfig::from_env_or_default();
    let mut log_level_provided = false;
    let mut log_filter_provided = false;
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
            filter.minimum = DiagnosticLogFilter::parse(value)?;
            log_level_provided = true;
            continue;
        }

        if let Some(value) = arg.strip_prefix("--log-level=") {
            if log_level_provided {
                return Err("--log-level was provided more than once".into());
            }
            filter.minimum = DiagnosticLogFilter::parse(value)?;
            log_level_provided = true;
            continue;
        }

        if arg == "--log-filter" {
            if log_filter_provided {
                return Err("--log-filter was provided more than once".into());
            }
            let Some(value) = args.next() else {
                return Err(format!(
                    "--log-filter requires a comma-separated filter such as warn,zircon_runtime::asset=debug; {DIAGNOSTIC_LOG_FILTER_ENV}, {DIAGNOSTIC_LOG_ENV}, and {RUST_LOG_ENV} accept the same values"
                )
                .into());
            };
            filter = DiagnosticLogFilterConfig::parse(value, filter.minimum)?;
            log_filter_provided = true;
            continue;
        }

        if let Some(value) = arg.strip_prefix("--log-filter=") {
            if log_filter_provided {
                return Err("--log-filter was provided more than once".into());
            }
            filter = DiagnosticLogFilterConfig::parse(value, filter.minimum)?;
            log_filter_provided = true;
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
    use zircon_runtime::diagnostic_log::{
        DiagnosticLogFilter, DiagnosticLogFilterConfig, DiagnosticLogLevel,
    };

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
            parsed.filter.minimum,
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
            parsed.filter.minimum,
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

    #[test]
    fn diagnostic_log_startup_args_strip_scoped_filter() {
        let parsed = parse_diagnostic_log_startup_args([
            "--log-level=warn".to_string(),
            "--log-filter".to_string(),
            "zircon_runtime::asset=debug".to_string(),
            "--headless".to_string(),
        ])
        .unwrap();

        assert_eq!(
            parsed.filter,
            DiagnosticLogFilterConfig {
                minimum: DiagnosticLogFilter::Minimum(DiagnosticLogLevel::Warn),
                module_filters: vec![zircon_runtime::diagnostic_log::DiagnosticLogModuleFilter {
                    scope_prefix: "zircon_runtime::asset".to_string(),
                    filter: DiagnosticLogFilter::Minimum(DiagnosticLogLevel::Debug),
                }],
            }
        );
        assert_eq!(parsed.remaining_args, ["--headless"]);
    }
}
