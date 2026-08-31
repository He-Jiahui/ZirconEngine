use crate::ui::retained_host::hierarchy_pointer::{
    hierarchy_row_width, hierarchy_row_y, HierarchyRowMetrics,
};

use crate::ui::retained_host::host_contract::data::FrameRect;

pub(super) fn hierarchy_row_frame(
    viewport: &FrameRect,
    index: usize,
    scroll_px: f32,
    metrics: HierarchyRowMetrics,
) -> FrameRect {
    FrameRect {
        x: viewport.x + metrics.row_x,
        y: viewport.y + hierarchy_row_y(metrics, index, scroll_px),
        width: hierarchy_row_width(viewport.width, metrics),
        height: metrics.row_height,
    }
}
