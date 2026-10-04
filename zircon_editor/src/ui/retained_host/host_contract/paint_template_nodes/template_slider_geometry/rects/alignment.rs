//! 为滑块本体及可选 halo 提供共同中心框；尺寸无效时返回退化框，由最终绘制器过滤。

use super::super::super::super::{data::FrameRect, paint_geometry::bounded_extent};

pub(in crate::ui::retained_host::host_contract::paint_template_nodes) fn centered_rect(
    center_x: f32,
    center_y: f32,
    size: f32,
) -> FrameRect {
    let size = bounded_extent(size);
    FrameRect {
        x: center_x - size * 0.5,
        y: center_y - size * 0.5,
        width: size,
        height: size,
    }
}

#[cfg(test)]
#[path = "tests/alignment.rs"]
mod tests;
