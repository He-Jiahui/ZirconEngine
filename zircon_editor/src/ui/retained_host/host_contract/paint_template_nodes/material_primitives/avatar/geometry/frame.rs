use crate::ui::retained_host::host_contract::data::FrameRect;

use super::metrics::{avatar_bounded_extent, AVATAR_DEFAULT_EDGE};

// 根序列将布局框收成默认最大边长的方框并垂直居中；遮罩与边框以此框为准。
pub(in crate::ui::retained_host::host_contract::paint_template_nodes) fn avatar_frame(
    rect: &FrameRect,
) -> FrameRect {
    let available_height = avatar_bounded_extent(rect.height);
    let size = avatar_bounded_extent(rect.width)
        .min(available_height)
        .min(AVATAR_DEFAULT_EDGE);
    FrameRect {
        x: rect.x,
        y: rect.y + (available_height - size) * 0.5,
        width: size,
        height: size,
    }
}

#[cfg(test)]
#[path = "tests/frame.rs"]
mod tests;
