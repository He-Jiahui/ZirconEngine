use std::{
    error::Error,
    fmt::{self, Display, Formatter},
};

use zircon_runtime::diagnostic_log::{DiagnosticLogFilter, DiagnosticLogFilterConfig};

#[derive(Clone, Debug, PartialEq, Eq)]
pub(super) struct DiagnosticLogStartupArgs {
    pub(super) filter: DiagnosticLogFilterConfig,
    pub(super) remaining_args: Vec<String>,
}

#[derive(Debug)]
struct DiagnosticLogStartupArgumentError {
    argument: &'static str,
    requested: String,
    cause: &'static str,
    recovery: &'static str,
}

impl DiagnosticLogStartupArgumentError {
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

impl Display for DiagnosticLogStartupArgumentError {
    fn fmt(&self, formatter: &mut Formatter<'_>) -> fmt::Result {
        write!(
            formatter,
            "runtime startup diagnostic: component=entry_runner argument={} requested={} cause={} recovery={}",
            self.argument, self.requested, self.cause, self.recovery
        )
    }
}

impl Error for DiagnosticLogStartupArgumentError {}

fn duplicate_log_level_error() -> DiagnosticLogStartupArgumentError {
    DiagnosticLogStartupArgumentError::new(
        "--log-level",
        "<duplicate>",
        "log level was provided more than once",
        "provide --log-level exactly once",
    )
}

fn missing_log_level_value_error() -> DiagnosticLogStartupArgumentError {
    DiagnosticLogStartupArgumentError::new(
        "--log-level",
        "<missing>",
        "log level value is missing",
        "provide verbose, debug, log, warn, error, or off after --log-level",
    )
}

fn empty_log_level_value_error() -> DiagnosticLogStartupArgumentError {
    DiagnosticLogStartupArgumentError::new(
        "--log-level",
        "<empty>",
        "log level value is empty",
        "provide verbose, debug, log, warn, error, or off after --log-level",
    )
}

fn invalid_log_level_error(value: &str) -> DiagnosticLogStartupArgumentError {
    DiagnosticLogStartupArgumentError::new(
        "--log-level",
        value,
        "log level is not supported",
        "provide verbose, debug, log, warn, error, or off after --log-level",
    )
}

fn parse_log_level_value(
    value: &str,
) -> Result<DiagnosticLogFilter, DiagnosticLogStartupArgumentError> {
    if value.trim().is_empty() {
        return Err(empty_log_level_value_error());
    }
    DiagnosticLogFilter::parse(value).map_err(|_| invalid_log_level_error(value))
}

fn duplicate_log_filter_error() -> DiagnosticLogStartupArgumentError {
    DiagnosticLogStartupArgumentError::new(
        "--log-filter",
        "<duplicate>",
        "log filter was provided more than once",
        "provide --log-filter exactly once",
    )
}

fn missing_log_filter_value_error() -> DiagnosticLogStartupArgumentError {
    DiagnosticLogStartupArgumentError::new(
        "--log-filter",
        "<missing>",
        "log filter value is missing",
        "provide a comma-separated filter such as warn,zircon_runtime::asset=debug after --log-filter",
    )
}

fn empty_log_filter_value_error() -> DiagnosticLogStartupArgumentError {
    DiagnosticLogStartupArgumentError::new(
        "--log-filter",
        "<empty>",
        "log filter value is empty",
        "provide a comma-separated filter such as warn,zircon_runtime::asset=debug after --log-filter",
    )
}

fn invalid_log_filter_error(value: &str) -> DiagnosticLogStartupArgumentError {
    DiagnosticLogStartupArgumentError::new(
        "--log-filter",
        value,
        "log filter is not supported",
        "provide a comma-separated filter such as warn,zircon_runtime::asset=debug after --log-filter",
    )
}

fn parse_log_filter_value(
    value: &str,
    fallback_minimum: DiagnosticLogFilter,
) -> Result<DiagnosticLogFilterConfig, DiagnosticLogStartupArgumentError> {
    if value.trim().is_empty() {
        return Err(empty_log_filter_value_error());
    }
    DiagnosticLogFilterConfig::parse(value, fallback_minimum)
        .map_err(|_| invalid_log_filter_error(value))
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
                return Err(duplicate_log_level_error().into());
            }
            let Some(value) = args.next() else {
                return Err(missing_log_level_value_error().into());
            };
            filter.minimum = parse_log_level_value(&value)?;
            log_level_provided = true;
            continue;
        }

        if let Some(value) = arg.strip_prefix("--log-level=") {
            if log_level_provided {
                return Err(duplicate_log_level_error().into());
            }
            filter.minimum = parse_log_level_value(value)?;
            log_level_provided = true;
            continue;
        }

        if arg == "--log-filter" {
            if log_filter_provided {
                return Err(duplicate_log_filter_error().into());
            }
            let Some(value) = args.next() else {
                return Err(missing_log_filter_value_error().into());
            };
            filter = parse_log_filter_value(&value, filter.minimum)?;
            log_filter_provided = true;
            continue;
        }

        if let Some(value) = arg.strip_prefix("--log-filter=") {
            if log_filter_provided {
                return Err(duplicate_log_filter_error().into());
            }
            filter = parse_log_filter_value(value, filter.minimum)?;
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

        assert_eq!(
            error.to_string(),
            "runtime startup diagnostic: component=entry_runner argument=--log-level requested=<duplicate> cause=log level was provided more than once recovery=provide --log-level exactly once"
        );
    }

    #[test]
    fn diagnostic_log_startup_args_reject_missing_level_value() {
        let error = parse_diagnostic_log_startup_args(["--log-level".to_string()]).unwrap_err();

        assert_eq!(
            error.to_string(),
            "runtime startup diagnostic: component=entry_runner argument=--log-level requested=<missing> cause=log level value is missing recovery=provide verbose, debug, log, warn, error, or off after --log-level"
        );
    }

    #[test]
    fn diagnostic_log_startup_args_reject_invalid_equals_level() {
        let error =
            parse_diagnostic_log_startup_args(["--log-level=notice".to_string()]).unwrap_err();

        assert_eq!(
            error.to_string(),
            "runtime startup diagnostic: component=entry_runner argument=--log-level requested=notice cause=log level is not supported recovery=provide verbose, debug, log, warn, error, or off after --log-level"
        );
    }

    #[test]
    fn diagnostic_log_startup_args_reject_empty_equals_level() {
        let error = parse_diagnostic_log_startup_args(["--log-level=".to_string()]).unwrap_err();

        assert_eq!(
            error.to_string(),
            "runtime startup diagnostic: component=entry_runner argument=--log-level requested=<empty> cause=log level value is empty recovery=provide verbose, debug, log, warn, error, or off after --log-level"
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

    #[test]
    fn diagnostic_log_startup_args_reject_duplicate_filters() {
        let error = parse_diagnostic_log_startup_args([
            "--log-filter=warn".to_string(),
            "--log-filter".to_string(),
            "zircon_runtime::asset=debug".to_string(),
        ])
        .unwrap_err();

        assert_eq!(
            error.to_string(),
            "runtime startup diagnostic: component=entry_runner argument=--log-filter requested=<duplicate> cause=log filter was provided more than once recovery=provide --log-filter exactly once"
        );
    }

    #[test]
    fn diagnostic_log_startup_args_reject_missing_filter_value() {
        let error = parse_diagnostic_log_startup_args(["--log-filter".to_string()]).unwrap_err();

        assert_eq!(
            error.to_string(),
            "runtime startup diagnostic: component=entry_runner argument=--log-filter requested=<missing> cause=log filter value is missing recovery=provide a comma-separated filter such as warn,zircon_runtime::asset=debug after --log-filter"
        );
    }

    #[test]
    fn diagnostic_log_startup_args_reject_invalid_equals_filter() {
        let error =
            parse_diagnostic_log_startup_args(["--log-filter==debug".to_string()]).unwrap_err();

        assert_eq!(
            error.to_string(),
            "runtime startup diagnostic: component=entry_runner argument=--log-filter requested==debug cause=log filter is not supported recovery=provide a comma-separated filter such as warn,zircon_runtime::asset=debug after --log-filter"
        );
    }

    #[test]
    fn diagnostic_log_startup_args_reject_empty_equals_filter() {
        let error = parse_diagnostic_log_startup_args(["--log-filter=".to_string()]).unwrap_err();

        assert_eq!(
            error.to_string(),
            "runtime startup diagnostic: component=entry_runner argument=--log-filter requested=<empty> cause=log filter value is empty recovery=provide a comma-separated filter such as warn,zircon_runtime::asset=debug after --log-filter"
        );
    }
}
