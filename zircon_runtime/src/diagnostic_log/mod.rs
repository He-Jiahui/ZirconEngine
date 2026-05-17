//! Lightweight process diagnostics mirrored to console and a per-run log file.

mod diagnostics;
mod level;
mod platform;
mod settings;
mod sink;
mod timestamp;

pub use diagnostics::{
    format_diagnostic_store_snapshot, write_diagnostic_store_snapshot, DiagnosticStoreLogSchedule,
    DEFAULT_DIAGNOSTIC_STORE_LOG_WAIT,
};
pub use level::{
    DiagnosticLogFilter, DiagnosticLogFilterConfig, DiagnosticLogLevel,
    DiagnosticLogLevelParseError, DiagnosticLogModuleFilter, DIAGNOSTIC_LOG_ENV,
    DIAGNOSTIC_LOG_FILTER_ENV, DIAGNOSTIC_LOG_LEVEL_ENV, RUST_LOG_ENV,
};
pub use settings::{DiagnosticLogSettings, LogSettings};
pub use sink::{
    diagnostic_log_allows, diagnostic_log_allows_for_scope, initialize_process_log,
    initialize_process_log_with_config, initialize_process_log_with_filter,
    initialize_process_log_with_location, initialize_process_log_with_location_and_filter,
    initialize_process_log_with_settings, initialize_unity_process_log,
    initialize_unity_process_log_with_config, initialize_unity_process_log_with_filter,
    write_debug_log, write_diagnostic_log, write_diagnostic_log_at, write_error, write_log,
    write_warn, DiagnosticLogLocation,
};
