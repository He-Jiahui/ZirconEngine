//! 组标签使用原始组框而非偏移后的主体框，使标题与选项主体保留独立排布。

use super::super::super::super::data::FrameRect;
use super::super::super::super::paint_geometry::bounded_extent;
use super::super::metrics::segment_group_label_height;

pub(in crate::ui::retained_host::host_contract::paint_template_nodes) fn segmented_group_label_rect(
    rect: &FrameRect,
) -> FrameRect {
    FrameRect {
        x: rect.x,
        y: rect.y,
        width: bounded_extent(rect.width),
        height: bounded_extent(segment_group_label_height()).min(bounded_extent(rect.height)),
    }
}

#[cfg(test)]
#[path = "tests/group.rs"]
mod tests;
