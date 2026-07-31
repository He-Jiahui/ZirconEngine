use super::super::super::super::data::{FrameRect, TemplatePaneNodeData};
use super::super::metrics::dialog_metrics;
use crate::ui::retained_host::host_contract::paint_text::measure_runtime_text_width;

pub(super) fn action_label(node: &TemplatePaneNodeData, index: usize) -> Option<String> {
    node.actions
        .row_data(index)
        .and_then(|action| non_empty(action.label.as_str()).map(str::to_string))
}

pub(super) fn action_width(text: &str) -> f32 {
    let metrics = dialog_metrics();
    (measure_runtime_text_width(text, metrics.action_font_size)
        + metrics.action_text_padding_x * 2.0
        + metrics.action_text_clip_guard)
        .max(metrics.action_min_width)
}

pub(super) fn action_text_frame(surface: &FrameRect, text: &str) -> FrameRect {
    let metrics = dialog_metrics();
    let available_width = (surface.width - metrics.action_text_padding_x * 2.0).max(0.0);
    let text_width = measure_runtime_text_width(text, metrics.action_font_size)
        .min(available_width)
        .max(0.0);
    FrameRect {
        x: surface.x + (surface.width - text_width).max(0.0) * 0.5,
        y: surface.y + (surface.height - metrics.action_line_height).max(0.0) * 0.5,
        width: text_width,
        height: metrics.action_line_height.min(surface.height).max(0.0),
    }
}

fn non_empty(value: &str) -> Option<&str> {
    let trimmed = value.trim();
    (!trimmed.is_empty()).then_some(trimmed)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn action_width_uses_runtime_text_measurement() {
        let text = "WWWiiiWWWiii";
        let metrics = dialog_metrics();
        let expected = (measure_runtime_text_width(text, metrics.action_font_size)
            + metrics.action_text_padding_x * 2.0
            + metrics.action_text_clip_guard)
            .max(metrics.action_min_width);
        let legacy_half_em_width = text.chars().count() as f32 * (metrics.action_font_size * 0.5)
            + metrics.action_text_padding_x * 2.0;

        assert!((action_width(text) - expected).abs() <= 0.01);
        assert_ne!(action_width(text).round(), legacy_half_em_width.round());
    }

    #[test]
    fn action_width_keeps_minimum_for_short_labels() {
        assert_eq!(action_width("OK"), dialog_metrics().action_min_width);
    }

    #[test]
    fn action_text_frame_centers_runtime_measured_text_inside_the_button_surface() {
        let surface = FrameRect {
            x: 20.0,
            y: 30.0,
            width: 96.0,
            height: dialog_metrics().action_height,
        };
        let frame = action_text_frame(&surface, "Confirm");

        assert!(frame.x >= surface.x);
        assert!(frame.x + frame.width <= surface.x + surface.width);
        assert!((frame.x + frame.width * 0.5 - (surface.x + surface.width * 0.5)).abs() < 0.01);
        assert!((frame.y + frame.height * 0.5 - (surface.y + surface.height * 0.5)).abs() < 0.01);
    }
}
