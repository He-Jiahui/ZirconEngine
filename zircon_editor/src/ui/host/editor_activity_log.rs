use std::sync::Arc;

use crate::core::editor_event::{ConsoleMessageFilter, ConsoleSourceFilter};
use crate::core::editor_event::{
    EditorAssetEvent, EditorEvent, EditorEventRecord, EditorEventSource,
};
use crate::core::logging::{EditorLogService, LogJumpTarget};
use crate::ui::host::EditorHostEventController;
use crate::ui::workbench::shell_state::WorkbenchShellStateData;
use crate::ui::workbench::snapshot::ConsoleOutputSnapshot;
use crate::ui::workbench::ActivityLogConsoleProjection;

pub(crate) const ACTIVITY_LOG_JUMP_ACTION_PREFIX: &str = "workbench.activity_log.jump.";

#[derive(Clone, Debug, PartialEq, Eq)]
pub(crate) enum ActivityLogJumpAction {
    Asset(Arc<str>),
    ScriptLocation {
        path: Arc<str>,
        line: u32,
        column: u32,
    },
}

pub(crate) fn activity_log_console_output(
    logs: &EditorLogService,
    message_filter: ConsoleMessageFilter,
    source_filter: ConsoleSourceFilter,
) -> ConsoleOutputSnapshot {
    ActivityLogConsoleProjection::default().project(logs, message_filter, source_filter)
}

pub(crate) fn activity_log_console_output_for_shell(
    shell: &mut WorkbenchShellStateData,
) -> ConsoleOutputSnapshot {
    let manager = Arc::clone(&shell.manager);
    shell.activity_log_console_projection.project(
        manager.context().logs(),
        shell.console_message_filter,
        shell.console_source_filter,
    )
}

pub(crate) fn activity_log_jump_action_id(sequence: u64) -> String {
    format!("{ACTIVITY_LOG_JUMP_ACTION_PREFIX}{sequence}")
}

pub(crate) fn parse_activity_log_jump_action_id(action_id: &str) -> Option<u64> {
    action_id
        .strip_prefix(ACTIVITY_LOG_JUMP_ACTION_PREFIX)?
        .parse()
        .ok()
}

pub(crate) fn activity_log_jump_action(
    logs: &EditorLogService,
    sequence: u64,
) -> Result<Option<ActivityLogJumpAction>, String> {
    let record = logs
        .record(sequence)
        .ok_or_else(|| format!("Activity log record {sequence} is no longer retained"))?;
    Ok(record.entry().jump().map(|jump| match jump.target() {
        LogJumpTarget::Asset(path) => ActivityLogJumpAction::Asset(Arc::clone(path)),
        LogJumpTarget::ScriptLocation { path, line, column } => {
            ActivityLogJumpAction::ScriptLocation {
                path: Arc::clone(path),
                line: *line,
                column: *column,
            }
        }
    }))
}

impl EditorHostEventController {
    pub(crate) fn dispatch_activity_log_jump(
        &self,
        sequence: u64,
    ) -> Result<Option<EditorEventRecord>, String> {
        let Some(action) = activity_log_jump_action(self.context().logs(), sequence)? else {
            return Ok(None);
        };
        let (asset_locator, script_location) = match action {
            ActivityLogJumpAction::Asset(path) => (path, None),
            ActivityLogJumpAction::ScriptLocation { path, line, column } => {
                (Arc::clone(&path), Some((path, line, column)))
            }
        };
        let record = self
            .dispatch_event(
                EditorEventSource::RetainedHost,
                EditorEvent::Asset(EditorAssetEvent::OpenAsset {
                    asset_locator: asset_locator.to_string(),
                }),
            )
            .map_err(|error| error.to_string())?;
        let opened = record
            .result
            .value
            .as_ref()
            .and_then(|value| value.get("changed"))
            .and_then(serde_json::Value::as_bool)
            .unwrap_or(false);
        if opened {
            if let Some((path, line, column)) = script_location {
                self.shell()
                    .lock()
                    .state
                    .set_status_line(format!("Opened {path} at line {line}, column {column}"));
            }
        }
        Ok(Some(record))
    }
}

#[cfg(test)]
mod tests {
    use crate::core::editor_event::{ConsoleMessageFilter, ConsoleSourceFilter};
    use crate::core::logging::{
        EditorLogConfig, EditorLogService, LogEntry, LogJump, LogSeverity, LogSource,
    };
    use crate::ui::workbench::snapshot::CONSOLE_OUTPUT_LOGICAL_LINE_CAPACITY;

    use super::{
        activity_log_console_output, activity_log_jump_action, activity_log_jump_action_id,
        parse_activity_log_jump_action_id, ActivityLogJumpAction,
    };

    fn emit(
        logs: &EditorLogService,
        source: LogSource,
        severity: LogSeverity,
        message: &str,
        frame: u64,
        jump: Option<LogJump>,
    ) -> u64 {
        logs.emit(LogEntry::new(source, severity, message, frame, jump).unwrap())
            .unwrap()
            .record()
            .sequence()
    }

    #[test]
    fn projection_uses_the_canonical_source_and_severity_filter() {
        let logs = EditorLogService::default();
        emit(
            &logs,
            LogSource::editor(),
            LogSeverity::Error,
            "editor failed",
            10,
            None,
        );
        let runtime_sequence = emit(
            &logs,
            LogSource::runtime(),
            LogSeverity::Warning,
            "runtime fallback",
            11,
            None,
        );
        emit(
            &logs,
            LogSource::runtime(),
            LogSeverity::Info,
            "runtime ready",
            12,
            None,
        );

        let output = activity_log_console_output(
            &logs,
            ConsoleMessageFilter::Warning,
            ConsoleSourceFilter::Runtime,
        );

        assert_eq!(output.levels().len(), 1);
        assert!(output.contains(&format!("#{runtime_sequence} [frame 11] [runtime]")));
        assert!(output.contains("runtime fallback"));
        assert!(!output.contains("editor failed"));
        assert!(!output.contains("runtime ready"));
    }

    #[test]
    fn refreshed_projection_replaces_records_evicted_by_the_log_service() {
        let logs = EditorLogService::new(EditorLogConfig::new(2, 16 * 1024).unwrap());
        let first = emit(
            &logs,
            LogSource::editor(),
            LogSeverity::Info,
            "first",
            1,
            None,
        );
        emit(
            &logs,
            LogSource::editor(),
            LogSeverity::Info,
            "second",
            2,
            None,
        );
        let before =
            activity_log_console_output(&logs, ConsoleMessageFilter::All, ConsoleSourceFilter::All);
        assert!(before.contains(&format!("#{first} ")));

        let third = emit(
            &logs,
            LogSource::editor(),
            LogSeverity::Info,
            "third",
            3,
            None,
        );
        let after =
            activity_log_console_output(&logs, ConsoleMessageFilter::All, ConsoleSourceFilter::All);

        assert!(!after.contains(&format!("#{first} ")));
        assert!(after.contains(&format!("#{third} ")));
        assert_eq!(after.levels().len(), 2);
    }

    #[test]
    fn projection_materializes_only_the_visible_record_tail() {
        let retained_records = CONSOLE_OUTPUT_LOGICAL_LINE_CAPACITY + 44;
        let logs =
            EditorLogService::new(EditorLogConfig::new(retained_records, 64 * 1024).unwrap());
        let mut first_sequence = 0;
        let mut last_sequence = 0;
        for index in 0..retained_records {
            let sequence = emit(
                &logs,
                LogSource::editor(),
                LogSeverity::Info,
                &format!("record-{index}"),
                index as u64,
                None,
            );
            if index == 0 {
                first_sequence = sequence;
            }
            last_sequence = sequence;
        }

        let output =
            activity_log_console_output(&logs, ConsoleMessageFilter::All, ConsoleSourceFilter::All);

        assert_eq!(output.levels().len(), CONSOLE_OUTPUT_LOGICAL_LINE_CAPACITY);
        assert!(!output.contains(&format!("#{first_sequence} ")));
        assert!(output.contains(&format!("#{last_sequence} ")));
    }

    #[test]
    fn jump_actions_requery_typed_asset_and_script_targets() {
        let logs = EditorLogService::default();
        let asset_sequence = emit(
            &logs,
            LogSource::import(),
            LogSeverity::Warning,
            "asset issue",
            4,
            Some(LogJump::asset("res://materials/terrain.zmat").unwrap()),
        );
        let script_sequence = emit(
            &logs,
            LogSource::script_build(),
            LogSeverity::Error,
            "script issue",
            5,
            Some(LogJump::script_location("scripts/player.zs", 12, 7).unwrap()),
        );
        let plain_sequence = emit(
            &logs,
            LogSource::editor(),
            LogSeverity::Info,
            "plain",
            6,
            None,
        );

        assert_eq!(
            activity_log_jump_action(&logs, asset_sequence).unwrap(),
            Some(ActivityLogJumpAction::Asset(
                "res://materials/terrain.zmat".into()
            ))
        );
        assert_eq!(
            activity_log_jump_action(&logs, script_sequence).unwrap(),
            Some(ActivityLogJumpAction::ScriptLocation {
                path: "scripts/player.zs".into(),
                line: 12,
                column: 7,
            })
        );
        assert_eq!(
            activity_log_jump_action(&logs, plain_sequence).unwrap(),
            None
        );
        let action_id = activity_log_jump_action_id(script_sequence);
        assert_eq!(
            parse_activity_log_jump_action_id(&action_id),
            Some(script_sequence)
        );
        assert_eq!(parse_activity_log_jump_action_id("script issue:12:7"), None);
    }
}
