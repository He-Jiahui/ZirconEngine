//! 把完整候选索引映射到面板行，再为标题和辅助说明提供文本区域。
//! 行位置与可见行筛选共享密度指标；调用方负责裁剪，不在此重排行或执行分页。

use super::super::super::super::data::FrameRect;
use super::common::symmetric_extent;
use super::metrics::command_palette_metrics;

pub(in crate::ui::retained_host::host_contract::paint_template_nodes) fn row_rect(
    panel_rect: &FrameRect,
    row: usize,
) -> FrameRect {
    let metrics = command_palette_metrics();
    FrameRect {
        x: panel_rect.x + metrics.row_inset_x,
        y: panel_rect.y + metrics.list_top + row as f32 * metrics.row_height,
        width: (panel_rect.width - symmetric_extent(metrics.row_inset_x))
            .max(metrics.min_frame_extent),
        height: metrics.row_height,
    }
}

pub(in crate::ui::retained_host::host_contract::paint_template_nodes) fn row_label_rect(
    row_rect: &FrameRect,
) -> FrameRect {
    // BUG: [CR-EDITOR-PAINT-OVERLAY-0001] 标题区域延伸至行右侧，与辅助说明区域重叠；
    // 两者由同一行入口同时绘制且没有列间裁剪，长标题会进入快捷键/说明列。应按实际辅助内容预留列宽并补长标题 fixture。
    let metrics = command_palette_metrics();
    FrameRect {
        x: row_rect.x + metrics.row_text_x,
        y: row_rect.y + metrics.row_text_y,
        width: (row_rect.width - symmetric_extent(metrics.row_text_x))
            .max(metrics.min_frame_extent),
        height: (row_rect.height - symmetric_extent(metrics.row_text_y)).max(metrics.line_height),
    }
}

pub(in crate::ui::retained_host::host_contract::paint_template_nodes) fn row_detail_rect(
    row_rect: &FrameRect,
) -> FrameRect {
    let metrics = command_palette_metrics();
    FrameRect {
        x: row_rect.x + row_rect.width * metrics.row_detail_left_ratio,
        y: row_rect.y + metrics.row_text_y,
        width: (row_rect.width * metrics.row_detail_width_ratio).max(metrics.min_frame_extent),
        height: (row_rect.height - symmetric_extent(metrics.row_text_y)).max(metrics.line_height),
    }
}
