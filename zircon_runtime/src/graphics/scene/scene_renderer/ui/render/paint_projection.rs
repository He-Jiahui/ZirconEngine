use zircon_runtime_interface::ui::layout::UiLayoutMetrics;
use zircon_runtime_interface::ui::surface::{
    UiPaintElement, UiPaintPayload, UiRenderCommand, UiRenderCommandKind, UiTextPaint,
};

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub(super) struct ScreenSpaceUiTextPaintProjectionReport {
    text_command_count: usize,
    rich_command_count: usize,
    paint_element_count: usize,
    layout_line_count: usize,
    layout_run_count: usize,
    paint_run_count: usize,
    source_text_bytes: usize,
    run_text_bytes: usize,
    style_string_bytes: usize,
    rich_layout_run_count: usize,
    rich_paint_run_count: usize,
    rich_run_text_bytes: usize,
}

impl ScreenSpaceUiTextPaintProjectionReport {
    fn record(
        &mut self,
        command: &UiRenderCommand,
        text_paint: &UiTextPaint,
        paint_element_count: usize,
    ) {
        self.text_command_count = self.text_command_count.saturating_add(1);
        self.paint_element_count = self.paint_element_count.saturating_add(paint_element_count);
        self.paint_run_count = self.paint_run_count.saturating_add(text_paint.runs.len());
        self.source_text_bytes = self
            .source_text_bytes
            .saturating_add(text_paint.source_text.len());
        self.run_text_bytes = self.run_text_bytes.saturating_add(
            text_paint
                .runs
                .iter()
                .map(|run| run.text.len())
                .fold(0_usize, usize::saturating_add),
        );
        self.style_string_bytes = self
            .style_string_bytes
            .saturating_add(text_paint_style_string_bytes(text_paint));

        let Some(layout) = command.text_layout.as_ref() else {
            return;
        };
        let layout_run_count = layout
            .lines
            .iter()
            .fold(0_usize, |count, line| count.saturating_add(line.runs.len()));
        self.layout_line_count = self.layout_line_count.saturating_add(layout.lines.len());
        self.layout_run_count = self.layout_run_count.saturating_add(layout_run_count);
        if layout.rich_text_artifact.is_none() {
            return;
        }

        self.rich_command_count = self.rich_command_count.saturating_add(1);
        self.rich_layout_run_count = self.rich_layout_run_count.saturating_add(layout_run_count);
        self.rich_paint_run_count = self
            .rich_paint_run_count
            .saturating_add(text_paint.runs.len());
        self.rich_run_text_bytes = self.rich_run_text_bytes.saturating_add(
            text_paint
                .runs
                .iter()
                .map(|run| run.text.len())
                .fold(0_usize, usize::saturating_add),
        );
    }

    pub(super) fn publish_profile_counters(self) {
        #[cfg(any(feature = "profiling", feature = "profiling-tracy"))]
        crate::core::diagnostics::profiling::record_counter_batch(
            "runtime",
            &[
                (
                    "ui_text.paint_projection.command_count",
                    self.text_command_count as f64,
                ),
                (
                    "ui_text.paint_projection.rich_command_count",
                    self.rich_command_count as f64,
                ),
                (
                    "ui_text.paint_projection.paint_element_count",
                    self.paint_element_count as f64,
                ),
                (
                    "ui_text.paint_projection.layout_line_count",
                    self.layout_line_count as f64,
                ),
                (
                    "ui_text.paint_projection.layout_run_count",
                    self.layout_run_count as f64,
                ),
                (
                    "ui_text.paint_projection.paint_run_count",
                    self.paint_run_count as f64,
                ),
                (
                    "ui_text.paint_projection.source_text_bytes",
                    self.source_text_bytes as f64,
                ),
                (
                    "ui_text.paint_projection.run_text_bytes",
                    self.run_text_bytes as f64,
                ),
                (
                    "ui_text.paint_projection.style_string_bytes",
                    self.style_string_bytes as f64,
                ),
                (
                    "ui_text.paint_projection.rich_layout_run_count",
                    self.rich_layout_run_count as f64,
                ),
                (
                    "ui_text.paint_projection.rich_paint_run_count",
                    self.rich_paint_run_count as f64,
                ),
                (
                    "ui_text.paint_projection.rich_run_text_bytes",
                    self.rich_run_text_bytes as f64,
                ),
            ],
        );
        #[cfg(not(any(feature = "profiling", feature = "profiling-tracy")))]
        let _ = self;
    }
}

pub(super) fn project_transient_paint_elements(
    command: &UiRenderCommand,
    first_paint_order: u64,
    metrics: UiLayoutMetrics,
    elements: &mut Vec<UiPaintElement>,
    report: &mut ScreenSpaceUiTextPaintProjectionReport,
) {
    let has_text_payload = command.text.as_ref().is_some_and(|text| !text.is_empty())
        || matches!(command.kind, UiRenderCommandKind::Text);
    if !has_text_payload {
        command.fill_transient_paint_elements(first_paint_order, metrics, elements);
        return;
    }

    crate::profile_scope!(
        "runtime",
        "text.paint_projection",
        "materialize_transient_text_paint"
    );
    command.fill_transient_paint_elements(first_paint_order, metrics, elements);
    let Some(text_paint) = elements.iter().find_map(|element| match &element.payload {
        UiPaintPayload::Text { text } => Some(text),
        _ => None,
    }) else {
        return;
    };
    report.record(command, text_paint, elements.len());
}

fn text_paint_style_string_bytes(text_paint: &UiTextPaint) -> usize {
    let root_bytes = option_string_bytes(&text_paint.color)
        .saturating_add(option_string_bytes(&text_paint.font))
        .saturating_add(option_string_bytes(&text_paint.font_family));
    let run_bytes = text_paint.runs.iter().fold(0_usize, |bytes, run| {
        bytes
            .saturating_add(option_string_bytes(&run.color))
            .saturating_add(option_string_bytes(&run.font))
            .saturating_add(option_string_bytes(&run.font_family))
    });
    let decoration_bytes = text_paint
        .decorations
        .iter()
        .fold(0_usize, |bytes, decoration| {
            bytes.saturating_add(decoration.color.len())
        });
    root_bytes
        .saturating_add(run_bytes)
        .saturating_add(decoration_bytes)
}

fn option_string_bytes(value: &Option<String>) -> usize {
    value.as_ref().map_or(0, String::len)
}
