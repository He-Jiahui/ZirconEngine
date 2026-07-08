use super::super::super::super::super::data::{FrameRect, TemplatePaneNodeData};

pub(in crate::ui::retained_host::host_contract::paint_template_nodes) const DIVIDER_THICKNESS: f32 =
    1.0;
pub(in crate::ui::retained_host::host_contract::paint_template_nodes) const DIVIDER_MIDDLE_HORIZONTAL_INSET: f32 = 16.0;
pub(in crate::ui::retained_host::host_contract::paint_template_nodes) const DIVIDER_INSET_HORIZONTAL_INSET: f32 = 72.0;
pub(in crate::ui::retained_host::host_contract::paint_template_nodes) const DIVIDER_MIDDLE_VERTICAL_INSET: f32 = 8.0;
pub(in crate::ui::retained_host::host_contract::paint_template_nodes) const DIVIDER_WRAPPER_HORIZONTAL_PADDING: f32 = 9.6;
pub(in crate::ui::retained_host::host_contract::paint_template_nodes) const DIVIDER_WRAPPER_VERTICAL_PADDING: f32 = 9.6;

const DIVIDER_DEFAULT_FONT_SIZE: f32 = 12.0;
const DIVIDER_MIN_FONT_SIZE: f32 = 8.0;
const DIVIDER_MAX_FONT_HEIGHT_RATIO: f32 = 0.82;
const DIVIDER_LABEL_LINE_HEIGHT_RATIO: f32 = 1.2;
const DIVIDER_LABEL_CENTER_RATIO: f32 = 0.5;
const DIVIDER_VERTICAL_TEXT_HORIZONTAL_PADDING_RATIO: f32 = 0.25;
const DIVIDER_MIN_TEXT_FRAME_EXTENT: f32 = 1.0;

pub(in crate::ui::retained_host::host_contract::paint_template_nodes) fn divider_font_size(
    node: &TemplatePaneNodeData,
    available_height: f32,
) -> f32 {
    let requested = if node.font_size.is_finite() && node.font_size > 0.0 {
        node.font_size
    } else {
        DIVIDER_DEFAULT_FONT_SIZE
    };
    requested
        .min((available_height * DIVIDER_MAX_FONT_HEIGHT_RATIO).max(DIVIDER_MIN_FONT_SIZE))
        .max(DIVIDER_MIN_FONT_SIZE)
}

pub(in crate::ui::retained_host::host_contract::paint_template_nodes) fn divider_label_line_height(
    font_size: f32,
) -> f32 {
    font_size * DIVIDER_LABEL_LINE_HEIGHT_RATIO
}

pub(in crate::ui::retained_host::host_contract::paint_template_nodes) fn divider_wrapped_label_width(
    measured_text_width: f32,
    available_width: f32,
) -> f32 {
    (measured_text_width + DIVIDER_WRAPPER_HORIZONTAL_PADDING * 2.0)
        .max(DIVIDER_WRAPPER_HORIZONTAL_PADDING * 2.0)
        .min(available_width.max(0.0))
}

pub(in crate::ui::retained_host::host_contract::paint_template_nodes) fn divider_centered_label_y(
    rect: &FrameRect,
    line_height: f32,
) -> f32 {
    rect.y + (rect.height - line_height).max(0.0) * DIVIDER_LABEL_CENTER_RATIO
}

pub(in crate::ui::retained_host::host_contract::paint_template_nodes) fn divider_vertical_label_height(
    font_size: f32,
    rect_height: f32,
) -> f32 {
    (divider_label_line_height(font_size) + DIVIDER_WRAPPER_VERTICAL_PADDING * 2.0)
        .max(DIVIDER_WRAPPER_VERTICAL_PADDING * 2.0)
        .min(rect_height.max(0.0))
}

pub(in crate::ui::retained_host::host_contract::paint_template_nodes) fn divider_vertical_text_horizontal_padding(
    rect_width: f32,
) -> f32 {
    DIVIDER_WRAPPER_HORIZONTAL_PADDING
        .min(rect_width * DIVIDER_VERTICAL_TEXT_HORIZONTAL_PADDING_RATIO)
}

pub(in crate::ui::retained_host::host_contract::paint_template_nodes) fn divider_min_text_frame_extent(
    extent: f32,
) -> f32 {
    extent.max(DIVIDER_MIN_TEXT_FRAME_EXTENT)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn divider_text_metrics_project_wrapper_and_centering_rules() {
        let rect = FrameRect {
            x: 0.0,
            y: 10.0,
            width: 160.0,
            height: 40.0,
        };

        let line_height = divider_label_line_height(12.0);
        let wrapped_width = divider_wrapped_label_width(32.0, 120.0);

        assert!((line_height - 14.4).abs() <= 0.01);
        assert!((wrapped_width - 51.2).abs() <= 0.01);
        assert!((divider_centered_label_y(&rect, line_height) - 22.8).abs() <= 0.01);
    }

    #[test]
    fn divider_vertical_text_metrics_clamp_padding_and_min_extent() {
        assert!((divider_vertical_text_horizontal_padding(24.0) - 6.0).abs() <= 0.01);
        assert!((divider_min_text_frame_extent(0.2) - 1.0).abs() <= 0.01);
        assert!((divider_vertical_label_height(12.0, 28.0) - 28.0).abs() <= 0.01);
    }
}
