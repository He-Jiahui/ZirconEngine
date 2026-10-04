//! 分隔线的行内位置合同；沿用文本左右边距，避免触及弹层框线。

use super::super::super::super::super::data::FrameRect;
use super::super::super::metrics::WorkbenchPopupRowMetrics;

pub(super) fn popup_separator_rect(
    row_rect: &FrameRect,
    metrics: &WorkbenchPopupRowMetrics,
) -> FrameRect {
    FrameRect {
        x: row_rect.x + metrics.text_left,
        y: row_rect.y + row_rect.height * 0.5,
        width: (row_rect.width - metrics.text_left - metrics.text_right).max(0.0),
        height: metrics.outline_width,
    }
}
