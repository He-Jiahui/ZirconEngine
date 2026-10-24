use super::super::super::super::super::paint_frame::HostRgbaFrame;
use super::super::super::super::super::paint_primitives::{
    draw_rgba_image_clipped_with_atlas, draw_shared_rgba_image_clipped_with_resource_key,
};
use super::super::super::command::HostPaintCommand;
use super::pixels::image_pixels_with_opacity;
use super::placeholder::draw_image_placeholder;

/// 有图集且完全不透明时优先保留图集资源；其余图像以共享像素进入绘制和录制路径。
pub(in crate::ui::retained_host::host_contract::paint_template_nodes) fn draw_image_command(
    frame: &mut HostRgbaFrame,
    command: &HostPaintCommand,
) -> bool {
    if let Some(image) = command.image_pixels.as_ref() {
        if command.opacity >= 1.0 {
            if let Some(atlas) = image.atlas.as_ref() {
                if draw_rgba_image_clipped_with_atlas(
                    frame,
                    command.frame.clone(),
                    command.clip_frame.as_ref(),
                    image.width,
                    image.height,
                    &image.rgba,
                    atlas,
                ) {
                    return true;
                }
            }
        }
        // Regression scope [CR-M19-PAINT-0001]: baked pixels require their own stable resource identity.
        let opacity_rgba =
            (command.opacity < 1.0).then(|| image_pixels_with_opacity(image, command.opacity));
        let shared_rgba = opacity_rgba.as_ref().unwrap_or(&image.rgba);
        let opacity_resource_key = opacity_rgba.as_ref().map(|rgba| {
            let mut identity = blake3::Hasher::new();
            identity.update(b"zircon-editor-opacity-rgba-v1\0");
            identity.update(&(image.resource_key.len() as u64).to_le_bytes());
            identity.update(image.resource_key.as_bytes());
            identity.update(&image.width.to_le_bytes());
            identity.update(&image.height.to_le_bytes());
            identity.update(&(rgba.len() as u64).to_le_bytes());
            identity.update(rgba.as_ref());
            format!("opacity-rgba:{}", identity.finalize().to_hex())
        });
        let resource_key = opacity_resource_key
            .as_deref()
            .unwrap_or(&image.resource_key);
        if draw_shared_rgba_image_clipped_with_resource_key(
            frame,
            command.frame.clone(),
            command.clip_frame.as_ref(),
            resource_key,
            0,
            image.width,
            image.height,
            shared_rgba,
        ) {
            return true;
        }
    }

    draw_image_placeholder(frame, command)
}
