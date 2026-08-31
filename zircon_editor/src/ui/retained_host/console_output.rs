use std::{ops::Range, rc::Rc};

use crate::ui::retained_host::primitives::SharedString;
use crate::ui::workbench::snapshot::{
    ConsoleOutputLineSnapshot, ConsoleOutputSnapshot, EditorConsoleMessageLevel,
};

mod viewport_size;

pub(in crate::ui::retained_host) use viewport_size::console_output_viewport_size;

pub(in crate::ui::retained_host) const CONSOLE_OUTPUT_LINE_HEIGHT: f32 = 18.0;
pub(in crate::ui::retained_host) const CONSOLE_OUTPUT_SEVERITY_SLOT_WIDTH: f32 = 64.0;
pub(in crate::ui::retained_host) const CONSOLE_OUTPUT_MIN_MESSAGE_SLOT_WIDTH: f32 = 32.0;
pub(in crate::ui::retained_host) const CONSOLE_OUTPUT_OVERSCAN_LINES: usize = 2;
pub(in crate::ui::retained_host) const CONSOLE_OUTPUT_BODY_CONTROL_ID: &str = "ConsoleBodySection";
pub(in crate::ui::retained_host) const CONSOLE_OUTPUT_LINE_PREFIX: &str = "ConsoleOutputLine";
pub(in crate::ui::retained_host) const CONSOLE_OUTPUT_SEVERITY_PREFIX: &str =
    "ConsoleOutputSeverity";
pub(in crate::ui::retained_host) const CONSOLE_OUTPUT_PROTOTYPE_CONTROL_ID: &str =
    "ConsoleOutputLinePrototype";

const CONSOLE_EMPTY_OUTPUT: &str = "No output yet";

#[derive(Clone, Copy, Debug, PartialEq)]
pub(in crate::ui::retained_host) struct ConsoleOutputViewport {
    pub x: f32,
    pub y: f32,
    pub width: f32,
    pub height: f32,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub(in crate::ui::retained_host) struct ConsoleOutputLogicalLine {
    text: SharedString,
    text_tone: SharedString,
    severity_text: Option<SharedString>,
    severity_tone: SharedString,
    dispatch_kind: SharedString,
    action_id: SharedString,
}

impl ConsoleOutputLogicalLine {
    pub(in crate::ui::retained_host) fn new(text: SharedString, text_tone: SharedString) -> Self {
        Self {
            text,
            text_tone,
            severity_text: None,
            severity_tone: SharedString::new(),
            dispatch_kind: SharedString::new(),
            action_id: SharedString::new(),
        }
    }

    pub(in crate::ui::retained_host) fn with_severity(
        mut self,
        text: SharedString,
        tone: SharedString,
    ) -> Self {
        self.severity_text = Some(text);
        self.severity_tone = tone;
        self
    }

    pub(in crate::ui::retained_host) fn with_action(
        mut self,
        dispatch_kind: SharedString,
        action_id: SharedString,
    ) -> Self {
        self.dispatch_kind = dispatch_kind;
        self.action_id = action_id;
        self
    }

    pub(in crate::ui::retained_host) fn text(&self) -> &str {
        self.text.as_str()
    }

    pub(in crate::ui::retained_host) fn text_tone(&self) -> &str {
        self.text_tone.as_str()
    }

    pub(in crate::ui::retained_host) fn severity_text(&self) -> Option<&str> {
        self.severity_text.as_deref()
    }

    pub(in crate::ui::retained_host) fn severity_tone(&self) -> &str {
        self.severity_tone.as_str()
    }

    pub(in crate::ui::retained_host) fn dispatch_kind(&self) -> &str {
        self.dispatch_kind.as_str()
    }

    pub(in crate::ui::retained_host) fn action_id(&self) -> &str {
        self.action_id.as_str()
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(in crate::ui::retained_host) enum ConsoleOutputSlotKind {
    Severity,
    Message,
}

#[derive(Clone, Copy, Debug)]
pub(in crate::ui::retained_host) struct ConsoleOutputSlotBinding<'a> {
    pub logical_index: usize,
    pub kind: ConsoleOutputSlotKind,
    pub line: ConsoleOutputLogicalLineRef<'a>,
}

#[derive(Clone, Copy, Debug)]
pub(in crate::ui::retained_host) enum ConsoleOutputLogicalLineRef<'a> {
    Owned(&'a ConsoleOutputLogicalLine),
    Snapshot(&'a ConsoleOutputLineSnapshot),
    Empty,
}

impl<'a> ConsoleOutputLogicalLineRef<'a> {
    pub(in crate::ui::retained_host) fn text(self) -> &'a str {
        match self {
            Self::Owned(line) => line.text(),
            Self::Snapshot(line) => line.text(),
            Self::Empty => CONSOLE_EMPTY_OUTPUT,
        }
    }

    pub(in crate::ui::retained_host) fn text_tone(self) -> &'a str {
        match self {
            Self::Owned(line) => line.text_tone(),
            Self::Snapshot(line) if line.action_id().is_some() => "accent",
            Self::Snapshot(_) => "secondary",
            Self::Empty => "muted",
        }
    }

    pub(in crate::ui::retained_host) fn severity_text(self) -> Option<&'a str> {
        match self {
            Self::Owned(line) => line.severity_text(),
            Self::Snapshot(line) => Some(console_output_level_label(line.level())),
            Self::Empty => None,
        }
    }

    pub(in crate::ui::retained_host) fn severity_tone(self) -> &'a str {
        match self {
            Self::Owned(line) => line.severity_tone(),
            Self::Snapshot(line) => console_output_text_tone(line.level()),
            Self::Empty => "",
        }
    }

    pub(in crate::ui::retained_host) fn dispatch_kind(self) -> &'a str {
        match self {
            Self::Owned(line) => line.dispatch_kind(),
            Self::Snapshot(line) if line.action_id().is_some() => "activity_log_jump",
            Self::Snapshot(_) | Self::Empty => "",
        }
    }

    pub(in crate::ui::retained_host) fn action_id(self) -> &'a str {
        match self {
            Self::Owned(line) => line.action_id(),
            Self::Snapshot(line) => line.action_id().unwrap_or_default(),
            Self::Empty => "",
        }
    }
}

#[derive(Clone, Debug)]
enum ConsoleOutputLogicalSource {
    Owned(Rc<[ConsoleOutputLogicalLine]>),
    Snapshot(ConsoleOutputSnapshot),
}

#[derive(Clone, Debug)]
pub(in crate::ui::retained_host) struct ConsoleOutputPaintMetadata {
    viewport: ConsoleOutputViewport,
    line_origin_y: f32,
    line_rows: Range<usize>,
    line_count: usize,
    nodes_per_line: usize,
    materialized_line_count: usize,
    overscan_lines: usize,
    logical_lines: Option<ConsoleOutputLogicalSource>,
}

impl ConsoleOutputPaintMetadata {
    pub(in crate::ui::retained_host) fn new(
        viewport: ConsoleOutputViewport,
        line_origin_y: f32,
        line_row_start: usize,
        line_count: usize,
    ) -> Option<Self> {
        Self::new_with_nodes_per_line(viewport, line_origin_y, line_row_start, line_count, 1)
    }

    pub(in crate::ui::retained_host) fn new_with_nodes_per_line(
        viewport: ConsoleOutputViewport,
        line_origin_y: f32,
        line_row_start: usize,
        line_count: usize,
        nodes_per_line: usize,
    ) -> Option<Self> {
        let line_node_count = line_count.checked_mul(nodes_per_line)?;
        (viewport.width > 0.0 && viewport.height > 0.0 && line_count > 0 && nodes_per_line > 0)
            .then_some(Self {
                viewport,
                line_origin_y,
                line_rows: line_row_start..line_row_start.saturating_add(line_node_count),
                line_count,
                nodes_per_line,
                materialized_line_count: line_count,
                overscan_lines: 0,
                logical_lines: None,
            })
    }

    pub(in crate::ui::retained_host) fn new_virtualized(
        viewport: ConsoleOutputViewport,
        line_origin_y: f32,
        line_row_start: usize,
        logical_lines: Vec<ConsoleOutputLogicalLine>,
        nodes_per_line: usize,
        overscan_lines: usize,
    ) -> Option<Self> {
        let line_count = logical_lines.len();
        if viewport.width <= 0.0 || viewport.height <= 0.0 || line_count == 0 || nodes_per_line == 0
        {
            return None;
        }
        let visible_line_capacity = (viewport.height / CONSOLE_OUTPUT_LINE_HEIGHT).ceil() as usize;
        let materialized_line_count = line_count.min(
            visible_line_capacity
                .saturating_add(1)
                .saturating_add(overscan_lines.saturating_mul(2)),
        );
        let line_node_count = materialized_line_count.checked_mul(nodes_per_line)?;
        Some(Self {
            viewport,
            line_origin_y,
            line_rows: line_row_start..line_row_start.saturating_add(line_node_count),
            line_count,
            nodes_per_line,
            materialized_line_count,
            overscan_lines,
            logical_lines: Some(ConsoleOutputLogicalSource::Owned(logical_lines.into())),
        })
    }

    pub(in crate::ui::retained_host) fn new_virtualized_snapshot(
        viewport: ConsoleOutputViewport,
        line_origin_y: f32,
        line_row_start: usize,
        output: ConsoleOutputSnapshot,
        nodes_per_line: usize,
        overscan_lines: usize,
    ) -> Option<Self> {
        let line_count = output.logical_line_count().max(1);
        if viewport.width <= 0.0 || viewport.height <= 0.0 || nodes_per_line == 0 {
            return None;
        }
        let visible_line_capacity = (viewport.height / CONSOLE_OUTPUT_LINE_HEIGHT).ceil() as usize;
        let materialized_line_count = line_count.min(
            visible_line_capacity
                .saturating_add(1)
                .saturating_add(overscan_lines.saturating_mul(2)),
        );
        let line_node_count = materialized_line_count.checked_mul(nodes_per_line)?;
        Some(Self {
            viewport,
            line_origin_y,
            line_rows: line_row_start..line_row_start.saturating_add(line_node_count),
            line_count,
            nodes_per_line,
            materialized_line_count,
            overscan_lines,
            logical_lines: Some(ConsoleOutputLogicalSource::Snapshot(output)),
        })
    }

    pub(in crate::ui::retained_host) fn replacing_snapshot(
        &self,
        output: ConsoleOutputSnapshot,
    ) -> Option<Self> {
        let replacement = Self::new_virtualized_snapshot(
            self.viewport,
            self.line_origin_y,
            self.line_rows.start,
            output,
            self.nodes_per_line,
            self.overscan_lines,
        )?;
        (replacement.materialized_line_count == self.materialized_line_count
            && replacement.nodes_per_line == self.nodes_per_line)
            .then_some(replacement)
    }

    pub(in crate::ui::retained_host) const fn viewport(&self) -> ConsoleOutputViewport {
        self.viewport
    }

    pub(in crate::ui::retained_host) fn content_extent(&self) -> f32 {
        self.line_count as f32 * CONSOLE_OUTPUT_LINE_HEIGHT
    }

    pub(in crate::ui::retained_host) const fn logical_line_count(&self) -> usize {
        self.line_count
    }

    pub(in crate::ui::retained_host) const fn materialized_line_count(&self) -> usize {
        self.materialized_line_count
    }

    pub(in crate::ui::retained_host) fn materialized_node_count(&self) -> usize {
        self.materialized_line_count
            .saturating_mul(self.nodes_per_line)
    }

    pub(in crate::ui::retained_host) const fn overscan_line_count(&self) -> usize {
        self.overscan_lines
    }

    pub(in crate::ui::retained_host) fn visible_logical_line_count(&self, scroll_px: f32) -> usize {
        self.visible_logical_line_range(scroll_px).len()
    }

    pub(in crate::ui::retained_host) const fn is_virtualized(&self) -> bool {
        self.logical_lines.is_some()
    }

    #[cfg(test)]
    pub(in crate::ui::retained_host) fn visible_node_rows(
        &self,
        row_count: usize,
        scroll_px: f32,
    ) -> Vec<usize> {
        let mut rows = Vec::new();
        self.stream_visible_node_rows(row_count, scroll_px, &mut |row| rows.push(row));
        rows
    }

    pub(in crate::ui::retained_host) fn stream_visible_node_rows(
        &self,
        row_count: usize,
        scroll_px: f32,
        visit: &mut dyn FnMut(usize),
    ) {
        let line_rows = self.line_node_rows(row_count);
        for row in 0..line_rows.start {
            visit(row);
        }
        for row in self.visible_line_node_rows(row_count, scroll_px) {
            visit(row);
        }
        for row in line_rows.end..row_count {
            visit(row);
        }
    }

    pub(in crate::ui::retained_host) fn visible_line_node_rows(
        &self,
        row_count: usize,
        scroll_px: f32,
    ) -> impl DoubleEndedIterator<Item = usize> + '_ {
        let line_start = self.line_rows.start.min(row_count);
        let line_end = self.line_rows.end.min(row_count).max(line_start);
        let nodes_per_line = self.nodes_per_line;
        let complete_line_end =
            line_start + ((line_end - line_start) / nodes_per_line).saturating_mul(nodes_per_line);
        let materialized_line_count = self.materialized_line_count.max(1);
        let virtualized = self.is_virtualized();
        self.visible_logical_line_range(scroll_px)
            .flat_map(move |logical_index| {
                let slot_index = if virtualized {
                    self.slot_index_for_logical_line(logical_index)
                } else {
                    logical_index
                };
                let start = line_start
                    .saturating_add(slot_index.saturating_mul(nodes_per_line))
                    .min(complete_line_end);
                let end = start.saturating_add(nodes_per_line).min(complete_line_end);
                start..end
            })
    }

    pub(in crate::ui::retained_host) fn logical_line_for_slot(
        &self,
        slot_index: usize,
        scroll_px: f32,
    ) -> Option<(usize, ConsoleOutputLogicalLineRef<'_>)> {
        let logical_lines = self.logical_lines.as_ref()?;
        let slot_count = self.materialized_line_count;
        if slot_count == 0 || slot_index >= slot_count {
            return None;
        }
        let window = self.materialized_logical_line_range(scroll_px);
        let first_slot = self.slot_index_for_logical_line(window.start);
        let slot_offset = (slot_index + slot_count - first_slot) % slot_count;
        let logical_index = window.start.saturating_add(slot_offset);
        if logical_index >= window.end {
            return None;
        }
        let line = match logical_lines {
            ConsoleOutputLogicalSource::Owned(lines) => {
                ConsoleOutputLogicalLineRef::Owned(lines.get(logical_index)?)
            }
            ConsoleOutputLogicalSource::Snapshot(output) => output
                .logical_line(logical_index)
                .map(ConsoleOutputLogicalLineRef::Snapshot)
                .or_else(|| {
                    (!output.has_output() && logical_index == 0)
                        .then_some(ConsoleOutputLogicalLineRef::Empty)
                })?,
        };
        Some((logical_index, line))
    }

    pub(in crate::ui::retained_host) fn slot_index_for_logical_line(
        &self,
        logical_index: usize,
    ) -> usize {
        let slot_count = self.materialized_line_count.max(1);
        let first_slot_id = match self.logical_lines.as_ref() {
            Some(ConsoleOutputLogicalSource::Snapshot(output)) => output
                .logical_line(0)
                .map(ConsoleOutputLineSnapshot::slot_id)
                .unwrap_or(0),
            Some(ConsoleOutputLogicalSource::Owned(_)) | None => 0,
        };
        ((first_slot_id % slot_count as u64) as usize + logical_index % slot_count) % slot_count
    }

    pub(in crate::ui::retained_host) fn slot_source_id(
        &self,
        slot_index: usize,
        scroll_px: f32,
    ) -> Option<u64> {
        let (logical_index, line) = self.logical_line_for_slot(slot_index, scroll_px)?;
        Some(match line {
            ConsoleOutputLogicalLineRef::Owned(_) => logical_index as u64,
            ConsoleOutputLogicalLineRef::Snapshot(line) => line.slot_id(),
            ConsoleOutputLogicalLineRef::Empty => 0,
        })
    }

    pub(in crate::ui::retained_host) const fn nodes_per_line(&self) -> usize {
        self.nodes_per_line
    }

    pub(in crate::ui::retained_host) const fn line_row_start(&self) -> usize {
        self.line_rows.start
    }

    pub(in crate::ui::retained_host) fn logical_line_for_node_row(
        &self,
        row: usize,
        row_count: usize,
        scroll_px: f32,
    ) -> Option<ConsoleOutputSlotBinding<'_>> {
        let line_rows = self.line_node_rows(row_count);
        if !line_rows.contains(&row) {
            return None;
        }
        let relative_row = row - line_rows.start;
        let complete_row_count =
            (line_rows.end - line_rows.start) / self.nodes_per_line * self.nodes_per_line;
        if relative_row >= complete_row_count {
            return None;
        }
        let slot_index = relative_row / self.nodes_per_line;
        let node_offset = relative_row % self.nodes_per_line;
        let (logical_index, line) = self.logical_line_for_slot(slot_index, scroll_px)?;
        let kind = if self.nodes_per_line > 1 && node_offset == 0 {
            ConsoleOutputSlotKind::Severity
        } else {
            ConsoleOutputSlotKind::Message
        };
        if kind == ConsoleOutputSlotKind::Severity && line.severity_text().is_none() {
            return None;
        }
        Some(ConsoleOutputSlotBinding {
            logical_index,
            kind,
            line,
        })
    }

    pub(in crate::ui::retained_host) fn line_frame_y(
        &self,
        logical_index: usize,
        scroll_px: f32,
    ) -> f32 {
        self.line_origin_y + logical_index as f32 * CONSOLE_OUTPUT_LINE_HEIGHT - scroll_px.max(0.0)
    }

    pub(in crate::ui::retained_host) fn line_node_rows(&self, row_count: usize) -> Range<usize> {
        let start = self.line_rows.start.min(row_count);
        start..self.line_rows.end.min(row_count).max(start)
    }

    pub(in crate::ui::retained_host) fn static_node_rows(
        &self,
        row_count: usize,
    ) -> impl DoubleEndedIterator<Item = usize> {
        let line_rows = self.line_node_rows(row_count);
        (0..line_rows.start).chain(line_rows.end..row_count)
    }

    fn visible_logical_line_range(&self, scroll_px: f32) -> Range<usize> {
        let scroll_px = scroll_px.max(0.0);
        let visible_start_px = (scroll_px + self.viewport.y - self.line_origin_y).max(0.0);
        let visible_end_px =
            (scroll_px + self.viewport.y + self.viewport.height - self.line_origin_y).max(0.0);
        let first_line = (visible_start_px / CONSOLE_OUTPUT_LINE_HEIGHT).floor() as usize;
        let line_end_exclusive = (visible_end_px / CONSOLE_OUTPUT_LINE_HEIGHT).ceil() as usize;
        let first_line = first_line.min(self.line_count);
        let line_end_exclusive = line_end_exclusive.min(self.line_count).max(first_line);
        first_line..line_end_exclusive
    }

    fn materialized_logical_line_range(&self, scroll_px: f32) -> Range<usize> {
        if !self.is_virtualized() || self.line_count <= self.materialized_line_count {
            return 0..self.line_count;
        }
        let visible = self.visible_logical_line_range(scroll_px);
        let capacity = self.materialized_line_count;
        let mut start = visible.start.saturating_sub(self.overscan_lines);
        let requested_end = visible
            .end
            .saturating_add(self.overscan_lines)
            .min(self.line_count);
        let end = requested_end
            .max(start.saturating_add(capacity))
            .min(self.line_count);
        if end - start < capacity {
            start = end.saturating_sub(capacity);
        }
        start..end
    }
}

fn console_output_text_tone(level: EditorConsoleMessageLevel) -> &'static str {
    match level {
        EditorConsoleMessageLevel::Info => "secondary",
        EditorConsoleMessageLevel::Warning => "warning",
        EditorConsoleMessageLevel::Error => "error",
    }
}

fn console_output_level_label(level: EditorConsoleMessageLevel) -> &'static str {
    match level {
        EditorConsoleMessageLevel::Info => "[Info]",
        EditorConsoleMessageLevel::Warning => "[Warning]",
        EditorConsoleMessageLevel::Error => "[Error]",
    }
}

pub(in crate::ui::retained_host) fn console_output_lines(text: &str) -> impl Iterator<Item = &str> {
    console_output_lines_with_presence(text, !text.is_empty())
}

pub(in crate::ui::retained_host) fn console_output_lines_with_presence(
    text: &str,
    has_output: bool,
) -> impl Iterator<Item = &str> {
    let is_empty = text.is_empty();
    std::iter::once(CONSOLE_EMPTY_OUTPUT)
        .filter(move |_| is_empty && !has_output)
        .chain(std::iter::once("").filter(move |_| is_empty && has_output))
        .chain(
            text.split('\n')
                .filter(move |_| !is_empty)
                .map(|line| line.strip_suffix('\r').unwrap_or(line)),
        )
}

pub(in crate::ui::retained_host) fn console_output_line_count(text: &str) -> usize {
    if text.is_empty() {
        1
    } else {
        text.as_bytes()
            .iter()
            .filter(|byte| **byte == b'\n')
            .count()
            + 1
    }
}

pub(in crate::ui::retained_host) fn console_content_extent(text: &str) -> f32 {
    console_output_line_count(text) as f32 * CONSOLE_OUTPUT_LINE_HEIGHT
}

pub(in crate::ui::retained_host) fn console_snapshot_content_extent(
    output: &ConsoleOutputSnapshot,
) -> f32 {
    output.logical_line_count().max(1) as f32 * CONSOLE_OUTPUT_LINE_HEIGHT
}

#[cfg(test)]
#[path = "console_output/tests.rs"]
mod tests;
