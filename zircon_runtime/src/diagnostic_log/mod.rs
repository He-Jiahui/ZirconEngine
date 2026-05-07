//! Lightweight process diagnostics mirrored to console and a per-run log file.

mod level;
mod platform;
mod sink;
mod timestamp;

pub use level::{
    DiagnosticLogFilter, DiagnosticLogLevel, DiagnosticLogLevelParseError, DIAGNOSTIC_LOG_LEVEL_ENV,
};
pub use sink::{
    diagnostic_log_allows, initialize_process_log, initialize_process_log_with_filter,
    initialize_process_log_with_location, initialize_process_log_with_location_and_filter,
    initialize_unity_process_log, initialize_unity_process_log_with_filter, write_debug_log,
    write_diagnostic_log, write_diagnostic_log_at, write_error, write_log, write_warn,
    DiagnosticLogLocation,
};
