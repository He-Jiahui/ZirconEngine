use super::super::super::style_selector::WorkbenchStatusSignalKind as StatusSignalKind;
use super::super::super::template_status_glyphs::centered_rect;
use super::constants::status_signal_metrics;
use crate::ui::retained_host::host_contract::data::{FrameRect, TemplatePaneNodeData};

pub(in crate::ui::retained_host::host_contract::paint_template_nodes) fn status_signal_icon_rect(
    node: &TemplatePaneNodeData,
    rect: &FrameRect,
    _kind: StatusSignalKind,
) -> FrameRect {
    let metrics = status_signal_metrics();
    let size = metrics.signal_marker_size;
    FrameRect {
        x: rect.x + metrics.signal_icon_left + node.layout_offset_x,
        y: rect.y + node.layout_offset_y + (rect.height - size).max(0.0) * 0.5,
        width: size,
        height: size,
    }
}

pub(in crate::ui::retained_host::host_contract::paint_template_nodes) fn status_signal_icon_paint_rect(
    _node: &TemplatePaneNodeData,
    rect: &FrameRect,
    _kind: StatusSignalKind,
) -> FrameRect {
    let metrics = status_signal_metrics();
    let size = metrics
        .signal_marker_size
        .min(rect.width.min(rect.height).max(0.0))
        .max(0.0);
    centered_rect(rect, size)
}
