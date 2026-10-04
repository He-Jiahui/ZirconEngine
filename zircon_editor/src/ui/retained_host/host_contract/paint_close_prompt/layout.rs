use super::super::data::{FrameRect, HostClosePromptData};
use super::super::paint_theme::HostControlMetrics;

#[derive(Clone, Debug, PartialEq)]
pub(super) struct PromptTextLayout {
    pub(super) title_x: f32,
    pub(super) title_y: f32,
    pub(super) message_x: f32,
    pub(super) message_y: f32,
    pub(super) details_frame: FrameRect,
    pub(super) details_x: f32,
    pub(super) details_y: f32,
}

pub(super) fn prompt_text_layout(prompt: &HostClosePromptData) -> PromptTextLayout {
    prompt_text_layout_with_metrics(prompt, super::super::paint_theme::current_host_metrics())
}

fn prompt_text_layout_with_metrics(
    prompt: &HostClosePromptData,
    metrics: HostControlMetrics,
) -> PromptTextLayout {
    let outer_inset = metrics.gap_l + metrics.gap_s + metrics.border_width * 2.0;
    let inner_inset = (metrics.gap_m - metrics.border_width * 2.0).max(0.0);
    let body_line_height = metrics
        .line_height(metrics.font_body)
        .round()
        .max(metrics.font_body.ceil());
    let title_x = prompt.dialog_frame.x + outer_inset;
    let title_y = prompt.dialog_frame.y + outer_inset;
    let title_stack_gap = (metrics.gap_l + metrics.gap_s - metrics.border_width * 2.0).max(0.0);
    let message_y = title_y + body_line_height + title_stack_gap;
    let details_y = message_y + body_line_height + metrics.gap_l;
    let details_frame = FrameRect {
        x: title_x,
        y: details_y,
        width: (prompt.dialog_frame.width - outer_inset * 2.0).max(0.0),
        height: body_line_height * 2.0 + metrics.gap_m + metrics.border_width * 2.0,
    };

    PromptTextLayout {
        title_x,
        title_y,
        message_x: title_x,
        message_y,
        details_x: details_frame.x + inner_inset,
        details_y: details_frame.y + metrics.gap_m + metrics.border_width * 2.0,
        details_frame,
    }
}

#[cfg(test)]
#[path = "tests/layout.rs"]
mod tests;
