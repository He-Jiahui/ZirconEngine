use super::super::super::style_selector::WorkbenchStatusSignalKind as StatusSignalKind;
use super::super::super::template_status_glyphs::{centered_rect, STATUS_ITEM_ICON_SIZE};
use super::constants::{STATUS_ITEM_ICON_LEFT, STATUS_READY_DOT_SIZE};
use crate::ui::retained_host::host_contract::data::{FrameRect, TemplatePaneNodeData};

pub(in crate::ui::retained_host::host_contract::paint_template_nodes) fn status_signal_icon_rect(
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

pub(in crate::ui::retained_host::host_contract::paint_template_nodes) fn status_signal_icon_paint_rect(
    node: &TemplatePaneNodeData,
    rect: &FrameRect,
    kind: StatusSignalKind,
) -> FrameRect {
    let size = status_signal_visual_icon_size(node, kind)
        .min(rect.width.min(rect.height).max(1.0))
        .max(1.0);
    centered_rect(rect, size)
}

fn status_signal_icon_size(node: &TemplatePaneNodeData, kind: StatusSignalKind) -> f32 {
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
