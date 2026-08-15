use std::sync::Arc;

use zircon_runtime_interface::{ScriptDiagnostic, ScriptDiagnosticSeverity};

use crate::core::logging::{
    EditorLogError, EditorLogService, LogEntry, LogJump, LogSeverity, LogSource,
};

use super::{ScriptBuildCompletion, ScriptBuildGeneration, ScriptBuildRequestId};

#[derive(Clone, Copy, Debug, Eq, Ord, PartialEq, PartialOrd)]
struct DiagnosticCursorKey {
    generation: ScriptBuildGeneration,
    request_id: ScriptBuildRequestId,
    step_index: usize,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
struct DiagnosticCursor {
    key: DiagnosticCursorKey,
    next_diagnostic_index: usize,
}

#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub struct ScriptDiagnosticProjectionReport {
    emitted_count: usize,
    duplicate_count: usize,
    stale: bool,
}

impl ScriptDiagnosticProjectionReport {
    pub const fn emitted_count(self) -> usize {
        self.emitted_count
    }

    pub const fn duplicate_count(self) -> usize {
        self.duplicate_count
    }

    pub const fn stale(self) -> bool {
        self.stale
    }
}

pub struct ScriptBuildDiagnosticsSink {
    log_service: Arc<EditorLogService>,
    cursor: Option<DiagnosticCursor>,
}

impl ScriptBuildDiagnosticsSink {
    pub fn new(log_service: Arc<EditorLogService>) -> Self {
        Self {
            log_service,
            cursor: None,
        }
    }

    pub fn project(
        &mut self,
        completion: &ScriptBuildCompletion,
        diagnostics: &[ScriptDiagnostic],
        timestamp_frame: u64,
    ) -> Result<ScriptDiagnosticProjectionReport, EditorLogError> {
        let key = DiagnosticCursorKey {
            generation: completion.generation(),
            request_id: completion.request_id(),
            step_index: completion.completed_step_index(),
        };
        let next_diagnostic_index = match self.cursor {
            Some(cursor) if key < cursor.key => {
                return Ok(ScriptDiagnosticProjectionReport {
                    stale: true,
                    ..ScriptDiagnosticProjectionReport::default()
                });
            }
            Some(cursor) if key == cursor.key => cursor.next_diagnostic_index,
            Some(_) | None => {
                self.cursor = Some(DiagnosticCursor {
                    key,
                    next_diagnostic_index: 0,
                });
                0
            }
        };
        let duplicate_count = next_diagnostic_index.min(diagnostics.len());
        let mut emitted_count = 0;
        for diagnostic in diagnostics.iter().skip(next_diagnostic_index) {
            let entry = diagnostic_entry(diagnostic, timestamp_frame)?;
            self.log_service.emit(entry)?;
            emitted_count += 1;
            self.cursor = Some(DiagnosticCursor {
                key,
                next_diagnostic_index: next_diagnostic_index + emitted_count,
            });
        }
        Ok(ScriptDiagnosticProjectionReport {
            emitted_count,
            duplicate_count,
            stale: false,
        })
    }

    pub fn cursor_generation(&self) -> Option<ScriptBuildGeneration> {
        self.cursor.map(|cursor| cursor.key.generation)
    }
}

fn diagnostic_entry(
    diagnostic: &ScriptDiagnostic,
    timestamp_frame: u64,
) -> Result<LogEntry, EditorLogError> {
    let severity = match diagnostic.severity {
        ScriptDiagnosticSeverity::Info => LogSeverity::Info,
        ScriptDiagnosticSeverity::Warning => LogSeverity::Warning,
        ScriptDiagnosticSeverity::Error => LogSeverity::Error,
    };
    let jump = diagnostic
        .location
        .as_ref()
        .map(|location| {
            LogJump::script_location(location.path.clone(), location.line, location.column)
        })
        .transpose()?;
    let message = format!(
        "[{}] {}: {}",
        diagnostic.code, diagnostic.module, diagnostic.message
    );
    LogEntry::new(
        LogSource::script_build(),
        severity,
        message,
        timestamp_frame,
        jump,
    )
}
