use super::parse_diagnostic_log_startup_args;
use zircon_runtime::diagnostic_log::{
    DiagnosticLogFilter, DiagnosticLogFilterConfig, DiagnosticLogLevel,
};

#[test]
fn diagnostic_log_startup_args_strip_space_separated_level() {
    let parsed = parse_diagnostic_log_startup_args([
        "--run".to_string(),
        "plugin-list".to_string(),
        "--log-level".to_string(),
        "warn".to_string(),
    ])
    .unwrap();

    assert_eq!(
        parsed.filter.minimum,
        DiagnosticLogFilter::Minimum(DiagnosticLogLevel::Warn)
    );
    assert_eq!(parsed.remaining_args, ["--run", "plugin-list"]);
}

#[test]
fn diagnostic_log_startup_args_strip_equals_level() {
    let parsed = parse_diagnostic_log_startup_args([
        "--log-level=debug".to_string(),
        "--run".to_string(),
        "plugin-list".to_string(),
    ])
    .unwrap();

    assert_eq!(
        parsed.filter.minimum,
        DiagnosticLogFilter::Minimum(DiagnosticLogLevel::Debug)
    );
    assert_eq!(parsed.remaining_args, ["--run", "plugin-list"]);
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
    let error = parse_diagnostic_log_startup_args(["--log-level=notice".to_string()]).unwrap_err();

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
    let error = parse_diagnostic_log_startup_args(["--log-filter==debug".to_string()]).unwrap_err();

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
