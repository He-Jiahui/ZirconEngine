use zircon_runtime::core::{
    TaskDiagnosticCursor, TaskDiagnosticObservation, TaskDiagnosticSeverity, TaskDiagnosticSource,
    TASK_DIAGNOSTIC_MAX_BATCH_ENTRIES,
};

use super::RuntimeTaskDiagnosticProjectionReport;
use crate::core::logging::{EditorLogError, EditorLogService, LogEntry, LogSeverity, LogSource};

pub(crate) struct RuntimeTaskDiagnosticLogBridge {
    source: TaskDiagnosticSource,
    cursor: TaskDiagnosticCursor,
}

impl RuntimeTaskDiagnosticLogBridge {
    pub(crate) fn new(source: TaskDiagnosticSource) -> Self {
        let cursor = source.initial_cursor();
        Self { source, cursor }
    }

    pub(crate) fn pump(
        &mut self,
        logs: &EditorLogService,
        timestamp_frame: u64,
    ) -> Result<RuntimeTaskDiagnosticProjectionReport, EditorLogError> {
        let batch = self
            .source
            .read_after(self.cursor, TASK_DIAGNOSTIC_MAX_BATCH_ENTRIES);
        let requires_gap_record = batch.source_changed() || batch.dropped_count() > 0;
        let gap_record_count = usize::from(requires_gap_record);

        if requires_gap_record {
            let message = if batch.source_changed() {
                "runtime task diagnostic source changed; resumed from its retained cursor"
                    .to_owned()
            } else {
                format!(
                    "runtime task diagnostic retention dropped {} observations before editor consumption",
                    batch.dropped_count()
                )
            };
            logs.emit(LogEntry::new(
                LogSource::runtime(),
                LogSeverity::Warning,
                message,
                timestamp_frame,
                None,
            )?)?;
            self.cursor = batch.recovery_cursor();
        }

        let mut observation_count = 0;
        for observation in batch.observations() {
            logs.emit(observation_entry(observation, timestamp_frame)?)?;
            self.cursor = observation.next_cursor();
            observation_count += 1;
        }
        if batch.observations().is_empty() {
            self.cursor = batch.next_cursor();
        }

        Ok(RuntimeTaskDiagnosticProjectionReport::new(
            observation_count,
            gap_record_count,
            batch.dropped_count(),
            batch.has_more(),
        ))
    }
}

fn observation_entry(
    observation: &TaskDiagnosticObservation,
    timestamp_frame: u64,
) -> Result<LogEntry, EditorLogError> {
    let identity = observation.identity();
    let message = format!(
        "[task {identity}] {}: {}",
        observation.kind().as_str(),
        observation.message()
    );
    let severity = log_severity(observation.severity());
    LogEntry::new(
        LogSource::runtime(),
        severity,
        message,
        timestamp_frame,
        None,
    )
}

pub(super) const fn log_severity(severity: TaskDiagnosticSeverity) -> LogSeverity {
    match severity {
        TaskDiagnosticSeverity::Warning => LogSeverity::Warning,
        TaskDiagnosticSeverity::Error => LogSeverity::Error,
    }
}
