use super::super::data::FrameRect;
use super::super::paint_frame::HostRgbaFrame;
use super::super::paint_primitives::{draw_border, draw_rect, draw_text_bars_clipped};
use super::super::paint_theme::{current_host_metrics, HostControlMetrics};
use super::colors::ClosePromptPalette;

pub(in crate::ui::retained_host::host_contract) fn draw_prompt_button(
    frame: &mut HostRgbaFrame,
    button: &FrameRect,
    label: &str,
    enabled: bool,
    palette: ClosePromptPalette,
) {
    let metrics = current_host_metrics();
    draw_rect(
        frame,
        button.clone(),
        if enabled {
            palette.button
        } else {
            palette.button_disabled
        },
    );
    draw_border(
        frame,
        button.clone(),
        if enabled {
            palette.accent
        } else {
            palette.text_muted
        },
    );
    draw_text_bars_clipped(
        frame,
        prompt_button_label_x(button, metrics),
        prompt_button_label_y(button, metrics),
        label,
        Some(button),
        if enabled {
            palette.text
        } else {
            palette.text_disabled
        },
    );
}

fn prompt_button_label_x(button: &FrameRect, metrics: HostControlMetrics) -> f32 {
    button.x + metrics.button_pad_x.min(button.width.max(0.0) * 0.5)
}

fn prompt_button_label_y(button: &FrameRect, metrics: HostControlMetrics) -> f32 {
    let line_height = metrics
        .line_height(metrics.font_body)
        .round()
        .max(metrics.font_body.ceil())
        .min(button.height.max(0.0));
    button.y + ((button.height - line_height).max(0.0) * 0.5)
}

#[cfg(test)]
#[path = "tests/button.rs"]
mod tests;
