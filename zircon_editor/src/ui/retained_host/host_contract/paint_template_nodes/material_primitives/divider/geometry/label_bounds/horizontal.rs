use super::super::super::super::super::super::{
    data::{FrameRect, TemplatePaneNodeData},
    paint_text::measure_runtime_text_width,
};
use super::super::align::pixel_aligned;
use super::super::metrics::{divider_font_size, divider_wrapped_label_width};
use super::align::{divider_text_align, divider_text_align_ratio};

pub(in crate::ui::retained_host::host_contract::paint_template_nodes) fn horizontal_label_bounds(
    node: &TemplatePaneNodeData,
    rect: &FrameRect,
    line_start: f32,
    line_end: f32,
    label: &str,
) -> (f32, f32) {
    let label_width = measured_horizontal_label_width(node, rect, label, line_end - line_start);
    let label_left = horizontal_label_left(node, line_start, line_end, label_width);
    (label_left, (label_left + label_width).min(line_end))
}

fn horizontal_label_left(
    node: &TemplatePaneNodeData,
    line_start: f32,
    line_end: f32,
    label_width: f32,
) -> f32 {
    let available = (line_end - line_start).max(0.0);
    let remaining = (available - label_width).max(0.0);
    let ratio = divider_text_align_ratio(divider_text_align(node));
    pixel_aligned(line_start + remaining * ratio)
}

fn measured_horizontal_label_width(
    node: &TemplatePaneNodeData,
    rect: &FrameRect,
    label: &str,
    available_width: f32,
) -> f32 {
    let font_size = divider_font_size(node, rect.height);
    let text_width = measure_runtime_text_width(label, font_size);
    divider_wrapped_label_width(text_width, available_width)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn rect(height: f32) -> FrameRect {
        FrameRect {
            x: 0.0,
            y: 0.0,
            width: 240.0,
            height,
        }
    }

    fn node(font_size: f32) -> TemplatePaneNodeData {
        TemplatePaneNodeData {
            font_size,
            text_align: "center".into(),
            ..TemplatePaneNodeData::default()
        }
    }

    #[test]
    fn horizontal_label_bounds_use_runtime_text_width_and_painted_font_size() {
        let node = node(24.0);
        let rect = rect(14.0);
        let line_start = 0.0;
        let line_end = 240.0;
        let label = "WWW iii";
        let font_size = divider_font_size(&node, rect.height);
        let expected_width = divider_wrapped_label_width(
            measure_runtime_text_width(label, font_size),
            line_end - line_start,
        );

        let (label_left, label_right) =
            horizontal_label_bounds(&node, &rect, line_start, line_end, label);

        assert!(
            ((label_right - label_left) - expected_width).abs() <= 0.01,
            "label bounds must use runtime text measurement width"
        );
        let old_heuristic_width = divider_wrapped_label_width(
            label.chars().count() as f32 * font_size * 0.56,
            line_end - line_start,
        );
        assert!(
            (expected_width - old_heuristic_width).abs() > 0.25,
            "fixture should catch regressions back to char-count width"
        );
    }
}
