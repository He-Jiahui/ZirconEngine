use super::super::super::super::super::{
    data::{FrameRect, TemplatePaneNodeData},
    paint_text::measure_runtime_text_width,
};
use super::super::identity::{chip_has_avatar, chip_has_icon, chip_is_deletable};
use super::delete::chip_delete_edge;
use super::leading::{chip_leading_edge, chip_leading_margin, chip_negative_slot_margin};
use super::metrics::{
    chip_font_size, chip_label_base_padding, chip_label_line_height, chip_label_width, chip_label_y,
};

/// 文本帧须同时让出前置头像或图标及删除槽位，并以运行时字形测量限制实际宽度。
pub(in crate::ui::retained_host::host_contract::paint_template_nodes) fn chip_label_frame(
    node: &TemplatePaneNodeData,
    rect: &FrameRect,
    label: &str,
) -> Option<(FrameRect, f32, f32)> {
    let font_size = chip_font_size(node, rect);
    let line_height = chip_label_line_height(font_size, rect);
    let left = rect.x + chip_label_left_padding(node);
    let right = rect.x + rect.width - chip_label_right_padding(node);
    if right <= left {
        return None;
    }
    let measured_width = measure_runtime_text_width(label, font_size);
    let width = chip_label_width(measured_width, right - left);
    Some((
        FrameRect {
            x: left,
            y: chip_label_y(rect, line_height),
            width,
            height: line_height,
        },
        font_size,
        line_height,
    ))
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

#[cfg(test)]
#[path = "tests/label.rs"]
mod tests;
