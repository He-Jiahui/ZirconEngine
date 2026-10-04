use crate::ui::retained_host::host_contract::data::FrameRect;

use super::metrics::AVATAR_FALLBACK_SCALE;

// 为默认图标分配回退框，供图像请求或内置 glyph 使用；极窄布局的下限问题见紧邻 BUG 标签。
pub(in crate::ui::retained_host::host_contract::paint_template_nodes) fn avatar_fallback_child_frame(
    rect: &FrameRect,
) -> FrameRect {
    centered_child_rect(rect, AVATAR_FALLBACK_SCALE)
}

fn centered_child_rect(rect: &FrameRect, scale: f32) -> FrameRect {
    // BUG: [CR-M18-AVATAR-0002] 小于 1 的 Avatar 仍把回退子框扩成 1×1；pane 级裁剪可让图标越过 Avatar 自身边界。
    let size = (rect.width.min(rect.height) * scale.clamp(0.0, 1.0)).max(1.0);
    FrameRect {
        x: rect.x + (rect.width - size) * 0.5,
        y: rect.y + (rect.height - size) * 0.5,
        width: size,
        height: size,
    }
}
