use std::sync::Arc;

use crate::core::editor_event::{ConsoleMessageFilter, ConsoleSourceFilter};
use crate::ui::workbench::snapshot::{
    ConsoleOutputLevelCounts, ConsoleOutputLineDelta, ConsoleOutputLineGeneration,
    ConsoleOutputLineSnapshot, ConsoleOutputSnapshot, EditorConsoleMessageLevel,
    CONSOLE_OUTPUT_LOGICAL_LINE_CAPACITY,
};

#[derive(Clone)]
pub(in crate::ui::workbench) struct EditorConsoleHistory {
    lines: Arc<ConsoleOutputLineGeneration>,
    output: ConsoleOutputSnapshot,
    counts: ConsoleOutputLevelCounts,
    filter: ConsoleMessageFilter,
    next_source_id: u64,
    next_visible_slot_id: u64,
    last_message_line_count: usize,
    last_message_level: EditorConsoleMessageLevel,
}

impl EditorConsoleHistory {
    pub(in crate::ui::workbench) fn new(initial_message: &str) -> Self {
        let mut history = Self {
            lines: Arc::new(ConsoleOutputLineGeneration::default()),
            output: ConsoleOutputSnapshot::default(),
            counts: ConsoleOutputLevelCounts::default(),
            filter: ConsoleMessageFilter::All,
            next_source_id: 0,
            next_visible_slot_id: 0,
            last_message_line_count: 0,
            last_message_level: EditorConsoleMessageLevel::Info,
        };
        history.push(initial_message);
        history
    }

    pub(super) fn push(&mut self, message: &str) {
        self.push_with_level(message, EditorConsoleMessageLevel::Info);
    }

    pub(super) fn push_with_level(&mut self, message: &str, level: EditorConsoleMessageLevel) {
        if message.trim().is_empty() {
            return;
        }
        let message = message_tail_with_max_lines(message, CONSOLE_OUTPUT_LOGICAL_LINE_CAPACITY);
        if self.matches_last_message(message, level) {
            return;
        }

        let entered_lines = message
            .split('\n')
            .map(|text| {
                let source_id = self.next_source_id;
                self.next_source_id = self.next_source_id.saturating_add(1);
                ConsoleOutputLineSnapshot::new(source_id, Arc::from(text), level, None, None)
            })
            .collect::<Vec<_>>();
        let retained_entered_start = entered_lines
            .len()
            .saturating_sub(CONSOLE_OUTPUT_LOGICAL_LINE_CAPACITY);
        let retained_entered = &entered_lines[retained_entered_start..];
        let previous_lines = Arc::clone(&self.lines);
        let (next_lines, all_delta) = previous_lines
            .append_bounded(entered_lines.clone(), CONSOLE_OUTPUT_LOGICAL_LINE_CAPACITY);
        self.update_counts(&previous_lines, retained_entered, all_delta);
        self.lines = Arc::new(next_lines);
        self.last_message_line_count = retained_entered.len();
        self.last_message_level = level;
        self.publish_visible_append(retained_entered, all_delta);
    }

    fn matches_last_message(&self, message: &str, level: EditorConsoleMessageLevel) -> bool {
        if self.last_message_line_count == 0 || self.last_message_level != level {
            return false;
        }
        let message_line_count = logical_line_count(message);
        if message_line_count != self.last_message_line_count
            || message_line_count > self.lines.len()
        {
            return false;
        }
        let start = self.lines.len() - message_line_count;
        self.lines
            .iter()
            .skip(start)
            .zip(message.split('\n'))
            .all(|(line, text)| line.raw_text() == text && line.level() == level)
    }

    fn update_counts(
        &mut self,
        previous_lines: &ConsoleOutputLineGeneration,
        entered_lines: &[ConsoleOutputLineSnapshot],
        delta: ConsoleOutputLineDelta,
    ) {
        if delta.expired == previous_lines.len() {
            self.counts = ConsoleOutputLevelCounts::default();
        } else {
            for index in 0..delta.expired {
                if let Some(line) = previous_lines.get(index) {
                    self.counts.remove_level(line.level());
                }
            }
        }
        for line in entered_lines {
            self.counts.add_level(line.level());
        }
    }

    fn publish_visible_append(
        &mut self,
        retained_entered: &[ConsoleOutputLineSnapshot],
        all_delta: ConsoleOutputLineDelta,
    ) {
        if self.filter == ConsoleMessageFilter::All {
            self.output = ConsoleOutputSnapshot::from_line_generation(
                Arc::clone(&self.lines),
                self.counts,
                self.filter,
                ConsoleSourceFilter::All,
                all_delta,
            );
            return;
        }

        let previous_visible = self.output.line_generation();
        let first_source_id = self
            .lines
            .first()
            .map(ConsoleOutputLineSnapshot::source_id)
            .unwrap_or(self.next_source_id);
        let (trimmed, expired) = previous_visible.trim_before_source_id(first_source_id);
        let filter = self.filter;
        let mut next_visible_slot_id = self.next_visible_slot_id;
        let matching = retained_entered
            .iter()
            .filter(|line| message_filter_matches(filter, line.level()))
            .cloned()
            .map(|line| {
                let slot_id = next_visible_slot_id;
                next_visible_slot_id = next_visible_slot_id.saturating_add(1);
                line.with_slot_id(slot_id)
            })
            .collect::<Vec<_>>();
        self.next_visible_slot_id = next_visible_slot_id;
        if expired == 0 && matching.is_empty() {
            self.output = ConsoleOutputSnapshot::from_line_generation(
                previous_visible,
                self.counts,
                self.filter,
                ConsoleSourceFilter::All,
                ConsoleOutputLineDelta::default(),
            );
            return;
        }
        let (next_visible, append_delta) =
            trimmed.append_bounded(matching, CONSOLE_OUTPUT_LOGICAL_LINE_CAPACITY);
        let total_expired = expired.saturating_add(append_delta.expired);
        self.output = ConsoleOutputSnapshot::from_line_generation(
            Arc::new(next_visible),
            self.counts,
            self.filter,
            ConsoleSourceFilter::All,
            ConsoleOutputLineDelta {
                entered: append_delta.entered,
                expired: total_expired,
                retained: previous_visible.len().saturating_sub(total_expired),
            },
        );
    }

    pub(super) fn output(&self) -> ConsoleOutputSnapshot {
        self.output.clone()
    }

    pub(super) fn set_filter(&mut self, filter: ConsoleMessageFilter) -> bool {
        if self.filter == filter {
            return false;
        }
        let previous_line_count = self.output.logical_line_count();
        self.filter = filter;
        let visible_lines = if filter == ConsoleMessageFilter::All {
            self.next_visible_slot_id = self.next_source_id;
            Arc::clone(&self.lines)
        } else {
            let lines = self
                .lines
                .iter()
                .filter(|line| message_filter_matches(filter, line.level()))
                .cloned()
                .enumerate()
                .map(|(slot_id, line)| line.with_slot_id(slot_id as u64))
                .collect::<Vec<_>>();
            self.next_visible_slot_id = lines.len() as u64;
            Arc::new(ConsoleOutputLineGeneration::from_lines(lines))
        };
        let entered = visible_lines.len();
        self.output = ConsoleOutputSnapshot::from_line_generation(
            visible_lines,
            self.counts,
            self.filter,
            ConsoleSourceFilter::All,
            ConsoleOutputLineDelta {
                entered,
                expired: previous_line_count,
                retained: 0,
            },
        );
        true
    }

    pub(super) fn clear(&mut self) -> bool {
        if self.lines.is_empty() {
            return false;
        }
        let expired = self.output.logical_line_count();
        self.lines = Arc::new(ConsoleOutputLineGeneration::default());
        self.counts = ConsoleOutputLevelCounts::default();
        self.last_message_line_count = 0;
        self.next_visible_slot_id = 0;
        self.output = ConsoleOutputSnapshot::from_line_generation(
            Arc::clone(&self.lines),
            self.counts,
            self.filter,
            ConsoleSourceFilter::All,
            ConsoleOutputLineDelta {
                entered: 0,
                expired,
                retained: 0,
            },
        );
        true
    }
}

fn message_filter_matches(filter: ConsoleMessageFilter, level: EditorConsoleMessageLevel) -> bool {
    matches!(filter, ConsoleMessageFilter::All)
        || matches!(
            (filter, level),
            (ConsoleMessageFilter::Info, EditorConsoleMessageLevel::Info)
                | (
                    ConsoleMessageFilter::Warning,
                    EditorConsoleMessageLevel::Warning
                )
                | (
                    ConsoleMessageFilter::Error,
                    EditorConsoleMessageLevel::Error
                )
        )
}

fn logical_line_count(message: &str) -> usize {
    message
        .as_bytes()
        .iter()
        .filter(|byte| **byte == b'\n')
        .count()
        + 1
}

fn message_tail_with_max_lines(message: &str, max_lines: usize) -> &str {
    let line_count = logical_line_count(message);
    if line_count <= max_lines {
        return message;
    }
    let lines_to_drop = line_count - max_lines;
    &message[byte_offset_after_logical_lines(message, lines_to_drop)..]
}

fn byte_offset_after_logical_lines(message: &str, line_count: usize) -> usize {
    if line_count == 0 {
        return 0;
    }
    message
        .match_indices('\n')
        .nth(line_count - 1)
        .map_or(message.len(), |(index, _)| index + 1)
}

#[cfg(test)]
#[path = "console_history/tests.rs"]
mod tests;
