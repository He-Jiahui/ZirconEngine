use super::super::super::super::super::data::{FrameRect, TemplatePaneNodeData};
pub(in crate::ui::retained_host::host_contract::paint_template_nodes) use super::super::super::bounded_extent as chip_bounded_extent;
use super::super::identity::{chip_is_outlined, chip_is_small};

pub(in crate::ui::retained_host::host_contract::paint_template_nodes) const CHIP_MEDIUM_HEIGHT:
    f32 = 32.0;
pub(in crate::ui::retained_host::host_contract::paint_template_nodes) const CHIP_SMALL_HEIGHT: f32 =
    24.0;
pub(in crate::ui::retained_host::host_contract::paint_template_nodes) const CHIP_LABEL_FONT_SIZE:
    f32 = 13.0;
pub(in crate::ui::retained_host::host_contract::paint_template_nodes) const CHIP_SMALL_LABEL_FONT_SIZE: f32 = 12.0;
pub(in crate::ui::retained_host::host_contract::paint_template_nodes) const CHIP_LABEL_PADDING:
    f32 = 12.0;
pub(in crate::ui::retained_host::host_contract::paint_template_nodes) const CHIP_LABEL_OUTLINED_PADDING: f32 = 11.0;
pub(in crate::ui::retained_host::host_contract::paint_template_nodes) const CHIP_SMALL_LABEL_PADDING: f32 = 8.0;
pub(in crate::ui::retained_host::host_contract::paint_template_nodes) const CHIP_SMALL_OUTLINED_LABEL_PADDING: f32 = 7.0;
pub(in crate::ui::retained_host::host_contract::paint_template_nodes) const CHIP_AVATAR_MEDIUM_EDGE: f32 = 24.0;
pub(in crate::ui::retained_host::host_contract::paint_template_nodes) const CHIP_AVATAR_SMALL_EDGE: f32 = 18.0;
pub(in crate::ui::retained_host::host_contract::paint_template_nodes) const CHIP_ICON_MEDIUM_EDGE: f32 = 20.0;
pub(in crate::ui::retained_host::host_contract::paint_template_nodes) const CHIP_ICON_SMALL_EDGE:
    f32 = 18.0;
pub(in crate::ui::retained_host::host_contract::paint_template_nodes) const CHIP_DELETE_MEDIUM_EDGE: f32 = 22.0;
pub(in crate::ui::retained_host::host_contract::paint_template_nodes) const CHIP_DELETE_SMALL_EDGE: f32 = 16.0;

const CHIP_LABEL_LINE_HEIGHT_RATIO: f32 = 1.5;
const CHIP_LABEL_VERTICAL_CENTER_RATIO: f32 = 0.5;

pub(in crate::ui::retained_host::host_contract::paint_template_nodes) fn chip_font_size(
    node: &TemplatePaneNodeData,
    rect: &FrameRect,
) -> f32 {
    let requested = if node.font_size.is_finite() && node.font_size > 0.0 {
        node.font_size
    } else if chip_is_small(node) {
        CHIP_SMALL_LABEL_FONT_SIZE
    } else {
        CHIP_LABEL_FONT_SIZE
    };
    requested.min(chip_bounded_extent(rect.width).min(chip_bounded_extent(rect.height)))
}

pub(in crate::ui::retained_host::host_contract::paint_template_nodes) fn chip_label_base_padding(
    node: &TemplatePaneNodeData,
) -> f32 {
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

pub(in crate::ui::retained_host::host_contract::paint_template_nodes) fn chip_label_line_height(
    font_size: f32,
    rect: &FrameRect,
) -> f32 {
    chip_bounded_extent(font_size * CHIP_LABEL_LINE_HEIGHT_RATIO)
        .min(chip_bounded_extent(rect.height))
}

pub(in crate::ui::retained_host::host_contract::paint_template_nodes) fn chip_label_width(
    measured_width: f32,
    available_width: f32,
) -> f32 {
    measured_width
        .is_finite()
        .then_some(measured_width.max(0.0))
        .unwrap_or(0.0)
        .min(chip_bounded_extent(available_width))
}

pub(in crate::ui::retained_host::host_contract::paint_template_nodes) fn chip_label_y(
    rect: &FrameRect,
    line_height: f32,
) -> f32 {
    rect.y
        + (chip_bounded_extent(rect.height) - line_height).max(0.0)
            * CHIP_LABEL_VERTICAL_CENTER_RATIO
}

#[cfg(test)]
mod tests {
    use super::*;

    fn node(font_size: f32) -> TemplatePaneNodeData {
        TemplatePaneNodeData {
            font_size,
            ..TemplatePaneNodeData::default()
        }
    }

    #[test]
    fn chip_label_metrics_project_font_line_height_and_y() {
        let rect = FrameRect {
            x: 0.0,
            y: 10.0,
            width: 120.0,
            height: 32.0,
        };
        let line_height = chip_label_line_height(chip_font_size(&node(13.0), &rect), &rect);

        assert!((line_height - 19.5).abs() <= 0.01);
        assert!((chip_label_y(&rect, line_height) - 16.25).abs() <= 0.01);
    }

    #[test]
    fn chip_label_width_clamps_to_available_bounds() {
        assert!((chip_label_width(80.0, 44.0) - 44.0).abs() <= 0.01);
        assert!((chip_label_width(0.0, 44.0) - 0.0).abs() <= 0.01);
    }
}
