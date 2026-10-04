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
#[path = "tests/editor_activity_log.rs"]
mod tests;
