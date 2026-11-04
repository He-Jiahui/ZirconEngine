//! 搜索控件内部的图标与文字共用位置合同；输入矩形已是面板内的搜索区域，避免重复加入面板原点。

use super::super::super::super::data::FrameRect;
use super::command_palette_metrics;
use super::common::{centered_offset, symmetric_extent};

pub(in crate::ui::retained_host::host_contract::paint_template_nodes) fn search_text_rect(
    search_rect: &FrameRect,
) -> FrameRect {
    let metrics = command_palette_metrics();
    FrameRect {
        x: search_rect.x + metrics.search_text_x,
        y: search_rect.y + metrics.search_text_y,
        width: (search_rect.width - symmetric_extent(metrics.search_text_x))
            .max(metrics.min_frame_extent),
        height: metrics.line_height,
    }
}

pub(in crate::ui::retained_host::host_contract::paint_template_nodes) fn search_icon_rect(
    search_rect: &FrameRect,
) -> FrameRect {
    let metrics = command_palette_metrics();
    FrameRect {
        x: search_rect.x + metrics.search_icon_x,
        y: search_rect.y + centered_offset(search_rect.height, metrics.search_icon_size),
        width: metrics.search_icon_size,
        height: metrics.search_icon_size,
    }
}
