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
mod tests {
    use super::{prompt_button_label_x, prompt_button_label_y};
    use crate::ui::retained_host::host_contract::{data::FrameRect, paint_theme::METRICS};

    #[test]
    fn prompt_button_label_offsets_follow_host_metrics_inside_narrow_buttons() {
        let button = FrameRect {
            x: 4.0,
            y: 8.0,
            width: 32.0,
            height: 32.0,
        };

        assert_eq!(prompt_button_label_x(&button, METRICS), 16.0);
        assert_eq!(prompt_button_label_y(&button, METRICS), 16.0);

        let narrow = FrameRect {
            width: 12.0,
            height: 10.0,
            ..button
        };
        assert!(prompt_button_label_x(&narrow, METRICS) <= narrow.x + narrow.width);
        assert!(prompt_button_label_y(&narrow, METRICS) >= narrow.y);
    }
}
