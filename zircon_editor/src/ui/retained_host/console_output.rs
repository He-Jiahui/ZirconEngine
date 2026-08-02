use std::ops::Range;

mod viewport_size;

pub(in crate::ui::retained_host) use viewport_size::console_output_viewport_size;

pub(in crate::ui::retained_host) const CONSOLE_OUTPUT_LINE_HEIGHT: f32 = 18.0;
pub(in crate::ui::retained_host) const CONSOLE_OUTPUT_SEVERITY_SLOT_WIDTH: f32 = 64.0;
pub(in crate::ui::retained_host) const CONSOLE_OUTPUT_MIN_MESSAGE_SLOT_WIDTH: f32 = 32.0;
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

#[derive(Clone, Debug)]
pub(in crate::ui::retained_host) struct ConsoleOutputPaintMetadata {
    viewport: ConsoleOutputViewport,
    line_origin_y: f32,
    line_rows: Range<usize>,
    line_count: usize,
    nodes_per_line: usize,
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
            })
    }

    pub(in crate::ui::retained_host) const fn viewport(&self) -> ConsoleOutputViewport {
        self.viewport
    }

    pub(in crate::ui::retained_host) fn content_extent(&self) -> f32 {
        self.line_count as f32 * CONSOLE_OUTPUT_LINE_HEIGHT
    }

    pub(in crate::ui::retained_host) fn visible_node_rows(
        &self,
        row_count: usize,
        scroll_px: f32,
    ) -> Vec<usize> {
        let line_start = self.line_rows.start.min(row_count);
        let line_end = self.line_rows.end.min(row_count).max(line_start);
        let available_line_count = (line_end - line_start) / self.nodes_per_line;
        let scroll_px = scroll_px.max(0.0);
        let visible_start_px = (scroll_px + self.viewport.y - self.line_origin_y).max(0.0);
        let visible_end_px =
            (scroll_px + self.viewport.y + self.viewport.height - self.line_origin_y).max(0.0);
        let first_line = (visible_start_px / CONSOLE_OUTPUT_LINE_HEIGHT).floor() as usize;
        let line_end_exclusive = (visible_end_px / CONSOLE_OUTPUT_LINE_HEIGHT).ceil() as usize;
        let first_line = first_line.min(available_line_count);
        let line_end_exclusive = line_end_exclusive.min(available_line_count).max(first_line);
        let visible_row_start = line_start + first_line * self.nodes_per_line;
        let visible_row_end = line_start + line_end_exclusive * self.nodes_per_line;

        let mut rows = Vec::with_capacity(
            row_count - (line_end - line_start) + visible_row_end.saturating_sub(visible_row_start),
        );
        rows.extend(0..line_start);
        rows.extend(visible_row_start..visible_row_end);
        rows.extend(line_end..row_count);
        rows
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

#[cfg(test)]
#[path = "console_output/tests.rs"]
mod tests;
