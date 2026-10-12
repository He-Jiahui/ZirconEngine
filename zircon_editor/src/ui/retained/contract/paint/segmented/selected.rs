//! 选中项内缩表面及其下划线区域；绘制器先核对可见性再发出相应层级。

use super::super::super::super::{data::FrameRect, paint_geometry::bounded_extent};
use super::super::metrics::segment_selected_inset;

pub(in crate::ui::retained_host::host_contract::paint_template_nodes) fn selected_segment_rect(
    segment: &FrameRect,
) -> FrameRect {
    inset_rect(segment, segment_selected_inset())
}

pub(in crate::ui::retained_host::host_contract::paint_template_nodes) fn selected_segment_underline_rect(
    selected_rect: &FrameRect,
    underline_height: f32,
) -> FrameRect {
    FrameRect {
        x: selected_rect.x,
        y: selected_rect.y + (selected_rect.height - underline_height).max(0.0),
        width: bounded_extent(selected_rect.width),
        height: bounded_extent(underline_height).min(bounded_extent(selected_rect.height)),
    }
}

fn inset_rect(rect: &FrameRect, inset: f32) -> FrameRect {
    let inset = bounded_extent(inset);
    FrameRect {
        x: rect.x + inset,
        y: rect.y + inset,
        width: bounded_extent(rect.width - inset * 2.0),
        height: bounded_extent(rect.height - inset * 2.0),
    }
}

#[cfg(test)]
#[path = "tests/selected.rs"]
mod tests;
