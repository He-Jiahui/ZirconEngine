#[path = "config.rs"]
mod config;
#[path = "entry.rs"]
mod entry;
#[path = "error.rs"]
mod error;
#[path = "filter.rs"]
mod filter;
#[path = "jump.rs"]
mod jump;
#[path = "record.rs"]
mod record;
#[path = "rolling_file.rs"]
mod rolling_file;
#[path = "runtime_task_diagnostics/mod.rs"]
mod runtime_task_diagnostics;
#[path = "service.rs"]
mod service;
#[path = "severity.rs"]
mod severity;
#[path = "source.rs"]
mod source;
#[path = "store.rs"]
mod store;

#[cfg(test)]
#[path = "tests/cases.rs"]
mod tests;

pub use config::EditorLogConfig;
pub use entry::LogEntry;
pub use error::EditorLogError;
pub use filter::LogFilter;
pub use jump::{LogJump, LogJumpTarget};
pub use record::LogRecord;
pub use rolling_file::RollingFileLogSink;
pub(crate) use runtime_task_diagnostics::{
    RuntimeTaskDiagnosticLogBridge, RuntimeTaskDiagnosticProjectionReport,
};
pub use service::{EditorLogEventSink, EditorLogService, LogEventDelivery, LogWriteReport};
pub use severity::LogSeverity;
pub use source::{LogChannel, LogSource};
pub(crate) use store::LogTailIdentity;
pub use store::{EditorLogDiagnostics, EditorLogStore};
