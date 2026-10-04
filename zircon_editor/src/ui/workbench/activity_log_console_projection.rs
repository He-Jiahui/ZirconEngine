use std::sync::Arc;

use crate::core::editor_event::{ConsoleMessageFilter, ConsoleSourceFilter};
use crate::core::logging::{
    EditorLogService, LogChannel, LogFilter, LogRecord, LogSeverity, LogTailIdentity,
};
use crate::ui::workbench::snapshot::{
    ConsoleOutputLevelCounts, ConsoleOutputLineDelta, ConsoleOutputLineGeneration,
    ConsoleOutputLineSnapshot, ConsoleOutputSnapshot, EditorConsoleMessageLevel,
    CONSOLE_OUTPUT_LOGICAL_LINE_CAPACITY,
};

const ACTIVITY_LOG_JUMP_ACTION_PREFIX: &str = "workbench.activity_log.jump.";

#[derive(Default)]
pub(crate) struct ActivityLogConsoleProjection {
    initialized: bool,
    message_filter: ConsoleMessageFilter,
    source_filter: ConsoleSourceFilter,
    tail_identity: Option<LogTailIdentity>,
    output: ConsoleOutputSnapshot,
}

impl ActivityLogConsoleProjection {
    pub(crate) fn project(
        &mut self,
        logs: &EditorLogService,
        message_filter: ConsoleMessageFilter,
        source_filter: ConsoleSourceFilter,
    ) -> ConsoleOutputSnapshot {
        let filter = activity_log_filter(message_filter, source_filter);
        let same_filter = self.initialized
            && self.message_filter == message_filter
            && self.source_filter == source_filter;
        let known_identity = if same_filter {
            self.tail_identity
        } else {
            None
        };
        let (tail_identity, records) = logs.snapshot_tail_if_changed(
            &filter,
            CONSOLE_OUTPUT_LOGICAL_LINE_CAPACITY,
            known_identity,
        );

        let next = match records {
            None => self.output.clone(),
            Some(records) if same_filter && self.matches_cached_generation(&records) => {
                self.output.clone()
            }
            Some(records) if same_filter => self
                .append_projection(&records, message_filter, source_filter)
                .unwrap_or_else(|| self.full_projection(&records, message_filter, source_filter)),
            Some(records) => self.full_projection(&records, message_filter, source_filter),
        };

        self.initialized = true;
        self.message_filter = message_filter;
        self.source_filter = source_filter;
        self.tail_identity = Some(tail_identity);
        self.output = next.clone();
        next
    }

    fn matches_cached_generation(&self, records: &[LogRecord]) -> bool {
        let generation = self.output.line_generation();
        if generation.len() != records.len() {
            return false;
        }
        match (records.first(), records.last()) {
            (None, None) => true,
            (Some(first), Some(last)) => {
                generation.first().map(ConsoleOutputLineSnapshot::source_id)
                    == Some(first.sequence())
                    && generation
                        .get(generation.len().saturating_sub(1))
                        .map(ConsoleOutputLineSnapshot::source_id)
                        == Some(last.sequence())
            }
            _ => false,
        }
    }

    fn append_projection(
        &self,
        records: &[LogRecord],
        message_filter: ConsoleMessageFilter,
        source_filter: ConsoleSourceFilter,
    ) -> Option<ConsoleOutputSnapshot> {
        let previous = self.output.line_generation();
        let previous_last = previous.get(previous.len().checked_sub(1)?)?.source_id();
        let first_sequence = records.first()?.sequence();
        let entered_start = records.partition_point(|record| record.sequence() <= previous_last);
        if entered_start == 0
            || records.get(entered_start - 1)?.sequence() != previous_last
            || records.last()?.sequence() <= previous_last
        {
            return None;
        }

        let (trimmed, expired_before_append) = previous.trim_before_source_id(first_sequence);
        if trimmed.len() != entered_start
            || trimmed.first().map(ConsoleOutputLineSnapshot::source_id)
                != records.first().map(LogRecord::sequence)
            || trimmed
                .get(trimmed.len().saturating_sub(1))
                .map(ConsoleOutputLineSnapshot::source_id)
                != records.get(entered_start - 1).map(LogRecord::sequence)
        {
            return None;
        }

        let mut entered = Vec::with_capacity(records.len().saturating_sub(entered_start));
        for record in &records[entered_start..] {
            entered.push(activity_log_line(record));
        }
        let (next, append_delta) =
            trimmed.append_bounded(entered, CONSOLE_OUTPUT_LOGICAL_LINE_CAPACITY);
        let total_expired = expired_before_append.saturating_add(append_delta.expired);
        let counts = ConsoleOutputLevelCounts::from_lines(&next);
        Some(ConsoleOutputSnapshot::from_line_generation(
            Arc::new(next),
            counts,
            message_filter,
            source_filter,
            ConsoleOutputLineDelta {
                entered: append_delta.entered,
                expired: total_expired,
                retained: previous.len().saturating_sub(total_expired),
            },
        ))
    }

    fn full_projection(
        &self,
        records: &[LogRecord],
        message_filter: ConsoleMessageFilter,
        source_filter: ConsoleSourceFilter,
    ) -> ConsoleOutputSnapshot {
        let mut lines = Vec::with_capacity(records.len());
        for record in records {
            lines.push(activity_log_line(record));
        }
        let generation = ConsoleOutputLineGeneration::from_lines(lines);
        let counts = ConsoleOutputLevelCounts::from_lines(&generation);
        let entered = generation.len();
        ConsoleOutputSnapshot::from_line_generation(
            Arc::new(generation),
            counts,
            message_filter,
            source_filter,
            ConsoleOutputLineDelta {
                entered,
                expired: self.output.logical_line_count(),
                retained: 0,
            },
        )
    }
}

fn activity_log_filter(
    message_filter: ConsoleMessageFilter,
    source_filter: ConsoleSourceFilter,
) -> LogFilter {
    let minimum_severity = match message_filter {
        ConsoleMessageFilter::All | ConsoleMessageFilter::Info => LogSeverity::Info,
        ConsoleMessageFilter::Warning => LogSeverity::Warning,
        ConsoleMessageFilter::Error => LogSeverity::Error,
    };
    LogFilter::from_channel(source_channel(source_filter), minimum_severity)
}

fn source_channel(filter: ConsoleSourceFilter) -> Option<LogChannel> {
    match filter {
        ConsoleSourceFilter::All => None,
        ConsoleSourceFilter::Editor => Some(LogChannel::Editor),
        ConsoleSourceFilter::Runtime => Some(LogChannel::Runtime),
        ConsoleSourceFilter::Play => Some(LogChannel::Play),
        ConsoleSourceFilter::Plugin => Some(LogChannel::Plugin),
        ConsoleSourceFilter::Import => Some(LogChannel::Import),
        ConsoleSourceFilter::ScriptBuild => Some(LogChannel::ScriptBuild),
    }
}

fn activity_log_line(record: &LogRecord) -> ConsoleOutputLineSnapshot {
    let jump_sequence = record.entry().jump().map(|_| record.sequence());
    let action_id = jump_sequence
        .map(|sequence| Arc::<str>::from(format!("{ACTIVITY_LOG_JUMP_ACTION_PREFIX}{sequence}")));
    ConsoleOutputLineSnapshot::new(
        record.sequence(),
        activity_log_row_text(record).into(),
        console_level(record.entry().severity()),
        jump_sequence,
        action_id,
    )
}

fn activity_log_row_text(record: &LogRecord) -> String {
    let message = record
        .entry()
        .message()
        .replace('\r', "\\r")
        .replace('\n', "\\n");
    format!(
        "#{} [frame {}] [{}] {message}",
        record.sequence(),
        record.entry().timestamp_frame(),
        record.entry().source()
    )
}

fn console_level(severity: LogSeverity) -> EditorConsoleMessageLevel {
    match severity {
        LogSeverity::Info => EditorConsoleMessageLevel::Info,
        LogSeverity::Warning => EditorConsoleMessageLevel::Warning,
        LogSeverity::Error => EditorConsoleMessageLevel::Error,
    }
}

#[cfg(test)]
#[path = "activity_log_console_projection/tests/capacity_tests.rs"]
mod capacity_tests;

#[cfg(test)]
#[path = "tests/activity_log_console_projection.rs"]
mod tests;
