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

/// 模板显式字号优先；最终字号受 Chip 当前帧约束，避免标签超过极窄控件。
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
#[path = "tests/metrics.rs"]
mod tests;
