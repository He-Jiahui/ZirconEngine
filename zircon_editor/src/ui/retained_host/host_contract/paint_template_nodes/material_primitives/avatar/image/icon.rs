use super::super::super::super::super::data::{FrameRect, TemplatePaneNodeData};
use super::super::super::super::visual_assets::{
    raster_size_from_frame, template_image_pixels, HostPaintImagePixels,
};

// 头像图像和文字均无内容时，按 icon_name 尝试图像回退；该请求返回 None 后内容序列才画内置 glyph。
pub(in crate::ui::retained_host::host_contract::paint_template_nodes) fn avatar_icon_pixels(
    node: &TemplatePaneNodeData,
    icon_rect: &FrameRect,
    foreground: [u8; 4],
    damage_frame: FrameRect,
) -> Option<HostPaintImagePixels> {
    if node.icon_name.is_empty() {
        return None;
    }
    let (target_width, target_height) = raster_size_from_frame(icon_rect.width, icon_rect.height)?;
    template_image_pixels(
        &node.preview_image,
        "",
        node.icon_name.as_str(),
        target_width,
        target_height,
        Some(foreground),
        false,
        Some(damage_frame),
    )
}
