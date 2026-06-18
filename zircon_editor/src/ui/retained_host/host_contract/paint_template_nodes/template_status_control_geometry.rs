use super::super::data::{FrameRect, TemplatePaneNodeData};
use super::style_selector::WorkbenchStatusSignalKind as StatusSignalKind;
use super::template_status_glyphs::{centered_rect, STATUS_ICON_GLYPH_SIZE, STATUS_ITEM_ICON_SIZE};

pub(super) const STATUS_FONT_SIZE: f32 = 12.0;
pub(super) const STATUS_CHIP_RADIUS: f32 = 6.0;
pub(super) const STATUS_ICON_BUTTON_RADIUS: f32 = 5.0;

const STATUS_ITEM_ICON_LEFT: f32 = 24.0;
const STATUS_ITEM_TEXT_GAP: f32 = 9.0;
const STATUS_READY_DOT_SIZE: f32 = 10.0;
const STATUS_CHIP_TEXT_LEFT: f32 = 12.0;
const STATUS_CHIP_RIGHT_RESERVE: f32 = 24.0;

pub(super) fn status_line_height() -> f32 {
    STATUS_FONT_SIZE * 1.2
}

pub(super) fn status_control_offset_rect(
    node: &TemplatePaneNodeData,
    rect: &FrameRect,
) -> FrameRect {
    FrameRect {
        x: rect.x + node.layout_offset_x,
        y: rect.y + node.layout_offset_y,
        width: rect.width,
        height: rect.height,
    }
}

pub(super) fn status_signal_icon_rect(
    node: &TemplatePaneNodeData,
    rect: &FrameRect,
    kind: StatusSignalKind,
) -> FrameRect {
    let size = status_signal_icon_size(node, kind);
    FrameRect {
        x: rect.x + STATUS_ITEM_ICON_LEFT + node.layout_offset_x,
        y: rect.y
            + node.layout_offset_y
            + (rect.height - size).max(0.0) * 0.5
            + node.layout_content_offset_y,
        width: size,
        height: size,
    }
}

pub(super) fn status_signal_icon_paint_rect(
    node: &TemplatePaneNodeData,
    rect: &FrameRect,
    kind: StatusSignalKind,
) -> FrameRect {
    let size = status_signal_visual_icon_size(node, kind)
        .min(rect.width.min(rect.height).max(1.0))
        .max(1.0);
    centered_rect(rect, size)
}

pub(super) fn status_signal_text_rect(
    node: &TemplatePaneNodeData,
    rect: &FrameRect,
    icon: &FrameRect,
) -> FrameRect {
    let line_height = status_line_height();
    let text_gap = status_signal_text_gap(node);
    FrameRect {
        x: icon.x + icon.width + text_gap,
        y: rect.y + node.layout_offset_y + (rect.height - line_height).max(0.0) * 0.5,
        width: (rect.x + rect.width - icon.x - icon.width - text_gap).max(1.0),
        height: line_height,
    }
}

pub(super) fn status_chip_text_rect(rect: &FrameRect) -> FrameRect {
    let line_height = status_line_height();
    FrameRect {
        x: rect.x + STATUS_CHIP_TEXT_LEFT,
        y: rect.y + (rect.height - line_height).max(0.0) * 0.5,
        width: (rect.width - STATUS_CHIP_TEXT_LEFT - STATUS_CHIP_RIGHT_RESERVE).max(1.0),
        height: line_height,
    }
}

pub(super) fn status_chip_chevron_rect(rect: &FrameRect) -> FrameRect {
    FrameRect {
        x: rect.x + rect.width - 18.0,
        y: rect.y + (rect.height - 12.0).max(0.0) * 0.5,
        width: 12.0,
        height: 12.0,
    }
}

pub(super) fn status_icon_button_glyph_rect(rect: &FrameRect) -> FrameRect {
    centered_rect(rect, STATUS_ICON_GLYPH_SIZE)
}

pub(super) fn status_signal_icon_size(node: &TemplatePaneNodeData, kind: StatusSignalKind) -> f32 {
    if node.value_number > 0.0 {
        return node.value_number;
    }
    match kind {
        StatusSignalKind::Ready => STATUS_READY_DOT_SIZE,
        StatusSignalKind::Success | StatusSignalKind::Warning | StatusSignalKind::Info => {
            STATUS_ITEM_ICON_SIZE
        }
    }
}

fn status_signal_visual_icon_size(node: &TemplatePaneNodeData, kind: StatusSignalKind) -> f32 {
    if node.layout_icon_size.is_finite() && node.layout_icon_size > 0.0 {
        node.layout_icon_size
    } else {
        status_signal_icon_size(node, kind)
    }
}

#[cfg(test)]
pub(super) fn status_signal_text_gap(node: &TemplatePaneNodeData) -> f32 {
    if node.layout_content_offset_x > 0.0 {
        node.layout_content_offset_x
    } else {
        STATUS_ITEM_TEXT_GAP
    }
}

#[cfg(not(test))]
fn status_signal_text_gap(node: &TemplatePaneNodeData) -> f32 {
    if node.layout_content_offset_x > 0.0 {
        node.layout_content_offset_x
    } else {
        STATUS_ITEM_TEXT_GAP
    }
}
