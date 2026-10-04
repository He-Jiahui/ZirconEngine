//! 通用模板图标的资产绘制门槛：只有名称、目标像素尺寸、可见交集和真实资源均存在时才追加命令。
//! 调用者可用返回值决定是否继续自己的后备外观；图标加载失败不生成缺失图标占位。

use super::super::data::FrameRect;
use super::super::paint_geometry::intersect;
use super::render_commands::HostPaintCommand;
use super::visual_assets::{load_existing_icon_asset_pixels_for_size, raster_size_from_frame};

pub(in crate::ui::retained_host::host_contract::paint_template_nodes) fn push_icon_asset_pixels(
    commands: &mut Vec<HostPaintCommand>,
    icon_name: &str,
    rect: &FrameRect,
    clip: &FrameRect,
    order: i32,
    tint: Option<[u8; 4]>,
    opacity: f32,
) -> bool {
    if icon_name.trim().is_empty() {
        return false;
    }
    let Some((target_width, target_height)) = raster_size_from_frame(rect.width, rect.height)
    else {
        return false;
    };
    let Some(damage_frame) = intersect(rect, clip) else {
        return false;
    };
    let Some(image) = load_existing_icon_asset_pixels_for_size(
        icon_name,
        target_width,
        target_height,
        tint,
        Some(damage_frame),
    ) else {
        return false;
    };

    commands.push(HostPaintCommand::image_pixels(
        rect.clone(),
        Some(clip.clone()),
        order,
        image.resource_key,
        image.width,
        image.height,
        image.rgba,
        image.atlas,
        opacity,
    ));
    true
}
