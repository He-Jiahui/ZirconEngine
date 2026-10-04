//! 把主体均分为选项框；末项吸收浮点除法余差，分隔线和标签均以各选项框为基准。

use super::super::super::super::{data::FrameRect, paint_geometry::bounded_extent};
use super::super::metrics::{
    segment_divider_inset_y, segment_divider_width, segment_text_inset_x, segment_text_inset_y,
};

pub(in crate::ui::retained_host::host_contract::paint_template_nodes) fn segment_divider_rect(
    segment: &FrameRect,
) -> FrameRect {
    let inset_y = segment_divider_inset_y();
    FrameRect {
        x: segment.x,
        y: segment.y + inset_y,
        width: bounded_extent(segment.width).min(bounded_extent(segment_divider_width())),
        height: bounded_extent(segment.height - inset_y * 2.0),
    }
}

pub(in crate::ui::retained_host::host_contract::paint_template_nodes) fn segment_label_rect(
    segment: &FrameRect,
) -> FrameRect {
    let inset_x = segment_text_inset_x();
    let inset_y = segment_text_inset_y();
    FrameRect {
        x: segment.x + inset_x,
        y: segment.y + inset_y,
        width: bounded_extent(segment.width - inset_x * 2.0),
        height: bounded_extent(segment.height - inset_y * 2.0),
    }
}

/// 调用方传入已过滤空项后的 index/count；最后一段吸收小数余差以覆盖整个主体宽度。
pub(in crate::ui::retained_host::host_contract::paint_template_nodes) fn segment_rect(
    rect: &FrameRect,
    index: usize,
    count: usize,
) -> FrameRect {
    let count = count.max(1);
    let width = bounded_extent(rect.width) / count as f32;
    FrameRect {
        x: rect.x + width * index as f32,
        y: rect.y,
        width: if index + 1 == count {
            rect.x + rect.width - (rect.x + width * index as f32)
        } else {
            width
        }
        .max(0.0),
        height: bounded_extent(rect.height),
    }
}

#[cfg(test)]
#[path = "tests/item.rs"]
mod tests;
