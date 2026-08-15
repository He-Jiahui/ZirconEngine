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
pub use settings::{
    DiagnosticLogSettings, DiagnosticLogSinkSettings, LogSettings,
    DEFAULT_DIAGNOSTIC_LOG_BATCH_BYTES, DEFAULT_DIAGNOSTIC_LOG_BATCH_RECORDS,
    DEFAULT_DIAGNOSTIC_LOG_CRASH_FLUSH_TIMEOUT, DEFAULT_DIAGNOSTIC_LOG_FLUSH_INTERVAL,
    DEFAULT_DIAGNOSTIC_LOG_QUEUE_CAPACITY, DEFAULT_DIAGNOSTIC_LOG_SHUTDOWN_TIMEOUT,
};
pub(crate) use sink::{acquire_dynamic_unity_process_log, DynamicProcessLogLease};
pub use sink::{
    diagnostic_log_allows, diagnostic_log_allows_for_scope, diagnostic_log_sink_snapshot,
    flush_process_log, initialize_process_log, initialize_process_log_with_config,
    initialize_process_log_with_filter, initialize_process_log_with_location,
    initialize_process_log_with_location_and_filter, initialize_process_log_with_settings,
    initialize_unity_process_log, initialize_unity_process_log_with_config,
    initialize_unity_process_log_with_filter, install_process_log_panic_flush,
    shutdown_process_log, write_debug_log, write_debug_log_lazy, write_diagnostic_log,
    write_diagnostic_log_at, write_diagnostic_log_lazy, write_diagnostic_log_lazy_at, write_error,
    write_error_lazy, write_log, write_log_lazy, write_warn, write_warn_lazy,
    DiagnosticLogLocation, DiagnosticLogSinkSnapshot,
};
