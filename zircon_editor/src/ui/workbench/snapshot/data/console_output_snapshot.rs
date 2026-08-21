use std::fmt;
use std::ops::Deref;
use std::sync::Arc;

use crate::core::editor_event::{ConsoleMessageFilter, ConsoleSourceFilter};

pub(crate) const CONSOLE_OUTPUT_LOGICAL_LINE_CAPACITY: usize = 256;

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum EditorConsoleMessageLevel {
    #[default]
    Info,
    Warning,
    Error,
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct ConsoleOutputLevelCounts {
    pub info: usize,
    pub warning: usize,
    pub error: usize,
}

impl ConsoleOutputLevelCounts {
    pub const fn total(self) -> usize {
        self.info + self.warning + self.error
    }

    fn from_levels(levels: &[EditorConsoleMessageLevel]) -> Self {
        let mut counts = Self::default();
        for level in levels {
            match level {
                EditorConsoleMessageLevel::Info => counts.info += 1,
                EditorConsoleMessageLevel::Warning => counts.warning += 1,
                EditorConsoleMessageLevel::Error => counts.error += 1,
            }
        }
        counts
    }
}

#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct ConsoleOutputSnapshot {
    text: Arc<str>,
    levels: Arc<[EditorConsoleMessageLevel]>,
    counts: ConsoleOutputLevelCounts,
    filter: ConsoleMessageFilter,
    source_filter: ConsoleSourceFilter,
    jump_sequences: Arc<[Option<u64>]>,
}

impl ConsoleOutputSnapshot {
    pub(crate) fn new(text: Arc<str>, levels: Arc<[EditorConsoleMessageLevel]>) -> Self {
        debug_assert!(text_and_levels_align(text.as_ref(), levels.as_ref()));
        let (text, levels) = bounded_output(text, levels);
        let counts = ConsoleOutputLevelCounts::from_levels(levels.as_ref());
        let jump_sequences = Arc::from(vec![None; levels.len()]);
        Self {
            text,
            levels,
            counts,
            filter: ConsoleMessageFilter::All,
            source_filter: ConsoleSourceFilter::All,
            jump_sequences,
        }
    }

    pub(crate) fn filtered(
        text: Arc<str>,
        levels: Arc<[EditorConsoleMessageLevel]>,
        counts: ConsoleOutputLevelCounts,
        filter: ConsoleMessageFilter,
    ) -> Self {
        debug_assert!(text_and_levels_align(text.as_ref(), levels.as_ref()));
        let (text, levels) = bounded_output(text, levels);
        let jump_sequences = Arc::from(vec![None; levels.len()]);
        Self {
            text,
            levels,
            counts,
            filter,
            source_filter: ConsoleSourceFilter::All,
            jump_sequences,
        }
    }

    pub(crate) fn activity(
        text: Arc<str>,
        levels: Arc<[EditorConsoleMessageLevel]>,
        filter: ConsoleMessageFilter,
        source_filter: ConsoleSourceFilter,
        jump_sequences: Arc<[Option<u64>]>,
    ) -> Self {
        debug_assert!(text_and_levels_align(text.as_ref(), levels.as_ref()));
        debug_assert_eq!(levels.len(), jump_sequences.len());
        let (text, levels, jump_sequences) = bounded_activity_output(text, levels, jump_sequences);
        let counts = ConsoleOutputLevelCounts::from_levels(levels.as_ref());
        Self {
            text,
            levels,
            counts,
            filter,
            source_filter,
            jump_sequences,
        }
    }

    pub fn levels(&self) -> &[EditorConsoleMessageLevel] {
        self.levels.as_ref()
    }

    pub fn has_output(&self) -> bool {
        !self.levels.is_empty()
    }

    pub const fn counts(&self) -> ConsoleOutputLevelCounts {
        self.counts
    }

    pub const fn filter(&self) -> ConsoleMessageFilter {
        self.filter
    }

    pub const fn source_filter(&self) -> ConsoleSourceFilter {
        self.source_filter
    }

    pub fn jump_sequences(&self) -> &[Option<u64>] {
        self.jump_sequences.as_ref()
    }

    pub(crate) fn levels_arc(&self) -> Arc<[EditorConsoleMessageLevel]> {
        Arc::clone(&self.levels)
    }

    pub(crate) fn text_arc(&self) -> Arc<str> {
        Arc::clone(&self.text)
    }

    pub(crate) fn jump_sequences_arc(&self) -> Arc<[Option<u64>]> {
        Arc::clone(&self.jump_sequences)
    }
}

impl AsRef<str> for ConsoleOutputSnapshot {
    fn as_ref(&self) -> &str {
        self.text.as_ref()
    }
}

impl Deref for ConsoleOutputSnapshot {
    type Target = str;

    fn deref(&self) -> &Self::Target {
        self.text.as_ref()
    }
}

impl fmt::Display for ConsoleOutputSnapshot {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str(self.text.as_ref())
    }
}

impl From<&str> for ConsoleOutputSnapshot {
    fn from(text: &str) -> Self {
        let text = bounded_text_tail(text);
        let levels = vec![EditorConsoleMessageLevel::Info; logical_line_count(text)];
        Self::new(Arc::from(text), Arc::from(levels))
    }
}

impl From<String> for ConsoleOutputSnapshot {
    fn from(text: String) -> Self {
        let retained_start = bounded_text_tail_start(&text);
        if retained_start > 0 {
            return Self::from(&text[retained_start..]);
        }
        let levels = vec![EditorConsoleMessageLevel::Info; logical_line_count(&text)];
        Self::new(Arc::from(text), Arc::from(levels))
    }
}

fn logical_line_count(text: &str) -> usize {
    if text.is_empty() {
        0
    } else {
        text.as_bytes()
            .iter()
            .filter(|byte| **byte == b'\n')
            .count()
            + 1
    }
}

fn bounded_text_tail(text: &str) -> &str {
    &text[bounded_text_tail_start(text)..]
}

fn bounded_text_tail_start(text: &str) -> usize {
    let line_count = logical_line_count(text);
    if line_count <= CONSOLE_OUTPUT_LOGICAL_LINE_CAPACITY {
        return 0;
    }
    byte_offset_after_logical_lines(text, line_count - CONSOLE_OUTPUT_LOGICAL_LINE_CAPACITY)
}

fn bounded_output(
    text: Arc<str>,
    levels: Arc<[EditorConsoleMessageLevel]>,
) -> (Arc<str>, Arc<[EditorConsoleMessageLevel]>) {
    let line_count = if text.is_empty() {
        levels.len()
    } else {
        logical_line_count(text.as_ref())
    };
    if line_count <= CONSOLE_OUTPUT_LOGICAL_LINE_CAPACITY {
        return (text, levels);
    }

    let lines_to_drop = line_count - CONSOLE_OUTPUT_LOGICAL_LINE_CAPACITY;
    let retained_text_start = byte_offset_after_logical_lines(text.as_ref(), lines_to_drop);
    let retained_levels_start = levels.len() - CONSOLE_OUTPUT_LOGICAL_LINE_CAPACITY;
    (
        Arc::from(&text[retained_text_start..]),
        Arc::from(&levels[retained_levels_start..]),
    )
}

fn bounded_activity_output(
    text: Arc<str>,
    levels: Arc<[EditorConsoleMessageLevel]>,
    jump_sequences: Arc<[Option<u64>]>,
) -> (
    Arc<str>,
    Arc<[EditorConsoleMessageLevel]>,
    Arc<[Option<u64>]>,
) {
    let line_count = if text.is_empty() {
        levels.len()
    } else {
        logical_line_count(text.as_ref())
    };
    if line_count <= CONSOLE_OUTPUT_LOGICAL_LINE_CAPACITY {
        return (text, levels, jump_sequences);
    }

    let lines_to_drop = line_count - CONSOLE_OUTPUT_LOGICAL_LINE_CAPACITY;
    let retained_text_start = byte_offset_after_logical_lines(text.as_ref(), lines_to_drop);
    (
        Arc::from(&text[retained_text_start..]),
        Arc::from(&levels[lines_to_drop..]),
        Arc::from(&jump_sequences[lines_to_drop..]),
    )
}

fn text_and_levels_align(text: &str, levels: &[EditorConsoleMessageLevel]) -> bool {
    if text.is_empty() {
        levels.len() <= 1
    } else {
        logical_line_count(text) == levels.len()
    }
}

fn byte_offset_after_logical_lines(text: &str, line_count: usize) -> usize {
    if line_count == 0 {
        return 0;
    }
    text.match_indices('\n')
        .nth(line_count - 1)
        .map_or(text.len(), |(index, _)| index + 1)
}

#[cfg(test)]
mod tests {
    use std::sync::Arc;

    use crate::core::editor_event::ConsoleMessageFilter;

    use super::{
        ConsoleOutputSnapshot, EditorConsoleMessageLevel, CONSOLE_OUTPUT_LOGICAL_LINE_CAPACITY,
    };

    #[test]
    fn string_conversion_assigns_info_to_each_logical_line() {
        let output = ConsoleOutputSnapshot::from("ready\ncompiled\n");

        assert_eq!(output.as_ref(), "ready\ncompiled\n");
        assert_eq!(
            output.levels(),
            &[
                EditorConsoleMessageLevel::Info,
                EditorConsoleMessageLevel::Info,
                EditorConsoleMessageLevel::Info,
            ]
        );
        assert_eq!(output.counts().info, 3);
        assert_eq!(output.counts().total(), 3);
        assert_eq!(output.filter(), ConsoleMessageFilter::All);
        assert!(Arc::ptr_eq(&output.text, &output.text_arc()));
        assert!(ConsoleOutputSnapshot::from("").levels().is_empty());
    }

    #[test]
    fn snapshot_construction_bounds_direct_multiline_inputs() {
        let text = (0..(CONSOLE_OUTPUT_LOGICAL_LINE_CAPACITY + 44))
            .map(|index| format!("line {index}"))
            .collect::<Vec<_>>()
            .join("\n");

        let output = ConsoleOutputSnapshot::from(text);
        let lines = output.lines().collect::<Vec<_>>();

        assert_eq!(lines.len(), CONSOLE_OUTPUT_LOGICAL_LINE_CAPACITY);
        assert_eq!(lines.first().copied(), Some("line 44"));
        assert_eq!(lines.last().copied(), Some("line 299"));
        assert_eq!(output.levels().len(), CONSOLE_OUTPUT_LOGICAL_LINE_CAPACITY);
        assert_eq!(
            output.counts().total(),
            CONSOLE_OUTPUT_LOGICAL_LINE_CAPACITY
        );
    }

    #[test]
    fn snapshot_distinguishes_empty_history_from_one_blank_logical_line() {
        let empty = ConsoleOutputSnapshot::from("");
        let blank_line = ConsoleOutputSnapshot::new(
            Arc::from(""),
            Arc::from([EditorConsoleMessageLevel::Warning]),
        );

        assert!(!empty.has_output());
        assert!(blank_line.has_output());
        assert!(blank_line.is_empty());
        assert_eq!(blank_line.counts().warning, 1);
    }
}
