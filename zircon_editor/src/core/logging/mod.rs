mod config;
mod entry;
mod error;
mod filter;
mod jump;
mod record;
mod rolling_file;
mod runtime_task_diagnostics;
mod service;
mod severity;
mod source;
mod store;

#[cfg(test)]
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
pub use store::{EditorLogDiagnostics, EditorLogStore};
