use std::fmt;
use std::ops::Deref;
use std::sync::{Arc, OnceLock};

use crate::core::editor_event::{ConsoleMessageFilter, ConsoleSourceFilter};

mod generation;

pub(crate) use generation::{
    ConsoleOutputLineDelta, ConsoleOutputLineGeneration, ConsoleOutputLineSnapshot,
};

pub(crate) const CONSOLE_OUTPUT_LOGICAL_LINE_CAPACITY: usize = 256;
pub(crate) const ACTIVITY_LOG_JUMP_ACTION_PREFIX: &str = "workbench.activity_log.jump.";

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

    pub(crate) fn add_level(&mut self, level: EditorConsoleMessageLevel) {
        match level {
            EditorConsoleMessageLevel::Info => self.info += 1,
            EditorConsoleMessageLevel::Warning => self.warning += 1,
            EditorConsoleMessageLevel::Error => self.error += 1,
        }
    }

    pub(crate) fn remove_level(&mut self, level: EditorConsoleMessageLevel) {
        let count = match level {
            EditorConsoleMessageLevel::Info => &mut self.info,
            EditorConsoleMessageLevel::Warning => &mut self.warning,
            EditorConsoleMessageLevel::Error => &mut self.error,
        };
        *count = count.saturating_sub(1);
    }

    pub(crate) fn from_lines(lines: &ConsoleOutputLineGeneration) -> Self {
        let mut counts = Self::default();
        for line in lines.iter() {
            counts.add_level(line.level());
        }
        counts
    }
}

#[derive(Debug)]
struct ConsoleOutputSnapshotData {
    lines: Arc<ConsoleOutputLineGeneration>,
    counts: ConsoleOutputLevelCounts,
    filter: ConsoleMessageFilter,
    source_filter: ConsoleSourceFilter,
    line_delta: ConsoleOutputLineDelta,
    flat_text: OnceLock<Arc<str>>,
    levels: OnceLock<Arc<[EditorConsoleMessageLevel]>>,
    jump_sequences: OnceLock<Arc<[Option<u64>]>>,
}

#[derive(Clone)]
pub struct ConsoleOutputSnapshot {
    data: Arc<ConsoleOutputSnapshotData>,
}

impl ConsoleOutputSnapshot {
    pub(crate) fn new(text: Arc<str>, levels: Arc<[EditorConsoleMessageLevel]>) -> Self {
        debug_assert!(text_and_levels_align(text.as_ref(), levels.as_ref()));
        let lines = generation_from_flat_parts(text.as_ref(), levels.as_ref(), &[]);
        let counts = ConsoleOutputLevelCounts::from_lines(&lines);
        let entered = lines.len();
        Self::from_line_generation(
            Arc::new(lines),
            counts,
            ConsoleMessageFilter::All,
            ConsoleSourceFilter::All,
            ConsoleOutputLineDelta {
                entered,
                expired: 0,
                retained: 0,
            },
        )
    }

    pub(crate) fn filtered(
        text: Arc<str>,
        levels: Arc<[EditorConsoleMessageLevel]>,
        counts: ConsoleOutputLevelCounts,
        filter: ConsoleMessageFilter,
    ) -> Self {
        debug_assert!(text_and_levels_align(text.as_ref(), levels.as_ref()));
        let lines = generation_from_flat_parts(text.as_ref(), levels.as_ref(), &[]);
        let entered = lines.len();
        Self::from_line_generation(
            Arc::new(lines),
            counts,
            filter,
            ConsoleSourceFilter::All,
            ConsoleOutputLineDelta {
                entered,
                expired: 0,
                retained: 0,
            },
        )
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
        let lines =
            generation_from_flat_parts(text.as_ref(), levels.as_ref(), jump_sequences.as_ref());
        let counts = ConsoleOutputLevelCounts::from_lines(&lines);
        let entered = lines.len();
        Self::from_line_generation(
            Arc::new(lines),
            counts,
            filter,
            source_filter,
            ConsoleOutputLineDelta {
                entered,
                expired: 0,
                retained: 0,
            },
        )
    }

    pub(crate) fn from_line_generation(
        lines: Arc<ConsoleOutputLineGeneration>,
        counts: ConsoleOutputLevelCounts,
        filter: ConsoleMessageFilter,
        source_filter: ConsoleSourceFilter,
        line_delta: ConsoleOutputLineDelta,
    ) -> Self {
        Self {
            data: Arc::new(ConsoleOutputSnapshotData {
                lines,
                counts,
                filter,
                source_filter,
                line_delta,
                flat_text: OnceLock::new(),
                levels: OnceLock::new(),
                jump_sequences: OnceLock::new(),
            }),
        }
    }

    pub fn levels(&self) -> &[EditorConsoleMessageLevel] {
        self.data
            .levels
            .get_or_init(|| {
                self.data
                    .lines
                    .iter()
                    .map(ConsoleOutputLineSnapshot::level)
                    .collect::<Vec<_>>()
                    .into()
            })
            .as_ref()
    }

    pub fn has_output(&self) -> bool {
        !self.data.lines.is_empty()
    }

    pub fn counts(&self) -> ConsoleOutputLevelCounts {
        self.data.counts
    }

    pub fn filter(&self) -> ConsoleMessageFilter {
        self.data.filter
    }

    pub fn source_filter(&self) -> ConsoleSourceFilter {
        self.data.source_filter
    }

    pub fn jump_sequences(&self) -> &[Option<u64>] {
        self.data
            .jump_sequences
            .get_or_init(|| {
                self.data
                    .lines
                    .iter()
                    .map(ConsoleOutputLineSnapshot::jump_sequence)
                    .collect::<Vec<_>>()
                    .into()
            })
            .as_ref()
    }

    pub(crate) fn levels_arc(&self) -> Arc<[EditorConsoleMessageLevel]> {
        Arc::clone(self.data.levels.get_or_init(|| {
            self.data
                .lines
                .iter()
                .map(ConsoleOutputLineSnapshot::level)
                .collect::<Vec<_>>()
                .into()
        }))
    }

    pub(crate) fn text_arc(&self) -> Arc<str> {
        Arc::clone(
            self.data
                .flat_text
                .get_or_init(|| flatten_lines(&self.data.lines)),
        )
    }

    pub(crate) fn jump_sequences_arc(&self) -> Arc<[Option<u64>]> {
        Arc::clone(self.data.jump_sequences.get_or_init(|| {
            self.data
                .lines
                .iter()
                .map(ConsoleOutputLineSnapshot::jump_sequence)
                .collect::<Vec<_>>()
                .into()
        }))
    }

    pub(crate) fn logical_line(&self, index: usize) -> Option<&ConsoleOutputLineSnapshot> {
        self.data.lines.get(index)
    }

    pub(crate) fn logical_line_count(&self) -> usize {
        self.data.lines.len()
    }

    pub(crate) fn line_generation(&self) -> Arc<ConsoleOutputLineGeneration> {
        Arc::clone(&self.data.lines)
    }

    pub(crate) fn line_delta(&self) -> ConsoleOutputLineDelta {
        self.data.line_delta
    }

    pub(crate) fn shares_logical_generation_with(&self, other: &Self) -> bool {
        Arc::ptr_eq(&self.data.lines, &other.data.lines)
    }

    pub(crate) fn shares_logical_storage_chunk_with(
        &self,
        other: &Self,
        self_index: usize,
        other_index: usize,
    ) -> bool {
        self.data
            .lines
            .shares_storage_chunk_with(&other.data.lines, self_index, other_index)
    }

    pub(crate) fn has_materialized_flat_text(&self) -> bool {
        self.data.flat_text.get().is_some()
    }

    fn flat_text(&self) -> &str {
        self.data
            .flat_text
            .get_or_init(|| flatten_lines(&self.data.lines))
            .as_ref()
    }
}

impl Default for ConsoleOutputSnapshot {
    fn default() -> Self {
        Self::from_line_generation(
            Arc::new(ConsoleOutputLineGeneration::default()),
            ConsoleOutputLevelCounts::default(),
            ConsoleMessageFilter::All,
            ConsoleSourceFilter::All,
            ConsoleOutputLineDelta::default(),
        )
    }
}

impl fmt::Debug for ConsoleOutputSnapshot {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter
            .debug_struct("ConsoleOutputSnapshot")
            .field("logical_line_count", &self.logical_line_count())
            .field("counts", &self.counts())
            .field("filter", &self.filter())
            .field("source_filter", &self.source_filter())
            .field("line_delta", &self.line_delta())
            .field("flat_text_materialized", &self.has_materialized_flat_text())
            .finish()
    }
}

impl PartialEq for ConsoleOutputSnapshot {
    fn eq(&self, other: &Self) -> bool {
        Arc::ptr_eq(&self.data, &other.data)
            || (self.counts() == other.counts()
                && self.filter() == other.filter()
                && self.source_filter() == other.source_filter()
                && self.data.lines == other.data.lines)
    }
}

impl Eq for ConsoleOutputSnapshot {}

impl AsRef<str> for ConsoleOutputSnapshot {
    fn as_ref(&self) -> &str {
        self.flat_text()
    }
}

impl Deref for ConsoleOutputSnapshot {
    type Target = str;

    fn deref(&self) -> &Self::Target {
        self.flat_text()
    }
}

impl fmt::Display for ConsoleOutputSnapshot {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str(self.flat_text())
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
        Self::from(text.as_str())
    }
}

fn generation_from_flat_parts(
    text: &str,
    levels: &[EditorConsoleMessageLevel],
    jump_sequences: &[Option<u64>],
) -> ConsoleOutputLineGeneration {
    if text.is_empty() && levels.is_empty() {
        return ConsoleOutputLineGeneration::default();
    }
    let line_count = logical_line_count(text).max(levels.len());
    let retained_start = line_count.saturating_sub(CONSOLE_OUTPUT_LOGICAL_LINE_CAPACITY);
    let lines = text
        .split('\n')
        .skip(retained_start)
        .enumerate()
        .map(|(retained_index, line)| {
            let source_index = retained_start + retained_index;
            let level = levels.get(source_index).copied().unwrap_or_default();
            let jump_sequence = jump_sequences.get(source_index).copied().flatten();
            let action_id = jump_sequence.map(|sequence| {
                Arc::<str>::from(format!("{ACTIVITY_LOG_JUMP_ACTION_PREFIX}{sequence}"))
            });
            ConsoleOutputLineSnapshot::new(
                source_index as u64,
                Arc::from(line),
                level,
                jump_sequence,
                action_id,
            )
        })
        .collect::<Vec<_>>();
    ConsoleOutputLineGeneration::from_lines(lines)
}

fn flatten_lines(lines: &ConsoleOutputLineGeneration) -> Arc<str> {
    if lines.is_empty() {
        return Arc::from("");
    }
    let text_len = lines
        .iter()
        .map(|line| line.raw_text().len())
        .sum::<usize>()
        .saturating_add(lines.len().saturating_sub(1));
    let mut text = String::with_capacity(text_len);
    for (index, line) in lines.iter().enumerate() {
        if index > 0 {
            text.push('\n');
        }
        text.push_str(line.raw_text());
    }
    text.into()
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
        assert!(Arc::ptr_eq(&output.text_arc(), &output.text_arc()));
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

    #[test]
    fn snapshot_preserves_raw_crlf_text_but_exposes_clean_presentation_lines() {
        let output = ConsoleOutputSnapshot::from("compile\r\nready");

        assert_eq!(output.as_ref(), "compile\r\nready");
        assert_eq!(
            output.logical_line(0).map(|line| line.text()),
            Some("compile")
        );
        assert_eq!(
            output.logical_line(1).map(|line| line.text()),
            Some("ready")
        );
    }
}
