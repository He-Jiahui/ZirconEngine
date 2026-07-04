use super::super::super::super::super::{
    data::{FrameRect, TemplatePaneNodeData},
    paint_text::measure_runtime_text_width,
};
use super::super::identity::{
    chip_has_avatar, chip_has_icon, chip_is_deletable, chip_is_outlined, chip_is_small,
};
use super::delete::chip_delete_edge;
use super::leading::{chip_leading_edge, chip_leading_margin, chip_negative_slot_margin};
use super::metrics::{
    CHIP_LABEL_FONT_SIZE, CHIP_LABEL_OUTLINED_PADDING, CHIP_LABEL_PADDING,
    CHIP_SMALL_LABEL_FONT_SIZE, CHIP_SMALL_LABEL_PADDING, CHIP_SMALL_OUTLINED_LABEL_PADDING,
};

pub(in crate::ui::retained_host::host_contract::paint_template_nodes) fn chip_label_frame(
    node: &TemplatePaneNodeData,
    rect: &FrameRect,
    label: &str,
) -> Option<(FrameRect, f32, f32)> {
    let font_size = chip_font_size(node);
    let line_height = font_size * 1.5;
    let left = rect.x + chip_label_left_padding(node);
    let right = rect.x + rect.width - chip_label_right_padding(node);
    if right <= left {
        return None;
    }
    let measured_width = measure_runtime_text_width(label, font_size);
    let width = measured_width.min(right - left).max(1.0);
    Some((
        FrameRect {
            x: left,
            y: rect.y + (rect.height - line_height).max(0.0) * 0.5,
            width,
            height: line_height,
        },
        font_size,
        line_height,
    ))
}

fn chip_font_size(node: &TemplatePaneNodeData) -> f32 {
    if node.font_size.is_finite() && node.font_size > 0.0 {
        node.font_size
    } else if chip_is_small(node) {
        CHIP_SMALL_LABEL_FONT_SIZE
    } else {
        CHIP_LABEL_FONT_SIZE
    }
}

fn chip_label_left_padding(node: &TemplatePaneNodeData) -> f32 {
    let base = chip_label_base_padding(node);
    if chip_has_avatar(node) || chip_has_icon(node) {
        base + chip_leading_margin(node) + chip_leading_edge(node) - chip_negative_slot_margin(node)
    } else {
        base
    }
}

fn chip_label_right_padding(node: &TemplatePaneNodeData) -> f32 {
    let base = chip_label_base_padding(node);
    if chip_is_deletable(node) {
        base + chip_delete_edge(node) - chip_negative_slot_margin(node)
    } else {
        base
    }
}

fn chip_label_base_padding(node: &TemplatePaneNodeData) -> f32 {
    if chip_is_small(node) {
        if chip_is_outlined(node) {
            CHIP_SMALL_OUTLINED_LABEL_PADDING
        } else {
            CHIP_SMALL_LABEL_PADDING
        }
    } else if chip_is_outlined(node) {
        CHIP_LABEL_OUTLINED_PADDING
    } else {
        CHIP_LABEL_PADDING
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn rect(width: f32) -> FrameRect {
        FrameRect {
            x: 8.0,
            y: 10.0,
            width,
            height: 32.0,
        }
    }

    fn node(font_size: f32) -> TemplatePaneNodeData {
        TemplatePaneNodeData {
            font_size,
            ..TemplatePaneNodeData::default()
        }
    }

    #[test]
    fn chip_label_frame_uses_runtime_text_width() {
        let node = node(13.0);
        let label = "WWW iii";
        let frame = chip_label_frame(&node, &rect(220.0), label)
            .expect("chip label has enough horizontal space")
            .0;
        let expected_width = measure_runtime_text_width(label, 13.0)
            .min(220.0 - chip_label_left_padding(&node) - chip_label_right_padding(&node))
            .max(1.0);

        assert!(
            (frame.width - expected_width).abs() <= 0.01,
            "chip label frame must use runtime text measurement width"
        );
        let old_heuristic_width = label.chars().count() as f32 * 13.0 * 0.56;
        assert!(
            (expected_width - old_heuristic_width).abs() > 0.25,
            "fixture should catch regressions back to char-count width"
        );
    }

    #[test]
    fn chip_label_frame_clamps_measured_width_to_available_space() {
        let node = node(13.0);
        let available_width =
            44.0 - chip_label_left_padding(&node) - chip_label_right_padding(&node);
        let frame = chip_label_frame(&node, &rect(44.0), "Long chip label")
            .expect("chip label has enough horizontal space")
            .0;

        assert!((frame.width - available_width).abs() <= 0.01);
    }
}
