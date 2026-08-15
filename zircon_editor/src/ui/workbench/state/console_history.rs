use std::collections::VecDeque;
use std::sync::Arc;

use crate::core::editor_event::ConsoleMessageFilter;
use crate::ui::workbench::snapshot::{
    CONSOLE_OUTPUT_LOGICAL_LINE_CAPACITY, ConsoleOutputLevelCounts, ConsoleOutputSnapshot,
    EditorConsoleMessageLevel,
};

#[derive(Clone)]
pub(in crate::ui::workbench) struct EditorConsoleHistory {
    lines: VecDeque<EditorConsoleHistoryLine>,
    logical_line_count: usize,
    output: ConsoleOutputSnapshot,
    filter: ConsoleMessageFilter,
}

#[derive(Clone)]
struct EditorConsoleHistoryLine {
    message: String,
    level: EditorConsoleMessageLevel,
}

impl EditorConsoleHistory {
    pub(in crate::ui::workbench) fn new(initial_message: &str) -> Self {
        let mut history = Self {
            lines: VecDeque::with_capacity(CONSOLE_OUTPUT_LOGICAL_LINE_CAPACITY),
            logical_line_count: 0,
            output: ConsoleOutputSnapshot::default(),
            filter: ConsoleMessageFilter::All,
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
        if self
            .lines
            .back()
            .is_some_and(|last| last.message == message && last.level == level)
        {
            return;
        }
        self.logical_line_count += logical_line_count(message);
        self.lines.push_back(EditorConsoleHistoryLine {
            message: message.to_owned(),
            level,
        });
        self.trim_to_logical_line_capacity();
        self.rebuild_output();
    }

    fn trim_to_logical_line_capacity(&mut self) {
        let mut excess = self
            .logical_line_count
            .saturating_sub(CONSOLE_OUTPUT_LOGICAL_LINE_CAPACITY);
        while excess > 0 {
            let Some(front) = self.lines.front_mut() else {
                break;
            };
            let front_line_count = logical_line_count(&front.message);
            if front_line_count <= excess {
                self.lines.pop_front();
                self.logical_line_count -= front_line_count;
                excess -= front_line_count;
                continue;
            }

            let retained_start = byte_offset_after_logical_lines(&front.message, excess);
            front.message.drain(..retained_start);
            self.logical_line_count -= excess;
            excess = 0;
        }
        debug_assert!(self.logical_line_count <= CONSOLE_OUTPUT_LOGICAL_LINE_CAPACITY);
    }

    pub(super) fn output(&self) -> ConsoleOutputSnapshot {
        self.output.clone()
    }

    pub(super) fn set_filter(&mut self, filter: ConsoleMessageFilter) -> bool {
        if self.filter == filter {
            return false;
        }
        self.filter = filter;
        self.rebuild_output();
        true
    }

    pub(super) fn clear(&mut self) -> bool {
        if self.lines.is_empty() {
            return false;
        }
        self.lines.clear();
        self.logical_line_count = 0;
        self.output = ConsoleOutputSnapshot::filtered(
            Arc::from(""),
            Arc::from([]),
            ConsoleOutputLevelCounts::default(),
            self.filter,
        );
        true
    }

    fn rebuild_output(&mut self) {
        let mut output_len = 0;
        let mut visible_message_count = 0usize;
        let mut visible_logical_line_count = 0;
        let mut counts = ConsoleOutputLevelCounts::default();
        for line in &self.lines {
            let logical_line_count = logical_line_count(&line.message);
            match line.level {
                EditorConsoleMessageLevel::Info => counts.info += logical_line_count,
                EditorConsoleMessageLevel::Warning => counts.warning += logical_line_count,
                EditorConsoleMessageLevel::Error => counts.error += logical_line_count,
            }
            if message_filter_matches(self.filter, line.level) {
                output_len += line.message.len();
                visible_message_count += 1;
                visible_logical_line_count += logical_line_count;
            }
        }
        output_len += visible_message_count.saturating_sub(1);

        let mut output = String::with_capacity(output_len);
        let mut levels = Vec::with_capacity(visible_logical_line_count);
        let mut has_visible_message = false;
        for line in &self.lines {
            let logical_line_count = logical_line_count(&line.message);
            if !message_filter_matches(self.filter, line.level) {
                continue;
            }
            if has_visible_message {
                output.push('\n');
            }
            has_visible_message = true;
            output.push_str(&line.message);
            levels.extend(std::iter::repeat_n(line.level, logical_line_count));
        }
        self.output = ConsoleOutputSnapshot::filtered(
            Arc::from(output),
            Arc::from(levels),
            counts,
            self.filter,
        );
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
