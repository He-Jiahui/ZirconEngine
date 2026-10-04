//! Shared parser for process diagnostic logging arguments.
//! 在各产品入口之前统一处理日志过滤，其余参数继续由编辑器、Runtime 或无头 runner 解释。

use std::{
    error::Error,
    fmt::{self, Display, Formatter},
};

use zircon_runtime::diagnostic_log::{DiagnosticLogFilter, DiagnosticLogFilterConfig};

#[derive(Clone, Debug, PartialEq, Eq)]
pub(crate) struct DiagnosticLogStartupArgs {
    pub(crate) filter: DiagnosticLogFilterConfig,
    pub(crate) remaining_args: Vec<String>,
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

/// 先于产品参数和动态库加载调用；显式日志级别覆盖环境最小级别，未知参数必须保留给下一层。
pub(crate) fn parse_diagnostic_log_startup_args<I, S>(
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
#[path = "tests/diagnostic_log_args.rs"]
mod tests;
